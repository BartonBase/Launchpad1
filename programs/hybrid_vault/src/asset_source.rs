//! Asset sourcing abstraction. The rest of the vault only needs: "asset `index` exists at
//! `asset_address(vault, index)`, is a Core asset of the vault's collection, and is owned by X".
//!
//! LAZY MINT (ADR-016, Barton 2026-09-25 5:13 PM MT): nothing is pre-minted. Settle mints asset `i`
//! straight to the user the first time VRF picks it (`mint_to_user`), paid from the request's mint
//! escrow; later picks of a returned asset transfer it (`deliver`). The pool bitmap guarantees each
//! index is minted at most once. Metadata comes only from the committed leaf (verified here).

use crate::{constants::*, core_cpi, error::VaultError, merkle};
use anchor_lang::prelude::*;

pub fn asset_address(vault: &Pubkey, index: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ASSET_SEED, vault.as_ref(), &index.to_le_bytes()], &crate::ID)
}

pub fn is_content_addressed(uri: &str) -> bool {
    ALLOWED_URI_PREFIXES.iter().any(|p| uri.len() > p.len() && uri.starts_with(p))
}

/// Leaf preimage supplied by the settler for a lazy mint (graduation-design §5.1).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LeafPreimage {
    pub trait_values: [u16; 8],
    pub salt: [u8; 32],
    pub image_sha256: [u8; 32],
    pub json_sha256: [u8; 32],
    /// Content-addressed URI of the metadata JSON; becomes the asset's on-chain URI verbatim.
    pub uri: String,
}

/// Settle's lazy-mint arguments: the committed leaf preimage for the picked index and its proof.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MintArgs {
    pub leaf: LeafPreimage,
    pub proof: Vec<[u8; 32]>,
}

/// Recompute leaf v2 from the preimage and verify it against the committed root.
pub fn check_leaf(
    trait_root: &[u8; 32],
    trait_schema_hash: &[u8; 32],
    launch_config: &Pubkey,
    collection_size: u32,
    index: u32,
    leaf: &LeafPreimage,
    proof: &[[u8; 32]],
) -> Result<()> {
    require!(index < collection_size, VaultError::IndexOutOfRange);
    require!(leaf.uri.len() <= MAX_URI_LEN, VaultError::MetadataTooLong);
    require!(is_content_addressed(&leaf.uri), VaultError::UriNotContentAddressed);
    require!(proof.len() <= MAX_MERKLE_DEPTH, VaultError::InvalidMerkleProof);
    let t = merkle::traits_hash(trait_schema_hash, &leaf.trait_values);
    let art = merkle::art_hash(&leaf.salt, &leaf.image_sha256, &leaf.json_sha256, &leaf.uri);
    let l = merkle::leaf_hash(&launch_config.to_bytes(), index, &t, &art);
    require!(merkle::verify(l, index, collection_size, proof, trait_root), VaultError::InvalidMerkleProof);
    Ok(())
}

/// The asset's on-chain name is derived from its index (never supplied by the crank).
pub fn asset_name(index: u32) -> String {
    format!("#{index}")
}

/// Drain-before-create (graduation-design T-GRAD-01, measured): anyone can pre-fund a PDA with
/// ~0.00089 SOL so a plain `create_account` fails forever. If the (system-owned, data-less) PDA
/// holds lamports, move them to `to` (the graduation fund: the vault_authority PDA) with the PDA's
/// own signature first; the caller then creates it in the same instruction.
pub fn drain_prefunded_pda<'info>(pda: &AccountInfo<'info>, to: &AccountInfo<'info>, system_program: &AccountInfo<'info>, pda_seeds: &[&[u8]]) -> Result<()> {
    if pda.lamports() == 0 {
        return Ok(());
    }
    require!(pda.data_is_empty() && *pda.owner == anchor_lang::system_program::ID, VaultError::AssetStateMismatch);
    let lamports = pda.lamports();
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            system_program.key(),
            anchor_lang::system_program::Transfer { from: pda.clone(), to: to.clone() },
            &[pda_seeds],
        ),
        lamports,
    )
}

pub fn mint_escrow_address(vault: &Pubkey, seq: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_ESCROW_SEED, vault.as_ref(), &seq.to_le_bytes()], &crate::ID)
}

/// Move ALL lamports of a data-less system-owned PDA (the mint escrow) to `to`, PDA-signed.
/// Used at settle (leftover after a mint, or all of it on the transfer path) and at expire.
pub fn refund_escrow<'info>(escrow: &AccountInfo<'info>, to: &AccountInfo<'info>, system_program: &AccountInfo<'info>, escrow_seeds: &[&[u8]]) -> Result<()> {
    drain_prefunded_pda(escrow, to, system_program, escrow_seeds)
}

/// Lazy mint of committed asset `index` straight to `user` at settle (ADR-016), paid by the
/// request's mint escrow PDA (never the settler, T-HV-16):
/// 1. a pre-funded asset PDA is drained into the escrow (T-GRAD-01),
/// 2. Core create: payer = escrow PDA (signed), authority = vault_authority, owner = user,
///    collection = the vault's (set + verified), name `#index`, uri = the verified leaf URI,
/// 3. the post-state is asserted; the caller then refunds whatever is left in the escrow to the user.
/// A Core create that needs more than the escrow holds fails the whole tx (MintEscrowShort is
/// checked up front against live rent).
#[allow(clippy::too_many_arguments)]
pub fn mint_to_user<'info>(
    core: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    vault_authority: &AccountInfo<'info>,
    escrow: &AccountInfo<'info>,
    user: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    vault_key: &Pubkey,
    index: u32,
    authority_bump: u8,
    escrow_seeds: &[&[u8]],
    name: String,
    uri: String,
) -> Result<()> {
    let idx = index.to_le_bytes();
    let (expected, asset_bump) = asset_address(vault_key, index);
    require_keys_eq!(*asset.key, expected, VaultError::WrongAsset);
    let asset_seeds: &[&[u8]] = &[ASSET_SEED, vault_key.as_ref(), &idx, &[asset_bump]];
    let auth_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];
    let need = Rent::get()?.minimum_balance(hybrid_launch::CORE_ASSET_SPACE_BYTES).saturating_add(hybrid_launch::CORE_CREATE_FEE_LAMPORTS);
    require!(escrow.lamports() >= need, VaultError::MintEscrowShort);
    drain_prefunded_pda(asset, escrow, system_program, asset_seeds)?;
    core_cpi::create_asset(core, asset, collection, vault_authority, escrow, user, system_program, name, uri, asset_seeds, auth_seeds, escrow_seeds)?;
    core_cpi::assert_asset_state(asset, collection.key, user.key)
}

/// Vault -> user (settle). Signed by vault_authority; post-state asserted.
#[allow(clippy::too_many_arguments)]
pub fn deliver<'info>(
    core: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    vault_authority: &AccountInfo<'info>,
    user: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    vault_key: &Pubkey,
    authority_bump: u8,
) -> Result<()> {
    let auth_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];
    core_cpi::transfer_asset(core, asset, collection, payer, vault_authority, user, system_program, Some(auth_seeds))?;
    core_cpi::assert_asset_state(asset, collection.key, user.key)
}

/// User -> vault (re-roll hand-in, unwrap). The user signs as owner; post-state asserted.
pub fn take_back<'info>(
    core: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    user: &AccountInfo<'info>,
    vault_authority: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    core_cpi::transfer_asset(core, asset, collection, user, user, vault_authority, system_program, None)?;
    core_cpi::assert_asset_state(asset, collection.key, vault_authority.key)
}
