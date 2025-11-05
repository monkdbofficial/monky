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

use std::collections::HashMap;
use url::form_urlencoded;

/// How to handle duplicate keys in the input.
#[derive(Debug, Clone, Copy)]
pub enum DuplicateBehavior {
    /// Keep the first occurrence seen (like `Map::entry().or_insert(...)`).
    KeepFirst,
    /// Keep the last occurrence (overwrites earlier values).
    KeepLast,
    /// Collect all values into a comma-separated string (preserves order).
    /// Note: values themselves are not quoted; choose a different collector if commas are valid values.
    CollectCommaSeparated,
}

/// Parse `application/x-www-form-urlencoded` payload into a `HashMap<String, String>`.
///
/// - Decodes percent-escapes and converts `+` to space.
/// - Handles empty keys/values.
/// - Default duplicate behavior is `KeepLast`.
///
/// # Examples
///
/// ```
/// use monky_utilities::url_parse::{parse_url_encoded, DuplicateBehavior};
///
/// let payload = "name=alice&age=30&note=hello+world";
/// let map = parse_url_encoded(payload, DuplicateBehavior::KeepLast);
/// assert_eq!(map.get("name").map(|s| s.as_str()), Some("alice"));
/// assert_eq!(map.get("age").map(|s| s.as_str()), Some("30"));
/// assert_eq!(map.get("note").map(|s| s.as_str()), Some("hello world"));
/// ```
///
pub fn parse_url_encoded(
    payload: &str,
    dup_behavior: DuplicateBehavior,
) -> HashMap<String, String> {
    // form_urlencoded::parse returns an iterator of (Cow<str>, Cow<str>) already decoded.
    let iter = form_urlencoded::parse(payload.as_bytes());

    match dup_behavior {
        DuplicateBehavior::KeepLast => {
            // Insert/overwrite semantics: later occurrences replace earlier ones.
            let mut map = HashMap::with_capacity(8);
            for (k, v) in iter {
                map.insert(k.into_owned(), v.into_owned());
            }
            map
        }

        DuplicateBehavior::KeepFirst => {
            // Only insert if the key is not already present.
            let mut map = HashMap::with_capacity(8);
            for (k, v) in iter {
                map.entry(k.into_owned()).or_insert_with(|| v.into_owned());
            }
            map
        }

        DuplicateBehavior::CollectCommaSeparated => {
            // Append values, comma-separated. Reserve small capacity.
            let mut map: HashMap<String, String> = HashMap::with_capacity(8);
            for (k, v) in iter {
                let key = k.into_owned();
                let val = v.into_owned();
                map.entry(key)
                    .and_modify(|existing| {
                        existing.push(',');
                        existing.push_str(&val);
                    })
                    .or_insert(val);
            }
            map
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let payload = "name=alice&age=30&note=hello+world";
        let m = parse_url_encoded(payload, DuplicateBehavior::KeepLast);
        assert_eq!(m.get("name").map(|s| s.as_str()), Some("alice"));
        assert_eq!(m.get("age").map(|s| s.as_str()), Some("30"));
        assert_eq!(m.get("note").map(|s| s.as_str()), Some("hello world"));
    }

    #[test]
    fn empty_values_and_keys() {
        // key without value -> empty string; empty key allowed
        let payload = "k1=&=emptykey&justkey";
        // `justkey` will be treated as key with empty value by form_urlencoded
        let m = parse_url_encoded(payload, DuplicateBehavior::KeepLast);
        assert_eq!(m.get("k1").map(|s| s.as_str()), Some(""));
        assert_eq!(m.get("").map(|s| s.as_str()), Some("emptykey")); // from "=emptykey"
        assert_eq!(m.get("justkey").map(|s| s.as_str()), Some(""));
    }

    #[test]
    fn percent_decoding() {
        let payload = "q=Rust%20%26%20Web&plus=1%2B1";
        let m = parse_url_encoded(payload, DuplicateBehavior::KeepLast);
        assert_eq!(m.get("q").map(|s| s.as_str()), Some("Rust & Web"));
        assert_eq!(m.get("plus").map(|s| s.as_str()), Some("1+1"));
    }

    #[test]
    fn duplicates_keep_first_vs_last() {
        let payload = "k=a&k=b&k=c";
        let m_last = parse_url_encoded(payload, DuplicateBehavior::KeepLast);
        assert_eq!(m_last.get("k").map(|s| s.as_str()), Some("c"));

        let m_first = parse_url_encoded(payload, DuplicateBehavior::KeepFirst);
        assert_eq!(m_first.get("k").map(|s| s.as_str()), Some("a"));

        let m_collected = parse_url_encoded(payload, DuplicateBehavior::CollectCommaSeparated);
        assert_eq!(m_collected.get("k").map(|s| s.as_str()), Some("a,b,c"));
    }

    #[test]
    fn reserved_characters() {
        let payload = "k=%2Fpath%2Fto%2Ffile&space=one+two";
        let m = parse_url_encoded(payload, DuplicateBehavior::KeepLast);
        assert_eq!(m.get("k").map(|s| s.as_str()), Some("/path/to/file"));
        assert_eq!(m.get("space").map(|s| s.as_str()), Some("one two"));
    }
}
