//! GaussDB/OpenGauss authentication tests
//!
//! This module contains tests for GaussDB-specific authentication methods:
//! - SHA256 authentication
//! - MD5_SHA256 authentication

#[cfg(test)]
mod tests {
    use postgres_protocol::authentication::{sha256_hash, md5_sha256_hash};

    const TEST_RANDOM_CODE: &str = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    #[test]
    fn test_sha256_authentication() {
        // Test SHA256 authentication with known values
        let username = b"gaussdb";
        let password = b"Gaussdb@123";
        let salt = b"test_salt_1234";

        let result = sha256_hash(username, password, salt);

        // Verify the result format
        assert!(result.starts_with("sha256"));
        assert_eq!(result.len(), 70); // "sha256" + 64 hex chars

        // Test consistency
        let result2 = sha256_hash(username, password, salt);
        assert_eq!(result, result2);
    }

    #[test]
    fn test_md5_sha256_authentication() {
        // Test MD5_SHA256 authentication with known values
        let password = "Gaussdb@123";
        let salt = b"random_salt_bytes";

        let result = md5_sha256_hash(password, TEST_RANDOM_CODE, salt);

        // Verify the result format
        assert!(result.starts_with("md5"));
        assert_eq!(result.len(), 35); // "md5" + 32 hex chars

        // Test consistency
        let result2 = md5_sha256_hash(password, TEST_RANDOM_CODE, salt);
        assert_eq!(result, result2);
    }

    #[test]
    fn test_gaussdb_authentication_differences() {
        // Test that SHA256 and MD5_SHA256 produce different results
        let username = b"testuser";
        let password = "testpass";
        let salt = b"testsalt";

        let sha256_result = sha256_hash(username, password.as_bytes(), salt);
        let md5_sha256_result = md5_sha256_hash(password, TEST_RANDOM_CODE, salt);

        // Both should be valid but different
        assert!(sha256_result.starts_with("sha256"));
        assert!(md5_sha256_result.starts_with("md5"));
        assert_ne!(sha256_result, md5_sha256_result);
    }

}
