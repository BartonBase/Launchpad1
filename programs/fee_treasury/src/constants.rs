//! Seeds and protocol constants for `fee_treasury`.

use anchor_lang::prelude::*;

/// Seed for the per-mint [`crate::state::TreasuryConfig`] PDA:
/// `["treasury_config", fee_mint]`.
#[constant]
pub const TREASURY_CONFIG_SEED: &[u8] = b"treasury_config";

/// Seed for the vault-authority PDA: `["vault_authority", config]`.
/// This PDA will be the Token-2022 `withdraw_withheld_authority` of the fee
/// mint and the owner of the vault token account. It holds no data.
#[constant]
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";

/// Token-2022 (Token Extensions) program id. Pinned explicitly so a spoofed
/// "token program" can never be substituted.
pub const TOKEN_2022_PROGRAM_ID: Pubkey =
    anchor_lang::prelude::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// Shortest allowed spending window (1 hour).
#[constant]
pub const MIN_SPEND_WINDOW_SECONDS: i64 = 60 * 60;

/// Longest allowed spending window (30 days).
#[constant]
pub const MAX_SPEND_WINDOW_SECONDS: i64 = 30 * 24 * 60 * 60;

/// Account layout version written at initialization.
pub const TREASURY_CONFIG_VERSION: u8 = 1;
