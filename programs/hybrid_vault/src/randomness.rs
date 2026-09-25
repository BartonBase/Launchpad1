//! Randomness interface (Switchboard On-Demand). Nothing else in the program touches Switchboard:
//! - `init_account`   (ix `init_randomness`): CPI `randomness_init` with authority = this vault's
//!   `randomness_authority` PDA. Switchboard requires the authority to SIGN init, commit and reveal,
//!   so only hybrid_vault can ever commit or reveal these accounts.
//! - `commit_for_request` (in `request_*` and `recommit_randomness`): CPI `randomness_commit` inside the
//!   same instruction that locks the user's payment, then require seed_slot == slot - 1 and "not revealed".
//! - `reveal` (ix `reveal_randomness`, PERMISSIONLESS): CPI `randomness_reveal` with the oracle's
//!   signed value fetched from the Switchboard gateway by anyone (our crank). The user can't withhold a
//!   reveal: they don't hold the authority, and anyone can submit it.
//! - `revealed_value` (in settle): only the value for the seed slot recorded in the request.
//! - `select_oracle` (M-04): the PROGRAM chooses the oracle for every commit from the pinned queue's
//!   oracle list: index = (queue.curr_idx + H(vault, seq) + commits) mod oracle_keys_len, skipping
//!   oracles already used by this request. The caller passes the oracle account (needed for the CPI)
//!   but a different one is rejected (`WrongOracle`), so the caller never chooses it. `curr_idx` is
//!   Switchboard's own rotation pointer (not caller-controlled); the seq term spreads requests.
//! SlotHashes / Clock / blockhash are NEVER used as randomness (SlotHashes is only passed through).

use crate::{constants::*, error::VaultError};
use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::invoke_signed,
    },
};
pub use switchboard_on_demand::QUEUE_ACCOUNT_DISCRIMINATOR;
use switchboard_on_demand::{OracleAccountData, QueueAccountData, RandomnessAccountData};

pub struct RandomnessSnapshot {
    pub authority: Pubkey,
    pub queue: Pubkey,
    pub seed_slot: u64,
    pub reveal_slot: u64,
    pub value: [u8; 32],
}

/// Parse a Switchboard randomness account (owner pinned to the Switchboard program id).
pub fn read(account: &AccountInfo) -> Result<RandomnessSnapshot> {
    require_keys_eq!(*account.owner, SWITCHBOARD_PROGRAM_ID, VaultError::InvalidRandomnessAccount);
    let data = account.try_borrow_data()?;
    let parsed = RandomnessAccountData::parse(data).map_err(|_| error!(VaultError::InvalidRandomnessAccount))?;
    Ok(RandomnessSnapshot {
        authority: Pubkey::new_from_array(parsed.authority.to_bytes()),
        queue: Pubkey::new_from_array(parsed.queue.to_bytes()),
        seed_slot: parsed.seed_slot,
        reveal_slot: parsed.reveal_slot,
        value: parsed.value,
    })
}

fn require_sb_program(program: &AccountInfo) -> Result<()> {
    require_keys_eq!(*program.key, SWITCHBOARD_PROGRAM_ID, VaultError::InvalidRandomnessAccount);
    require!(program.executable, VaultError::InvalidRandomnessAccount);
    Ok(())
}

/// CPI Switchboard `randomness_init` (accounts in Switchboard order):
/// 0 randomness (w, s) | 1 reward_escrow (w) | 2 authority (s) = our PDA | 3 queue (w) | 4 payer (w, s)
/// | 5 system | 6 token | 7 associated token | 8 wrapped SOL mint | 9 program_state | 10 lut_signer
/// | 11 lut (w) | 12 address lookup table program. Args: recent_slot (u64).
pub fn init_account<'info>(accounts: &[AccountInfo<'info>; 13], recent_slot: u64, switchboard_program: &AccountInfo<'info>, authority_seeds: &[&[u8]]) -> Result<()> {
    require_sb_program(switchboard_program)?;
    let metas = vec![
        AccountMeta::new(*accounts[0].key, true),
        AccountMeta::new(*accounts[1].key, false),
        AccountMeta::new_readonly(*accounts[2].key, true),
        AccountMeta::new(*accounts[3].key, false),
        AccountMeta::new(*accounts[4].key, true),
        AccountMeta::new_readonly(*accounts[5].key, false),
        AccountMeta::new_readonly(*accounts[6].key, false),
        AccountMeta::new_readonly(*accounts[7].key, false),
        AccountMeta::new_readonly(*accounts[8].key, false),
        AccountMeta::new_readonly(*accounts[9].key, false),
        AccountMeta::new_readonly(*accounts[10].key, false),
        AccountMeta::new(*accounts[11].key, false),
        AccountMeta::new_readonly(*accounts[12].key, false),
    ];
    let mut data = SB_RANDOMNESS_INIT_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&recent_slot.to_le_bytes());
    let mut infos: Vec<AccountInfo<'info>> = accounts.to_vec();
    infos.push(switchboard_program.clone());
    invoke_signed(&Instruction { program_id: SWITCHBOARD_PROGRAM_ID, accounts: metas, data }, &infos, &[authority_seeds])?;
    let snap = read(&accounts[0])?;
    require_keys_eq!(snap.authority, *accounts[2].key, VaultError::RandomnessAuthorityMismatch);
    Ok(())
}

