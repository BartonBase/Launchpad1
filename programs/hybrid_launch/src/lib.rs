//! # hybrid_launch
//!
//! **Status: in development. Not audited. Devnet/localnet only. Not mainnet.**
//!
//! Mode 2 (reversible hybrid, Track A) is [`hybrid_launch::launch`] and
//! [`hybrid_launch::register_dbc_launch`]. They write an immutable [`state::LaunchConfig`].
//! The swap engine is `hybrid_vault`, which loads only that account type.
//!
//! Mode 1 (plain SPL) is [`hybrid_launch::launch_plain`] and
//! [`hybrid_launch::register_plain_dbc`]. They write a [`state::PlainLaunchConfig`] at the same
//! seeds. No ratio, collection, fee, wrap, unwrap, or NFT. Token-2022 is rejected on both modes
//! (`Program<Token>`). There is no update instruction on either account.
//!
//! Mode 3 (burn hybrid) is [`hybrid_launch::launch_burn`] and
//! [`hybrid_launch::register_burn_dbc`]. They write a [`state::BurnLaunchConfig`] at the same seeds.
//! `hybrid_vault` mints from that collection and burns the tokens in the same instruction. There is
//! no release. Token-2022 is rejected.

pub mod constants;
pub mod dbc;
pub mod error;
pub mod instructions;
pub mod needs_barton;
pub mod stable_layout;
pub mod state;
pub mod t22;
pub mod validation;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use stable_layout::*;
pub use state::*;
pub use validation::*;

declare_id!("9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8");

#[program]
pub mod hybrid_launch {
    use super::*;

    /// Create the mint, mint the fixed 1B supply, revoke authorities and record
    /// the immutable LaunchConfig (see `instructions::launch`).
    pub fn launch(ctx: Context<Launch>, params: LaunchParams) -> Result<()> {
        instructions::launch::handle_launch(ctx, params)
    }

    /// Register a Meteora DBC-created token as a hybrid launch (ADR-014). Verifies DBC's pool, config
    /// and mint, and creates the locked buffer ATA. Signed by the DBC pool creator, before graduation.
    pub fn register_dbc_launch(
        ctx: Context<RegisterDbcLaunch>,
        params: RegisterDbcParams,
    ) -> Result<()> {
        instructions::register_dbc::handle_register_dbc_launch(ctx, params)
    }

    /// Mode 1. Create a classic SPL mint, mint the fixed 1B supply, revoke authorities, and record
    /// an immutable `PlainLaunchConfig`. No ratio, collection, fee, or NFT. Not a curve.
    pub fn launch_plain(ctx: Context<LaunchPlain>, params: PlainLaunchParams) -> Result<()> {
        instructions::plain::handle_launch_plain(ctx, params)
    }

    /// Mode 1 curve path. Bind a DBC pool to a `PlainLaunchConfig` only when its config is on
    /// `APPROVED_DBC_CONFIGS`. No ratio, collection, fee, or NFT. No wrap.
    pub fn register_plain_dbc(ctx: Context<RegisterPlainDbc>) -> Result<()> {
        instructions::register_plain_dbc::handle_register_plain_dbc(ctx)
    }

    /// Mode 3. Classic SPL mint, fixed 1B, revoked authorities, immutable `BurnLaunchConfig`.
    /// Ratio and collection size are fixed here. Not a curve. Wrapping is `hybrid_vault`, and it burns.
    pub fn launch_burn(ctx: Context<LaunchBurn>, params: LaunchParams) -> Result<()> {
        instructions::launch_burn::handle_launch_burn(ctx, params)
    }

    /// Mode 3 curve path. Bind an allowlisted DBC pool to a `BurnLaunchConfig`. Same checks as
    /// `register_dbc_launch`. No release path is created here.
    pub fn register_burn_dbc(
        ctx: Context<RegisterBurnDbc>,
        params: RegisterDbcParams,
    ) -> Result<()> {
        instructions::register_burn_dbc::handle_register_burn_dbc(ctx, params)
    }

    /// Mode 4. Token-2022 mint with a transfer tax chosen here and locked (no fee-config authority).
    /// Mint and freeze revoked. Withheld tax withdraws only to the vault program's tax PDA.
    /// This path does not burn.
    pub fn launch_token22(ctx: Context<LaunchToken22>, params: TaxLaunchParams) -> Result<()> {
        instructions::launch_token22::handle_launch_token22(ctx, params)
    }

    /// Mode 5. Same locked tax and buyback price as Mode 4. The vault raffles the whole round
    /// to one snapshotted NFT instead of splitting it.
    pub fn launch_raffle(ctx: Context<LaunchToken22>, params: TaxLaunchParams) -> Result<()> {
        instructions::launch_token22::handle_launch_tax(ctx, params, LAUNCH_MODE_RAFFLE)
    }

    /// Buy launch inventory with SOL from the tax PDA. Tokens go only to the tax treasury.
    /// Price is the lamports-per-whole-token stored at launch.
    pub fn buy_inventory(ctx: Context<BuyInventory>, sol_amount: u64) -> Result<()> {
        instructions::launch_token22::handle_buy_inventory(ctx, sol_amount)
    }
}
