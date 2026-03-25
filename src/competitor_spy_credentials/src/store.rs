// competitor_spy_credentials/src/store.rs
//
// File-backed encrypted credential store. Each credential is individually
// encrypted with age passphrase-based encryption (scrypt). The on-disk
// format is a JSON object: { "adapter_id": "<base64-encoded ciphertext>" }.
//
// Decrypted values are wrapped in SecretValue which zeroes memory on drop.
// Nothing here may be written to logs or stdout.

use age::secrecy::Secret;
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};
use thiserror::Error;
use zeroize::Zeroizing;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("encryption error: {0}")]
    Encrypt(#[from] age::EncryptError),
    #[error("decryption error: {0}")]
    Decrypt(#[from] age::DecryptError),
    #[error("serialisation error: {0}")]
    Serialise(#[from] serde_json::Error),
    #[error("corrupt store: {0}")]
    CorruptStore(String),
}

// ---------------------------------------------------------------------------
// SecretValue -- zeroed on drop
// ---------------------------------------------------------------------------

/// Holds a decrypted credential value. Memory is overwritten with zeroes on drop.
pub struct SecretValue(Vec<u8>);

impl SecretValue {
    fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Exposes the credential as a UTF-8 string slice.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }

    /// Raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        // Zero the memory before deallocation.
        for b in self.0.iter_mut() {
            *b = 0;
        }
        // Volatile write to prevent optimiser from eliding the zeroing.
        if !self.0.is_empty() {
            // SAFETY: `self.0.as_mut_ptr()` is valid and non-null because
            // `self.0.is_empty()` is false, guaranteeing at least one byte.
            // `write_volatile` ensures the compiler does not elide this write
            // even though the memory is immediately freed after Drop completes.
            unsafe {
                std::ptr::write_volatile(self.0.as_mut_ptr(), 0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CredentialStore
// ---------------------------------------------------------------------------

/// File-backed store for encrypted adapter credentials.
///
/// Each credential is individually encrypted with age passphrase-based
/// encryption (scrypt). The file contains a JSON object mapping adapter IDs
/// to base64-encoded ciphertext. The file is read once on open and
/// written on every mutation.
pub struct CredentialStore {
    path: PathBuf,
    /// SEC-003: passphrase is wrapped in `Zeroizing` so heap memory is
    /// overwritten with zeroes when `CredentialStore` is dropped.
    passphrase: Zeroizing<String>,
    /// adapter_id -> base64-encoded age ciphertext
    entries: HashMap<String, String>,
}

impl CredentialStore {
    /// Open (or create) a credential store at `path`, using `passphrase` for
    /// all encryption and decryption operations.
    ///
    /// If the file does not exist, an empty store is constructed in memory.
    /// The file is not written until the first mutation.
    pub fn open(path: PathBuf, passphrase: String) -> Result<Self, StoreError> {
        let entries = if path.exists() {
            let data = std::fs::read(&path)?;
            let map: HashMap<String, String> = serde_json::from_slice(&data)?;
            map
        } else {
            HashMap::new()
        };
        Ok(Self { path, passphrase: Zeroizing::new(passphrase), entries })
    }

    /// Encrypt `plaintext` and store it under `adapter_id`, overwriting any
    /// existing entry. Persists to disk immediately.
    pub fn store(&mut self, adapter_id: &str, plaintext: &str) -> Result<(), StoreError> {
        let ciphertext = age_encrypt(plaintext.as_bytes(), &self.passphrase)?;
        let encoded = base64_encode(&ciphertext);
        self.entries.insert(adapter_id.to_owned(), encoded);
        self.persist()
    }

    /// Retrieve and decrypt the credential for `adapter_id`.
    /// Returns `Ok(None)` if no entry exists for that adapter.
    pub fn retrieve(&self, adapter_id: &str) -> Result<Option<SecretValue>, StoreError> {
        match self.entries.get(adapter_id) {
            None => Ok(None),
            Some(encoded) => {
                let ciphertext = base64_decode(encoded)
                    .map_err(|e| StoreError::CorruptStore(format!("base64: {e}")))?;
                let plaintext = age_decrypt(&ciphertext, &self.passphrase)?;
                Ok(Some(SecretValue::new(plaintext)))
            }
        }
    }

    /// Remove the credential for `adapter_id`. Returns `true` if an entry
    /// existed and was deleted, `false` if no entry was found.
    /// Persists to disk only if a deletion occurred.
    pub fn delete(&mut self, adapter_id: &str) -> Result<bool, StoreError> {
        if self.entries.remove(adapter_id).is_some() {
            self.persist()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if a credential is stored for the given adapter_id.
    pub fn contains(&self, adapter_id: &str) -> bool {
        self.entries.contains_key(adapter_id)
    }

    // Write the current entries map to disk as JSON.
    fn persist(&self) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(&self.entries)?;
        std::fs::write(&self.path, &json)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// age encrypt/decrypt helpers (passphrase-based)
// ---------------------------------------------------------------------------

fn age_encrypt(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, age::EncryptError> {
    let encryptor =
        age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));
    let mut ciphertext = Vec::new();
    let mut writer = encryptor.wrap_output(&mut ciphertext)?;
    // write_all on Vec<u8> is infallible.
    writer.write_all(plaintext).expect("write_all on Vec must not fail");
    writer.finish()?;
    Ok(ciphertext)
}

fn age_decrypt(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>, age::DecryptError> {
    let decryptor = match age::Decryptor::new(ciphertext)? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err(age::DecryptError::DecryptionFailed),
    };
    let mut plaintext = Vec::new();
    // Accept high-work-factor entries from older/newer age clients.
    // This avoids false decryption failures when a credential was encrypted
    // with stronger scrypt parameters than the default threshold.
    let mut reader = decryptor.decrypt(&Secret::new(passphrase.to_owned()), Some(20))?;
    reader.read_to_end(&mut plaintext).map_err(|_| age::DecryptError::DecryptionFailed)?;
    Ok(plaintext)
}

// ---------------------------------------------------------------------------
// base64 helpers (no external crate; standard RFC 4648 alphabet)
// ---------------------------------------------------------------------------

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let combined = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((combined >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((combined >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((combined >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(combined & 0x3F) as usize] as char } else { '=' });
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim_end_matches('=');
    const TABLE: [i8; 128] = {
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut t = [-1i8; 128];
        let mut i = 0usize;
        while i < 64 {
            t[alphabet[i] as usize] = i as i8;
            i += 1;
        }
        t
    };
    let chars: Vec<u8> = s.bytes().collect();
    let mut out = Vec::with_capacity(chars.len() * 3 / 4 + 1);
    let mut i = 0;
    while i < chars.len() {
        let get = |idx: usize| -> Result<u32, String> {
            let c = chars.get(idx).copied().unwrap_or(b'A') as usize;
            if c >= 128 || TABLE[c] < 0 {
                return Err(format!("invalid base64 char {} at index {idx}", chars[idx] as char));
            }
            Ok(TABLE[c] as u32)
        };
        let b0 = get(i)?;
        let b1 = get(i + 1)?;
        out.push(((b0 << 2) | (b1 >> 4)) as u8);
        if i + 2 < chars.len() {
            let b2 = get(i + 2)?;
            out.push(((b1 << 4) | (b2 >> 2)) as u8);
            if i + 3 < chars.len() {
                let b3 = get(i + 3)?;
                out.push(((b2 << 6) | b3) as u8);
            }
        }
        i += 4;
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("cspy_store_tests");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    // --- SecretValue ---

    #[test]
    fn secret_value_as_str_returns_utf8() {
        let sv = SecretValue::new(b"my-api-key".to_vec());
        assert_eq!(sv.as_str().unwrap(), "my-api-key");
    }

    #[test]
    fn secret_value_as_bytes_returns_slice() {
        let sv = SecretValue::new(vec![1, 2, 3]);
        assert_eq!(sv.as_bytes(), &[1, 2, 3]);
    }

    // --- base64 ---

    #[test]
    fn base64_round_trip_empty() {
        let encoded = base64_encode(&[]);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, b"");
    }

    #[test]
    fn base64_round_trip_all_byte_values() {
        let data: Vec<u8> = (0..=255u8).collect();
        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    // --- age helpers ---

    #[test]
    fn age_encrypt_decrypt_round_trip() {
        let plaintext = b"super-secret-api-key-12345";
        let passphrase = "test-passphrase";
        let ciphertext = age_encrypt(plaintext, passphrase).unwrap();
        assert_ne!(&ciphertext, plaintext);
        let decrypted = age_decrypt(&ciphertext, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn age_wrong_passphrase_returns_error() {
        let ciphertext = age_encrypt(b"secret", "correct-passphrase").unwrap();
        let result = age_decrypt(&ciphertext, "wrong-passphrase");
        assert!(result.is_err());
    }

    // --- CredentialStore ---

    #[test]
    fn store_and_retrieve_credential() {
        let path = temp_path("store_retrieve.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path.clone(), "passphrase123".into()).unwrap();
        store.store("yelp", "yelp-api-key-abc").unwrap();

        let sv = store.retrieve("yelp").unwrap().expect("should be present");
        assert_eq!(sv.as_str().unwrap(), "yelp-api-key-abc");
    }

    #[test]
    fn retrieve_absent_adapter_returns_none() {
        let path = temp_path("retrieve_absent.json");
        let _ = std::fs::remove_file(&path);

        let store = CredentialStore::open(path, "passphrase123".into()).unwrap();
        let result = store.retrieve("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn store_persists_across_store_instances() {
        let path = temp_path("persist_test.json");
        let _ = std::fs::remove_file(&path);

        {
            let mut store = CredentialStore::open(path.clone(), "my-pass".into()).unwrap();
            store.store("google", "google-key-xyz").unwrap();
        }

        let store2 = CredentialStore::open(path, "my-pass".into()).unwrap();
        let sv = store2.retrieve("google").unwrap().expect("should persist");
        assert_eq!(sv.as_str().unwrap(), "google-key-xyz");
    }

    #[test]
    fn store_overwrites_existing_entry() {
        let path = temp_path("overwrite_test.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path, "pass".into()).unwrap();
        store.store("osm", "first-key").unwrap();
        store.store("osm", "second-key").unwrap();

        let sv = store.retrieve("osm").unwrap().expect("present");
        assert_eq!(sv.as_str().unwrap(), "second-key");
    }

    #[test]
    fn delete_removes_entry_and_returns_true() {
        let path = temp_path("delete_test.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path, "pass".into()).unwrap();
        store.store("yelp", "key").unwrap();
        let deleted = store.delete("yelp").unwrap();
        assert!(deleted);
        assert!(store.retrieve("yelp").unwrap().is_none());
    }

    #[test]
    fn delete_absent_returns_false() {
        let path = temp_path("delete_absent.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path, "pass".into()).unwrap();
        let deleted = store.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn contains_reflects_store_state() {
        let path = temp_path("contains_test.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path, "pass".into()).unwrap();
        assert!(!store.contains("yelp"));
        store.store("yelp", "key").unwrap();
        assert!(store.contains("yelp"));
        store.delete("yelp").unwrap();
        assert!(!store.contains("yelp"));
    }

    #[test]
    fn multiple_adapters_stored_independently() {
        let path = temp_path("multi_adapter.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path, "pass".into()).unwrap();
        store.store("yelp", "yelp-key").unwrap();
        store.store("google", "google-key").unwrap();

        assert_eq!(store.retrieve("yelp").unwrap().unwrap().as_str().unwrap(), "yelp-key");
        assert_eq!(
            store.retrieve("google").unwrap().unwrap().as_str().unwrap(),
            "google-key"
        );
    }

    #[test]
    fn wrong_passphrase_on_reopen_fails_decrypt() {
        let path = temp_path("wrong_pass.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path.clone(), "correct".into()).unwrap();
        store.store("yelp", "secret").unwrap();

        let store_wrong = CredentialStore::open(path, "wrong".into()).unwrap();
        let result = store_wrong.retrieve("yelp");
        assert!(result.is_err());
    }

    #[test]
    fn open_nonexistent_file_creates_empty_store() {
        let path = temp_path("nonexistent_xyz_99.json");
        let _ = std::fs::remove_file(&path);

        let store = CredentialStore::open(path, "pass".into()).unwrap();
        assert!(!store.contains("anything"));
    }

    #[test]
    fn delete_persists_across_reopen() {
        let path = temp_path("delete_persist.json");
        let _ = std::fs::remove_file(&path);

        let mut store = CredentialStore::open(path.clone(), "pass".into()).unwrap();
        store.store("yelp", "key").unwrap();
        store.delete("yelp").unwrap();

        let store2 = CredentialStore::open(path, "pass".into()).unwrap();
        assert!(!store2.contains("yelp"));
    }
}
