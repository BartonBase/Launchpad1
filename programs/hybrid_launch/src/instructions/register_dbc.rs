//! `register_dbc_launch` (ADR-014): the Meteora DBC curve path. DBC creates the mint itself
//! (`initialize_virtual_pool_with_spl_token` needs `base_mint` as a signer of that transaction: a keypair,
//! or a PDA of a program that CPIs into DBC). We don't create or own the mint. Instead we VERIFY what DBC
//! created and bind it to an immutable LaunchConfig:
//!
//! - pool: owner DBC, VirtualPool discriminator + length, `base_mint == mint`, `config == dbc_config`,
//!   SPL pool type, NOT yet migrated (registration happens before graduation, so the supply check below
//!   isn't fooled by holders burning later);
//! - config: on the platform allowlist `APPROVED_DBC_CONFIGS` (compile-time; upgrade-only), owner DBC, PoolConfig discriminator + length, quote = wSOL, SPL token type, `token_decimal`
//!   == the mint's decimals == params, fixed supply with pre == post == 1B * 10^dec (DBC burns nothing at
//!   migration), Immutable token authority, `leftover_receiver` == our buffer PDA, threshold == params;
//! - mint: classic Token program (typed `Account<Mint>`), mint + freeze authority None, supply exactly 1B;
//! - the signer is the pool's creator (nobody else can register someone's token with other parameters);
//! - creates ATA(buffer PDA, mint): DBC's permissionless `withdraw_leftover` sends the unsold buffer there,
//!   and no instruction can ever sign for it (T-GRAD-03).
//!
//! The LaunchConfig PDA is `init` (one registration per mint, and a mint made by native `launch` can't be
//! registered again). `launch_destination` = DBC's base vault and `launch_vault` = DBC's pool authority.

use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token::{Mint, Token},
};

