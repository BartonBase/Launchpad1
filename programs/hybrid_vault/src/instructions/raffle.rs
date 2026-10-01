//! Mode 5. One NFT wins the whole round.
//!
//! The pot uses the same market-cap tiers as Mode 4. When it opens, every NFT that already
//! exists is snapshotted, owner included, before Switchboard is asked for a number. The winner
//! is that snapshotted owner. Buying the NFT after the draw does not move the prize.

use crate::{
    asset_source, config,
    constants::*,
    core_cpi,
    error::VaultError,
    randomness::{self, RevealArgs},
    selection::{self, uniform_below},
    state::{RaffleSeat, RandLock, TaxVault},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token_2022::Token2022,
};
use hybrid_launch::T22BurnLaunchConfig;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RaffleInitParams {
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub collection_name: String,
    pub collection_uri: String,
    pub sb_queue: Pubkey,
}

#[derive(Accounts)]
pub struct InitRaffleVault<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [hybrid_launch::LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        seeds::program = hybrid_launch::ID,
        bump = launch_config.bump,
        has_one = mint,
        constraint = launch_config.creator == creator.key() @ VaultError::NotCreator,
        constraint = launch_config.tax_authority == tax_authority.key() @ VaultError::UnsupportedLaunchConfig,
        constraint = launch_config.launch_mode == hybrid_launch::LAUNCH_MODE_RAFFLE @ VaultError::UnsupportedLaunchConfig,
    )]
    pub launch_config: Box<Account<'info, T22BurnLaunchConfig>>,

    /// CHECK: Token-2022 mint with a locked transfer fee.
    pub mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + TaxVault::INIT_SPACE,
        seeds = [VAULT_SEED, launch_config.key().as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, TaxVault>>,

    /// CHECK: collection authority.
    #[account(mut, seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: signs Switchboard commits and reveals. Not a person.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump)]
    pub randomness_authority: UncheckedAccount<'info>,

    /// CHECK: tax withdraw authority.
    #[account(seeds = [TAX_AUTHORITY_SEED, mint.key().as_ref()], bump)]
    pub tax_authority: UncheckedAccount<'info>,

    /// CHECK: locked wrap principal.
    #[account(mut)]
    pub lock_tokens: UncheckedAccount<'info>,

    /// CHECK: raffle pot.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: Core collection created here.
    #[account(mut, seeds = [COLLECTION_SEED, vault.key().as_ref()], bump)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: the queue this vault will use for every draw. Owned by Switchboard.
    pub sb_queue: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_raffle_vault(
    ctx: Context<InitRaffleVault>,
    params: RaffleInitParams,
) -> Result<()> {
    require_keys_eq!(
        *ctx.accounts.sb_queue.owner,
        SWITCHBOARD_PROGRAM_ID,
        VaultError::WrongQueue
    );
    require_keys_eq!(
        ctx.accounts.sb_queue.key(),
        params.sb_queue,
        VaultError::WrongQueue
    );
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
        tax.freeze_authority_none
            && tax.config_authority_none
            && tax.tax_bps == ctx.accounts.launch_config.tax_bps
            && tax.withdraw_authority == ctx.accounts.tax_authority.key(),
        VaultError::MintMismatch
    );
    require!(
        params.collection_name.len() <= MAX_NAME_LEN && params.collection_uri.len() <= MAX_URI_LEN,
        VaultError::MetadataTooLong
    );
    require!(
        asset_source::is_content_addressed(&params.collection_uri),
        VaultError::UriNotContentAddressed
    );
    let econ = config::t22_econ(&ctx.accounts.launch_config)?;
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
    v.randomness_authority_bump = ctx.bumps.randomness_authority;
    v.collection_bump = collection_bump;
    v.launch_mode = hybrid_launch::LAUNCH_MODE_RAFFLE;
    v.launch_config = cfg.key();
    v.mint = cfg.mint;
    v.creator = cfg.creator;
    v.collection = ctx.accounts.collection.key();
    v.trait_root = params.trait_root;
    v.trait_schema_hash = params.trait_schema_hash;
    v.minted_count = 0;
    v.collection_size = econ.collection_size;
    v.kind = KIND_RAFFLE;
    v.sb_queue = params.sb_queue;
    v.buyback_lamports_per_whole = cfg.buyback_lamports_per_whole;
    v.total_supply_base = cfg.total_supply_base;
    v.decimals = cfg.decimals;
    v.tax_bps = cfg.tax_bps;
    Ok(())
}

