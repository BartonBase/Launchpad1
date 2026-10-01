//! Mode 3 (burn hybrid). Same collection and ratio as Mode 2, but the wrap burns the tokens and
//! mints the NFT in one instruction. There is no unwrap, re-roll, expire, or other release.
//!
//! The NFT index is `minted_count`, not a Switchboard draw. Burning before a later reveal would
//! destroy tokens that no instruction can return. The user cannot choose the index.

use crate::{
    asset_source, config, constants::*, core_cpi, error::VaultError, graduation,
    state::PermanentVault,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{self, Burn, Mint, Token, TokenAccount},
};
use hybrid_launch::BurnLaunchConfig;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PermanentInitParams {
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub collection_name: String,
    pub collection_uri: String,
}

#[derive(Accounts)]
pub struct InitPermanentVault<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [hybrid_launch::LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        has_one = mint,
        constraint = launch_config.creator == creator.key() @ VaultError::NotCreator,
    )]
    pub launch_config: Box<Account<'info, BurnLaunchConfig>>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        space = 8 + PermanentVault::INIT_SPACE,
        seeds = [VAULT_SEED, launch_config.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, PermanentVault>>,

    /// CHECK: data-less PDA. Collection update authority. Nothing here signs it to move tokens.
    #[account(mut, seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: PDA created here by the Core CPI.
    #[account(mut, seeds = [COLLECTION_SEED, vault.key().as_ref()], bump)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_permanent_vault(
    ctx: Context<InitPermanentVault>,
    params: PermanentInitParams,
) -> Result<()> {
    require!(
        params.collection_name.len() <= MAX_NAME_LEN,
        VaultError::MetadataTooLong
    );
    require!(
        params.collection_uri.len() <= MAX_URI_LEN,
        VaultError::MetadataTooLong
    );
    require!(
        asset_source::is_content_addressed(&params.collection_uri),
        VaultError::UriNotContentAddressed
    );
    let econ = config::burn_econ(&ctx.accounts.launch_config)?;
    require!(
        econ.collection_size > 0,
        VaultError::UnsupportedLaunchConfig
    );

    let vault_key = ctx.accounts.vault.key();
    let collection_bump = ctx.bumps.collection;
    let collection_seeds: &[&[u8]] = &[COLLECTION_SEED, vault_key.as_ref(), &[collection_bump]];
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

    let cfg = &ctx.accounts.launch_config;
    let v = &mut ctx.accounts.vault;
    v.version = PERMANENT_VAULT_VERSION;
    v.bump = ctx.bumps.vault;
    v.authority_bump = ctx.bumps.vault_authority;
    v.collection_bump = collection_bump;
    v.open = false;
    v.launch_mode = hybrid_launch::LAUNCH_MODE_BURN;
    v.launch_config = cfg.key();
    v.mint = cfg.mint;
    v.creator = cfg.creator;
    v.collection = ctx.accounts.collection.key();
    v.trait_root = params.trait_root;
    v.trait_schema_hash = params.trait_schema_hash;
    v.minted_count = 0;
    v.total_burned_base = 0;
    v.total_fee_lamports = 0;
    v.total_wraps = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct OpenPermanentVault<'info> {
    pub caller: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, PermanentVault>>,

    #[account(address = vault.launch_config)]
    pub launch_config: Box<Account<'info, BurnLaunchConfig>>,

    /// CHECK: the vault's Core collection.
    #[account(address = vault.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: interpreted only by graduation::verify_recorded.
    pub graduation_proof: UncheckedAccount<'info>,
}

pub fn handle_open_permanent_vault(ctx: Context<OpenPermanentVault>) -> Result<()> {
    let econ = config::burn_econ(&ctx.accounts.launch_config)?;
    let v = &ctx.accounts.vault;
    require!(!v.open, VaultError::VaultAlreadyOpen);
    {
        let c = &ctx.accounts.collection;
        require_keys_eq!(*c.owner, MPL_CORE_ID, VaultError::AssetStateMismatch);
        let data = c.try_borrow_data()?;
        let col = mpl_core::accounts::BaseCollectionV1::from_bytes(&data)
            .map_err(|_| error!(VaultError::AssetStateMismatch))?;
        let (va, _) =
            Pubkey::find_program_address(&[VAULT_AUTHORITY_SEED, v.key().as_ref()], &crate::ID);
        require!(
            col.update_authority.to_bytes() == va.to_bytes(),
            VaultError::AssetStateMismatch
        );
        require!(
            col.num_minted == v.minted_count && col.current_size == v.minted_count,
            VaultError::AssetAccountingBroken
        );
    }
    graduation::verify_recorded(
        &ctx.accounts.launch_config.mint,
        &ctx.accounts.launch_config.dbc_config,
        &ctx.accounts.launch_config.dbc_pool,
        &ctx.accounts.graduation_proof.to_account_info(),
    )?;
    require!(
        econ.collection_size > 0,
        VaultError::UnsupportedLaunchConfig
    );
    let v = &mut ctx.accounts.vault;
    v.open = true;
    Ok(())
}

