//! `initialize`: create the treasury config for one Token-2022 fee mint.

use anchor_lang::prelude::*;

use crate::{constants::*, error::TreasuryError, state::TreasuryConfig};

/// Spending-rule parameters supplied at initialization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitializeTreasuryParams {
    pub max_spend_per_purchase: u64,
    pub max_spend_per_window: u64,
    pub spend_window_seconds: i64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Pays rent and becomes `config.admin`.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Only the owning program is verified here (must be Token-2022).
    /// TODO(next milestone): deserialize as `InterfaceAccount<Mint>` via
    /// anchor-spl `token_interface` and require the `TransferFeeConfig`
    /// extension with `withdraw_withheld_authority == vault_authority`.
    #[account(owner = TOKEN_2022_PROGRAM_ID @ TreasuryError::MintNotToken2022)]
    pub fee_mint: UncheckedAccount<'info>,

    /// Config PDA. `init` guarantees it cannot be re-initialized.
    #[account(
        init,
        payer = admin,
        space = 8 + TreasuryConfig::INIT_SPACE,
        seeds = [TREASURY_CONFIG_SEED, fee_mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, TreasuryConfig>,

    /// CHECK: Data-less PDA used only as a signer / token authority.
    /// Seeds + canonical bump are verified by Anchor.
    #[account(seeds = [VAULT_AUTHORITY_SEED, config.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>, params: InitializeTreasuryParams) -> Result<()> {
    require!(params.max_spend_per_purchase > 0, TreasuryError::ZeroPurchaseCap);
    require!(
        params.max_spend_per_purchase <= params.max_spend_per_window,
        TreasuryError::PurchaseCapExceedsWindowCap
    );
    require!(
        (MIN_SPEND_WINDOW_SECONDS..=MAX_SPEND_WINDOW_SECONDS).contains(&params.spend_window_seconds),
        TreasuryError::InvalidSpendWindow
    );

    let now = Clock::get()?.unix_timestamp;
    let config = &mut ctx.accounts.config;
    config.set_inner(TreasuryConfig {
        version: TREASURY_CONFIG_VERSION,
        bump: ctx.bumps.config,
        vault_authority_bump: ctx.bumps.vault_authority,
        paused: false,
        admin: ctx.accounts.admin.key(),
        fee_mint: ctx.accounts.fee_mint.key(),
        max_spend_per_purchase: params.max_spend_per_purchase,
        max_spend_per_window: params.max_spend_per_window,
        spend_window_seconds: params.spend_window_seconds,
        window_start_ts: now,
        spent_in_window: 0,
        total_harvested: 0,
        total_spent: 0,
    });

    msg!("fee_treasury initialized for mint {}", config.fee_mint);
    Ok(())
}
