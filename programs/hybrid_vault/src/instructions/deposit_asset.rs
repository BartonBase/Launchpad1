//! `deposit_asset` (v1 default = creator pre-deposit): the creator mints committed asset `index`
//! straight into the vault. The asset is a Core asset at PDA ["asset", vault, index], created by the
//! vault's CPI into the vault's own collection, owner = vault_authority, no plugins. (name, uri) must
//! be proven against the trait root committed at init. When every index is deposited the vault is
//! sealed and captures open. No asset can ever be added after sealing.

use crate::{constants::*, core_cpi, error::VaultError, invariants, merkle, pool::PoolView, state::Vault};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(index: u32)]
pub struct DepositAsset<'info> {
    #[account(mut, address = vault.creator @ VaultError::NotCreator)]
    pub creator: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: validated by PoolView::load (discriminator, vault, size) + address pinned.
    #[account(mut, address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: PDA.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: the vault's own collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: PDA for this index; must be empty (created here).
    #[account(mut, seeds = [ASSET_SEED, vault.key().as_ref(), &index.to_le_bytes()], bump)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_deposit_asset(
    ctx: Context<DepositAsset>,
    index: u32,
    name: String,
    uri: String,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let vault = &ctx.accounts.vault;
    require!(!vault.sealed, VaultError::VaultSealed);
    require!(index < vault.collection_size, VaultError::IndexOutOfRange);
    require!(name.len() <= MAX_NAME_LEN && uri.len() <= MAX_URI_LEN, VaultError::MetadataTooLong);
    require!(proof.len() <= MAX_MERKLE_DEPTH, VaultError::InvalidMerkleProof);
    require!(ctx.accounts.asset.data_is_empty(), VaultError::AssetAlreadyDeposited);
    require!(
        merkle::verify(merkle::leaf_hash(index, &name, &uri), index, vault.collection_size, &proof, &vault.trait_root),
        VaultError::InvalidMerkleProof
    );

    let vault_key = vault.key();
    let idx = index.to_le_bytes();
    let asset_bump = ctx.bumps.asset;
    let asset_seeds: &[&[u8]] = &[ASSET_SEED, vault_key.as_ref(), &idx, &[asset_bump]];
    let auth_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[vault.authority_bump]];
    core_cpi::create_asset_in_vault(
        &ctx.accounts.mpl_core_program.to_account_info(),
        &ctx.accounts.asset.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.vault_authority.to_account_info(),
        &ctx.accounts.creator.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        name,
        uri,
        asset_seeds,
        auth_seeds,
    )?;
    core_cpi::assert_asset_state(&ctx.accounts.asset, &vault.collection, &ctx.accounts.vault_authority.key())?;

    let vault = &mut ctx.accounts.vault;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let mut pool = PoolView::load(&mut data, &vault_key)?;
    pool.pool_push(index)?;
    vault.deposited_count = vault.deposited_count.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if vault.deposited_count == vault.collection_size {
        vault.sealed = true;
    }
    // No tokens move here; balances are zero-requirement (no NFT outside yet).
    invariants::check(vault, &pool, u64::MAX, u64::MAX)?;
    Ok(())
}
