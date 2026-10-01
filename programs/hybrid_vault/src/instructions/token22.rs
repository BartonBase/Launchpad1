//! Mode 4. Token-2022 transfer tax, locked at launch.
//!
//! Wrap locks `ratio` tokens and mints the next project NFT. It does not burn.
//! `harvest_tax` moves withheld fees into a program treasury.
//! `harvest_tax` and `buyback` freeze one exact share per NFT that already exists.
//! `claim_tax` pays only the next NFT in that frozen order, to that NFT's current owner.
//! The caller cannot choose the NFT, the amount, or the wallet. There is no admin.
//! There is no unwrap and no instruction that changes the tax rate.

use crate::{
    asset_source, config, constants::*, core_cpi, error::VaultError, state::TaxShare,
    state::TaxVault,
};
use anchor_lang::{prelude::*, InstructionData, ToAccountMetas};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token_2022::Token2022,
};
use hybrid_launch::T22BurnLaunchConfig;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Token22InitParams {
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub collection_name: String,
    pub collection_uri: String,
}

#[derive(Accounts)]
pub struct InitToken22Vault<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [hybrid_launch::LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        has_one = mint,
        constraint = launch_config.creator == creator.key() @ VaultError::NotCreator,
        constraint = launch_config.tax_authority == tax_authority.key() @ VaultError::UnsupportedLaunchConfig,
    )]
    pub launch_config: Box<Account<'info, T22BurnLaunchConfig>>,

    /// CHECK: Token-2022 mint with a locked transfer fee. Checked below.
    pub mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + TaxVault::INIT_SPACE,
        seeds = [VAULT_SEED, launch_config.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, TaxVault>>,

    /// CHECK: collection authority. Does not sign token withdrawals.
    #[account(mut, seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: `["tax_authority", mint]`. Only this PDA can withdraw withheld tax.
    #[account(seeds = [TAX_AUTHORITY_SEED, mint.key().as_ref()], bump)]
    pub tax_authority: UncheckedAccount<'info>,

    /// CHECK: ATA(vault_authority, mint). Locked wrap principal. No instruction moves it out.
    #[account(mut)]
    pub lock_tokens: UncheckedAccount<'info>,

    /// CHECK: ATA(tax_authority, mint). Tax paid to NFT holders leaves from here only.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: Core collection created here.
    #[account(mut, seeds = [COLLECTION_SEED, vault.key().as_ref()], bump)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_token22_vault(
    ctx: Context<InitToken22Vault>,
    params: Token22InitParams,
) -> Result<()> {
    require_keys_eq!(
        *ctx.accounts.mint.owner,
        anchor_spl::token_2022::ID,
        VaultError::ExtensionsNotAllowed
    );
    let tax = {
        let data = ctx.accounts.mint.try_borrow_data()?;
        hybrid_launch::t22::read_tax_mint(&data)
            .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
    };
    require!(
        tax.mint_authority_none
            && tax.freeze_authority_none
            && tax.config_authority_none
            && tax.tax_bps == ctx.accounts.launch_config.tax_bps
            && tax.withdraw_authority == ctx.accounts.tax_authority.key(),
        VaultError::MintMismatch
    );
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
    let econ = config::t22_econ(&ctx.accounts.launch_config)?;
    require!(
        ctx.accounts.launch_config.launch_mode == hybrid_launch::LAUNCH_MODE_TOKEN22,
        VaultError::UnsupportedLaunchConfig
    );
    let token_program_id = ctx.accounts.token_program.key();
    let mint_key = ctx.accounts.mint.key();
    require_keys_eq!(
        ctx.accounts.lock_tokens.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.vault_authority.key(),
            &mint_key,
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    require_keys_eq!(
        ctx.accounts.treasury.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.tax_authority.key(),
            &mint_key,
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    associated_token::create_idempotent(CpiContext::new(
        ctx.accounts.associated_token_program.key(),
        associated_token::Create {
            payer: ctx.accounts.creator.to_account_info(),
            associated_token: ctx.accounts.lock_tokens.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;
    associated_token::create_idempotent(CpiContext::new(
        ctx.accounts.associated_token_program.key(),
        associated_token::Create {
            payer: ctx.accounts.creator.to_account_info(),
            associated_token: ctx.accounts.treasury.to_account_info(),
            authority: ctx.accounts.tax_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

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
    v.version = TAX_VAULT_VERSION;
    v.bump = ctx.bumps.vault;
    v.authority_bump = ctx.bumps.vault_authority;
    v.tax_authority_bump = ctx.bumps.tax_authority;
    v.collection_bump = collection_bump;
    v.launch_mode = hybrid_launch::LAUNCH_MODE_TOKEN22;
    v.launch_config = cfg.key();
    v.mint = cfg.mint;
    v.creator = cfg.creator;
    v.collection = ctx.accounts.collection.key();
    v.trait_root = params.trait_root;
    v.trait_schema_hash = params.trait_schema_hash;
    v.minted_count = 0;
    v.collection_size = econ.collection_size;
    v.payout_cursor = 0;
    v.round_minted = 0;
    v.round_share = 0;
    v.round_open = 0;
    v.credited_base = 0;
    v.paid_base = 0;
    v.pending_base = 0;
    v.buyback_lamports_per_whole = cfg.buyback_lamports_per_whole;
    v.total_supply_base = cfg.total_supply_base;
    v.decimals = cfg.decimals;
    v.total_locked_base = 0;
    v.total_fee_lamports = 0;
    v.total_wraps = 0;
    v.tax_bps = cfg.tax_bps;
    v.kind = KIND_SPLIT;
    let _ = econ;
    Ok(())
}

#[derive(Accounts)]
pub struct WrapToken22<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    #[account(
        seeds = [hybrid_launch::LAUNCH_CONFIG_SEED, vault.mint.as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        constraint = launch_config.key() == vault.launch_config @ VaultError::UnsupportedLaunchConfig,
        constraint = launch_config.mint == vault.mint @ VaultError::MintMismatch,
    )]
    pub launch_config: Box<Account<'info, T22BurnLaunchConfig>>,

    /// CHECK: address-pinned to `vault.mint` (the Token-2022 mint this vault was opened for).
    #[account(mut, address = vault.mint @ VaultError::MintMismatch)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: the user's Token-2022 ATA.
    #[account(mut)]
    pub user_token: UncheckedAccount<'info>,

    /// CHECK: lock ATA. Receives the locked tokens. Not burned.
    #[account(mut)]
    pub lock_tokens: UncheckedAccount<'info>,

    /// CHECK: platform fee recipient. Not caller-chosen.
    #[account(mut, address = hybrid_launch::PLATFORM_FEE_RECIPIENT @ VaultError::FeeRecipientNotPlatform)]
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: collection authority. Signs the NFT mint only.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: this vault's collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: asset PDA for `minted_count`.
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + TaxShare::INIT_SPACE,
        seeds = [TAX_SHARE_SEED, vault.key().as_ref(), asset.key().as_ref()],
        bump
    )]
    pub tax_share: Box<Account<'info, TaxShare>>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_wrap_token22(ctx: Context<WrapToken22>, args: asset_source::MintArgs) -> Result<()> {
    let econ = config::t22_econ(&ctx.accounts.launch_config)?;
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
        get_associated_token_address_with_program_id(
            &ctx.accounts.user.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    require_keys_eq!(
        ctx.accounts.lock_tokens.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.vault_authority.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    let supply_before = {
        let data = ctx.accounts.mint.try_borrow_data()?;
        let mint = hybrid_launch::t22::read_tax_mint(&data)
            .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?;
        require!(
            mint.mint_authority_none
                && mint.freeze_authority_none
                && mint.config_authority_none
                && mint.tax_bps == ctx.accounts.vault.tax_bps,
            VaultError::MintMismatch
        );
        mint.supply
    };
    let ratio = econ.ratio_base;
    let fee_sol = econ.request_fee()?;
    let epoch = Clock::get()?.epoch;
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let transfer_fee = hybrid_launch::t22::transfer_fee(&mint_data, epoch, ratio)
        .map_err(|_| error!(VaultError::MathOverflow))?;
    let decimals = ctx.accounts.launch_config.decimals;
    drop(mint_data);
    anchor_spl::token_2022_extensions::transfer_fee::transfer_checked_with_fee(
        CpiContext::new(
            token_program_id,
            anchor_spl::token_2022_extensions::transfer_fee::TransferCheckedWithFee {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                source: ctx.accounts.user_token.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                destination: ctx.accounts.lock_tokens.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        ratio,
        decimals,
        transfer_fee,
    )?;
    crate::instructions::vault_token_ops::sol_fee(
        &ctx.accounts.system_program,
        &ctx.accounts.user,
        &ctx.accounts.fee_recipient.to_account_info(),
        fee_sol,
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

    let supply_after = hybrid_launch::t22::read_tax_mint(&ctx.accounts.mint.try_borrow_data()?)
        .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
        .supply;
    require!(
        supply_after == supply_before,
        VaultError::AssetAccountingBroken
    );

    // The payout is not stored here. The vault cursor pays NFT 0, then 1, then 2.
    ctx.accounts.tax_share.set_inner(TaxShare {
        vault: vault_key,
        asset: ctx.accounts.asset.key(),
        index: 0,
    });
    let v = &mut ctx.accounts.vault;
    v.minted_count = index.checked_add(1).ok_or(VaultError::MathOverflow)?;
    v.total_locked_base = v
        .total_locked_base
        .checked_add(ratio)
        .ok_or(VaultError::MathOverflow)?;
    v.total_fee_lamports = v
        .total_fee_lamports
        .checked_add(fee_sol)
        .ok_or(VaultError::MathOverflow)?;
    v.total_wraps = v
        .total_wraps
        .checked_add(1)
        .ok_or(VaultError::MathOverflow)?;
    Ok(())
}

#[derive(Accounts)]
pub struct HarvestTax<'info> {
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    /// CHECK: address-pinned to `vault.mint` (the Token-2022 mint this vault was opened for).
    #[account(mut, address = vault.mint @ VaultError::MintMismatch)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: ATA(tax_authority, mint). Not caller-chosen.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: withdraw authority. Signs only this withdrawal into `treasury`.
    #[account(seeds = [TAX_AUTHORITY_SEED, vault.mint.as_ref()], bump = vault.tax_authority_bump)]
    pub tax_authority: UncheckedAccount<'info>,

    /// CHECK: a token account of this mint. Its withheld tax is moved to the mint, then withdrawn.
    #[account(mut)]
    pub source: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handle_harvest_tax(ctx: Context<HarvestTax>) -> Result<()> {
    let token_program_id = ctx.accounts.token_program.key();
    require_keys_eq!(
        ctx.accounts.treasury.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.tax_authority.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    let before = hybrid_launch::t22::read_token_account(&ctx.accounts.treasury.try_borrow_data()?)
        .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
        .amount;

    let ix = anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::instruction::harvest_withheld_tokens_to_mint(
        &token_program_id,
        &ctx.accounts.mint.key(),
        &[ctx.accounts.source.key],
    )?;
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.source.to_account_info(),
        ],
    )?;

    let bump = ctx.accounts.vault.tax_authority_bump;
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[TAX_AUTHORITY_SEED, mint_key.as_ref(), &[bump]];
    let ix = anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::instruction::withdraw_withheld_tokens_from_mint(
        &token_program_id,
        &mint_key,
        &ctx.accounts.treasury.key(),
        &ctx.accounts.tax_authority.key(),
        &[],
    )?;
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.tax_authority.to_account_info(),
        ],
        &[seeds],
    )?;

    let after = hybrid_launch::t22::read_token_account(&ctx.accounts.treasury.try_borrow_data()?)
        .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
        .amount;
    let gained = after.checked_sub(before).ok_or(VaultError::MathOverflow)?;
    accrue(&mut ctx.accounts.vault, gained)?;
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimTax<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: current owner of `asset`. Destination is their ATA, not a caller-chosen account.
    pub holder: UncheckedAccount<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    /// CHECK: address-pinned to `vault.mint` (the Token-2022 mint this vault was opened for).
    #[account(mut, address = vault.mint @ VaultError::MintMismatch)]
    pub mint: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [TAX_SHARE_SEED, vault.key().as_ref(), asset.key().as_ref()],
        bump,
        constraint = tax_share.vault == vault.key() @ VaultError::UnsupportedLaunchConfig,
        constraint = tax_share.asset == asset.key() @ VaultError::WrongAsset,
    )]
    pub tax_share: Box<Account<'info, TaxShare>>,

    /// CHECK: a Core NFT of this vault's collection.
    pub asset: UncheckedAccount<'info>,

    /// CHECK: treasury ATA.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: ATA(holder, mint). Created here if needed.
    #[account(mut)]
    pub holder_token: UncheckedAccount<'info>,

    /// CHECK: tax PDA. Signs the payout only.
    #[account(seeds = [TAX_AUTHORITY_SEED, vault.mint.as_ref()], bump = vault.tax_authority_bump)]
    pub tax_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_claim_tax(ctx: Context<ClaimTax>) -> Result<()> {
    require!(
        ctx.accounts.vault.kind == KIND_SPLIT,
        VaultError::WrongRequestKind
    );
    core_cpi::assert_asset_state(
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.vault.collection,
        &ctx.accounts.holder.key(),
    )?;
    let token_program_id = ctx.accounts.token_program.key();
    require_keys_eq!(
        ctx.accounts.holder_token.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.holder.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    require_keys_eq!(
        ctx.accounts.treasury.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.tax_authority.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    associated_token::create_idempotent(CpiContext::new(
        ctx.accounts.associated_token_program.key(),
        associated_token::Create {
            payer: ctx.accounts.payer.to_account_info(),
            associated_token: ctx.accounts.holder_token.to_account_info(),
            authority: ctx.accounts.holder.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    // No round is frozen: there is nothing to pay. A later NFT cannot pull an old round.
    if ctx.accounts.vault.round_open == 0 {
        return Ok(());
    }
    let cursor = ctx.accounts.vault.payout_cursor;
    let (expected_asset, _) = Pubkey::find_program_address(
        &[
            ASSET_SEED,
            ctx.accounts.vault.key().as_ref(),
            &cursor.to_le_bytes(),
        ],
        &crate::ID,
    );
    // The caller cannot skip ahead, pay themselves, or pick a different NFT.
    require_keys_eq!(
        ctx.accounts.asset.key(),
        expected_asset,
        VaultError::OutOfOrder
    );
    let owed = ctx.accounts.vault.round_share;
    let treasury_bal =
        hybrid_launch::t22::read_token_account(&ctx.accounts.treasury.try_borrow_data()?)
            .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
            .amount;
    let room = ctx
        .accounts
        .vault
        .credited_base
        .checked_sub(ctx.accounts.vault.paid_base)
        .ok_or(VaultError::MathOverflow)?;
    require!(
        owed > 0 && owed <= treasury_bal && owed <= room,
        VaultError::DistributionExceeded
    );
    ctx.accounts.vault.paid_base = ctx
        .accounts
        .vault
        .paid_base
        .checked_add(owed)
        .ok_or(VaultError::MathOverflow)?;
    ctx.accounts.vault.payout_cursor = cursor.checked_add(1).ok_or(VaultError::MathOverflow)?;
    if ctx.accounts.vault.payout_cursor == ctx.accounts.vault.round_minted {
        ctx.accounts.vault.round_open = 0;
        ctx.accounts.vault.round_share = 0;
    }
    let epoch = Clock::get()?.epoch;
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let transfer_fee = hybrid_launch::t22::transfer_fee(&mint_data, epoch, owed)
        .map_err(|_| error!(VaultError::MathOverflow))?;
    let decimals = hybrid_launch::t22::read_tax_mint(&mint_data)
        .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
        .decimals;
    drop(mint_data);
    let bump = ctx.accounts.vault.tax_authority_bump;
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[TAX_AUTHORITY_SEED, mint_key.as_ref(), &[bump]];
    anchor_spl::token_2022_extensions::transfer_fee::transfer_checked_with_fee(
        CpiContext::new_with_signer(
            token_program_id,
            anchor_spl::token_2022_extensions::transfer_fee::TransferCheckedWithFee {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                source: ctx.accounts.treasury.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                destination: ctx.accounts.holder_token.to_account_info(),
                authority: ctx.accounts.tax_authority.to_account_info(),
            },
            &[seeds],
        ),
        owed,
        decimals,
        transfer_fee,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct Buyback<'info> {
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    #[account(
        mut,
        seeds = [hybrid_launch::LAUNCH_CONFIG_SEED, vault.mint.as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        constraint = launch_config.key() == vault.launch_config @ VaultError::UnsupportedLaunchConfig,
    )]
    pub launch_config: Box<Account<'info, hybrid_launch::T22BurnLaunchConfig>>,

    /// CHECK: address-pinned to `vault.mint` (the Token-2022 mint this vault was opened for).
    #[account(mut, address = vault.mint @ VaultError::MintMismatch)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: hybrid_launch inventory PDA. Receives the SOL.
    #[account(mut, address = launch_config.launch_vault @ VaultError::UnsupportedLaunchConfig)]
    pub launch_vault: UncheckedAccount<'info>,

    /// CHECK: inventory ATA.
    #[account(mut, address = launch_config.launch_destination @ VaultError::TokenSourceNotDerived)]
    pub launch_destination: UncheckedAccount<'info>,

    /// CHECK: tax treasury ATA.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: pays the SOL. Signs only the buy.
    #[account(mut, seeds = [TAX_AUTHORITY_SEED, vault.mint.as_ref()], bump = vault.tax_authority_bump)]
    pub tax_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    /// CHECK: hybrid_launch. Address is pinned.
    #[account(address = hybrid_launch::ID)]
    pub launch_program: UncheckedAccount<'info>,
}

pub fn handle_buyback(ctx: Context<Buyback>) -> Result<()> {
    let token_program_id = ctx.accounts.token_program.key();
    require_keys_eq!(
        ctx.accounts.treasury.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.tax_authority.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        VaultError::TokenSourceNotDerived
    );
    let rent = Rent::get()?.minimum_balance(0);
    let have = ctx.accounts.tax_authority.lamports();
    let sol_amount = have.saturating_sub(rent);
    if sol_amount == 0 {
        return Ok(());
    }
    let before = hybrid_launch::t22::read_token_account(&ctx.accounts.treasury.try_borrow_data()?)
        .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
        .amount;
    let bump = ctx.accounts.vault.tax_authority_bump;
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[TAX_AUTHORITY_SEED, mint_key.as_ref(), &[bump]];
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: hybrid_launch::ID,
        accounts: hybrid_launch::accounts::BuyInventory {
            launch_config: ctx.accounts.launch_config.key(),
            mint: mint_key,
            launch_vault: ctx.accounts.launch_vault.key(),
            launch_destination: ctx.accounts.launch_destination.key(),
            treasury: ctx.accounts.treasury.key(),
            tax_authority: ctx.accounts.tax_authority.key(),
            token_program: token_program_id,
            system_program: ctx.accounts.system_program.key(),
        }
        .to_account_metas(None),
        data: hybrid_launch::instruction::BuyInventory { sol_amount }.data(),
    };
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.launch_config.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.launch_vault.to_account_info(),
            ctx.accounts.launch_destination.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.tax_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[seeds],
    )?;
    let after = hybrid_launch::t22::read_token_account(&ctx.accounts.treasury.try_borrow_data()?)
        .map_err(|_| error!(VaultError::ExtensionsNotAllowed))?
        .amount;
    let gained = after.checked_sub(before).ok_or(VaultError::MathOverflow)?;
    accrue(&mut ctx.accounts.vault, gained)?;
    Ok(())
}

/// Freeze newly arrived tokens into one exact share per NFT that already exists.
/// An open round is never edited. A new round waits until the pot is worth at least
/// the Stonk-style market-cap tier. Unminted NFTs get nothing. There is no $20 minimum.
fn accrue(vault: &mut TaxVault, gained: u64) -> Result<()> {
    vault.pending_base = vault
        .pending_base
        .checked_add(gained)
        .ok_or(VaultError::MathOverflow)?;
    if vault.round_open == 1 {
        return Ok(());
    }
    let n = vault.minted_count as u64;
    if n == 0 || vault.pending_base == 0 {
        return Ok(());
    }
    let pot = value_lamports(
        vault.pending_base,
        vault.buyback_lamports_per_whole,
        vault.decimals,
    )
    .ok_or(VaultError::MathOverflow)?;
    let mcap = value_lamports(
        vault.total_supply_base,
        vault.buyback_lamports_per_whole,
        vault.decimals,
    )
    .ok_or(VaultError::MathOverflow)?;
    if pot < pot_threshold_lamports(mcap) {
        return Ok(());
    }
    let share = vault.pending_base / n;
    if share == 0 {
        return Ok(());
    }
    let booked = share.checked_mul(n).ok_or(VaultError::MathOverflow)?;
    vault.pending_base = vault
        .pending_base
        .checked_sub(booked)
        .ok_or(VaultError::MathOverflow)?;
    vault.credited_base = vault
        .credited_base
        .checked_add(booked)
        .ok_or(VaultError::MathOverflow)?;
    vault.round_minted = vault.minted_count;
    vault.payout_cursor = 0;
    vault.round_open = 1;
    if vault.kind == KIND_RAFFLE {
        vault.round_pot = booked;
        vault.round_share = 0;
        vault.raffle_phase = RAFFLE_SNAPSHOT;
        vault.live_seats = 0;
        vault.raffle_commits = 0;
        vault.raffle_oracles = [Pubkey::default(); 4];
        vault.raffle_deadline_slot = 0;
        vault.round_id = vault
            .round_id
            .checked_add(1)
            .ok_or(VaultError::MathOverflow)?;
        return Ok(());
    }
    vault.round_share = share;
    Ok(())
}

/// Stonk.fun pot tiers (published 2026-08-07), measured in lamports of the locked buyback price.
/// The dollar labels match at 1 SOL = $100. There is no live USD feed and no caller-supplied price.
/// Above $125k the bar is 0.1% of market cap, capped at $50,000.
pub fn pot_threshold_lamports(mcap_lamports: u128) -> u128 {
    const SOL: u128 = 1_000_000_000;
    const MCAP_50K: u128 = 500 * SOL;
    const MCAP_100K: u128 = 1_000 * SOL;
    const MCAP_125K: u128 = 1_250 * SOL;
    const POT_50: u128 = SOL / 2;
    const POT_200: u128 = 2 * SOL;
    const POT_250: u128 = 5 * SOL / 2;
    const POT_CAP: u128 = 500 * SOL;
    if mcap_lamports < MCAP_50K {
        POT_50
    } else if mcap_lamports < MCAP_100K {
        POT_200
    } else if mcap_lamports < MCAP_125K {
        POT_250
    } else {
        let pct = mcap_lamports / 1000;
        if pct > POT_CAP {
            POT_CAP
        } else {
            pct
        }
    }
}

fn value_lamports(amount_base: u64, lamports_per_whole: u64, decimals: u8) -> Option<u128> {
    let mut unit = 1u128;
    let mut d = 0u8;
    while d < decimals {
        unit = unit.checked_mul(10)?;
        d = d.checked_add(1)?;
    }
    (amount_base as u128)
        .checked_mul(lamports_per_whole as u128)?
        .checked_div(unit)
}

#[cfg(test)]
mod pot_tier_tests {
    use super::pot_threshold_lamports;

    const SOL: u128 = 1_000_000_000;

    #[test]
    fn stonk_tiers_ignore_a_holder_minimum() {
        assert_eq!(pot_threshold_lamports(0), SOL / 2);
        assert_eq!(pot_threshold_lamports(499 * SOL), SOL / 2);
        assert_eq!(pot_threshold_lamports(500 * SOL), 2 * SOL);
        assert_eq!(pot_threshold_lamports(999 * SOL), 2 * SOL);
        assert_eq!(pot_threshold_lamports(1_000 * SOL), 5 * SOL / 2);
        assert_eq!(pot_threshold_lamports(1_249 * SOL), 5 * SOL / 2);
        assert_eq!(pot_threshold_lamports(1_250 * SOL), 1_250 * SOL / 1000);
        assert_eq!(pot_threshold_lamports(10_000 * SOL), 10 * SOL);
        assert_eq!(pot_threshold_lamports(500_000 * SOL), 500 * SOL);
        assert_eq!(pot_threshold_lamports(5_000_000 * SOL), 500 * SOL);
    }
}
