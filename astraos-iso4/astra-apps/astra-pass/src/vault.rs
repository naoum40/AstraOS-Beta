// vault.rs - The AstraPass vault model + persistence layer.
//
// The vault is the in-memory representation of the user's password
// store. At rest it lives at `~/.config/astra/astrapass/vault.bin` as a
// JSON-serialized `EncryptedBlob` (salt + nonce + AES-GCM ciphertext).
// Inside the ciphertext is a JSON-serialized `VaultData` (the `entries`
// vector + a schema version for forward-compatibility).
//
// Lifecycle:
//
//   first launch  ─► Vault::create(master_password) → save() writes
//                    a fresh empty vault file.
//   later launch  ─► Vault::load(path) returns Option<EncryptedBlob>
//                    → Vault::unlock(master_password, blob) decrypts
//                    and parses the entries.
//
// All file I/O is checked through `std::io::Error`; the caller maps
// those into UI states (error banner, etc.).

use crate::crypto::{self, EncryptedBlob, KEY_LEN, SALT_LEN};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// On-disk vault schema version. Bump when adding fields to `VaultData`
/// — old vaults with a lower `version` will be migrated forward.
pub const VAULT_VERSION: u32 = 1;

/// A single password entry.
///
/// `id` is a 16-byte random hex string generated at creation time so we
/// can address entries by stable identifier (reordering the list, etc.)
/// without depending on array index.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub favorite: bool,
    pub created_at: i64,
}

impl Entry {
    /// Build a new empty entry with a fresh random `id` and the given
    /// `created_at` timestamp (Unix seconds).
    pub fn new(created_at: i64) -> Self {
        Self {
            id: new_entry_id(),
            title: String::new(),
            username: String::new(),
            password: String::new(),
            notes: String::new(),
            favorite: false,
            created_at,
        }
    }
}

