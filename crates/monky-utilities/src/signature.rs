use hmac::{Hmac, Mac};
use sha2::Sha256;
use sha1::{Digest, Sha1};
use hex;

/// Constant header name used for passing content signature in requests or responses.
pub const CONTENT_SIGNATURE_HEADER: &str = "X-Monky-Content-Signature";

/// Type alias for HMAC using SHA256 hash function.
type HmacSha256 = Hmac<Sha256>;

/// Type alias for HMAC using SHA1 hash function.
type HmacSha1 = Hmac<Sha1>;

/// Computes the HMAC-SHA256 of `content` using the provided `key`.
///
/// # Arguments
///
/// * `key` - A string slice that holds the secret key used for HMAC computation.
/// * `content` - The message content to compute the HMAC over.
///
/// # Returns
///
/// On success, returns a lowercase hexadecimal string representing the HMAC-SHA256 signature.
/// Returns an `HmacError` if the key is invalid.
///
/// # Example
///
/// ```
/// let key = "my_secret_key";
/// let data = "Important message";
/// let signature = monky_utilities::signature::get_signature(key, data).unwrap();
/// println!("Signature: {}", signature);
/// ```
pub fn get_signature(key: &str, content: &str) -> Result<String, HmacError> {
    get_hmac_sha256_bytes(key.as_bytes(), content.as_bytes())
        .map(|bytes| hex::encode(bytes))
}

/// Computes the SHA-1 digest of the provided `content`.
///
/// # Arguments
///
/// * `content` - The message content to hash.
///
/// # Returns
///
/// A lowercase hexadecimal string representing the SHA-1 digest.
///
/// # Example
///
/// ```
/// let digest = monky_utilities::signature::get_sha1("test message");
/// println!("SHA-1 digest: {}", digest);
/// ```
pub fn get_sha1(content: &str) -> String {
    let digest = Sha1::digest(content.as_bytes());
    hex::encode(digest)
}

/// Computes the HMAC-SHA1 of `content` using the provided `key`.
///
/// # Arguments
///
/// * `key` - A string slice that holds the secret key used for HMAC computation.
/// * `content` - The message content to compute the HMAC over.
///
/// # Returns
///
/// On success, returns a lowercase hexadecimal string representing the HMAC-SHA1 signature.
/// Returns an `HmacError` if the key is invalid.
///
/// # Example
///
/// ```
/// let key = "my_key";
/// let data = "Sample data";
/// let hmac = monky_utilities::signature::get_hmac(key, data).unwrap();
/// println!("HMAC-SHA1: {}", hmac);
/// ```
pub fn get_hmac(key: &str, content: &str) -> Result<String, HmacError> {
    get_hmac_sha1_bytes(key.as_bytes(), content.as_bytes())
        .map(|bytes| hex::encode(bytes))
}


/// Internal helper function that computes the raw HMAC-SHA256 bytes.
/// 
/// # Arguments
/// 
/// * `key` - Secret key bytes.
/// * `content` - Message content bytes.
/// 
/// # Returns
/// 
/// A `Result` wrapping a vector of raw HMAC bytes, or an `HmacError` on invalid key.
fn get_hmac_sha256_bytes(key: &[u8], content: &[u8]) -> Result<Vec<u8>, HmacError> {
    // new_from_slice only fails if key length is unacceptable for the implementation.
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| HmacError::InvalidKey)?;
    mac.update(content);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(code_bytes.to_vec())
}

/// Internal helper function that computes the raw HMAC-SHA1 bytes.
/// 
/// # Arguments
/// 
/// * `key` - Secret key bytes.
/// * `content` - Message content bytes.
/// 
/// # Returns
/// 
/// A `Result` wrapping a vector of raw HMAC bytes, or an `HmacError` on invalid key.
fn get_hmac_sha1_bytes(key: &[u8], content: &[u8]) -> Result<Vec<u8>, HmacError> {
    let mut mac = HmacSha1::new_from_slice(key).map_err(|_| HmacError::InvalidKey)?;
    mac.update(content);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(code_bytes.to_vec())
}

/// Enum representing errors that can occur during HMAC operations.
#[derive(Debug)]
pub enum HmacError {
    /// The provided key was invalid for HMAC initialization.
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

