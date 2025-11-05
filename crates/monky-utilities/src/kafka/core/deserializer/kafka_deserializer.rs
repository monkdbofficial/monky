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

use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::str;

use crate::kafka::core::serdes::hybrid_object_mapper::HybridObjectMapper;

/// Error type analogous to Kafka's `SerializationException` (no external crates).
#[derive(Debug)]
pub enum SerializationError {
    /// The incoming payload is too short to skip the header byte.
    PayloadTooShort,
    /// Payload is not valid UTF-8.
    InvalidUtf8(std::str::Utf8Error),
    /// JSON (de)serialization error from serde_json.
    Json(serde_json::Error),
}


impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::PayloadTooShort => {
                write!(f, "payload too short (need at least 1 byte to skip header)")
            }
            SerializationError::InvalidUtf8(e) => write!(f, "invalid utf-8 payload: {}", e),
            SerializationError::Json(e) => write!(f, "json deserialization error: {}", e),
        }
    }
}


impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SerializationError::PayloadTooShort => None,
            SerializationError::InvalidUtf8(e) => Some(e),
            SerializationError::Json(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for SerializationError {
    fn from(e: serde_json::Error) -> Self {
        SerializationError::Json(e)
    }
}


impl From<std::str::Utf8Error> for SerializationError {
    fn from(e: std::str::Utf8Error) -> Self {
        SerializationError::InvalidUtf8(e)
    }
}

/// Stateless deserializer. It uses a `HybridObjectMapper` instance to parse the JSON body after skipping
/// the first byte of the payload.
#[derive(Debug, Default)]
pub struct KafkaDeserializer {
    mapper: HybridObjectMapper,
}

impl KafkaDeserializer {
    /// Create a new instance. Uses a default-configured `HybridObjectMapper`.
    pub fn new() -> Self {
        KafkaDeserializer {
            mapper: HybridObjectMapper::new(),
        }
    }

    /// Construct with a preconfigured HybridObjectMapper.
    pub fn with_mapper(mapper: HybridObjectMapper) -> Self {
        KafkaDeserializer { mapper }
    }

    /// Deserialize the given Kafka payload bytes into `serde_json::Value`.
    ///
    /// `topic` parameter kept for API parity but unused.
    ///
    /// Returns `SerializationError` on any problem (short payload, invalid utf-8,
    /// or JSON parse issues).
    pub fn deserialize(&self, _topic: &str, bytes: &[u8]) -> Result<Value, SerializationError> {
        if bytes.len() < 1 {
            return Err(SerializationError::PayloadTooShort);
        }

        // Skip first byte without copying
        let payload = &bytes[1..];

        // Convert to &str (serde_json::from_str/HybridObjectMapper methods operate on str)
        let s = str::from_utf8(payload)?;

        // Try to use mapper.deserialize_with_type to support type-wrapped payloads
        // Fallback to plain deserialize if needed.
        //
        // hybrid mapper's deserialize_with_type returns Result<T, serde_json::Error>.
        match self.mapper.deserialize_with_type::<Value>(s) {
            Ok(v) => Ok(v),
            Err(e) => Err(SerializationError::Json(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_valid_payload_without_type_wrapper() {
        let des = KafkaDeserializer::new();
        let payload = b"\x00{\"k\":\"v\"}";
        let v = des.deserialize("topic", payload).expect("should parse");
        assert_eq!(v, json!({"k":"v"}));
    }

    #[test]
    fn deserialize_valid_payload_with_type_wrapper() {
        // Build a mapper configured to accept type-wrapped values (adjacent style)
        let mut mapper = HybridObjectMapper::new();
        mapper.type_tagging = crate::kafka::core::serdes::hybrid_object_mapper::TypeTagging::Adjacent;
        let des = KafkaDeserializer::with_mapper(mapper);

        // Prepare adjacent-wrapped JSON: {"@type":"com.example","value":{"k":"v"}}
        let _wrapped = br#"\x00{\"@type\":\"com.example\",\"value\":{\"k\":\"v\"}}"#;
        // Note: the raw bytes above include \x00 as two characters; construct properly:
        let mut buf = vec![0u8];
        buf.extend_from_slice(b"{\"@type\":\"com.example\",\"value\":{\"k\":\"v\"}}");

        let v = des.deserialize("topic", &buf).expect("should parse wrapped");
        assert_eq!(v, json!({"k":"v"}));
    }

    #[test]
    fn deserialize_empty_payload() {
        let des = KafkaDeserializer::new();
        let err = des.deserialize("topic", &[]).unwrap_err();
        matches!(err, SerializationError::PayloadTooShort);
    }

    #[test]
    fn deserialize_invalid_utf8() {
        let des = KafkaDeserializer::new();
        // invalid UTF-8 after skipping first byte
        let payload = &[0u8, 0xff, 0xff, 0xff];
        let err = des.deserialize("t", payload).unwrap_err();
        match err {
            SerializationError::InvalidUtf8(_) => {}
            _ => panic!("expected InvalidUtf8"),
        }
    }

    #[test]
    fn deserialize_invalid_json() {
        let des = KafkaDeserializer::new();
        let mut buf = vec![0u8];
        buf.extend_from_slice(b"{not:json}");
        let err = des.deserialize("t", &buf).unwrap_err();
        match err {
            SerializationError::Json(_) => {}
            _ => panic!("expected Json error"),
        }
    }
}