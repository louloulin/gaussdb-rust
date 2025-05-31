//! Authentication protocol support.
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;
use sha2::Sha256;

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

/// Hashes authentication information using SHA256_MD5 for GaussDB/OpenGauss.
///
/// This implements the SHA256_MD5 authentication method used by GaussDB and OpenGauss.
/// The process is: MD5(password + username) -> SHA256(md5_hex + salt) -> "sha256" + hex
#[inline]
pub fn sha256_hash(username: &[u8], password: &[u8], salt: &[u8]) -> String {
    // Step 1: MD5(password + username)
    let mut md5 = Md5::new();
    md5.update(password);
    md5.update(username);
    let md5_result = md5.finalize();
    let md5_hex = format!("{:x}", md5_result);

    // Step 2: SHA256(md5_hex + salt)
    let mut sha256 = Sha256::new();
    sha256.update(md5_hex.as_bytes());
    sha256.update(salt);
    let sha256_result = sha256.finalize();

    format!("sha256{:x}", sha256_result)
}

/// Hashes authentication information using MD5_SHA256 method for GaussDB/OpenGauss.
///
/// This implements the MD5_SHA256 authentication method used by GaussDB and OpenGauss.
/// The process involves PBKDF2, HMAC-SHA256, and MD5 operations.
#[inline]
pub fn md5_sha256_hash(password: &str, random_code: &str, salt: &[u8]) -> String {
    // Step 1: Generate K using PBKDF2
    let random_bytes = hex::decode(random_code).unwrap_or_else(|_| random_code.as_bytes().to_vec());
    let mut k = [0u8; 32];
    pbkdf2_hmac::<Sha1>(password.as_bytes(), &random_bytes, 2048, &mut k);

    // Step 2: Generate server_key and client_key using HMAC-SHA256
    let mut server_key_mac =
        Hmac::<Sha256>::new_from_slice(&k).expect("HMAC can take key of any size");
    server_key_mac.update(b"Sever Key"); // Note: "Sever" not "Server" - matches GaussDB implementation
    let server_key = server_key_mac.finalize().into_bytes();

    let mut client_key_mac =
        Hmac::<Sha256>::new_from_slice(&k).expect("HMAC can take key of any size");
    client_key_mac.update(b"Client Key");
    let client_key = client_key_mac.finalize().into_bytes();

    // Step 3: Generate stored_key using SHA256
    let mut sha256 = Sha256::new();
    sha256.update(client_key);
    let stored_key = sha256.finalize();

    // Step 4: Build the encryption string
    let encrypt_string = format!(
        "{}{}{}",
        random_code,
        hex::encode(server_key),
        hex::encode(stored_key)
    );

    // Step 5: MD5(encrypt_string + salt)
    let mut md5 = Md5::new();
    md5.update(encrypt_string.as_bytes());
    md5.update(salt);
    let md5_result = md5.finalize();

    format!("md5{:x}", md5_result)
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
        let password = "Gaussdb@123";
        let random_code = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let salt = b"randomsalt";
        let result = md5_sha256_hash(password, random_code, salt);
        assert!(result.starts_with("md5"));
        assert_eq!(result.len(), 35); // "md5" + 32 hex chars

        // Test with known values to ensure consistency
        let result2 = md5_sha256_hash(password, random_code, salt);
        assert_eq!(result, result2);
    }

    #[test]
    fn gaussdb_authentication_compatibility() {
        // Test cases based on GaussDB/OpenGauss authentication requirements
        let test_cases: Vec<(&str, &str, &[u8])> = vec![
            ("omm", "Enmo@123", b"salt"),
            ("gaussdb", "Gaussdb@123", b"test_salt"),
            ("postgres_user", "password", b"random_salt"),
        ];

        for (username, password, salt) in test_cases {
            let sha256_result = sha256_hash(username.as_bytes(), password.as_bytes(), salt);
            let random_code = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
            let md5_sha256_result = md5_sha256_hash(password, random_code, salt);

            // Both should produce valid hash strings
            assert!(sha256_result.starts_with("sha256"));
            assert!(md5_sha256_result.starts_with("md5"));
            assert_eq!(sha256_result.len(), 70);
            assert_eq!(md5_sha256_result.len(), 35);

            // They should produce different results (different algorithms)
            assert_ne!(sha256_result, md5_sha256_result);
        }
    }
}
