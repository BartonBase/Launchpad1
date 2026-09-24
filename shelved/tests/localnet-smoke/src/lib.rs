//! Intentionally empty. See `tests/smoke.rs`.
//!
//! The smoke tests only run when `ANCHOR_PROVIDER_URL` points at a local
//! validator (set automatically by `anchor test`). They are skipped otherwise
//! and they hard-fail if the URL is anything other than localhost/127.0.0.1,
//! so they can never be pointed at devnet or mainnet by accident.
