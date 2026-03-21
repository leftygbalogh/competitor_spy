// competitor_spy_credentials
//
// Sync credential store. Encrypts/decrypts API keys using the `age` crate.
// Decrypted values are held in memory only for the duration of a run and
// zeroed on drop. Nothing in this crate is ever written to logs or stdout.

pub mod store;
