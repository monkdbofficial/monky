use hmac::{Hmac, Mac};
use sha2::Sha256;
use sha1::{Digest, Sha1};
use hex;

pub const CONTENT_SIGNATURE_HEADER: &str = "X-Monky-Content-Signature";

type HmacSha256 = Hmac<Sha256>;
type HmacSha1 = Hmac<Sha1>;

/// Compute HMAC-SHA256 of `content` using `key`.
/// Returns lowercase hex string.
pub fn get_signature(key: &str, content: &str) -> Result<String, HmacError> {
    get_hmac_sha256_bytes(key.as_bytes(), content.as_bytes())
        .map(|bytes| hex::encode(bytes))
}

/// Compute SHA-1 digest of `content`.
/// Returns lowercase hex string.
pub fn get_sha1(content: &str) -> String {
    let digest = Sha1::digest(content.as_bytes());
    hex::encode(digest)
}

/// Compute HMAC-SHA1 of `content` using `key`.
/// Returns lowercase hex string.
pub fn get_hmac(key: &str, content: &str) -> Result<String, HmacError> {
    get_hmac_sha1_bytes(key.as_bytes(), content.as_bytes())
        .map(|bytes| hex::encode(bytes))
}

/// Internal helper: compute HMAC-SHA256 and return raw bytes.
fn get_hmac_sha256_bytes(key: &[u8], content: &[u8]) -> Result<Vec<u8>, HmacError> {
    // new_from_slice only fails if key length is unacceptable for the implementation.
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| HmacError::InvalidKey)?;
    mac.update(content);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(code_bytes.to_vec())
}

/// Internal helper: compute HMAC-SHA1 and return raw bytes.
fn get_hmac_sha1_bytes(key: &[u8], content: &[u8]) -> Result<Vec<u8>, HmacError> {
    let mut mac = HmacSha1::new_from_slice(key).map_err(|_| HmacError::InvalidKey)?;
    mac.update(content);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(code_bytes.to_vec())
}

/// Simple error type for HMAC operations.
#[derive(Debug)]
pub enum HmacError {
    InvalidKey,
}

impl std::fmt::Display for HmacError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HmacError::InvalidKey => write!(f, "invalid HMAC key"),
        }
    }
}

impl std::error::Error for HmacError {}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use sha1::Sha1;

    fn compute_expected_hmac_sha256(key: &[u8], content: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("Invalid key");
        mac.update(content);
        hex::encode(mac.finalize().into_bytes())
    }

    fn compute_expected_hmac_sha1(key: &[u8], content: &[u8]) -> String {
        let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("Invalid key");
        mac.update(content);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn test_get_signature_valid() {
        let key = "secretkey";
        let content = "Hello, world!";
        let expected = compute_expected_hmac_sha256(key.as_bytes(), content.as_bytes());
        let signature = get_signature(key, content).expect("Failed to get signature");
        assert_eq!(signature, expected);
    }

    #[test]
    fn test_get_signature_invalid_key() {
        // The hmac crate accepts any key length, so this may not error.
        // Adjust test or implementation if you want stricter checks.
        let key = "";
        let content = "data";
        let result = get_signature(key, content);
        // Instead of expecting error, test result is Ok with some output
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_sha1() {
        let content = "hello";
        let sha1_hash = get_sha1(content);
        let expected = "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d";
        assert_eq!(sha1_hash, expected);
    }

    #[test]
    fn test_get_hmac_valid() {
        let key = "mykey";
        let content = "some content";
        let expected = compute_expected_hmac_sha1(key.as_bytes(), content.as_bytes());
        let hmac = get_hmac(key, content).expect("Failed to get HMAC");
        assert_eq!(hmac, expected);
    }

    #[test]
    fn test_get_hmac_invalid_key() {
        // Similar to SHA256, empty key is valid but trivial; adjust as needed.
        let key = "";
        let content = "sample";
        let result = get_hmac(key, content);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}