/// Validate the account, CPI Switchboard `randomness_commit` signed by the vault's randomness
/// authority PDA, then confirm the commit happened in this slot and is unrevealed.
/// Returns the new seed slot to store in the request.
#[allow(clippy::too_many_arguments)]
pub fn commit_for_request<'info, 'p>(
    randomness: &AccountInfo<'info>,
    queue: &AccountInfo<'info>,
    oracle: &AccountInfo<'info>,
    slot_hashes: &AccountInfo<'info>,
    randomness_authority: &AccountInfo<'info>,
    switchboard_program: &AccountInfo<'info>,
    pinned_queue: &Pubkey,
    authority_seeds: &[&[u8]],
    vault_key: &Pubkey,
    seq: u64,
    used_oracles: &[Pubkey],
    stale_proofs: &[AccountInfo<'p>],
) -> Result<u64> {
    require_sb_program(switchboard_program)?;
    require_keys_eq!(*slot_hashes.key, SLOT_HASHES_SYSVAR_ID, VaultError::InvalidRandomnessAccount);
    require_keys_eq!(*queue.key, *pinned_queue, VaultError::WrongQueue);
    let now = Clock::get()?.unix_timestamp;
    let chosen = select_oracle(queue, vault_key, seq, used_oracles, stale_proofs, now)?;
    require_keys_eq!(*oracle.key, chosen, VaultError::WrongOracle);
    // The chosen oracle must itself be live; if it isn't, the caller supplies it as a stale proof and retries.
    let hb = oracle_last_heartbeat(oracle).ok_or_else(|| error!(VaultError::WrongOracle))?;
    require!(heartbeat_is_fresh(hb, now), VaultError::OracleStale);
    let before = read(randomness)?;
    require_keys_eq!(before.authority, *randomness_authority.key, VaultError::RandomnessAuthorityMismatch);
    require_keys_eq!(before.queue, *pinned_queue, VaultError::WrongQueue);

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
    invoke_signed(
        &ix,
        &[randomness.clone(), queue.clone(), oracle.clone(), slot_hashes.clone(), randomness_authority.clone(), switchboard_program.clone()],
        &[authority_seeds],
    )?;

    let after = read(randomness)?;
    let slot = Clock::get()?.slot;
    require!(after.seed_slot.checked_add(1) == Some(slot), VaultError::RandomnessNotFresh);
    require!(after.reveal_slot <= after.seed_slot, VaultError::RandomnessNotFresh);
    Ok(after.seed_slot)
}

/// Reveal args (Switchboard `RandomnessRevealInstructionArgs`).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RevealArgs {
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub value: [u8; 32],
}

/// CPI Switchboard `randomness_reveal` (accounts in Switchboard order):
/// 0 randomness (w) | 1 oracle | 2 queue | 3 stats (w) | 4 authority (s) = our PDA | 5 payer (w, s)
/// | 6 slothashes | 7 system | 8 reward_escrow (w) | 9 token | 10 wrapped SOL mint | 11 program_state.
/// Switchboard verifies the oracle signature over the committed seed; we only sign as authority.
pub fn reveal<'info>(accounts: &[AccountInfo<'info>; 12], args: &RevealArgs, switchboard_program: &AccountInfo<'info>, authority_seeds: &[&[u8]]) -> Result<()> {
    require_sb_program(switchboard_program)?;
    let metas = vec![
        AccountMeta::new(*accounts[0].key, false),
        AccountMeta::new_readonly(*accounts[1].key, false),
        AccountMeta::new_readonly(*accounts[2].key, false),
        AccountMeta::new(*accounts[3].key, false),
        AccountMeta::new_readonly(*accounts[4].key, true),
        AccountMeta::new(*accounts[5].key, true),
        AccountMeta::new_readonly(*accounts[6].key, false),
        AccountMeta::new_readonly(*accounts[7].key, false),
        AccountMeta::new(*accounts[8].key, false),
        AccountMeta::new_readonly(*accounts[9].key, false),
        AccountMeta::new_readonly(*accounts[10].key, false),
        AccountMeta::new_readonly(*accounts[11].key, false),
    ];
    let mut data = SB_RANDOMNESS_REVEAL_DISCRIMINATOR.to_vec();
    args.serialize(&mut data)?;
    let mut infos: Vec<AccountInfo<'info>> = accounts.to_vec();
    infos.push(switchboard_program.clone());
    invoke_signed(&Instruction { program_id: SWITCHBOARD_PROGRAM_ID, accounts: metas, data }, &infos, &[authority_seeds])?;
    Ok(())
}

