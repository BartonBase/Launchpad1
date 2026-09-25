//! Metaplex Core CPIs (program id hard-coded in constants::MPL_CORE_ID and checked on every call).

use crate::{constants::*, error::VaultError};
use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::{CreateCollectionV2CpiBuilder, CreateV2CpiBuilder, TransferV1CpiBuilder},
    types::UpdateAuthority,
};

fn check_program(p: &AccountInfo) -> Result<()> {
    require_keys_eq!(*p.key, MPL_CORE_ID, VaultError::AssetStateMismatch);
    Ok(())
}

pub fn create_collection<'info>(
    core: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    update_authority: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    name: String,
    uri: String,
    collection_seeds: &[&[u8]],
) -> Result<()> {
    check_program(core)?;
    CreateCollectionV2CpiBuilder::new(core)
        .collection(collection)
        .update_authority(Some(update_authority))
        .payer(payer)
        .system_program(system_program)
        .name(name)
        .uri(uri)
        .invoke_signed(&[collection_seeds])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
/// Core CreateV2 of a vault asset: collection = the vault's, authority = vault_authority (collection
/// update authority), no plugins, `owner` as given (the user, for a lazy mint at settle).
pub fn create_asset<'info>(
    core: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    vault_authority: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    name: String,
    uri: String,
    asset_seeds: &[&[u8]],
    authority_seeds: &[&[u8]],
    payer_seeds: &[&[u8]],
) -> Result<()> {
    check_program(core)?;
    CreateV2CpiBuilder::new(core)
        .asset(asset)
        .collection(Some(collection))
        .authority(Some(vault_authority))
        .payer(payer)
        .owner(Some(owner))
        .system_program(system_program)
        .name(name)
        .uri(uri)
        .invoke_signed(&[asset_seeds, authority_seeds, payer_seeds])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn transfer_asset<'info>(
    core: &AccountInfo<'info>,
    asset: &AccountInfo<'info>,
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    owner_authority: &AccountInfo<'info>,
    new_owner: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    signer_seeds: Option<&[&[u8]]>,
) -> Result<()> {
    check_program(core)?;
    let mut b = TransferV1CpiBuilder::new(core);
    b.asset(asset)
        .collection(Some(collection))
        .payer(payer)
        .authority(Some(owner_authority))
        .new_owner(new_owner)
        .system_program(Some(system_program));
    match signer_seeds {
        Some(s) => b.invoke_signed(&[s])?,
        None => b.invoke()?,
    }
    Ok(())
}

/// Defence in depth: after a transfer, confirm the asset is a Core asset of `collection` owned by `owner`.
pub fn assert_asset_state(asset: &AccountInfo, collection: &Pubkey, owner: &Pubkey) -> Result<()> {
    require_keys_eq!(*asset.owner, MPL_CORE_ID, VaultError::AssetStateMismatch);
    let data = asset.try_borrow_data()?;
    let base = BaseAssetV1::from_bytes(&data).map_err(|_| error!(VaultError::AssetStateMismatch))?;
    require!(base.owner.to_bytes() == owner.to_bytes(), VaultError::AssetStateMismatch);
    match base.update_authority {
        UpdateAuthority::Collection(c) if c.to_bytes() == collection.to_bytes() => Ok(()),
        _ => err!(VaultError::AssetStateMismatch),
    }
}
