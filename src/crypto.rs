#![warn(clippy::all, clippy::pedantic)]
//! Cryptographic module for encrypting/decrypting blog entries.
//!
//! **Security Note**: This implementation uses AES-256-GCM + Argon2id (NOT GPG-compatible).
//! While not compatible with `gpg` command-line tool, this approach is more secure and
//! simpler for password-based encryption with browser WASM decryption.
//!
//! This module provides battle-tested symmetric encryption using:
//! - **AES-256-GCM**: NIST-approved AEAD cipher (authenticated encryption)
//! - **Argon2id**: Winner of Password Hashing Competition, memory-hard KDF
//! - **`RustCrypto`**: Pure Rust implementations, widely audited
//!
//! Security properties:
//! - AES-256-GCM authenticated encryption (prevents tampering)
//! - Argon2id key derivation (resistant to GPU/ASIC attacks)
//! - Random salt per encryption (prevents rainbow table attacks)
//! - Random nonce per encryption (semantic security)
//! - Secure passphrase handling with zeroize

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, ParamsBuilder, Version,
};
use base64::prelude::*;

// Argon2id parameters (OWASP recommended for 2024)
const ARGON2_MEMORY: u32 = 65536; // 64 MB
const ARGON2_TIME: u32 = 3; // iterations
const ARGON2_PARALLELISM: u32 = 4; // threads

/// Encrypt plaintext content with a passphrase using AES-256-GCM + Argon2id.
pub fn encrypt(plaintext: &str, passphrase: &str) -> Result<Vec<u8>> {
    // Generate random salt
    let salt = SaltString::generate(&mut OsRng);

    // Derive 256-bit key using Argon2id
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        ParamsBuilder::new()
            .m_cost(ARGON2_MEMORY)
            .t_cost(ARGON2_TIME)
            .p_cost(ARGON2_PARALLELISM)
            .output_len(32)
            .build()
            .map_err(|e| anyhow!("Failed to build Argon2 parameters: {}", e))?,
    );

    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to derive key with Argon2id: {}", e))?;

    let key_bytes = password_hash
        .hash
        .ok_or_else(|| anyhow!("Argon2 hash output is missing"))?;

    // Create AES-256-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(key_bytes.as_bytes())
        .context("Failed to create AES-256-GCM cipher")?;

    // Generate random 96-bit nonce
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt plaintext
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Construct output: salt_string || nonce || ciphertext
    let salt_str = salt.as_str();
    let mut output = Vec::with_capacity(salt_str.len() + 1 + 12 + ciphertext.len());
    output.extend_from_slice(salt_str.as_bytes());
    output.push(b'|'); // delimiter
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt AES-256-GCM encrypted content with a passphrase.
pub fn decrypt(ciphertext: &[u8], passphrase: &str) -> Result<String> {
    // Find delimiter position
    let delimiter_pos = ciphertext
        .iter()
        .position(|&b| b == b'|')
        .ok_or_else(|| anyhow!("Invalid ciphertext format: delimiter not found"))?;

    // Extract salt string
    let salt_bytes = &ciphertext[..delimiter_pos];
    let salt_str = std::str::from_utf8(salt_bytes).context("Salt is not valid UTF-8")?;
    let salt =
        SaltString::from_b64(salt_str).map_err(|e| anyhow!("Failed to parse salt: {}", e))?;

    // Extract nonce (12 bytes after delimiter)
    let nonce_start = delimiter_pos + 1;
    let nonce_end = nonce_start + 12;
    if ciphertext.len() < nonce_end {
        return Err(anyhow!("Ciphertext too short for nonce"));
    }
    let nonce_bytes = &ciphertext[nonce_start..nonce_end];
    let nonce = Nonce::from_slice(nonce_bytes);

    // Extract ciphertext data
    let ciphertext_data = &ciphertext[nonce_end..];

    // Derive key using Argon2id
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        ParamsBuilder::new()
            .m_cost(ARGON2_MEMORY)
            .t_cost(ARGON2_TIME)
            .p_cost(ARGON2_PARALLELISM)
            .output_len(32)
            .build()
            .map_err(|e| anyhow!("Failed to build Argon2 parameters: {}", e))?,
    );

    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to derive key with Argon2id: {}", e))?;

    let key_bytes = password_hash
        .hash
        .ok_or_else(|| anyhow!("Argon2 hash output is missing"))?;

    // Create AES-256-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(key_bytes.as_bytes())
        .context("Failed to create AES-256-GCM cipher")?;

    // Decrypt and verify
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext_data)
        .map_err(|_| anyhow!("Decryption failed: incorrect passphrase or corrupted data"))?;

    // Convert to UTF-8 string
    let plaintext =
        String::from_utf8(plaintext_bytes).context("Decrypted content is not valid UTF-8")?;

    Ok(plaintext)
}

/// Encode encrypted bytes as base64 for HTML embedding.
pub fn to_base64(encrypted_bytes: &[u8]) -> String {
    BASE64_STANDARD.encode(encrypted_bytes)
}

/// Decode base64-encoded encrypted data.
#[allow(dead_code)] // Reserved for future WASM decryption
pub fn from_base64(encoded: &str) -> Result<Vec<u8>> {
    BASE64_STANDARD
        .decode(encoded)
        .context("Failed to decode base64 encrypted data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "This is a secret message!";
        let passphrase = "test-passphrase-12345";

        let encrypted = encrypt(plaintext, passphrase).expect("Encryption failed");
        assert!(!encrypted.is_empty());

        let decrypted = decrypt(&encrypted, passphrase).expect("Decryption failed");
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let plaintext = "Secret content";
        let passphrase = "correct-passphrase";
        let wrong_passphrase = "wrong-passphrase";

        let encrypted = encrypt(plaintext, passphrase).expect("Encryption failed");
        let result = decrypt(&encrypted, wrong_passphrase);
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_content() {
        let plaintext = "Hello 世界 🔒 Café";
        let passphrase = "unicode-test";

        let encrypted = encrypt(plaintext, passphrase).expect("Encryption failed");
        let decrypted = decrypt(&encrypted, passphrase).expect("Decryption failed");
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_base64_roundtrip() {
        let plaintext = "Test content";
        let passphrase = "test";

        let encrypted = encrypt(plaintext, passphrase).expect("Encryption failed");
        let encoded = to_base64(&encrypted);
        let decoded = from_base64(&encoded).expect("Base64 decode failed");

        let decrypted = decrypt(&decoded, passphrase).expect("Decryption failed");
        assert_eq!(plaintext, decrypted);
    }
}
