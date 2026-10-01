//! # hybrid_vault (Track A, SPL-404 engine, ADR-008 / ADR-012 / ADR-013)
//!
//! **Status: in development. NOT AUDITED. Localnet/devnet only.**
//!
//! Token <-> NFT vault bound 1:1 to a `hybrid_launch::LaunchConfig` (immutable economics):
//! Economics (ratio, N, flat SOL fees, fee recipient) are read from the immutable LaunchConfig on
//! every use (config.rs); the vault keeps no copies.
//! - `init_vault`: creator binds the vault to its LaunchConfig, creates the Core collection
//!   (update authority = vault PDA, no plugins), commits the trait root, pins the Switchboard queue.
//! - LAZY MINT (ADR-016): no pre-mint; settle mints a never-minted pick straight to the user from the
//!   request's mint escrow (leaf + proof verified against the committed root).
//! - `open_vault`: permissionless, one-way; needs a full mint AND verified graduation.
//! - `init_randomness` / `reveal_randomness` / `recommit_randomness`: permissionless Switchboard
//!   On-Demand lifecycle; the vault PDA is the randomness authority; reveal records the value.
//! - `expire_request`: last resort after 1 + MAX_RECOMMITS unrevealed commits + ~1 day: principal
//!   only (N tokens or the handed-in NFT); fees never refunded; impossible once revealed.
//! - `request_capture` / `request_reroll`: ratio into the vault (capture), flat SOL fee
//!   min(stored, tier, MAX) to PLATFORM_FEE_RECIPIENT (no token fee), Switchboard commit in the same instruction. Pool floor first.
//! - `settle_capture` / `settle_reroll`: permissionless, strict FIFO, uniform VRF pick.
//! - `unwrap` (release): FREE (Barton 2026-09-25 5:04 PM MT): exactly `ratio` back, no SOL fee, no fee
//!   account, one tx, never gated. It reads only mint/ratio/collection_size from the LaunchConfig at
//!   frozen offsets (M-41), so no launch-program upgrade can freeze it.
//! - NO pause/guardian of any kind (ADR-015): no key can halt any instruction.
//! - `merge_incoming`: permissionless crank.
//!
//! Randomness is ONLY the Switchboard On-Demand value. SlotHashes/Clock/blockhash are never used.
//! No instruction changes ratio, fees, fee recipient, mint, collection, queue or trait root; there
//! is no token withdraw, burn or close instruction (A-01/A-02/A-07). Solvency is asserted on-chain at the
//! end of every instruction that moves tokens or NFTs (invariants.rs).

