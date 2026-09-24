//! Seeds and protocol constants for `hybrid_launch` (Track A, SPL-404).

use anchor_lang::prelude::*;

/// `["launch_config", mint]`: one immutable config per launched mint. Its
/// existence proves the mint was created by this launchpad.
#[constant]
pub const LAUNCH_CONFIG_SEED: &[u8] = b"launch_config";

/// `["mint_authority", launch_config]`: temporary mint authority, revoked
/// (set to `None`) inside `launch` right after minting the fixed supply.
#[constant]
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

/// Classic SPL Token program (Token-2022 is rejected: INV-13).
pub const SPL_TOKEN_PROGRAM_ID: Pubkey = anchor_spl::token::ID;

/// Fixed total supply in WHOLE tokens (BRIEF hard requirement #1).
#[constant]
pub const TOTAL_SUPPLY_WHOLE_TOKENS: u64 = 1_000_000_000;

/// 1e9 * 10^9 = 1e18 < u64::MAX.
#[constant]
pub const MAX_DECIMALS: u8 = 9;

/// Allowed wrap ratios (whole tokens per NFT). All divide 1B and are multiples
/// of 10_000, so `ratio * bps / 10_000` is always an exact integer.
pub const ALLOWED_RATIOS: [u64; 5] = [10_000, 50_000, 100_000, 200_000, 1_000_000];

/// Hard cap for capture / re-roll token fees: 1_000 bps = 10% of the ratio.
#[constant]
pub const MAX_TOKEN_FEE_BPS: u16 = 1_000;

/// Fee destination codes. Only BURN exists (Barton, 2026-09-24): no program or
/// wallet ever custodies a fee pile.
pub const FEE_DESTINATION_BURN: u8 = 0;

/// LaunchConfig layout version.
pub const LAUNCH_CONFIG_VERSION: u8 = 1;
