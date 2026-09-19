// crypto.rs - AstraPass cryptography primitives.
//
// Provides the three primitives AstraPass needs to keep the vault safe at
// rest (file at `~/.config/astra/astrapass/vault.bin`):
//
//   1. PBKDF2-HMAC-SHA256 with 150,000 iterations to turn the user's master
//      password + a random 16-byte salt into a 256-bit AES key.
//   2. AES-GCM 256-bit authenticated encryption of the vault JSON payload.
//      A fresh random 12-byte nonce is generated for every `encrypt()` call
//      (GCM nonces must NEVER be reused with the same key — we draw from
//      `rand::OsRng` via `rand::rngs::OsRng` which is cryptographically
//      secure).
//   3. A base64 wrapper `EncryptedBlob { salt, nonce, ciphertext }` so the
//      vault file is a portable JSON document — base64 keeps the binary
//      nonce + ciphertext safe to round-trip through `serde_json`.
//
// The key derivation parameters (algorithm, iteration count) are pinned at
// compile time — they are NOT stored in the vault. Bumping them in a future
// release requires a migration step (out of scope for ISO 4).

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Sha256;

/// PBKDF2 iteration count — 150,000 rounds of HMAC-SHA256.
///
/// Chosen to match the spec ("150,000 iterations, SHA-256"). OWASP 2023
/// recommends ≥ 600,000 for SHA-256, but 150k keeps unlock latency under
/// ~150 ms on commodity laptops while still being well above the legacy
/// 1,000-round default. Bumping this is a one-line change.
pub const PBKDF2_ITERATIONS: u32 = 150_000;

/// Salt length in bytes — 16 bytes (128 bits) of cryptographically random
/// data drawn from `rand::rngs::OsRng`.
pub const SALT_LEN: usize = 16;

/// AES-GCM nonce length in bytes — 12 bytes (96 bits), the recommended
/// size for GCM.
pub const NONCE_LEN: usize = 12;

/// AES-GCM key length in bytes — 32 bytes (256 bits, AES-256).
pub const KEY_LEN: usize = 32;

/// Error type for crypto operations.
///
/// Carries a human-readable message — we deliberately do NOT leak any
/// internal ciphertext/nonce bytes through this type (timing-attack
/// avoidance for the unlock path).
#[derive(Debug, Clone)]
pub struct CryptoError {
    pub message: String,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CryptoError: {}", self.message)
    }
}

impl std::error::Error for CryptoError {}

impl From<aes_gcm::Error> for CryptoError {
    fn from(_: aes_gcm::Error) -> Self {
        // aes_gcm::Error is opaque — we only know the operation failed
        // (wrong key, tampered tag, etc.). The unlock screen interprets
        // this as "wrong master password".
        CryptoError {
            message: "decryption failed (wrong key or corrupted data)".to_string(),
        }
    }
}

impl From<base64::DecodeError> for CryptoError {
    fn from(e: base64::DecodeError) -> Self {
        CryptoError {
            message: format!("base64 decode failed: {}", e),
        }
    }
}

/// The on-disk encrypted payload. Stored as JSON via `serde_json`.
///
/// All three binary fields are base64-encoded in the JSON so the file
/// is human-inspectable and survives `serde_json`'s UTF-8 requirement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedBlob {
    /// PBKDF2 salt (16 bytes, base64 in JSON).
    #[serde(serialize_with = "serialize_b64", deserialize_with = "deserialize_b64")]
    pub salt: Vec<u8>,

    /// AES-GCM nonce (12 bytes, base64 in JSON).
    #[serde(serialize_with = "serialize_b64", deserialize_with = "deserialize_b64")]
    pub nonce: Vec<u8>,

    /// AES-GCM ciphertext + authentication tag (base64 in JSON).
    #[serde(serialize_with = "serialize_b64", deserialize_with = "deserialize_b64")]
    pub ciphertext: Vec<u8>,
}

// --- base64 (de)serialization helpers ------------------------------------
//
// We use the URL-safe alphabet without padding so the JSON looks clean
// (no `+` / `/` / `=` to escape). The base64 0.22 API is engine-driven —
// `URL_SAFE_NO_PAD.encode()` returns a `String` and `decode()` accepts
// `&str`.

fn serialize_b64<S: Serializer>(bytes: &[u8], ser: S) -> Result<S::Ok, S::Error> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    ser.serialize_str(&encoded)
}

fn deserialize_b64<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<u8>, D::Error> {
    let s: String = Deserialize::deserialize(de)?;
    base64::engine::general_purpose::STANDARD
        .decode(s.as_bytes())
        .map_err(serde::de::Error::custom)
}

