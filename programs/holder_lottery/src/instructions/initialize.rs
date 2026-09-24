//! `initialize`: create the lottery config for one Token-2022 ticket mint.

use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::LotteryError,
    state::{LotteryConfig, RandomnessProvider},
};

/// Fairness parameters supplied at initialization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitializeLotteryParams {
    pub ticket_threshold: u64,
    pub min_holding_seconds: i64,
    pub randomness_provider: RandomnessProvider,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Pays rent and becomes `config.admin`.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Only the owning program is verified here (must be Token-2022).
    /// TODO(next milestone): deserialize via anchor-spl `token_interface`.
    #[account(owner = TOKEN_2022_PROGRAM_ID @ LotteryError::MintNotToken2022)]
    pub ticket_mint: UncheckedAccount<'info>,

    /// Config PDA. `init` guarantees it cannot be re-initialized.
    #[account(
        init,
        payer = admin,
        space = 8 + LotteryConfig::INIT_SPACE,
        seeds = [LOTTERY_CONFIG_SEED, ticket_mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, LotteryConfig>,

    /// CHECK: Data-less PDA that will own prize assets. Seeds + canonical
    /// bump are verified by Anchor.
    #[account(seeds = [PRIZE_VAULT_SEED, config.key().as_ref()], bump)]
    pub prize_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>, params: InitializeLotteryParams) -> Result<()> {
    require!(params.ticket_threshold > 0, LotteryError::ZeroTicketThreshold);
    require!(
        (MIN_HOLDING_SECONDS_FLOOR..=MAX_HOLDING_SECONDS_CEIL).contains(&params.min_holding_seconds),
        LotteryError::InvalidHoldingPeriod
    );

    let config = &mut ctx.accounts.config;
    config.set_inner(LotteryConfig {
        version: LOTTERY_CONFIG_VERSION,
        bump: ctx.bumps.config,
        prize_vault_bump: ctx.bumps.prize_vault,
        paused: false,
        admin: ctx.accounts.admin.key(),
        ticket_mint: ctx.accounts.ticket_mint.key(),
        ticket_threshold: params.ticket_threshold,
        min_holding_seconds: params.min_holding_seconds,
        randomness_provider: params.randomness_provider,
        current_round: 0,
    });

    let provider: RandomnessProvider = config.randomness_provider;
    msg!(
        "holder_lottery initialized: mint {} threshold {} provider {:?}",
        config.ticket_mint,
        config.ticket_threshold,
        provider
    );
    Ok(())
}