#[derive(Accounts)]
pub struct WrapPermanent<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, PermanentVault>>,

    #[account(
        seeds = [hybrid_launch::LAUNCH_CONFIG_SEED, vault.mint.as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        constraint = launch_config.key() == vault.launch_config @ VaultError::UnsupportedLaunchConfig,
        constraint = launch_config.mint == vault.mint @ VaultError::MintMismatch,
    )]
    pub launch_config: Box<Account<'info, BurnLaunchConfig>>,

    #[account(mut, address = vault.mint @ VaultError::MintMismatch)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
        token::token_program = token_program,
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    /// CHECK: PLATFORM_FEE_RECIPIENT. Not a caller-chosen address.
    #[account(mut, address = hybrid_launch::PLATFORM_FEE_RECIPIENT @ VaultError::FeeRecipientNotPlatform)]
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: PDA. Signs the Core create. Does not hold the burned tokens.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: this vault's collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: asset PDA for `minted_count`. The user does not choose the index.
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_wrap_permanent(
    ctx: Context<WrapPermanent>,
    args: asset_source::MintArgs,
) -> Result<()> {
    let econ = config::burn_econ(&ctx.accounts.launch_config)?;
    require!(ctx.accounts.vault.open, VaultError::VaultNotOpen);
    let index = ctx.accounts.vault.minted_count;
    require!(
        (index as u32) < econ.collection_size,
        VaultError::NoAssetAvailable
    );
    asset_source::check_leaf(
        &ctx.accounts.vault.trait_root,
        &ctx.accounts.vault.trait_schema_hash,
        &ctx.accounts.launch_config.key(),
        econ.collection_size,
        index,
        &args.leaf,
        &args.proof,
    )?;
    let token_program_id = ctx.accounts.token_program.key();
    require_keys_eq!(
        ctx.accounts.user_token.key(),
        get_associated_token_address(&ctx.accounts.user.key(), &ctx.accounts.mint.key()),
        VaultError::TokenSourceNotDerived
    );
    require!(
        ctx.accounts.mint.mint_authority.is_none() && ctx.accounts.mint.freeze_authority.is_none(),
        VaultError::MintMismatch
    );
    let supply_before = ctx.accounts.mint.supply;
    let ratio = econ.ratio_base;
    let fee = econ.request_fee()?;

    token::burn(
        CpiContext::new(
            token_program_id,
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        ratio,
    )?;
    crate::instructions::vault_token_ops::sol_fee(
        &ctx.accounts.system_program,
        &ctx.accounts.user,
        &ctx.accounts.fee_recipient.to_account_info(),
        fee,
    )?;

    let vault_key = ctx.accounts.vault.key();
    let idx = index.to_le_bytes();
    let (expected, asset_bump) = asset_source::asset_address(&vault_key, index);
    require_keys_eq!(ctx.accounts.asset.key(), expected, VaultError::WrongAsset);
    let asset_seeds: &[&[u8]] = &[ASSET_SEED, vault_key.as_ref(), &idx, &[asset_bump]];
    let auth_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.accounts.vault.authority_bump],
    ];
    asset_source::drain_prefunded_pda(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        asset_seeds,
    )?;
    core_cpi::create_asset_user_pays(
        &ctx.accounts.mpl_core_program.to_account_info(),
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.vault_authority.to_account_info(),
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        asset_source::asset_name(index),
        args.leaf.uri,
        asset_seeds,
        auth_seeds,
    )?;
    core_cpi::assert_asset_state(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.key(),
        &ctx.accounts.user.key(),
    )?;

    ctx.accounts.mint.reload()?;
    require!(
        ctx.accounts.mint.supply
            == supply_before
                .checked_sub(ratio)
                .ok_or(VaultError::MathOverflow)?,
        VaultError::AssetAccountingBroken
    );
    require!(
        ctx.accounts.mint.mint_authority.is_none() && ctx.accounts.mint.freeze_authority.is_none(),
        VaultError::MintMismatch
    );

    let v = &mut ctx.accounts.vault;
    v.minted_count = index.checked_add(1).ok_or(VaultError::MathOverflow)?;
    v.total_burned_base = v
        .total_burned_base
        .checked_add(ratio)
        .ok_or(VaultError::MathOverflow)?;
    v.total_fee_lamports = v
        .total_fee_lamports
        .checked_add(fee)
        .ok_or(VaultError::MathOverflow)?;
    v.total_wraps = v
        .total_wraps
        .checked_add(1)
        .ok_or(VaultError::MathOverflow)?;
    Ok(())
}