#[derive(Accounts)]
pub struct SnapshotRaffle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: current owner. Recorded now, before any random number exists.
    pub holder: UncheckedAccount<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    #[account(
        init,
        payer = payer,
        space = 8 + RaffleSeat::INIT_SPACE,
        seeds = [RAFFLE_SEAT_SEED, vault.key().as_ref(), &vault.round_id.to_le_bytes(), &vault.payout_cursor.to_le_bytes()],
        bump
    )]
    pub seat: Box<Account<'info, RaffleSeat>>,

    /// CHECK: the NFT at `payout_cursor`.
    pub asset: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_snapshot_raffle(ctx: Context<SnapshotRaffle>) -> Result<()> {
    let vault = &ctx.accounts.vault;
    require!(vault.kind == KIND_RAFFLE, VaultError::WrongRequestKind);
    require!(
        vault.raffle_phase == RAFFLE_SNAPSHOT,
        VaultError::OutOfOrder
    );
    let cursor = vault.payout_cursor;
    require!(cursor < vault.round_minted, VaultError::OutOfOrder);
    let (expected, _) = Pubkey::find_program_address(
        &[ASSET_SEED, vault.key().as_ref(), &cursor.to_le_bytes()],
        &crate::ID,
    );
    require_keys_eq!(ctx.accounts.asset.key(), expected, VaultError::OutOfOrder);
    core_cpi::assert_asset_state(
        &ctx.accounts.asset.to_account_info(),
        &vault.collection,
        &ctx.accounts.holder.key(),
    )?;
    let round_id = vault.round_id;
    ctx.accounts.seat.set_inner(RaffleSeat {
        vault: vault.key(),
        round_id,
        index: cursor,
        owner: ctx.accounts.holder.key(),
    });
    let v = &mut ctx.accounts.vault;
    v.payout_cursor = cursor.checked_add(1).ok_or(VaultError::MathOverflow)?;
    if v.payout_cursor == v.round_minted {
        v.raffle_phase = RAFFLE_READY;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct InitRaffleRandomness<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,
    /// CHECK: new keypair, created by Switchboard.
    #[account(mut)]
    pub randomness: Signer<'info>,
    /// CHECK: PDA.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_reward_escrow: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(mut, address = vault.sb_queue @ VaultError::WrongQueue)]
    pub sb_queue: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: address pinned.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = WRAPPED_SOL_MINT)]
    pub wrapped_sol_mint: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_program_state: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_lut_signer: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_lut: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = ADDRESS_LOOKUP_TABLE_PROGRAM_ID)]
    pub address_lookup_table_program: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,
}

pub fn handle_init_raffle_randomness(
    ctx: Context<InitRaffleRandomness>,
    recent_slot: u64,
) -> Result<()> {
    require!(
        ctx.accounts.vault.kind == KIND_RAFFLE,
        VaultError::WrongRequestKind
    );
    let vault_key = ctx.accounts.vault.key();
    let seeds: &[&[u8]] = &[
        RANDOMNESS_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.accounts.vault.randomness_authority_bump],
    ];
    let a = &ctx.accounts;
    let accounts = [
        a.randomness.to_account_info(),
        a.sb_reward_escrow.to_account_info(),
        a.randomness_authority.to_account_info(),
        a.sb_queue.to_account_info(),
        a.payer.to_account_info(),
        a.system_program.to_account_info(),
        a.token_program.to_account_info(),
        a.associated_token_program.to_account_info(),
        a.wrapped_sol_mint.to_account_info(),
        a.sb_program_state.to_account_info(),
        a.sb_lut_signer.to_account_info(),
        a.sb_lut.to_account_info(),
        a.address_lookup_table_program.to_account_info(),
    ];
    randomness::init_account(
        &accounts,
        recent_slot,
        &a.switchboard_program.to_account_info(),
        seeds,
    )
}

