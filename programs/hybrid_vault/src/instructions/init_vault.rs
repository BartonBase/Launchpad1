//! `init_vault`: bind a vault 1:1 to a hybrid_launch LaunchConfig, create the vault's token accounts
//! and its Metaplex Core collection (update authority = vault PDA, no plugins), commit the trait root.

use crate::{constants::*, core_cpi, error::VaultError, pool, state::Vault};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use hybrid_launch::{LaunchConfig, FEE_DESTINATION_BURN, LAUNCH_CONFIG_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitVaultParams {
    pub trait_root: [u8; 32],
    pub collection_name: String,
    pub collection_uri: String,
    /// Pubkey::default() => no guardian, no pause power at all.
    pub guardian: Pubkey,
}

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Owner = hybrid_launch (Account<> checks owner + discriminator), canonical PDA of the mint.
    #[account(
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        has_one = mint,
        constraint = launch_config.creator == creator.key() @ VaultError::NotCreator,
    )]
    pub launch_config: Box<Account<'info, LaunchConfig>>,

    /// Classic SPL Token mint (Account<Mint> from anchor_spl::token checks the classic owner).
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        space = 8 + Vault::INIT_SPACE,
        seeds = [VAULT_SEED, launch_config.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: data-less PDA; owns vault tokens, fee escrow and pooled NFTs; collection update authority.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: data-less PDA; the only authority allowed on randomness accounts used by this vault.
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

    #[account(
        init,
        payer = creator,
        seeds = [FEE_ESCROW_SEED, vault.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = vault_authority,
        token::token_program = token_program,
    )]
    pub fee_escrow: Box<Account<'info, TokenAccount>>,

    /// CHECK: pre-created (top-level create_account, too big for CPI init), owned by this program,
    /// zeroed, sized for collection_size; validated in pool::init.
    #[account(mut, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: PDA; created here by the Core CPI (no plugins, update authority = vault_authority).
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
    require!(cfg.fee_destination == FEE_DESTINATION_BURN, VaultError::FeeDestinationNotBurn);
    require!(params.collection_name.len() <= MAX_NAME_LEN, VaultError::MetadataTooLong);
    require!(params.collection_uri.len() <= MAX_URI_LEN, VaultError::MetadataTooLong);
    let collection_size: u32 = cfg.collection_size.try_into().map_err(|_| error!(VaultError::MathOverflow))?;
    require!(collection_size > 0, VaultError::IndexOutOfRange);

    let vault_key = ctx.accounts.vault.key();
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        pool::init(&mut data, &vault_key, collection_size)?;
    }

    let collection_bump = ctx.bumps.collection;
    let collection_seeds: &[&[u8]] = &[COLLECTION_SEED, vault_key.as_ref(), &[collection_bump]];
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
    v.version = 1;
    v.bump = ctx.bumps.vault;
    v.authority_bump = ctx.bumps.vault_authority;
    v.randomness_authority_bump = ctx.bumps.randomness_authority;
    v.vault_tokens_bump = ctx.bumps.vault_tokens;
    v.fee_escrow_bump = ctx.bumps.fee_escrow;
    v.collection_bump = collection_bump;
    v.sealed = false;
    v.launch_config = cfg.key();
    v.mint = cfg.mint;
    v.creator = cfg.creator;
    v.guardian = params.guardian;
    v.collection = ctx.accounts.collection.key();
    v.pool = ctx.accounts.pool.key();
    v.vault_tokens = ctx.accounts.vault_tokens.key();
    v.fee_escrow = ctx.accounts.fee_escrow.key();
    v.trait_root = params.trait_root;
    v.collection_size = collection_size;
    v.ratio_base = cfg.ratio_base;
    v.capture_fee_amount = cfg.capture_fee_amount;
    v.reroll_fee_amount = cfg.reroll_fee_amount;
    Ok(())
}