pub fn is_revealed(snap: &RandomnessSnapshot, expected_seed_slot: u64) -> bool {
    snap.seed_slot == expected_seed_slot && snap.reveal_slot > snap.seed_slot
}


/// Queue layout (from the crate's own `QueueAccountData`): (body size, oracle_keys, oracle_keys_len,
/// curr_idx) offsets relative to the byte after the 8-byte discriminator.
pub fn queue_layout() -> (usize, usize, usize, usize) {
    (
        core::mem::size_of::<QueueAccountData>(),
        core::mem::offset_of!(QueueAccountData, oracle_keys),
        core::mem::offset_of!(QueueAccountData, oracle_keys_len),
        core::mem::offset_of!(QueueAccountData, curr_idx),
    )
}

/// Switchboard `OracleAccountData` discriminator (from the crate's `Discriminator` impl).
pub const ORACLE_ACCOUNT_DISCRIMINATOR: [u8; 8] = [128, 30, 16, 241, 170, 73, 55, 54];

/// (body size, offset of `last_heartbeat`) from the crate's own `OracleAccountData`.
pub fn oracle_layout() -> (usize, usize) {
    (core::mem::size_of::<OracleAccountData>(), core::mem::offset_of!(OracleAccountData, last_heartbeat))
}

/// `last_heartbeat` of a Switchboard oracle account, or None if it isn't one (owner/discriminator/size).
pub fn oracle_last_heartbeat(acc: &AccountInfo) -> Option<i64> {
    if *acc.owner != SWITCHBOARD_PROGRAM_ID {
        return None;
    }
    let data = acc.try_borrow_data().ok()?;
    let (size, hb) = oracle_layout();
    if data.len() < 8 + size || data[..8] != ORACLE_ACCOUNT_DISCRIMINATOR {
        return None;
    }
    Some(i64::from_le_bytes(data[8 + hb..8 + hb + 8].try_into().ok()?))
}

/// Heartbeat freshness (M-04 staleness filter): heartbeated within MAX_ORACLE_HEARTBEAT_AGE_SECS.
pub fn heartbeat_is_fresh(last_heartbeat: i64, now: i64) -> bool {
    now.saturating_sub(last_heartbeat) <= crate::constants::MAX_ORACLE_HEARTBEAT_AGE_SECS
}

/// Program-side oracle choice (M-04). Checks the queue account's owner, then walks the deterministic
/// candidate order. A candidate is skipped ONLY if the caller supplied that oracle's own account in
/// `stale_proofs` and it proves the oracle stale (heartbeat older than the limit). The caller can
/// therefore skip dead oracles but never a live one, and never pick one.
pub fn select_oracle(queue: &AccountInfo, vault_key: &Pubkey, seq: u64, used: &[Pubkey], stale_proofs: &[AccountInfo], now: i64) -> Result<Pubkey> {
    require_keys_eq!(*queue.owner, SWITCHBOARD_PROGRAM_ID, VaultError::WrongQueue);
    let stale: Vec<Pubkey> = stale_proofs
        .iter()
        .filter(|a| oracle_last_heartbeat(a).is_some_and(|hb| !heartbeat_is_fresh(hb, now)))
        .map(|a| *a.key)
        .collect();
    let data = queue.try_borrow_data()?;
    select_oracle_in(&data, vault_key, seq, used, &stale)
}

/// Pure selection over queue account data (discriminator and size checked). Skips zero keys, keys
/// already used by this request, and keys proven stale.
pub fn select_oracle_in(data: &[u8], vault_key: &Pubkey, seq: u64, used: &[Pubkey], stale: &[Pubkey]) -> Result<Pubkey> {
    let (size, keys_off, len_off, curr_off) = queue_layout();
    require!(data.len() >= 8 + size && data[..8] == QUEUE_ACCOUNT_DISCRIMINATOR, VaultError::WrongQueue);
    let body = &data[8..8 + size];
    let rd_u32 = |off: usize| u32::from_le_bytes(body[off..off + 4].try_into().unwrap());
    let len = rd_u32(len_off) as usize;
    let curr = rd_u32(curr_off) as u64;
    require!(len > 0 && len <= 78, VaultError::NoFreshOracle);
    let h = solana_sha256_hasher::hashv(&[b"hybrid_vault/oracle", vault_key.as_ref(), &seq.to_le_bytes()]).to_bytes();
    let spread = u64::from_le_bytes(h[..8].try_into().unwrap());
    let start = (curr % len as u64 + spread % len as u64 + used.len() as u64) % len as u64;
    for step in 0..len as u64 {
        let i = ((start + step) % len as u64) as usize;
        let k = Pubkey::new_from_array(body[keys_off + 32 * i..keys_off + 32 * (i + 1)].try_into().unwrap());
        if k != Pubkey::default() && !used.contains(&k) && !stale.contains(&k) {
            return Ok(k);
        }
    }
    err!(VaultError::NoFreshOracle)
}
