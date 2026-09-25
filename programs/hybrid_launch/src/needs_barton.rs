//! Barton-owned economic constants, in ONE place. Items marked DECIDED carry the date of Barton's
//! decision; anything still marked NEEDS BARTON is a documented default. See docs/DECISIONS.md
//! ADR-013 (flat SOL fee) and ADR-014 (graduation).

use anchor_lang::prelude::*;

/// Hard cap on collection size for batched pre-mint at graduation (graduation-design §2.3/§4.2:
/// measured 0.00509 SOL per NFT). DECIDED (Barton 2026-09-25 4:55 PM MT): 10,000.
/// Effective max N = min(1B / ratio, MAX_COLLECTION_SIZE).
#[constant]
pub const MAX_COLLECTION_SIZE: u64 = 10_000;

// ---------------------------------------------------------------------------------------------
// FEE POLICY (ADR-013, Barton 2026-09-25 4:49 PM + 4:52 PM MT). NO token fee of any kind. ONE flat
// SOL fee, the same on every capture, release and re-roll, TIERED BY RATIO from the const table
// below. Derived by hybrid_launch at launch (never creator-supplied) and stored immutably in the
// LaunchConfig. Never tied to an oracle or live price. Re-roll uses the same fee, so
// re-roll <= capture + release holds trivially in every tier (M-06).
// ---------------------------------------------------------------------------------------------

/// (ratio in whole tokens, flat SOL fee in lamports). Covers exactly ALLOWED_RATIOS.
/// DECIDED (Barton 2026-09-25 4:52 PM MT; 10k ratio dropped 4:55 PM MT).
pub const FEE_TIERS: [(u64, u64); 7] = [
    (50_000, 2_000_000),     // 0.002 SOL
    (100_000, 5_000_000),    // 0.005 SOL
    (200_000, 5_000_000),    // 0.005 SOL
    (500_000, 10_000_000),   // 0.01 SOL
    (1_000_000, 10_000_000), // 0.01 SOL
    (2_500_000, 10_000_000), // 0.01 SOL
    (5_000_000, 10_000_000), // 0.01 SOL
];

/// HARD CAP on the fee (M-08): exactly 0.01 SOL. A code constant: no launch parameter, instruction
/// or multisig action can raise it (only a program upgrade, 3-of-5 + 7-day timelock until the
/// post-audit freeze). hybrid_vault re-asserts it on every use.
#[constant]
pub const MAX_FEE_LAMPORTS: u64 = 10_000_000;

/// FLOOR (M-26 / M-30): the lowest tier. It must stay >= the 890_880-lamport rent-exempt minimum of
/// a 0-data system account so a transfer to an empty/missing recipient creates it instead of failing.
#[constant]
pub const MIN_FEE_LAMPORTS: u64 = 2_000_000;

/// Rent-exempt minimum for a 0-data account at the default rent (3480 lamports/byte-year x 2 years
/// x 128 bytes of account overhead).
pub const RENT_EXEMPT_MIN_0_BYTES: u64 = 890_880;

/// Flat SOL fee for a ratio, from FEE_TIERS. `None` for a ratio outside the set.
pub const fn fee_for_ratio(ratio_whole_tokens: u64) -> Option<u64> {
    let mut i = 0;
    while i < FEE_TIERS.len() {
        if FEE_TIERS[i].0 == ratio_whole_tokens {
            return Some(FEE_TIERS[i].1);
        }
        i += 1;
    }
    None
}

const fn tiers_within_bounds() -> bool {
    let mut i = 0;
    while i < FEE_TIERS.len() {
        let f = FEE_TIERS[i].1;
        if f < MIN_FEE_LAMPORTS || f > MAX_FEE_LAMPORTS {
            return false;
        }
        i += 1;
    }
    true
}
const _: () = assert!(tiers_within_bounds(), "every tier within [MIN_FEE_LAMPORTS, MAX_FEE_LAMPORTS]");
const _: () = assert!(MIN_FEE_LAMPORTS >= RENT_EXEMPT_MIN_0_BYTES, "fee floor must be >= rent-exempt minimum");
const _: () = assert!(MAX_FEE_LAMPORTS == 10_000_000, "hard cap is exactly 0.01 SOL");

