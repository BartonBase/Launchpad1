//! TEST-ONLY mock of the Switchboard On-Demand randomness program (LiteSVM only, never deployed).
//! Loaded at the Switchboard program id so hybrid_vault's REAL CPIs (init, commit, reveal) and its
//! account parsing run unchanged. Real account layout (`RandomnessAccountData`,
//! switchboard-on-demand 0.13.0: 8-byte discriminator + repr(C) fields) and real discriminators /
//! account order (codama `switchboard_on_demand_sol` builders):
//! - `randomness_init` (args recent_slot u64): accounts 0 randomness (signer, new), 2 authority
//!   (must sign), 3 queue, 4 payer (signer). Creates the 408-byte account owned by this program.
//! - `randomness_commit`: accounts 0 randomness, 1 queue (== stored), 2 oracle, 4 authority (must
//!   sign and equal stored). Sets seed_slot = slot - 1, stores the oracle.
//! - `randomness_reveal` (args signature [u8;64], recovery_id u8, value [u8;32]): accounts
//!   0 randomness, 1 oracle (== stored), 4 authority (must sign and equal stored). Sets
//!   reveal_slot = slot and value. DIFFERENCE FROM REAL: the oracle's secp256k1 signature is NOT
//!   verified (the test supplies the value) and reward/escrow accounts are ignored.


/// Marker logged on every instruction (so it's in the binary's rodata); deploy guards
/// (scripts/lib/deploy-guards.sh) always reject a binary that contains it.
pub const MOCK_SWITCHBOARD_MARKER: &str = "TEST-ONLY mock_switchboard: never deploy";

use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub const INIT_DISCRIMINATOR: [u8; 8] = [9, 9, 204, 33, 50, 116, 113, 15];
pub const COMMIT_DISCRIMINATOR: [u8; 8] = [52, 170, 152, 201, 179, 133, 242, 141];
pub const REVEAL_DISCRIMINATOR: [u8; 8] = [197, 181, 187, 10, 30, 58, 20, 73];
pub const RANDOMNESS_DISCRIMINATOR: [u8; 8] = [10, 66, 229, 135, 220, 239, 217, 114];
pub const ACCOUNT_SIZE: usize = 408;
pub const OFF_AUTHORITY: usize = 8;
pub const OFF_QUEUE: usize = 40;
pub const OFF_SEED_SLOT: usize = 104;
pub const OFF_ORACLE: usize = 112;
pub const OFF_REVEAL_SLOT: usize = 144;
pub const OFF_VALUE: usize = 152;
const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0u8; 32]);

entrypoint!(process);

fn acc<'a, 'b>(accounts: &'a [AccountInfo<'b>], i: usize) -> Result<&'a AccountInfo<'b>, ProgramError> {
    accounts.get(i).ok_or(ProgramError::NotEnoughAccountKeys)
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    solana_program::msg!("{}", MOCK_SWITCHBOARD_MARKER);
    if data.len() < 8 || accounts.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let disc = <[u8; 8]>::try_from(&data[..8]).unwrap();
    let r = &accounts[0];
    let slot = Clock::get()?.slot;

    if disc == INIT_DISCRIMINATOR {
        let (auth, queue, payer) = (acc(accounts, 2)?, acc(accounts, 3)?, acc(accounts, 4)?);
        if !r.is_signer || !auth.is_signer || !payer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let system = accounts.iter().find(|a| *a.key == SYSTEM_PROGRAM_ID).ok_or(ProgramError::NotEnoughAccountKeys)?;
        let lamports = Rent::get()?.minimum_balance(ACCOUNT_SIZE);
        let mut ix_data = vec![0u8, 0, 0, 0];
        ix_data.extend_from_slice(&lamports.to_le_bytes());
        ix_data.extend_from_slice(&(ACCOUNT_SIZE as u64).to_le_bytes());
        ix_data.extend_from_slice(program_id.as_ref());
        invoke(
            &Instruction {
                program_id: SYSTEM_PROGRAM_ID,
                accounts: vec![AccountMeta::new(*payer.key, true), AccountMeta::new(*r.key, true)],
                data: ix_data,
            },
            &[payer.clone(), r.clone(), system.clone()],
        )?;
        let mut d = r.try_borrow_mut_data()?;
        d[..8].copy_from_slice(&RANDOMNESS_DISCRIMINATOR);
        d[OFF_AUTHORITY..OFF_AUTHORITY + 32].copy_from_slice(auth.key.as_ref());
        d[OFF_QUEUE..OFF_QUEUE + 32].copy_from_slice(queue.key.as_ref());
        return Ok(());
    }

    if r.owner != program_id || !r.is_writable {
        return Err(ProgramError::IllegalOwner);
    }
    let mut d = r.try_borrow_mut_data()?;
    if d.len() != ACCOUNT_SIZE || d[..8] != RANDOMNESS_DISCRIMINATOR {
        return Err(ProgramError::InvalidAccountData);
    }
    let auth = acc(accounts, 4)?;
    if !auth.is_signer || d[OFF_AUTHORITY..OFF_AUTHORITY + 32] != auth.key.to_bytes() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    match disc {
        COMMIT_DISCRIMINATOR => {
            let (queue, oracle) = (acc(accounts, 1)?, acc(accounts, 2)?);
            if d[OFF_QUEUE..OFF_QUEUE + 32] != queue.key.to_bytes() {
                return Err(ProgramError::InvalidArgument);
            }
            d[OFF_SEED_SLOT..OFF_SEED_SLOT + 8].copy_from_slice(&(slot - 1).to_le_bytes());
            d[OFF_ORACLE..OFF_ORACLE + 32].copy_from_slice(oracle.key.as_ref());
            Ok(())
        }
        REVEAL_DISCRIMINATOR => {
            if data.len() != 8 + 64 + 1 + 32 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let oracle = acc(accounts, 1)?;
            if d[OFF_ORACLE..OFF_ORACLE + 32] != oracle.key.to_bytes() {
                return Err(ProgramError::InvalidArgument);
            }
            d[OFF_REVEAL_SLOT..OFF_REVEAL_SLOT + 8].copy_from_slice(&slot.to_le_bytes());
            d[OFF_VALUE..OFF_VALUE + 32].copy_from_slice(&data[8 + 65..8 + 65 + 32]);
            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