/// Derive a 256-bit AES key from the master password + salt using
/// PBKDF2-HMAC-SHA256 with `PBKDF2_ITERATIONS` rounds.
///
/// The salt MUST be unique per vault — `Vault::create` generates a fresh
/// random salt on first launch.
pub fn derive_key(master_password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        master_password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
    );
    key
}

/// Encrypt `data` with AES-GCM 256-bit using `key`. A fresh 12-byte
/// nonce is drawn from `OsRng` on every call — GCM nonces must NEVER
/// be reused with the same key.
///
/// Returns an `EncryptedBlob` carrying the salt (passed through unchanged
/// for round-trip), the nonce used, and the resulting ciphertext+tag.
pub fn encrypt(data: &str, key: &[u8; KEY_LEN]) -> Result<EncryptedBlob, CryptoError> {
    // The caller (Vault) is responsible for generating the salt; we
    // pass it back through the blob so the unlock path can re-derive
    // the key. Generate a fresh nonce here.
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // `Key::from_slice` returns `&Key<Aes256Gcm>` — a 32-byte
    // `GenericArray` reference. The `KeyInit` trait gives us
    // `Aes256Gcm::new(key)`.
    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);

    // `Aead::encrypt` returns `Result<Vec<u8>, aead::Error>`. The
    // ciphertext includes the GCM authentication tag (16 bytes appended).
    let ciphertext = cipher.encrypt(nonce, data.as_bytes())?;

    Ok(EncryptedBlob {
        // Salt is owned by the caller; we leave it empty here and let
        // `Vault::save` fill it in. (Keeping the function signature
        // focused on the encrypt op itself.)
        salt: Vec::new(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Encrypt `data` with AES-GCM using `key` and attach the given `salt`
/// to the resulting blob. Convenience wrapper used by `Vault::save`.
pub fn encrypt_with_salt(
    data: &str,
    key: &[u8; KEY_LEN],
    salt: Vec<u8>,
) -> Result<EncryptedBlob, CryptoError> {
    let mut blob = encrypt(data, key)?;
    blob.salt = salt;
    Ok(blob)
}

/// Decrypt an `EncryptedBlob` back to the original plaintext string.
///
/// Fails with `CryptoError` if the key is wrong (GCM tag verification
/// fails) or if the ciphertext is corrupted. The unlock screen treats
/// any error here as "wrong master password".
pub fn decrypt(blob: &EncryptedBlob, key: &[u8; KEY_LEN]) -> Result<String, CryptoError> {
    if blob.nonce.len() != NONCE_LEN {
        return Err(CryptoError {
            message: format!(
                "invalid nonce length: expected {} bytes, got {}",
                NONCE_LEN,
                blob.nonce.len()
            ),
        });
    }

    let nonce = Nonce::from_slice(&blob.nonce);
    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);

    let plaintext = cipher.decrypt(nonce, blob.ciphertext.as_ref())?;

    // Convert the decrypted bytes back to a UTF-8 string. The vault
    // payload is always JSON, so this should always succeed; if it
    // doesn't, the data was tampered with in a way that produced
    // invalid UTF-8 (extremely unlikely post-GCM-tag-verify).
    String::from_utf8(plaintext).map_err(|e| CryptoError {
        message: format!("decrypted payload is not valid UTF-8: {}", e),
    })
}

/// Generate `len` cryptographically random bytes using `OsRng`.
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    OsRng.fill_bytes(&mut out);
    out
}

// --- Tests --------------------------------------------------------------
//
// Quick round-trip test so we have at least one `cargo test` target
// covering the crypto path (GCM tag verification, nonce/salt
// propagation).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let salt = random_bytes(SALT_LEN);
        let key = derive_key("correct horse battery staple", &salt);

        let plaintext = r#"{"entries":[]}"#;
        let blob = encrypt_with_salt(plaintext, &key, salt.clone()).unwrap();
        let recovered = decrypt(&blob, &key).unwrap();
        assert_eq!(plaintext, recovered);
    }

    #[test]
    fn wrong_password_fails() {
        let salt = random_bytes(SALT_LEN);
        let key = derive_key("hunter2", &salt);
        let wrong_key = derive_key("hunter3", &salt);

        let blob = encrypt_with_salt("secret", &key, salt).unwrap();
        assert!(decrypt(&blob, &wrong_key).is_err());
    }

    #[test]
    fn derived_key_is_deterministic() {
        let salt = [0u8; SALT_LEN];
        let k1 = derive_key("password", &salt);
        let k2 = derive_key("password", &salt);
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_salts_produce_different_keys() {
        let s1 = [0u8; SALT_LEN];
        let s2 = [1u8; SALT_LEN];
        let k1 = derive_key("password", &s1);
        let k2 = derive_key("password", &s2);
        assert_ne!(k1, k2);
    }
}