/// The ONE fee recipient (Barton's fee wallet). Program constant, never a launch parameter or
/// instruction argument (M-05, F-06); recorded in every LaunchConfig and checked (fail closed) by
/// hybrid_vault. NEEDS BARTON: custody (recommended: a Squads vault he controls, M-17/B-07) and a
/// published policy that the fee wallet never converts or re-rolls (M-16).
/// Localnet/devnet: THROWAWAY key `.keys/devnet-only-fee-owner.json`.
/// A mainnet build refuses to compile until it is set.
#[cfg(not(feature = "mainnet"))]
pub const PLATFORM_FEE_RECIPIENT: Pubkey = pubkey!("7J3AajxfajAMgGmmwzfZeYGNieRtHRCuEgNTfd4gTjDN");
#[cfg(feature = "mainnet")]
compile_error!("hybrid_launch: set PLATFORM_FEE_RECIPIENT to Barton's mainnet Squads vault before a mainnet build");
#[cfg(feature = "mainnet")]
pub const PLATFORM_FEE_RECIPIENT: Pubkey = Pubkey::new_from_array([0u8; 32]);

// ---------------------------------------------------------------------------------------------
// MINT COST (LAZY MINT, ADR-016, Barton 2026-09-25 5:13 PM MT). Nothing is pre-minted; the
// affordability rule (ADR-014's graduation slice) is DROPPED. These constants size the per-request
// mint escrow in hybrid_vault (MINT_ESCROW_LAMPORTS = (rent + Core fee) x 125%). The graduation
// slice / margin constants below are kept for reference only and are not enforced.
// ---------------------------------------------------------------------------------------------

/// Core asset account rent, CONSERVATIVE: 385-byte asset (collection member + 8-trait Attributes
/// plugin + URI) = (128 + 385) x 6_960 = 3_570_480 lamports exactly (was 3_570_000, 480 short; caught by the live-rent guard) (graduation-design §1). `launch` also checks the live
/// Rent sysvar for CORE_ASSET_SPACE_BYTES does not exceed this (fails closed if rent rises).
pub const CORE_ASSET_RENT_LAMPORTS: u64 = 3_570_480;
pub const CORE_ASSET_SPACE_BYTES: usize = 385;
/// Metaplex Core protocol fee on create (measured 1_500_000 lamports = 0.0015 SOL).
pub const CORE_CREATE_FEE_LAMPORTS: u64 = 1_500_000;
/// Tx base share + crank reward per asset (graduation-design §1.4); sized so the all-in stays 0.00509 SOL.
pub const MINT_OVERHEAD_LAMPORTS: u64 = 19_520;
/// All-in per-NFT mint cost estimate: 0.00509 SOL (reference; the escrow uses rent + Core fee x 125%).
pub const PER_NFT_MINT_COST_LAMPORTS: u64 = CORE_ASSET_RENT_LAMPORTS + CORE_CREATE_FEE_LAMPORTS + MINT_OVERHEAD_LAMPORTS;
const _: () = assert!(PER_NFT_MINT_COST_LAMPORTS == 5_090_000);
/// Reference only since ADR-016 (was the pre-mint fundability margin).
pub const GRADUATION_MARGIN_PCT: u64 = 125;
/// Reference only since ADR-016 (was the max graduation slice for the pre-mint).
#[constant]
pub const MAX_GRADUATION_SLICE_PCT: u8 = 10;
/// Default graduation threshold (DBC migration_quote_threshold): 85 SOL. DECIDED.
#[constant]
pub const DEFAULT_GRADUATION_THRESHOLD_LAMPORTS: u64 = 85_000_000_000_u64;
/// Bounds: >= 10 SOL (Meteora keeper auto-migration floor); <= 100,000 SOL (sanity cap).
#[constant]
pub const MIN_GRADUATION_THRESHOLD_LAMPORTS: u64 = 10_000_000_000_u64;
#[constant]
pub const MAX_GRADUATION_THRESHOLD_LAMPORTS: u64 = 100_000_000_000_000_u64;
