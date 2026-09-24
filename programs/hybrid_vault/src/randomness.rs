//! Randomness interface (Switchboard On-Demand). The rest of the program only calls
//! `commit_for_request` (in request_*), `revealed_value` (in settle_*) and `is_revealed` (in expire).
//!
//! Design (closes the "user controls the randomness account" attacks):
//! - The randomness account's `authority` must be this vault's `randomness_authority` PDA, so only
//!   hybrid_vault can commit it. hybrid_vault commits via CPI *inside* `request_*`, atomically with
//!   locking the user's payment, so the value can't be known before the request exists, and the
//!   user can't re-commit later to force a failure + refund.
//! - A `RandLock` PDA per randomness account prevents a second request from re-committing it while
//!   the first is pending.
//! - Settle reads the value only if the account's `seed_slot` equals the one recorded at request and
//!   `reveal_slot > seed_slot`. Reveal is done by the Switchboard oracle flow (anyone can crank it).
//! - SlotHashes/Clock/blockhash are never used as randomness here (SlotHashes is only passed through
//!   to Switchboard's own commit instruction).

use crate::{constants::*, error::VaultError};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::{AccountMeta, Instruction}},
};
use switchboard_on_demand::RandomnessAccountData;

pub struct RandomnessSnapshot {
    pub authority: Pubkey,
    pub seed_slot: u64,
    pub reveal_slot: u64,
    pub value: [u8; 32],
}

pub fn read(account: &AccountInfo) -> Result<RandomnessSnapshot> {
    require_keys_eq!(*account.owner, SWITCHBOARD_PROGRAM_ID, VaultError::InvalidRandomnessAccount);
    let data = account.try_borrow_data()?;
    let parsed = RandomnessAccountData::parse(data).map_err(|_| error!(VaultError::InvalidRandomnessAccount))?;
    Ok(RandomnessSnapshot {
        authority: Pubkey::new_from_array(parsed.authority.to_bytes()),
        seed_slot: parsed.seed_slot,
        reveal_slot: parsed.reveal_slot,
        value: parsed.value,
    })
}

/// Validate the account, CPI Switchboard `randomness_commit` signed by the vault's randomness
/// authority PDA, then confirm the commit happened in this slot and is unrevealed.
/// Returns the new seed slot to store in the request.
#[allow(clippy::too_many_arguments)]
pub fn commit_for_request<'info>(
    randomness: &AccountInfo<'info>,
    queue: &AccountInfo<'info>,
    oracle: &AccountInfo<'info>,
    slot_hashes: &AccountInfo<'info>,
    randomness_authority: &AccountInfo<'info>,
    switchboard_program: &AccountInfo<'info>,
    authority_seeds: &[&[u8]],
) -> Result<u64> {
    require_keys_eq!(*switchboard_program.key, SWITCHBOARD_PROGRAM_ID, VaultError::InvalidRandomnessAccount);
    require_keys_eq!(*slot_hashes.key, SLOT_HASHES_SYSVAR_ID, VaultError::InvalidRandomnessAccount);
    let before = read(randomness)?;
    require_keys_eq!(before.authority, *randomness_authority.key, VaultError::InvalidRandomnessAccount);

    let ix = Instruction {
        program_id: SWITCHBOARD_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*randomness.key, false),
            AccountMeta::new_readonly(*queue.key, false),
            AccountMeta::new(*oracle.key, false),
            AccountMeta::new_readonly(SLOT_HASHES_SYSVAR_ID, false),
            AccountMeta::new_readonly(*randomness_authority.key, true),
        ],
        data: SB_RANDOMNESS_COMMIT_DISCRIMINATOR.to_vec(),
    };
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            randomness.clone(),
            queue.clone(),
            oracle.clone(),
            slot_hashes.clone(),
            randomness_authority.clone(),
            switchboard_program.clone(),
        ],
        &[authority_seeds],
    )?;

    let after = read(randomness)?;
    let slot = Clock::get()?.slot;
    require!(after.seed_slot.checked_add(1) == Some(slot), VaultError::RandomnessNotFresh);
    require!(after.reveal_slot <= after.seed_slot, VaultError::RandomnessNotFresh);
    Ok(after.seed_slot)
}

pub fn is_revealed(snap: &RandomnessSnapshot, expected_seed_slot: u64) -> bool {
    snap.seed_slot == expected_seed_slot && snap.reveal_slot > snap.seed_slot
}

/// The revealed value for the request's commit, or an error if not (yet) revealed or re-committed.
pub fn revealed_value(account: &AccountInfo, expected_seed_slot: u64) -> Result<[u8; 32]> {
    let snap = read(account)?;
    require!(snap.seed_slot == expected_seed_slot, VaultError::RandomnessMismatch);
    require!(snap.reveal_slot > snap.seed_slot, VaultError::RandomnessNotRevealed);
    Ok(snap.value)
}
