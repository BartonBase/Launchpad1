//! Seeds and protocol constants for `holder_lottery`.

use anchor_lang::prelude::*;

/// Seed for the per-mint [`crate::state::LotteryConfig`] PDA:
/// `["lottery_config", ticket_mint]`.
#[constant]
pub const LOTTERY_CONFIG_SEED: &[u8] = b"lottery_config";

/// Seed for the prize-vault authority PDA: `["prize_vault", config]`.
/// Owns the prize NFTs / token accounts. Holds no data.
#[constant]
pub const PRIZE_VAULT_SEED: &[u8] = b"prize_vault";

/// Token-2022 (Token Extensions) program id.
pub const TOKEN_2022_PROGRAM_ID: Pubkey =
    anchor_lang::prelude::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// Minimum holding period that may be configured (24 hours).
#[constant]
pub const MIN_HOLDING_SECONDS_FLOOR: i64 = 24 * 60 * 60;

/// Maximum holding period that may be configured (90 days).
#[constant]
pub const MAX_HOLDING_SECONDS_CEIL: i64 = 90 * 24 * 60 * 60;

/// Account layout version written at initialization.
pub const LOTTERY_CONFIG_VERSION: u8 = 1;
