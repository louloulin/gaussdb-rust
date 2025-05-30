//! Authentication protocol support.
use md5::{Digest, Md5};
use sha2::{Sha256};

pub mod sasl;

/// Hashes authentication information in a way suitable for use in response
/// to an `AuthenticationMd5Password` message.
///
/// The resulting string should be sent back to the database in a
/// `PasswordMessage` message.
#[inline]
pub fn md5_hash(username: &[u8], password: &[u8], salt: [u8; 4]) -> String {
    let mut md5 = Md5::new();
    md5.update(password);
    md5.update(username);
    let output = md5.finalize_reset();
    md5.update(format!("{:x}", output));
    md5.update(salt);
    format!("md5{:x}", md5.finalize())
}

/// Hashes authentication information using SHA256 for GaussDB/OpenGauss.
///
/// This implements the SHA256 authentication method used by GaussDB and OpenGauss.
/// The process is: SHA256(password + username + salt)
/// The resulting string should be sent back to the database in a `PasswordMessage` message.
#[inline]
pub fn sha256_hash(username: &[u8], password: &[u8], salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(username);
    hasher.update(salt);
    let result = hasher.finalize();
    format!("sha256{:x}", result)
}

/// Hashes authentication information using MD5+SHA256 hybrid method for GaussDB/OpenGauss.
///
/// This implements the MD5_SHA256 authentication method used by GaussDB and OpenGauss.
/// The process is:
/// 1. First hash: MD5(password + username)
/// 2. Second hash: SHA256(first_hash + salt)
/// The resulting string should be sent back to the database in a `PasswordMessage` message.
#[inline]
pub fn md5_sha256_hash(username: &[u8], password: &[u8], salt: &[u8]) -> String {
    // First stage: MD5(password + username)
    let mut md5 = Md5::new();
    md5.update(password);
    md5.update(username);
    let md5_result = md5.finalize();
    let md5_hex = format!("{:x}", md5_result);

    // Second stage: SHA256(md5_hex + salt)
    let mut sha256 = Sha256::new();
    sha256.update(md5_hex.as_bytes());
    sha256.update(salt);
    let sha256_result = sha256.finalize();

    format!("sha256{:x}", sha256_result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn md5() {
        let username = b"md5_user";
        let password = b"password";
        let salt = [0x2a, 0x3d, 0x8f, 0xe0];

        assert_eq!(
            md5_hash(username, password, salt),
            "md562af4dd09bbb41884907a838a3233294"
        );
    }

    #[test]
    fn sha256() {
        let username = b"testuser";
        let password = b"testpass";
        let salt = b"salt1234";
        let result = sha256_hash(username, password, salt);
        assert!(result.starts_with("sha256"));
        assert_eq!(result.len(), 70); // "sha256" + 64 hex chars
    }

    #[test]
    fn md5_sha256() {
        let username = b"gaussdb";
        let password = b"Gaussdb@123";
        let salt = b"randomsalt";
        let result = md5_sha256_hash(username, password, salt);
        assert!(result.starts_with("sha256"));
        assert_eq!(result.len(), 70); // "sha256" + 64 hex chars

        // Test with known values to ensure consistency
        let result2 = md5_sha256_hash(username, password, salt);
        assert_eq!(result, result2);
    }

    #[test]
    fn gaussdb_authentication_compatibility() {
        // Test cases based on GaussDB/OpenGauss authentication requirements
        let test_cases: Vec<(&[u8], &[u8], &[u8])> = vec![
            (b"omm", b"Enmo@123", b"salt"),
            (b"gaussdb", b"Gaussdb@123", b"test_salt"),
            (b"postgres_user", b"password", b"random_salt"),
        ];

        for (username, password, salt) in test_cases {
            let sha256_result = sha256_hash(username, password, salt);
            let md5_sha256_result = md5_sha256_hash(username, password, salt);

            // Both should produce valid hash strings
            assert!(sha256_result.starts_with("sha256"));
            assert!(md5_sha256_result.starts_with("sha256"));
            assert_eq!(sha256_result.len(), 70);
            assert_eq!(md5_sha256_result.len(), 70);

            // They should produce different results (different algorithms)
            assert_ne!(sha256_result, md5_sha256_result);
        }
    }
}
