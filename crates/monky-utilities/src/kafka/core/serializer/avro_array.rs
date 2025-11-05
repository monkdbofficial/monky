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

use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

/// A small wrapper type representing a generic Avro-style array of JSON values.
///
/// It serializes each element of the contained vector into the output array.
#[derive(Debug, Clone, Default)]
pub struct AvroGenericArray(pub Vec<Value>);

impl From<Vec<Value>> for AvroGenericArray {
    fn from(v: Vec<Value>) -> Self {
        AvroGenericArray(v)
    }
}

impl Serialize for AvroGenericArray {
    /// Serialize the wrapper as a JSON array and stream elements to the serializer.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Reserve the exact size if known to avoid reallocation in serializer internals.
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for v in &self.0 {
            seq.serialize_element(v)?;
        }
        seq.end()
    }
}

impl AvroGenericArray {
    /// Serialize the array with a type tag wrapper.
    ///
    /// The produced JSON looks like:
    /// {
    ///   "@type": "<type_name>",
    ///   "value": [ ... array elements ... ]
    /// }
    pub fn serialize_with_type<S>(&self, serializer: S, type_name: &str) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // map with 2 entries: @type and value
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("@type", type_name)?;
        // we can serialize the Vec<Value> directly as the "value" entry:
        map.serialize_entry("value", &self.0)?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialize_array() {
        let arr = AvroGenericArray(vec![json!(1), json!("two"), json!({ "k": "v" })]);
        let s = serde_json::to_string(&arr).unwrap();
        assert_eq!(s, r#"[1,"two",{"k":"v"}]"#);
    }

    #[test]
    fn serialize_with_type_wrapper() {
        let arr = AvroGenericArray(vec![json!(true), json!(null)]);
        // serialize_with_type into serde_json::Value by using serde_json::to_value + a tiny helper
        let wrapped = {
            // Use a small adapter to call `serialize_with_type` producing a serde_json::Value.
            struct Helper<'a>(&'a AvroGenericArray, &'a str);
            impl<'a> Serialize for Helper<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    self.0.serialize_with_type(serializer, self.1)
                }
            }

            serde_json::to_value(Helper(&arr, "com.example.Type")).unwrap()
        };

        // Expected JSON: {"@type":"com.example.Type","value":[true,null]}
        assert_eq!(
            wrapped,
            json!({
                "@type": "com.example.Type",
                "value": [true, null]
            })
        );
    }

    #[test]
    fn empty_array_serializes_to_empty_array() {
        let arr = AvroGenericArray::default();
        let s = serde_json::to_string(&arr).unwrap();
        assert_eq!(s, "[]");
    }
}
