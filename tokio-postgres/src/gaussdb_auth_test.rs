//! GaussDB/OpenGauss authentication tests
//! 
//! This module contains tests for GaussDB-specific authentication methods:
//! - SHA256 authentication
//! - MD5_SHA256 authentication

#[cfg(test)]
mod tests {
    use postgres_protocol::authentication::{sha256_hash, md5_sha256_hash};

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
        let username = b"omm";
        let password = b"Gaussdb@123";
        let salt = b"random_salt_bytes";
        
        let result = md5_sha256_hash(username, password, salt);
        
        // Verify the result format
        assert!(result.starts_with("sha256"));
        assert_eq!(result.len(), 70); // "sha256" + 64 hex chars
        
        // Test consistency
        let result2 = md5_sha256_hash(username, password, salt);
        assert_eq!(result, result2);
    }

    #[test]
    fn test_gaussdb_authentication_differences() {
        // Test that SHA256 and MD5_SHA256 produce different results
        let username = b"testuser";
        let password = b"testpass";
        let salt = b"testsalt";
        
        let sha256_result = sha256_hash(username, password, salt);
        let md5_sha256_result = md5_sha256_hash(username, password, salt);
        
        // Both should be valid but different
        assert!(sha256_result.starts_with("sha256"));
        assert!(md5_sha256_result.starts_with("sha256"));
        assert_ne!(sha256_result, md5_sha256_result);
    }

    #[test]
    fn test_gaussdb_real_world_scenarios() {
        // Test with realistic GaussDB/OpenGauss scenarios
        let test_cases: Vec<(&[u8], &[u8], &str)> = vec![
            // (username, password, description)
            (b"omm", b"Gaussdb@123", "Default OpenGauss admin user"),
            (b"gaussdb", b"Gaussdb@123", "Custom GaussDB user"),
            (b"postgres_user", b"password", "PostgreSQL compatible user"),
            (b"test_user", b"Test@123", "Test user with complex password"),
        ];

        for (username, password, description) in test_cases {
            let salt = b"test_salt_for_scenario";
            
            let sha256_result = sha256_hash(username, password, salt);
            let md5_sha256_result = md5_sha256_hash(username, password, salt);
            
            // Verify both authentication methods work
            assert!(sha256_result.starts_with("sha256"), 
                "SHA256 failed for {}", description);
            assert!(md5_sha256_result.starts_with("sha256"), 
                "MD5_SHA256 failed for {}", description);
            assert_eq!(sha256_result.len(), 70, 
                "SHA256 length incorrect for {}", description);
            assert_eq!(md5_sha256_result.len(), 70, 
                "MD5_SHA256 length incorrect for {}", description);
        }
    }

    #[test]
    fn test_empty_and_edge_cases() {
        // Test edge cases
        let test_cases: Vec<(&[u8], &[u8], &[u8], &str)> = vec![
            (b"", b"", b"", "Empty values"),
            (b"a", b"b", b"c", "Single character values"),
            (b"very_long_username_that_exceeds_normal_limits",
             b"very_long_password_with_special_chars_!@#$%^&*()",
             b"very_long_salt_value_for_testing_edge_cases",
             "Long values"),
        ];

        for (username, password, salt, description) in test_cases {
            let sha256_result = sha256_hash(username, password, salt);
            let md5_sha256_result = md5_sha256_hash(username, password, salt);
            
            // Even edge cases should produce valid results
            assert!(sha256_result.starts_with("sha256"), 
                "SHA256 failed for {}", description);
            assert!(md5_sha256_result.starts_with("sha256"), 
                "MD5_SHA256 failed for {}", description);
        }
    }

    #[test]
    fn test_salt_sensitivity() {
        // Test that different salts produce different results
        let username = b"testuser";
        let password = b"testpass";
        let salt1 = b"salt1";
        let salt2 = b"salt2";
        
        let sha256_result1 = sha256_hash(username, password, salt1);
        let sha256_result2 = sha256_hash(username, password, salt2);
        let md5_sha256_result1 = md5_sha256_hash(username, password, salt1);
        let md5_sha256_result2 = md5_sha256_hash(username, password, salt2);
        
        // Different salts should produce different results
        assert_ne!(sha256_result1, sha256_result2, 
            "SHA256 should be salt-sensitive");
        assert_ne!(md5_sha256_result1, md5_sha256_result2, 
            "MD5_SHA256 should be salt-sensitive");
    }

    #[test]
    fn test_unicode_handling() {
        // Test with Unicode characters (common in international deployments)
        let username = "用户名".as_bytes(); // Chinese characters
        let password = "密码@123".as_bytes(); // Chinese + ASCII
        let salt = b"unicode_test_salt";
        
        let sha256_result = sha256_hash(username, password, salt);
        let md5_sha256_result = md5_sha256_hash(username, password, salt);
        
        // Should handle Unicode correctly
        assert!(sha256_result.starts_with("sha256"));
        assert!(md5_sha256_result.starts_with("sha256"));
        assert_eq!(sha256_result.len(), 70);
        assert_eq!(md5_sha256_result.len(), 70);
    }
}
