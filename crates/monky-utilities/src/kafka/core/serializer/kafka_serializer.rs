/*
 * Copyright (C) 2025 Movibase Platform Private Limited
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::Serialize;
use std::{error::Error, fmt, io};

use crate::kafka::core::{
    MONKY_MAGIC_BYTE,
    serdes::hybrid_object_mapper::{HybridObjectMapper, TypeTagging},
};

/// Errors for serialization (no external crates).
#[derive(Debug)]
pub enum SerializationError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::Io(e) => write!(f, "io error during serialization: {}", e),
            SerializationError::Json(e) => write!(f, "json serialization error: {}", e),
        }
    }
}

impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SerializationError::Io(e) => Some(e),
            SerializationError::Json(e) => Some(e),
        }
    }
}

impl From<io::Error> for SerializationError {
    fn from(e: io::Error) -> Self {
        SerializationError::Io(e)
    }
}

impl From<serde_json::Error> for SerializationError {
    fn from(e: serde_json::Error) -> Self {
        SerializationError::Json(e)
    }
}

/// Stateless serializer. It prepends `MONKY_MAGIC_BYTE` and writes the JSON serialization of `data`.
#[derive(Debug, Default)]
pub struct KafkaSerializer {
    mapper: HybridObjectMapper,
}

impl KafkaSerializer {
    /// Create a default serializer using a default-configured `HybridObjectMapper`.
    pub fn new() -> Self {
        KafkaSerializer {
            mapper: HybridObjectMapper::new(),
        }
    }

    /// Create with a preconfigured mapper.
    pub fn with_mapper(mapper: HybridObjectMapper) -> Self {
        KafkaSerializer { mapper }
    }

    /// Serialize `data` into a `Vec<u8>` that begins with the magic byte.
    ///
    /// `topic` is accepted for API parity but currently unused.
    pub fn serialize<T: Serialize>(
        &self,
        _topic: &str,
        data: &T,
    ) -> Result<Vec<u8>, SerializationError> {
        // Fast path: no tagging, no null filtering, stream directly
        if self.mapper.type_tagging == TypeTagging::None && !self.mapper.omit_null_values {
            let mut out = Vec::with_capacity(1024);
            out.push(MONKY_MAGIC_BYTE);
            serde_json::to_writer(&mut out, data)?;
            return Ok(out);
        }

        // Adjacent type tagging: wrap data and stream value
        if self.mapper.type_tagging == TypeTagging::Adjacent {
            let payload_value = self.mapper.to_json_value(data)?;
            let mut map = serde_json::map::Map::with_capacity(2);
            map.insert(
                "@type".to_string(),
                serde_json::Value::String(std::any::type_name::<T>().to_string()),
            );
            map.insert("value".to_string(), payload_value);
            let wrapped = serde_json::Value::Object(map);

            let mut out = Vec::with_capacity(1024);
            out.push(MONKY_MAGIC_BYTE);
            serde_json::to_writer(&mut out, &wrapped)?;
            return Ok(out);
        }

        // Default: apply omit_null_values, then stream
        let value = self.mapper.to_json_value(data)?;
        let mut out = Vec::with_capacity(1024);
        out.push(MONKY_MAGIC_BYTE);
        serde_json::to_writer(&mut out, &value)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;
    use std::collections::HashMap;

    #[derive(Serialize)]
    struct Item {
        id: u32,
        name: String,
        optional: Option<String>,
    }

    #[test]
    fn serialize_basic() {
        let ser = KafkaSerializer::new();
        let item = Item {
            id: 1,
            name: "alice".to_string(),
            optional: None,
        };

        let bytes = ser.serialize("topic", &item).expect("serialize");
        assert_eq!(bytes[0], MONKY_MAGIC_BYTE);
        let json_part = &bytes[1..];
        let v: serde_json::Value = serde_json::from_slice(json_part).expect("parse json");
        assert_eq!(v["id"], 1);
        assert_eq!(v["name"], "alice");
        assert!(v.get("optional").is_none());
    }

    #[test]
    fn serialize_with_custom_mapper_adjacent_type_tag() {
        let mut mapper = HybridObjectMapper::new();
        mapper.type_tagging = TypeTagging::Adjacent;
        let ser = KafkaSerializer::with_mapper(mapper);

        let mut map = HashMap::new();
        map.insert("k", "v");

        let bytes = ser.serialize("t", &map).expect("serialize");
        assert_eq!(bytes[0], MONKY_MAGIC_BYTE);

        let v: serde_json::Value = serde_json::from_slice(&bytes[1..]).unwrap();
        assert!(v.get("@type").is_some());
        assert!(v.get("value").is_some());
    }

    #[test]
    fn serialize_array_removes_nulls() {
        use crate::kafka::core::serializer::avro_array::AvroGenericArray;
        let ser = KafkaSerializer::new();
        let arr = AvroGenericArray(vec![json!(1), json!(null), json!(2)]);
        let bytes = ser.serialize("t", &arr).expect("serialize");
        let v: serde_json::Value = serde_json::from_slice(&bytes[1..]).unwrap();
        assert_eq!(v, json!([1, 2]));
    }
}