/// Generate a 16-byte random hex string for use as an entry id.
fn new_entry_id() -> String {
    let bytes = crypto::random_bytes(16);
    let mut out = String::with_capacity(32);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// In-memory vault state. The plaintext `entries` vector lives here
/// only while the vault is unlocked; once dropped, the data is gone
/// from RAM (modulo Rust's allocator not zeroing memory — out of scope
/// for ISO 4).
pub struct Vault {
    pub entries: Vec<Entry>,
    pub salt: Vec<u8>,
    /// The master key — kept in memory so `save()` can re-encrypt
    /// without prompting the user again.
    key: [u8; KEY_LEN],
}

/// The plaintext JSON shape that gets encrypted into the vault file.
#[derive(Serialize, Deserialize)]
struct VaultData {
    version: u32,
    entries: Vec<Entry>,
}

/// Errors that can occur during vault load/unlock/save.
#[derive(Debug)]
pub enum VaultError {
    Io(io::Error),
    Json(serde_json::Error),
    Crypto(crate::crypto::CryptoError),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Io(e) => write!(f, "I/O error: {}", e),
            VaultError::Json(e) => write!(f, "JSON error: {}", e),
            VaultError::Crypto(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<io::Error> for VaultError {
    fn from(e: io::Error) -> Self {
        VaultError::Io(e)
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(e: serde_json::Error) -> Self {
        VaultError::Json(e)
    }
}

impl From<crate::crypto::CryptoError> for VaultError {
    fn from(e: crate::crypto::CryptoError) -> Self {
        VaultError::Crypto(e)
    }
}

impl Vault {
    /// Load an encrypted blob from `path`.
    ///
    /// Returns `Ok(None)` if the file does not exist — the caller treats
    /// this as the "first-time setup" path.
    /// Returns `Ok(Some(blob))` if the file was parsed successfully.
    pub fn load(path: &Path) -> Result<Option<EncryptedBlob>, VaultError> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        let blob: EncryptedBlob = serde_json::from_slice(&bytes)?;
        Ok(Some(blob))
    }

    /// Create a brand-new empty vault with a fresh random salt + key
    /// derived from `master_password`. Does NOT write to disk — call
    /// `save()` to persist.
    pub fn create(master_password: &str) -> Self {
        let salt = crypto::random_bytes(SALT_LEN);
        let key = crypto::derive_key(master_password, &salt);
        Self {
            entries: Vec::new(),
            salt,
            key,
        }
    }

    /// Unlock an existing encrypted blob with `master_password`.
    ///
    /// Returns `Err(VaultError::Crypto(_))` if the password is wrong
    /// (GCM tag verification fails). On success the entries are
    /// decrypted and parsed into a `Vault`.
    pub fn unlock(master_password: &str, blob: &EncryptedBlob) -> Result<Self, VaultError> {
        let key = crypto::derive_key(master_password, &blob.salt);
        let plaintext = crypto::decrypt(blob, &key)?;
        let data: VaultData = serde_json::from_str(&plaintext)?;
        Ok(Self {
            entries: data.entries,
            salt: blob.salt.clone(),
            key,
        })
    }

    /// Append an entry to the vault. The caller is responsible for
    /// calling `save()` to persist the change.
    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Borrow the entries slice read-only.
    pub fn get_entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Find an entry by id, returning a mutable reference so the detail
    /// pane can edit fields in place.
    pub fn find_entry_mut(&mut self, id: &str) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Remove an entry by id. Returns `true` if an entry was removed.
    pub fn remove_entry(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() != before
    }

    /// Toggle the `favorite` flag on the entry with the given id.
    #[allow(dead_code)]
    pub fn toggle_favorite(&mut self, id: &str) {
        if let Some(e) = self.find_entry_mut(id) {
            e.favorite = !e.favorite;
        }
    }

    /// Persist the vault to `path` as an encrypted JSON blob. Creates
    /// the parent directory if it doesn't exist.
    pub fn save(&self, path: &Path) -> Result<(), VaultError> {
        // Ensure the parent directory exists (~/.config/astra/astrapass/).
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = VaultData {
            version: VAULT_VERSION,
            entries: self.entries.clone(),
        };
        let json = serde_json::to_string(&data)?;
        let blob = crypto::encrypt_with_salt(&json, &self.key, self.salt.clone())?;
        let bytes = serde_json::to_vec(&blob)?;
        fs::write(path, bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn create_save_unlock_round_trip() {
        // Use a temp path so we don't clobber the real vault.
        let tmp = env::temp_dir().join("astrapass-test-vault.bin");
        let _ = fs::remove_file(&tmp);

        // Create + add an entry + save.
        {
            let mut v = Vault::create("masterpw");
            v.add_entry(Entry {
                id: "abc".to_string(),
                title: "GitHub".to_string(),
                username: "octocat".to_string(),
                password: "hunter2".to_string(),
                notes: "personal".to_string(),
                favorite: true,
                created_at: 1_700_000_000,
            });
            v.save(&tmp).expect("save");
        }

        // Load the blob back.
        let blob = Vault::load(&tmp).expect("load").expect("some blob");

        // Unlock with the right password.
        let v = Vault::unlock("masterpw", &blob).expect("unlock");
        assert_eq!(v.entries.len(), 1);
        assert_eq!(v.entries[0].title, "GitHub");
        assert_eq!(v.entries[0].username, "octocat");
        assert!(v.entries[0].favorite);

        // Wrong password fails.
        let bad = Vault::unlock("wrongpw", &blob);
        assert!(bad.is_err());

        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn load_missing_returns_none() {
        let p = env::temp_dir().join("does-not-exist.bin");
        let r = Vault::load(&p).unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn entry_id_is_32_hex_chars() {
        let e = Entry::new(0);
        assert_eq!(e.id.len(), 32);
        assert!(e.id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn toggle_favorite_flips_flag() {
        let mut v = Vault::create("pw");
        v.add_entry(Entry {
            id: "x".to_string(),
            title: "T".to_string(),
            username: String::new(),
            password: String::new(),
            notes: String::new(),
            favorite: false,
            created_at: 0,
        });
        v.toggle_favorite("x");
        assert!(v.get_entries()[0].favorite);
        v.toggle_favorite("x");
        assert!(!v.get_entries()[0].favorite);
    }
}
