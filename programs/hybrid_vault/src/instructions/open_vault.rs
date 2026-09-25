//! `open_vault`: PERMISSIONLESS, ONE-WAY (LAZY MINT, ADR-016). Opens converting once (a) the vault's
//! collection exists as a Core collection whose update authority is the vault_authority PDA and whose
//! Core counter agrees with the vault's minted_count (0 before any settle), (b) the pool is sized for
//! exactly N indices (the collection's max size, enforced by the pool + minted bitmap), and (c)
//! graduation is verified by `graduation::verify` (fail-closed in release builds until DBC is wired).
//! No pre-mint, no minted == N requirement. There is no close/un-open instruction.

use crate::{config, constants::*, error::VaultError, graduation, invariants, pool::PoolView, state::Vault};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use hybrid_launch::LaunchConfig;

#[derive(Accounts)]
pub struct OpenVault<'info> {
    pub caller: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,

    #[account(address = vault.launch_config)]
    pub launch_config: Box<Account<'info, LaunchConfig>>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    #[account(seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: the vault's Core collection (owner Core, update authority = vault_authority).
    #[account(address = vault.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: interpreted only by graduation::verify.
    pub graduation_proof: UncheckedAccount<'info>,
}

pub fn handle_open_vault(ctx: Context<OpenVault>) -> Result<()> {
    let econ = config::econ(&ctx.accounts.launch_config)?;
    let v = &ctx.accounts.vault;
    require!(!v.open, VaultError::VaultAlreadyOpen);
    {
        // The collection exists, is ours, and Core agrees with our mint count (nothing minted outside us).
        let c = &ctx.accounts.collection;
        require_keys_eq!(*c.owner, MPL_CORE_ID, VaultError::AssetStateMismatch);
        let data = c.try_borrow_data()?;
        let col = mpl_core::accounts::BaseCollectionV1::from_bytes(&data).map_err(|_| error!(VaultError::AssetStateMismatch))?;
        let (va, _) = Pubkey::find_program_address(&[VAULT_AUTHORITY_SEED, v.key().as_ref()], &crate::ID);
        require!(col.update_authority.to_bytes() == va.to_bytes(), VaultError::AssetStateMismatch);
        require!(col.num_minted == v.minted_count && col.current_size == v.minted_count, VaultError::AssetAccountingBroken);
    }
    graduation::verify(&v.mint, &ctx.accounts.graduation_proof.to_account_info())?;

    let vault_key = v.key();
    let v = &mut ctx.accounts.vault;
    v.open = true;
    v.opened_at_slot = Clock::get()?.slot;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    require!(pool.capacity() == econ.collection_size, VaultError::InvalidPoolAccount);
    invariants::check(v, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}