#[derive(Accounts)]
pub struct CommitRaffle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    #[account(
        init,
        payer = payer,
        space = 8 + RandLock::INIT_SPACE,
        seeds = [RAND_LOCK_SEED, randomness.key().as_ref()],
        bump
    )]
    pub rand_lock: Box<Account<'info, RandLock>>,

    /// CHECK: Switchboard randomness. Authority is this vault's PDA.
    #[account(mut)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: pinned queue.
    pub sb_queue: UncheckedAccount<'info>,
    /// CHECK: the oracle the program selects. A different one is rejected.
    #[account(mut)]
    pub sb_oracle: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = SLOT_HASHES_SYSVAR_ID)]
    pub slot_hashes: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_commit_raffle(ctx: Context<CommitRaffle>) -> Result<()> {
    let vault = &ctx.accounts.vault;
    require!(vault.kind == KIND_RAFFLE, VaultError::WrongRequestKind);
    require!(vault.raffle_phase == RAFFLE_READY, VaultError::OutOfOrder);
    let vault_key = vault.key();
    let round_id = vault.round_id;
    let seeds: &[&[u8]] = &[
        RANDOMNESS_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[vault.randomness_authority_bump],
    ];
    let seed_slot = randomness::commit_for_request(
        &ctx.accounts.randomness.to_account_info(),
        &ctx.accounts.sb_queue.to_account_info(),
        &ctx.accounts.sb_oracle.to_account_info(),
        &ctx.accounts.slot_hashes.to_account_info(),
        &ctx.accounts.randomness_authority.to_account_info(),
        &ctx.accounts.switchboard_program.to_account_info(),
        &ctx.accounts.vault.sb_queue,
        seeds,
        &vault_key,
        round_id,
        &[],
        ctx.remaining_accounts,
    )?;
    ctx.accounts.rand_lock.set_inner(RandLock {
        bump: ctx.bumps.rand_lock,
        vault: vault_key,
        seq: round_id,
    });
    let v = &mut ctx.accounts.vault;
    v.randomness = ctx.accounts.randomness.key();
    v.raffle_oracle = ctx.accounts.sb_oracle.key();
    v.seed_slot = seed_slot;
    v.raffle_phase = RAFFLE_COMMITTED;
    Ok(())
}

#[derive(Accounts)]
pub struct RevealRaffle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,
    #[account(
        seeds = [RAND_LOCK_SEED, randomness.key().as_ref()],
        bump = rand_lock.bump,
        constraint = rand_lock.vault == vault.key() @ VaultError::RandomnessMismatch,
        constraint = rand_lock.seq == vault.round_id @ VaultError::RandomnessMismatch,
    )]
    pub rand_lock: Box<Account<'info, RandLock>>,
    /// CHECK: the account committed for this round.
    #[account(mut, address = vault.randomness @ VaultError::RandomnessMismatch)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: the oracle stored at commit.
    #[account(address = vault.raffle_oracle @ VaultError::WrongOracle)]
    pub sb_oracle: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = vault.sb_queue @ VaultError::WrongQueue)]
    pub sb_queue: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_stats: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = SLOT_HASHES_SYSVAR_ID)]
    pub slot_hashes: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: forwarded.
    #[account(mut)]
    pub sb_reward_escrow: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = WRAPPED_SOL_MINT)]
    pub wrapped_sol_mint: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_program_state: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,
}