pub mod asset_source;
pub mod config;
pub mod constants;
pub mod core_cpi;
pub mod error;
pub mod graduation;
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

    pub fn open_vault(ctx: Context<OpenVault>) -> Result<()> {
        instructions::open_vault::handle_open_vault(ctx)
    }

    pub fn init_randomness(ctx: Context<InitRandomness>, recent_slot: u64) -> Result<()> {
        instructions::randomness_ix::handle_init_randomness(ctx, recent_slot)
    }

    pub fn request_capture(ctx: Context<RequestCapture>) -> Result<()> {
        instructions::request::handle_request_capture(ctx)
    }

    pub fn request_reroll(ctx: Context<RequestReroll>, index: u32) -> Result<()> {
        instructions::request::handle_request_reroll(ctx, index)
    }

    pub fn reveal_randomness(
        ctx: Context<RevealRandomness>,
        args: randomness::RevealArgs,
    ) -> Result<()> {
        instructions::randomness_ix::handle_reveal_randomness(ctx, args)
    }

    pub fn recommit_randomness(ctx: Context<RecommitRandomness>) -> Result<()> {
        instructions::randomness_ix::handle_recommit_randomness(ctx)
    }

    pub fn settle_capture(
        ctx: Context<Settle>,
        mint: Option<asset_source::MintArgs>,
    ) -> Result<()> {
        instructions::settle::handle_settle(ctx, REQUEST_KIND_CAPTURE, mint)
    }

    pub fn settle_reroll(ctx: Context<Settle>, mint: Option<asset_source::MintArgs>) -> Result<()> {
        instructions::settle::handle_settle(ctx, REQUEST_KIND_REROLL, mint)
    }

    pub fn unwrap(ctx: Context<Unwrap>, index: u32) -> Result<()> {
        instructions::unwrap::handle_unwrap(ctx, index)
    }

    pub fn expire_request(ctx: Context<ExpireRequest>) -> Result<()> {
        instructions::expire::handle_expire(ctx)
    }

    /// M-04 batch expire: up to MAX_EXPIRE_PER_CALL consecutive queue heads in one instruction
    /// (7 remaining accounts per request; see instructions/expire.rs).
    pub fn expire_requests<'info>(
        ctx: Context<'info, ExpireRequests<'info>>,
        count: u8,
    ) -> Result<()> {
        instructions::expire::handle_expire_batch(ctx, count)
    }

    pub fn merge_incoming(ctx: Context<MergeIncoming>, max: u32) -> Result<()> {
        instructions::admin::handle_merge_incoming(ctx, max)
    }

    /// Mode 3. Bind a permanent vault to a `BurnLaunchConfig`. No token vault, no release.
    pub fn init_permanent_vault(
        ctx: Context<InitPermanentVault>,
        params: PermanentInitParams,
    ) -> Result<()> {
        instructions::permanent::handle_init_permanent_vault(ctx, params)
    }

    /// Mode 3. Permissionless, one-way. Opens wrapping once graduation is verified.
    pub fn open_permanent_vault(ctx: Context<OpenPermanentVault>) -> Result<()> {
        instructions::permanent::handle_open_permanent_vault(ctx)
    }

    /// Mode 3 wrap. Burns exactly `ratio` tokens and mints the next NFT. Irreversible.
    pub fn wrap_permanent(ctx: Context<WrapPermanent>, args: asset_source::MintArgs) -> Result<()> {
        instructions::permanent::handle_wrap_permanent(ctx, args)
    }

    /// Mode 4. Bind a tax vault to a Token-2022 launch. Creates the project collection.
    /// Wrap locks tokens; it does not burn them.
    pub fn init_token22_vault(
        ctx: Context<InitToken22Vault>,
        params: Token22InitParams,
    ) -> Result<()> {
        instructions::token22::handle_init_token22_vault(ctx, params)
    }

    /// Mode 4. Lock `ratio` tokens and mint the next project NFT. Supply does not change.
    pub fn wrap_token22(ctx: Context<WrapToken22>, args: asset_source::MintArgs) -> Result<()> {
        instructions::token22::handle_wrap_token22(ctx, args)
    }

    /// Move withheld transfer tax into the program treasury and accrue it per project NFT.
    pub fn harvest_tax(ctx: Context<HarvestTax>) -> Result<()> {
        instructions::token22::handle_harvest_tax(ctx)
    }

    /// Pay the next frozen NFT its exact share. The caller cannot choose the NFT,
    /// the amount, or the destination. If no round is open, this pays nothing.
    pub fn claim_tax(ctx: Context<ClaimTax>) -> Result<()> {
        instructions::token22::handle_claim_tax(ctx)
    }

    /// Spend SOL on the tax PDA to buy this launch's tokens at the locked price.
    /// Bought tokens accrue to project NFT holders only.
    pub fn buyback(ctx: Context<Buyback>) -> Result<()> {
        instructions::token22::handle_buyback(ctx)
    }

    /// Mode 5. Bind a raffle vault to a Mode 5 tax launch and pin its Switchboard queue.
    pub fn init_raffle_vault(
        ctx: Context<InitRaffleVault>,
        params: RaffleInitParams,
    ) -> Result<()> {
        instructions::raffle::handle_init_raffle_vault(ctx, params)
    }

    /// Record the current owner of the next frozen NFT. Must finish before the draw is requested.
    pub fn snapshot_raffle(ctx: Context<SnapshotRaffle>) -> Result<()> {
        instructions::raffle::handle_snapshot_raffle(ctx)
    }

    /// Create a Switchboard randomness account whose authority is this vault's PDA.
    pub fn init_raffle_randomness(
        ctx: Context<InitRaffleRandomness>,
        recent_slot: u64,
    ) -> Result<()> {
        instructions::raffle::handle_init_raffle_randomness(ctx, recent_slot)
    }

    /// Commit the pinned Switchboard account. The program picks the oracle. No second try after a reveal.
    pub fn commit_raffle(ctx: Context<CommitRaffle>) -> Result<()> {
        instructions::raffle::handle_commit_raffle(ctx)
    }

    /// Submit the oracle reveal. The value is stored. It cannot be replaced.
    pub fn reveal_raffle(ctx: Context<RevealRaffle>, args: randomness::RevealArgs) -> Result<()> {
        instructions::raffle::handle_reveal_raffle(ctx, args)
    }

    /// Pay the snapshotted owner of the winning NFT the whole pot.
    pub fn settle_raffle(ctx: Context<SettleRaffle>) -> Result<()> {
        instructions::raffle::handle_settle_raffle(ctx)
    }
}
