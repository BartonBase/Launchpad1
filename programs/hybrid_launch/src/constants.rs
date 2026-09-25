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

/// Allowed wrap ratios (whole tokens per NFT). All divide 1B. Max collection size =
/// min(1B / ratio, MAX_COLLECTION_SIZE).
/// 10k was DROPPED by Barton 2026-09-25 4:55 PM MT (its mint cost exceeds the NFT's token value).
pub const ALLOWED_RATIOS: [u64; 7] = [50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000];

/// Minimum collection size for every ratio (Barton, 2026-09-24).
#[constant]
pub const MIN_COLLECTION_SIZE: u64 = 100;

// Values awaiting a Barton decision (collection cap, fee cap/defaults, fee recipient) live in
// `needs_barton.rs` and are re-exported here.
pub use crate::needs_barton::*;

/// LaunchConfig layout version.
/// v3 = flat SOL fee model (Barton 2026-09-25 4:49 PM MT); v2 (2% token fee) never shipped.
pub const LAUNCH_CONFIG_VERSION: u8 = 3;

const _: () = {
    // FEE_TIERS covers exactly ALLOWED_RATIOS, in order.
    let mut i = 0;
    while i < ALLOWED_RATIOS.len() {
        assert!(crate::needs_barton::FEE_TIERS[i].0 == ALLOWED_RATIOS[i]);
        i += 1;
    }
};
