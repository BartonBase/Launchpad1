//! Mode 1 curve path — `register_plain_dbc` (Barton 2026-10-01).
//!
//! Does not create the mint. DBC already did, including the mint-authority revoke. We verify the
//! classic mint, the pool, and the config, and accept only `APPROVED_DBC_CONFIGS`. The unsold
//! leftover ATA is owned by the buffer PDA. This file never signs: the buffer seeds are an
//! address check only.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token::{Mint, Token},
};

use crate::{
    constants::*,
    dbc::{
        self, DBC_POOL_AUTHORITY, DBC_TOKEN_AUTHORITY_IMMUTABLE, DBC_TOKEN_TYPE_SPL,
        WRAPPED_SOL_MINT,
    },
    error::LaunchError,
    state::PlainLaunchConfig,
    validation::{check_graduation_threshold, plain_total_supply},
};

#[derive(Accounts)]
pub struct RegisterPlainDbc<'info> {
    /// Pays rent; must be the DBC pool's `creator`. Holds no powers afterwards.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// The mint DBC created. Typed as classic SPL, so a Token-2022 mint is rejected.
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: parsed by `dbc::load_config` (owner DBC, discriminator, length) and allowlisted.
    pub dbc_config: UncheckedAccount<'info>,

    /// CHECK: parsed by `dbc::load_pool` (owner DBC, discriminator, length).
    pub dbc_pool: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + PlainLaunchConfig::INIT_SPACE,
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub launch_config: Box<Account<'info, PlainLaunchConfig>>,

    /// CHECK: data-less buffer PDA. No instruction signs with it.
    #[account(seeds = [DBC_BUFFER_SEED], bump)]
    pub buffer_authority: UncheckedAccount<'info>,

    /// CHECK: must be the classic ATA of the buffer PDA for this mint; created here.
    #[account(mut)]
    pub buffer_tokens: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_register_plain_dbc(ctx: Context<RegisterPlainDbc>) -> Result<()> {
    let a = &ctx.accounts;
    require!(
        is_approved_dbc_config(&a.dbc_config.key()),
        LaunchError::DbcConfigNotApproved
    );
    let mint_key = a.mint.key();
    let pool =
        dbc::load_pool(&a.dbc_pool.to_account_info()).ok_or(LaunchError::DbcAccountInvalid)?;
    let cfg =
        dbc::load_config(&a.dbc_config.to_account_info()).ok_or(LaunchError::DbcAccountInvalid)?;
    require_keys_eq!(pool.base_mint, mint_key, LaunchError::DbcAccountInvalid);
    require_keys_eq!(
        pool.config,
        a.dbc_config.key(),
        LaunchError::DbcAccountInvalid
    );
    require!(
        pool.pool_type == DBC_TOKEN_TYPE_SPL,
        LaunchError::DbcAccountInvalid
    );
    require_keys_eq!(
        pool.creator,
        a.creator.key(),
        LaunchError::DbcCreatorMismatch
    );
    require!(
        !pool.is_migrated && !pool.graduated(),
        LaunchError::DbcAlreadyGraduated
    );

    let total_supply_base = plain_total_supply(a.mint.decimals)?;
    check_graduation_threshold(cfg.migration_quote_threshold)?;

    let (buffer, _) = dbc::buffer_authority();
    require_keys_eq!(
        buffer,
        a.buffer_authority.key(),
        LaunchError::DbcConfigRejected
    );
    require!(
        cfg.quote_mint == WRAPPED_SOL_MINT
            && cfg.token_type == DBC_TOKEN_TYPE_SPL
            && cfg.token_decimal == a.mint.decimals
            && cfg.fixed_token_supply
            && cfg.pre_migration_token_supply == total_supply_base
            && cfg.post_migration_token_supply == total_supply_base
            && cfg.token_update_authority == DBC_TOKEN_AUTHORITY_IMMUTABLE
            && cfg.leftover_receiver == buffer,
        LaunchError::DbcConfigRejected
    );
    require!(
        a.mint.mint_authority.is_none()
            && a.mint.freeze_authority.is_none()
            && a.mint.supply == total_supply_base,
        LaunchError::DbcMintRejected
    );

    let token_program_id = a.token_program.key();
    require_keys_eq!(
        a.buffer_tokens.key(),
        get_associated_token_address_with_program_id(&buffer, &mint_key, &token_program_id),
        LaunchError::DbcConfigRejected
    );
    associated_token::create_idempotent(CpiContext::new(
        a.associated_token_program.key(),
        associated_token::Create {
            payer: a.creator.to_account_info(),
            associated_token: a.buffer_tokens.to_account_info(),
            authority: a.buffer_authority.to_account_info(),
            mint: a.mint.to_account_info(),
            system_program: a.system_program.to_account_info(),
            token_program: a.token_program.to_account_info(),
        },
    ))?;

    let cfg_key = a.dbc_config.key();
    let pool_key = a.dbc_pool.key();
    let creator = a.creator.key();
    let decimals = a.mint.decimals;
    let threshold = cfg.migration_quote_threshold;
    let destination = pool.base_vault;
    ctx.accounts.launch_config.set_inner(PlainLaunchConfig {
        version: PLAIN_LAUNCH_CONFIG_VERSION,
        bump: ctx.bumps.launch_config,
        mint_authority_bump: 0,
        launch_mode: LAUNCH_MODE_PLAIN,
        creator,
        mint: mint_key,
        launch_destination: destination,
        launch_vault: DBC_POOL_AUTHORITY,
        launch_vault_bump: 0,
        decimals,
        total_supply_base,
        launched_at: Clock::get()?.unix_timestamp,
        dbc_config: cfg_key,
        dbc_pool: pool_key,
        graduation_threshold_lamports: threshold,
    });
    msg!(
        "hybrid_launch: plain DBC pool {} mint {}",
        pool_key,
        mint_key
    );
    Ok(())
}
