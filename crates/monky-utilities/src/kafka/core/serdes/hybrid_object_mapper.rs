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

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use std::collections::HashSet;
use std::fmt;

/// Control how type metadata is emitted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeTagging {
    /// Do not emit type metadata.
    None,
    /// Emit adjacent style: { "@type": "<type name>", "value": <payload> }
    /// (Useful to mimic Jackson's default-typing behavior for consumers that
    /// expect a type tag.)
    Adjacent,
}

/// A small wrapper to hold configuration similar to the Java ObjectMapper.
#[derive(Debug, Clone)]
pub struct HybridObjectMapper {
    /// Whether to include type metadata and its tagging style.
    pub type_tagging: TypeTagging,
    /// If true, serialization will remove any `null` fields from JSON objects and maps.
    pub omit_null_values: bool,
    /// Optional set of type names to ignore (for dynamic JSON payloads) - those entries
    /// are dropped or replaced with `null` depending on context.
    pub ignore_type_names: HashSet<String>,
}

impl Default for HybridObjectMapper {
    fn default() -> Self {
        HybridObjectMapper {
            type_tagging: TypeTagging::None,
            omit_null_values: true,
            ignore_type_names: HashSet::new(),
        }
    }
}

impl HybridObjectMapper {
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a type name to ignore during dynamic-value serialization.
    pub fn add_ignored_type<S: Into<String>>(&mut self, name: S) {
        self.ignore_type_names.insert(name.into());
    }

    /// Serialize a serde `value` into a JSON string honoring mapper settings.
    ///
    /// - If `omit_null_values` is true, drops `null` entries from objects/maps recursively.
    /// - If `type_tagging` is `Adjacent`, wraps the payload in `{"@type": "<typename>", "value": ...}`.
    ///
    /// `type_name` is optional and used only when `type_tagging != None`. If not provided,
    /// we attempt to infer a name from the Rust type using `std::any::type_name`, which
    /// is not stable across builds but often informative for debugging.
    pub fn serialize<T: Serialize>(
        &self,
        value: &T,
        type_name: Option<&str>,
    ) -> Result<String, serde_json::Error> {
        // First convert the value into a serde_json::Value so we can mutate/filter it
        let mut v = serde_json::to_value(value)?;

        if self.omit_null_values {
            v = remove_nulls(v);
        }

        if let TypeTagging::Adjacent = self.type_tagging {
            // Determine a type name
            let tn = type_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| infer_type_name::<T>());

            // adjacent-style wrapper
            let mut map = Map::with_capacity(2);
            map.insert("@type".to_string(), Value::String(tn));
            map.insert("value".to_string(), v);
            let wrapped = Value::Object(map);
            serde_json::to_string(&wrapped)
        } else {
            serde_json::to_string(&v)
        }
    }

    /// Convenience: serialize a value but return serde_json::Value instead of String.
    pub fn to_json_value<T: Serialize>(&self, value: &T) -> Result<Value, serde_json::Error> {
        let mut v = serde_json::to_value(value)?;
        if self.omit_null_values {
            v = remove_nulls(v);
        }
        Ok(v)
    }

    /// Deserialize JSON text into a concrete type T.
    pub fn deserialize<T: DeserializeOwned>(&self, s: &str) -> Result<T, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Like `deserialize`, but first strips an adjacent type wrapper if present.
    /// If input is `{"@type":"foo","value": ... }` this returns the deserialized `T`
    /// from the `value` field. Otherwise it tries to deserialize the whole payload.
    pub fn deserialize_with_type<T: DeserializeOwned>(&self, s: &str) -> Result<T, serde_json::Error> {
        let v: Value = serde_json::from_str(s)?;
        if let Value::Object(mut m) = v {
            if let Some(Value::String(_tn)) = m.get("@type") {
                if let Some(val) = m.remove("value") {
                    return serde_json::from_value(val);
                }
            }
            // not a wrapped object or missing value => fall through
            let as_value = Value::Object(m);
            serde_json::from_value(as_value)
        } else {
            serde_json::from_value(v)
        }
    }

    /// Helper: take an already-built serde_json::Value and apply ignore-type and omit-null filters.
    /// This is useful when working with dynamic payloads where certain type names should be ignored.
    pub fn filter_dynamic_value(&self, mut v: Value) -> Value {
        // If it's an object, remove any entries whose keys match `ignore_type_names`
        if let Value::Object(ref mut map) = v {
            map.retain(|k, _| !self.ignore_type_names.contains(k));
            // recursively clean nested structures
            for (_k, val) in map.iter_mut() {
                let cleaned = remove_nulls(val.take());
                *val = cleaned;
            }
            Value::Object(map.clone())
        } else {
            // For arrays or scalars, just remove nulls if configured
            if self.omit_null_values {
                remove_nulls(v)
            } else {
                v
            }
        }
    }
}

