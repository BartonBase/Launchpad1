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

/// `["launch_vault", mint, launch_config]`: data-less PDA that OWNS the launch
/// destination token account (its ATA) holding the full 1B supply. No instruction
/// in this program signs with it, so no person can withdraw (QA-HL-02 / N2).
/// Distribution (bonding curve) is TBD and must be added under the program's
/// multisig + timelock upgrade process.
#[constant]
pub const LAUNCH_VAULT_SEED: &[u8] = b"launch_vault";

/// Classic SPL Token program (Token-2022 is rejected: INV-13).
pub const SPL_TOKEN_PROGRAM_ID: Pubkey = anchor_spl::token::ID;

/// Fixed total supply in WHOLE tokens (BRIEF hard requirement #1).
#[constant]
pub const TOTAL_SUPPLY_WHOLE_TOKENS: u64 = 1_000_000_000;

/// 1e9 * 10^9 = 1e18 < u64::MAX.
#[constant]
pub const MAX_DECIMALS: u8 = 9;

/// Allowed wrap ratios (whole tokens per NFT), Barton 2026-09-24. All divide 1B
/// and are multiples of 10_000, so `ratio * bps / 10_000` is always exact.
/// Max collection size = 1B / ratio: 100_000 / 20_000 / 10_000 / 5_000 / 2_000 /
/// 1_000 / 400 / 200.
pub const ALLOWED_RATIOS: [u64; 8] = [10_000, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000];

/// Minimum collection size for every ratio (Barton, 2026-09-24).
#[constant]
pub const MIN_COLLECTION_SIZE: u64 = 100;

/// Hard cap for capture / re-roll token fees: 1_000 bps = 10% of the ratio.
#[constant]
pub const MAX_TOKEN_FEE_BPS: u16 = 1_000;

/// Fee destination codes. Only BURN exists (Barton, 2026-09-24): no program or
/// wallet ever custodies a fee pile.
pub const FEE_DESTINATION_BURN: u8 = 0;

/// LaunchConfig layout version.
pub const LAUNCH_CONFIG_VERSION: u8 = 1;
