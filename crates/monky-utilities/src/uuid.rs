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

//! UUID v5 utilities — RFC-4122 namespace-based UUID v5 helpers and utilities.
//!
//! This module provides:
//! - `from_bytes([u8; 16]) -> Uuid` — set version/variant and construct a UUID.
//! - `from_name(name: &str) -> Uuid` — SHA-1(name) -> first 16 bytes -> UUID v5 (name-only).
//! - `from_namespace_and_name(namespace: &Uuid, name: &str) -> Uuid` — RFC-4122 correct.
//! - `from_reader<R: Read>(reader: &mut R) -> Result<Uuid, io::Error>` — stream SHA-1 over reader.

use sha1::Digest;
use sha1::Sha1;
use std::io::{self, Read};
use uuid::Uuid;

/// Set the version (5) and variant (RFC 4122 / IETF) bits on a 16-byte array and construct a `Uuid`.
pub fn uuid_from_bytes(mut bytes: [u8; 16]) -> Uuid {
    // Clear version nibble and set to 5 (0101)
    bytes[6] = (bytes[6] & 0x0f) | (5u8 << 4); // 0x50

    // Set the variant to RFC 4122 (10xx_xxxx)
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Uuid::from_bytes(bytes)
}

/// Compute UUID v5 from `name` bytes (NOT namespace-aware).
/// It computes SHA-1(name) and uses the first 16 bytes to build the UUID v5.
///
/// Note: RFC-4122 specifies v5 as SHA1(namespace || name). If you want the RFC behavior,
/// use `from_namespace_and_name(namespace, name)`.
pub fn uuid_from_name(name: &str) -> Uuid {
    let mut hasher = Sha1::new();
    hasher.update(name.as_bytes());
    let full = hasher.finalize(); // 20 bytes
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&full[..16]);
    uuid_from_bytes(bytes)
}

/// Compute UUID v5 according to RFC-4122: SHA-1(namespace_bytes || name_bytes).
///
/// `namespace` is a UUID (for example, `uuid::Uuid::NAMESPACE_DNS`), `name` is the name string.
/// Returns a UUID v5.
pub fn uuid_from_namespace_and_name(namespace: &Uuid, name: &str) -> Uuid {
    let mut hasher = Sha1::new();

    // namespace as 16 bytes in network (big-endian) order
    hasher.update(namespace.as_bytes());
    hasher.update(name.as_bytes());

    let full = hasher.finalize(); // 20 bytes
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&full[..16]);
    uuid_from_bytes(bytes)
}

/// Compute SHA-1 over a reader (streaming) and return uuid v5 from resulting digest.
///
/// The reader is consumed (read until EOF). If you need to reuse the stream, caller must
/// provide a seekable/resettable reader and handle rewinding.
pub fn uuid_from_reader<R: Read>(reader: &mut R) -> io::Result<Uuid> {
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8 * 1024];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let full = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&full[..16]);
    Ok(uuid_from_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use uuid::Uuid;

    #[test]
    fn test_uuid_from_bytes_sets_version_and_variant() {
        let bytes = [0xffu8; 16];
        let uuid = uuid_from_bytes(bytes);

        // version should be 5
        assert_eq!(uuid.get_version_num(), 5);

        // variant should be RFC 4122 (i.e., variant = DCE 1.1)
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn test_uuid_from_name_non_namespace_consistent() {
        let name = "test name";
        let uuid1 = uuid_from_name(name);
        let uuid2 = uuid_from_name(name);
        assert_eq!(uuid1, uuid2);

        assert_eq!(uuid1.get_version_num(), 5);
        assert_eq!(uuid1.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn test_uuid_from_namespace_and_name_matches_rfc() {
        let namespace = Uuid::NAMESPACE_DNS;
        let name = "example.com";
        let uuid = uuid_from_namespace_and_name(&namespace, name);

        // The result should be the same as the standard uuid crate's v5 generation
        let expected = Uuid::new_v5(&namespace, name.as_bytes());

        assert_eq!(uuid, expected);
        assert_eq!(uuid.get_version_num(), 5);
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn test_uuid_from_reader_correctness() {
        let data = b"some test input for sha1";
        let mut cursor = Cursor::new(data);
        let uuid = uuid_from_reader(&mut cursor).expect("Failed to create UUID from reader");

        // The function must produce a RFC 4122 variant and version 5 UUID
        assert_eq!(uuid.get_version_num(), 5);
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);

        // Calling again on the same data should produce the same UUID
        let mut cursor2 = Cursor::new(data);
        let uuid2 =
            uuid_from_reader(&mut cursor2).expect("Failed to create UUID from second reader");
        assert_eq!(uuid, uuid2);
    }
}
