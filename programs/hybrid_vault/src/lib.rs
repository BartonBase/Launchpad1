//! # hybrid_vault (Track A, SPL-404 engine, ADR-008)
//!
//! **Status: in development. NOT AUDITED. Localnet/devnet only.**
//!
//! Token <-> NFT vault bound 1:1 to a `hybrid_launch::LaunchConfig`:
//! - `init_vault`: binds to the LaunchConfig (owner program, PDA, mint, creator), creates the vault's
//!   Metaplex Core collection (update authority = vault PDA, no plugins) and commits a trait root.
//! - `deposit_asset`: creator pre-deposits committed asset `index` (Merkle-proven) into the vault.
//!   The vault seals when all are in.
//! - `request_capture` / `request_reroll`: lock payment (+ fee in escrow), commit Switchboard VRF by CPI.
//! - `settle_capture` / `settle_reroll`: permissionless, strict FIFO, uniform VRF pick, fee burned.
//! - `unwrap`: exact `ratio` back, one tx, no token fee, never pausable.
//! - `expire_request`: refund if the VRF never revealed after the deadline.
//! - `pause` / `unpause`: optional guardian; blocks only new requests; auto-expires.
//! - `merge_incoming`: permissionless crank.
//!
//! Randomness is ONLY the Switchboard On-Demand VRF value. SlotHashes/Clock/blockhash are never used.
//! There is no instruction that changes ratio, fees, mint, collection or trait root, and no withdraw.

pub mod constants;
pub mod core_cpi;
pub mod error;
pub mod instructions;
pub mod invariants;
pub mod merkle;
pub mod pool;
pub mod randomness;
pub mod selection;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG");

// switchboard-on-demand pulls getrandom 0.2; on SBF we register a backend that ALWAYS FAILS.
// This program never uses OS randomness.
#[cfg(target_os = "solana")]
fn getrandom_always_fails(_buf: &mut [u8]) -> core::result::Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}
#[cfg(target_os = "solana")]
getrandom::register_custom_getrandom!(getrandom_always_fails);

#[program]
pub mod hybrid_vault {
    use super::*;

    pub fn init_vault(ctx: Context<InitVault>, params: InitVaultParams) -> Result<()> {
        instructions::init_vault::handle_init_vault(ctx, params)
    }

    pub fn deposit_asset(
        ctx: Context<DepositAsset>,
        index: u32,
        name: String,
        uri: String,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::deposit_asset::handle_deposit_asset(ctx, index, name, uri, proof)
    }

    pub fn request_capture(ctx: Context<RequestCapture>) -> Result<()> {
        instructions::request::handle_request_capture(ctx)
    }

    pub fn settle_capture(ctx: Context<Settle>) -> Result<()> {
        instructions::settle::handle_settle(ctx, REQUEST_KIND_CAPTURE)
    }

    pub fn request_reroll(ctx: Context<RequestReroll>, index: u32) -> Result<()> {
        instructions::request::handle_request_reroll(ctx, index)
    }

    pub fn settle_reroll(ctx: Context<Settle>) -> Result<()> {
        instructions::settle::handle_settle(ctx, REQUEST_KIND_REROLL)
    }

    pub fn unwrap(ctx: Context<Unwrap>, index: u32) -> Result<()> {
        instructions::unwrap::handle_unwrap(ctx, index)
    }

    pub fn expire_request(ctx: Context<ExpireRequest>) -> Result<()> {
        instructions::expire::handle_expire(ctx)
    }

    pub fn pause(ctx: Context<GuardianAction>, duration_slots: u64) -> Result<()> {
        instructions::admin::handle_pause(ctx, duration_slots)
    }

    pub fn unpause(ctx: Context<GuardianAction>) -> Result<()> {
        instructions::admin::handle_unpause(ctx)
    }

    pub fn merge_incoming(ctx: Context<MergeIncoming>, max: u32) -> Result<()> {
        instructions::admin::handle_merge_incoming(ctx, max)
    }
}