pub fn handle_reveal_raffle(ctx: Context<RevealRaffle>, args: RevealArgs) -> Result<()> {
    require!(
        ctx.accounts.vault.kind == KIND_RAFFLE
            && ctx.accounts.vault.raffle_phase == RAFFLE_COMMITTED,
        VaultError::OutOfOrder
    );
    let vault_key = ctx.accounts.vault.key();
    let seeds: &[&[u8]] = &[
        RANDOMNESS_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.accounts.vault.randomness_authority_bump],
    ];
    let a = &ctx.accounts;
    let accounts = [
        a.randomness.to_account_info(),
        a.sb_oracle.to_account_info(),
        a.sb_queue.to_account_info(),
        a.sb_stats.to_account_info(),
        a.randomness_authority.to_account_info(),
        a.payer.to_account_info(),
        a.slot_hashes.to_account_info(),
        a.system_program.to_account_info(),
        a.sb_reward_escrow.to_account_info(),
        a.token_program.to_account_info(),
        a.wrapped_sol_mint.to_account_info(),
        a.sb_program_state.to_account_info(),
    ];
    randomness::reveal(
        &accounts,
        &args,
        &a.switchboard_program.to_account_info(),
        seeds,
    )?;
    let snap = randomness::read(&ctx.accounts.randomness.to_account_info())?;
    require!(
        randomness::is_revealed(&snap, ctx.accounts.vault.seed_slot),
        VaultError::RandomnessNotRevealed
    );
    let v = &mut ctx.accounts.vault;
    v.revealed = snap.value;
    v.raffle_phase = RAFFLE_REVEALED;
    Ok(())
}

#[derive(Accounts)]
pub struct SettleRaffle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: the snapshotted owner. Not whoever holds the NFT now.
    pub holder: UncheckedAccount<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    #[account(mut, address = vault.mint @ VaultError::MintMismatch)]
    pub mint: UncheckedAccount<'info>,

    pub seat: Box<Account<'info, RaffleSeat>>,

    /// CHECK: treasury.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: ATA of the snapshotted owner.
    #[account(mut)]
    pub holder_token: UncheckedAccount<'info>,

    /// CHECK: tax PDA.
    #[account(seeds = [TAX_AUTHORITY_SEED, vault.mint.as_ref()], bump = vault.tax_authority_bump)]
    pub tax_authority: UncheckedAccount<'info>,

    /// CHECK: the committed randomness, still the revealed value.
    #[account(address = vault.randomness @ VaultError::RandomnessMismatch)]
    pub randomness: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_settle_raffle(ctx: Context<SettleRaffle>) -> Result<()> {
    let vault = &ctx.accounts.vault;
    require!(vault.kind == KIND_RAFFLE, VaultError::WrongRequestKind);
    require!(
        vault.raffle_phase == RAFFLE_REVEALED,
        VaultError::OutOfOrder
    );
    let snap = randomness::read(&ctx.accounts.randomness.to_account_info())?;
    require!(
        randomness::is_revealed(&snap, vault.seed_slot) && snap.value == vault.revealed,
        VaultError::RandomnessMismatch
    );
    let mixed = selection::request_randomness(&vault.revealed, &vault.key(), vault.round_id);
    let index = uniform_below(&mixed, vault.round_minted);
    let (expected, _) = Pubkey::find_program_address(
        &[
            RAFFLE_SEAT_SEED,
            vault.key().as_ref(),
            &vault.round_id.to_le_bytes(),
            &index.to_le_bytes(),
        ],
        &crate::ID,
    );
    require_keys_eq!(ctx.accounts.seat.key(), expected, VaultError::OutOfOrder);
    require!(
        ctx.accounts.seat.round_id == vault.round_id && ctx.accounts.seat.index == index,
        VaultError::WrongAsset
    );
    require_keys_eq!(
        ctx.accounts.seat.owner,
        ctx.accounts.holder.key(),
        VaultError::WrongAsset
    );
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
    let owed = ctx.accounts.vault.round_pot;
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
    ctx.accounts.vault.round_open = 0;
    ctx.accounts.vault.round_pot = 0;
    ctx.accounts.vault.raffle_phase = 0;
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
