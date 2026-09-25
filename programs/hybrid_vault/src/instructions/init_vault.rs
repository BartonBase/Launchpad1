//! `init_vault`: bind a vault 1:1 to a hybrid_launch LaunchConfig (program-owned PDA of the mint,
//! version 2, economics re-validated by config::econ), create the vault's token account and Core
//! collection, commit the trait root, pin the Switchboard queue. Economics are NOT copied: every
//! instruction reads them from the LaunchConfig (M-01). The vault starts CLOSED (`open == false`).
//! No instruction ever changes the vault's fields listed here and there is no close instruction.

use crate::{asset_source, config, constants::*, core_cpi, error::VaultError, pool, state::Vault};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use hybrid_launch::{LaunchConfig, LAUNCH_CONFIG_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitVaultParams {
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub collection_name: String,
    pub collection_uri: String,
    /// Switchboard On-Demand queue every randomness account must use.
    pub sb_queue: Pubkey,
}

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        has_one = mint,
        constraint = launch_config.creator == creator.key() @ VaultError::NotCreator,
    )]
    pub launch_config: Box<Account<'info, LaunchConfig>>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(init, payer = creator, space = 8 + Vault::INIT_SPACE, seeds = [VAULT_SEED, launch_config.key().as_ref()], bump)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: data-less PDA; owns vault tokens and pooled NFTs; collection update authority; holds the
    /// graduation fund (drained pre-fund lamports land here). `mut` only for that drain.
    #[account(mut, seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: data-less PDA; the authority of every randomness account used by this vault.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub randomness_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = vault_authority,
        token::token_program = token_program,
    )]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: pre-created top-level, owned by this program, zeroed, sized; validated in pool::init
    /// (which also refuses an already-initialised pool).
    #[account(mut, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: PDA; created here by the Core CPI.
    #[account(mut, seeds = [COLLECTION_SEED, vault.key().as_ref()], bump)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_vault(ctx: Context<InitVault>, params: InitVaultParams) -> Result<()> {
    let cfg = &ctx.accounts.launch_config;
    require!(params.collection_name.len() <= MAX_NAME_LEN, VaultError::MetadataTooLong);
    require!(params.collection_uri.len() <= MAX_URI_LEN, VaultError::MetadataTooLong);
    require!(asset_source::is_content_addressed(&params.collection_uri), VaultError::UriNotContentAddressed);
    require!(params.sb_queue != Pubkey::default(), VaultError::WrongQueue);
    let econ = config::econ(cfg)?;
    let collection_size = econ.collection_size;

    let vault_key = ctx.accounts.vault.key();
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        pool::init(&mut data, &vault_key, collection_size)?;
    }

    let collection_bump = ctx.bumps.collection;
    let collection_seeds: &[&[u8]] = &[COLLECTION_SEED, vault_key.as_ref(), &[collection_bump]];
    // A pre-funded collection PDA would block Core's create forever (T-GRAD-01): drain it into the
    // graduation fund (vault_authority PDA) and create in the same instruction.
    asset_source::drain_prefunded_pda(
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.vault_authority.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        collection_seeds,
    )?;
    core_cpi::create_collection(
        &ctx.accounts.mpl_core_program.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.vault_authority.to_account_info(),
        &ctx.accounts.creator.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        params.collection_name,
        params.collection_uri,
        collection_seeds,
    )?;

    let v = &mut ctx.accounts.vault;
    v.version = VAULT_VERSION;
    v.bump = ctx.bumps.vault;
    v.authority_bump = ctx.bumps.vault_authority;
    v.randomness_authority_bump = ctx.bumps.randomness_authority;
    v.vault_tokens_bump = ctx.bumps.vault_tokens;
    v.collection_bump = collection_bump;
    v.open = false;
    v.launch_config = cfg.key();
    v.mint = cfg.mint;
    v.creator = cfg.creator;
    v.collection = ctx.accounts.collection.key();
    v.pool = ctx.accounts.pool.key();
    v.vault_tokens = ctx.accounts.vault_tokens.key();
    v.sb_queue = params.sb_queue;
    v.trait_root = params.trait_root;
    v.trait_schema_hash = params.trait_schema_hash;
    Ok(())
}
