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
pub const ALLOWED_RATIOS: [u64; 7] = [
    50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000,
];

/// Minimum collection size for every ratio (Barton, 2026-09-24).
#[constant]
pub const MIN_COLLECTION_SIZE: u64 = 100;

// Values awaiting a Barton decision (collection cap, fee cap/defaults, fee recipient) live in
// `needs_barton.rs` and are re-exported here.
pub use crate::needs_barton::*;

/// LaunchConfig layout version.
/// v3 = flat SOL fee model (Barton 2026-09-25 4:49 PM MT); v2 (2% token fee) never shipped.
/// v4 appends `dbc_config` + `dbc_pool` (ADR-014). The frozen exit-path prefix is unchanged.
pub const LAUNCH_CONFIG_VERSION: u8 = 4;

/// Mode 1 account layout version (`PlainLaunchConfig`). Not a `LaunchConfig` version.
pub const PLAIN_LAUNCH_CONFIG_VERSION: u8 = 1;

/// Mode 1: classic SPL memecoin. No wrap, unwrap, NFT, or Token-2022.
pub const LAUNCH_MODE_PLAIN: u8 = 1;
/// Mode 2: reversible hybrid. The account type is `LaunchConfig` (this byte is not stored there).
pub const LAUNCH_MODE_HYBRID: u8 = 2;

/// Mode 3: burn hybrid. Wrap burns the tokens and mints one NFT. No release.
pub const LAUNCH_MODE_BURN: u8 = 3;

/// Mode 3 account layout version (`BurnLaunchConfig`). Not a `LaunchConfig` version.
pub const BURN_LAUNCH_CONFIG_VERSION: u8 = 1;

const _: () = assert!(LAUNCH_MODE_PLAIN != LAUNCH_MODE_HYBRID);
const _: () = assert!(LAUNCH_MODE_BURN != LAUNCH_MODE_PLAIN);
const _: () = assert!(LAUNCH_MODE_BURN != LAUNCH_MODE_HYBRID);

/// Mode 4: Token-2022 transfer tax. The rate is chosen at launch and the config authority is None.
/// Withheld tokens are paid to this project's NFT holders. Nothing in this mode burns.
pub const LAUNCH_MODE_TOKEN22: u8 = 4;

/// Mode 5: same locked Token-2022 tax as Mode 4, but one snapshotted NFT wins the whole round.
pub const LAUNCH_MODE_RAFFLE: u8 = 5;

/// Mode 4 account layout version (`T22BurnLaunchConfig`).
pub const T22_BURN_LAUNCH_CONFIG_VERSION: u8 = 1;

/// Inclusive range for the transfer-tax rate, in basis points (1 = 0.01%, 1000 = 10%).
pub const MIN_TAX_BPS: u16 = 1;
pub const MAX_TAX_BPS: u16 = 1000;

/// SOL paid per whole token when the tax account buys inventory for NFT holders. Locked at launch.
pub const MIN_BUYBACK_LAMPORTS_PER_WHOLE: u64 = 1;
pub const MAX_BUYBACK_LAMPORTS_PER_WHOLE: u64 = 1_000_000_000;

/// `hybrid_vault` program id. The tax withdraw authority is a PDA of that program.
pub const HYBRID_VAULT_ID: Pubkey = pubkey!("BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG");

/// Seeds `["tax_authority", mint]` signed only by `hybrid_vault`.
pub const TAX_AUTHORITY_SEED: &[u8] = b"tax_authority";

const _: () = assert!(LAUNCH_MODE_TOKEN22 != LAUNCH_MODE_PLAIN);
const _: () = assert!(LAUNCH_MODE_TOKEN22 != LAUNCH_MODE_HYBRID);
const _: () = assert!(LAUNCH_MODE_TOKEN22 != LAUNCH_MODE_BURN);
const _: () = assert!(LAUNCH_MODE_RAFFLE != LAUNCH_MODE_TOKEN22);
const _: () = assert!(LAUNCH_MODE_RAFFLE != LAUNCH_MODE_PLAIN);
const _: () = assert!(LAUNCH_MODE_RAFFLE != LAUNCH_MODE_HYBRID);
const _: () = assert!(LAUNCH_MODE_RAFFLE != LAUNCH_MODE_BURN);

/// Seeds of the single buffer-authority PDA named as every DBC config's `leftover_receiver`.
/// NO instruction signs with it (T-GRAD-03).
#[constant]
pub const DBC_BUFFER_SEED: &[u8] = b"dbc_buffer";

const _: () = {
    // FEE_TIERS covers exactly ALLOWED_RATIOS, in order.
    let mut i = 0;
    while i < ALLOWED_RATIOS.len() {
        assert!(crate::needs_barton::FEE_TIERS[i].0 == ALLOWED_RATIOS[i]);
        i += 1;
    }
};