use crate::{
    constants::*,
    dbc::{self, DBC_POOL_AUTHORITY, DBC_TOKEN_AUTHORITY_IMMUTABLE, DBC_TOKEN_TYPE_SPL, WRAPPED_SOL_MINT},
    error::LaunchError,
    state::LaunchConfig,
    validation::{validate, LaunchParams},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegisterDbcParams {
    pub ratio_whole_tokens: u64,
    pub collection_size: u64,
}

#[derive(Accounts)]
pub struct RegisterDbcLaunch<'info> {
    /// Pays rent; must be the DBC pool's `creator`. Holds no powers afterwards.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// The mint DBC created. Typed as a classic SPL Token mint, so a Token-2022 mint is rejected.
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: parsed and verified by `dbc::load_config` (owner DBC, discriminator, length).
    pub dbc_config: UncheckedAccount<'info>,

    /// CHECK: parsed and verified by `dbc::load_pool` (owner DBC, discriminator, length).
    pub dbc_pool: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + LaunchConfig::INIT_SPACE,
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub launch_config: Box<Account<'info, LaunchConfig>>,

    /// CHECK: data-less PDA `["dbc_buffer"]` (canonical bump). No instruction signs with it.
    #[account(seeds = [DBC_BUFFER_SEED], bump)]
    pub buffer_authority: UncheckedAccount<'info>,

    /// CHECK: must be ATA(buffer_authority, mint, classic Token); created (idempotently) here.
    #[account(mut)]
    pub buffer_tokens: UncheckedAccount<'info>,

    /// CHECK: the platform fee recipient constant (ADR-013), same checks as `launch`.
    #[account(address = PLATFORM_FEE_RECIPIENT @ LaunchError::FeeRecipientInvalid)]
    pub fee_recipient: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_register_dbc_launch(ctx: Context<RegisterDbcLaunch>, params: RegisterDbcParams) -> Result<()> {
    let a = &ctx.accounts;
    {
        let r = a.fee_recipient.to_account_info();
        require_keys_eq!(*r.owner, system_program::ID, LaunchError::FeeRecipientInvalid);
        require!(!r.executable && r.data_is_empty(), LaunchError::FeeRecipientInvalid);
    }
    require!(
        Rent::get()?.minimum_balance(CORE_ASSET_SPACE_BYTES) <= CORE_ASSET_RENT_LAMPORTS,
        LaunchError::MintCostConstantStale
    );

    // Platform allowlist first: only configs the platform approved (compile-time, upgrade-only).
    require!(is_approved_dbc_config(&a.dbc_config.key()), LaunchError::DbcConfigNotApproved);
    let mint_key = a.mint.key();
    let pool = dbc::load_pool(&a.dbc_pool.to_account_info()).ok_or(LaunchError::DbcAccountInvalid)?;
    let cfg = dbc::load_config(&a.dbc_config.to_account_info()).ok_or(LaunchError::DbcAccountInvalid)?;
    require_keys_eq!(pool.base_mint, mint_key, LaunchError::DbcAccountInvalid);
    require_keys_eq!(pool.config, a.dbc_config.key(), LaunchError::DbcAccountInvalid);
    require!(pool.pool_type == DBC_TOKEN_TYPE_SPL, LaunchError::DbcAccountInvalid);
    require_keys_eq!(pool.creator, a.creator.key(), LaunchError::DbcCreatorMismatch);
    require!(!pool.is_migrated && !pool.graduated(), LaunchError::DbcAlreadyGraduated);

    // Launch parameters: decimals come from the mint, threshold from DBC's config.
    let lp = LaunchParams {
        decimals: a.mint.decimals,
        ratio_whole_tokens: params.ratio_whole_tokens,
        collection_size: params.collection_size,
        graduation_threshold_lamports: cfg.migration_quote_threshold,
    };
    let amounts = validate(&lp)?;

    let (buffer, _) = dbc::buffer_authority();
    require_keys_eq!(buffer, a.buffer_authority.key(), LaunchError::DbcConfigRejected);
    require!(
        cfg.quote_mint == WRAPPED_SOL_MINT
            && cfg.token_type == DBC_TOKEN_TYPE_SPL
            && cfg.token_decimal == a.mint.decimals
            && cfg.fixed_token_supply
            && cfg.pre_migration_token_supply == amounts.total_supply_base
            && cfg.post_migration_token_supply == amounts.total_supply_base
            && cfg.token_update_authority == DBC_TOKEN_AUTHORITY_IMMUTABLE
            && cfg.leftover_receiver == buffer,
        LaunchError::DbcConfigRejected
    );
    require!(
        a.mint.mint_authority.is_none()
            && a.mint.freeze_authority.is_none()
            && a.mint.supply == amounts.total_supply_base,
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

    let bump = ctx.bumps.launch_config;
    let cfg_key = ctx.accounts.dbc_config.key();
    let pool_key = ctx.accounts.dbc_pool.key();
    let creator = ctx.accounts.creator.key();
    // QA-FEE-03: the stored fee must be exactly the tier (shared derivation), never 0 or off-tier.
    require!(crate::validation::is_exact_tier_fee(amounts.fee_lamports, params.ratio_whole_tokens), LaunchError::FeeNotTier);
    ctx.accounts.launch_config.set_inner(LaunchConfig {
        version: LAUNCH_CONFIG_VERSION,
        bump,
        mint_authority_bump: 0,
        creator,
        mint: mint_key,
        launch_destination: pool.base_vault,
        launch_vault: DBC_POOL_AUTHORITY,
        launch_vault_bump: 0,
        decimals: lp.decimals,
        total_supply_base: amounts.total_supply_base,
        ratio_whole_tokens: params.ratio_whole_tokens,
        ratio_base: amounts.ratio_base,
        collection_size: params.collection_size,
        max_tokens_in_nft_form: amounts.max_tokens_in_nft_form,
        fee_lamports: amounts.fee_lamports,
        fee_recipient: PLATFORM_FEE_RECIPIENT,
        graduation_threshold_lamports: cfg.migration_quote_threshold,
        graduation_slice_pct: amounts.graduation_slice_pct,
        launched_at: Clock::get()?.unix_timestamp,
        dbc_config: cfg_key,
        dbc_pool: pool_key,
    });
    msg!("hybrid_launch: registered DBC pool {} mint {} ratio {} size {}", pool_key, mint_key, params.ratio_whole_tokens, params.collection_size);
    Ok(())
}