/// Remove all `null` entries from objects and arrays recursively.
/// - For `Object`: removes keys with Value::Null and cleans nested values.
/// - For `Array`: removes elements equal to Value::Null and cleans nested elements.
fn remove_nulls(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = Map::with_capacity(map.len());
            for (k, val) in map {
                let cleaned = remove_nulls(val);
                if !cleaned.is_null() {
                    out.insert(k, cleaned);
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for val in arr {
                let cleaned = remove_nulls(val);
                if !cleaned.is_null() {
                    out.push(cleaned);
                }
            }
            Value::Array(out)
        }
        other => other,
    }
}

/// A tiny helper to provide a best-effort type name from Rust type `T`.
/// Note: `std::any::type_name::<T>()` is not stable across versions/optimizations
/// (it includes module paths) but is useful for debugging or interop when you
/// control both producer and consumer.
fn infer_type_name<T>() -> String {
    std::any::type_name::<T>().to_string()
}

impl fmt::Display for HybridObjectMapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!(
            "HybridObjectMapper(type_tagging={:?}, omit_nulls={}, ignore_count={})",
            self.type_tagging,
            self.omit_null_values,
            self.ignore_type_names.len()
        );
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Sample {
        a: String,
        b: Option<String>,
        c: Option<i32>,
    }

    #[test]
    fn serialize_omit_nulls() {
        let mapper = HybridObjectMapper {
            type_tagging: TypeTagging::None,
            omit_null_values: true,
            ignore_type_names: HashSet::new(),
        };
        let s = Sample {
            a: "x".into(),
            b: None,
            c: Some(3),
        };
        let json = mapper.serialize(&s, None).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("b").is_none());
        assert_eq!(v.get("a").unwrap(), "x");
    }

    #[test]
    fn serialize_with_type_tagging() {
        let mapper = HybridObjectMapper {
            type_tagging: TypeTagging::Adjacent,
            omit_null_values: true,
            ignore_type_names: HashSet::new(),
        };
        let s = Sample {
            a: "y".into(),
            b: None,
            c: None,
        };
        let json = mapper.serialize(&s, Some("com.example.Sample")).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v.get("@type").unwrap(), "com.example.Sample");
        // value should be present and not contain nulls
        let val = &v["value"];
        assert!(val.get("b").is_none());
    }

    #[test]
    fn avro_generic_array_serialize_works() {
        // Example: reuse AvroGenericArray from earlier
        let arr = crate::kafka::core::serializer::avro_array::AvroGenericArray(vec![json!(1), json!("two"), json!(null)]);
        let mapper = HybridObjectMapper::new();
        // Ensure omit_null_values will remove the null element inside nested arrays too
        let value = mapper.to_json_value(&arr).unwrap();
        // Because AvroGenericArray serializes as an array, and we run remove_nulls,
        // the resulting Value should be an array without 'null' elements.
        assert_eq!(
            value,
            Value::Array(vec![json!(1), json!("two")])
        );
    }

    #[test]
    fn filter_dynamic_value_ignores_types() {
        let mut mapper = HybridObjectMapper::new();
        mapper.add_ignored_type("org.apache.avro.Schema");
        let mut map = Map::new();
        map.insert("org.apache.avro.Schema".to_string(), json!({"foo":"bar"}));
        map.insert("keep".to_string(), json!("x"));
        let v = Value::Object(map);
        let filtered = mapper.filter_dynamic_value(v);
        assert!(filtered.get("org.apache.avro.Schema").is_none());
        assert_eq!(filtered.get("keep").unwrap(), "x");
    }
}
