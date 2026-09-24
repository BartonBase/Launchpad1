//! TEST-ONLY mock of the Switchboard On-Demand randomness program (LiteSVM only, never deployed).
//!
//! Implements just enough for hybrid_vault's tests, with the real account layout
//! (`RandomnessAccountData`, switchboard-on-demand 0.13.0, 8-byte discriminator + repr(C) fields):
//! - `randomness_commit` (real discriminator): authority (account #4) must sign and equal the
//!   account's stored authority; sets `seed_slot = clock.slot - 1`.
//! - `MOCKREVL` + 32 bytes: test hook standing in for the oracle reveal; sets `reveal_slot = slot`
//!   and `value`. (The real reveal requires an oracle signature; this mock does not.)

use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, sysvar::Sysvar,
};

pub const COMMIT_DISCRIMINATOR: [u8; 8] = [52, 170, 152, 201, 179, 133, 242, 141];
pub const MOCK_REVEAL_DISCRIMINATOR: [u8; 8] = *b"MOCKREVL";
pub const RANDOMNESS_DISCRIMINATOR: [u8; 8] = [10, 66, 229, 135, 220, 239, 217, 114];
pub const ACCOUNT_SIZE: usize = 408;
pub const OFF_AUTHORITY: usize = 8;
pub const OFF_SEED_SLOT: usize = 104;
pub const OFF_REVEAL_SLOT: usize = 144;
pub const OFF_VALUE: usize = 152;

entrypoint!(process);

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if data.len() < 8 || accounts.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let r = &accounts[0];
    if r.owner != program_id || !r.is_writable {
        return Err(ProgramError::IllegalOwner);
    }
    let slot = Clock::get()?.slot;
    let mut d = r.try_borrow_mut_data()?;
    if d.len() != ACCOUNT_SIZE || d[..8] != RANDOMNESS_DISCRIMINATOR {
        return Err(ProgramError::InvalidAccountData);
    }
    match <[u8; 8]>::try_from(&data[..8]).unwrap() {
        COMMIT_DISCRIMINATOR => {
            let auth = accounts.get(4).ok_or(ProgramError::NotEnoughAccountKeys)?;
            if !auth.is_signer || d[OFF_AUTHORITY..OFF_AUTHORITY + 32] != auth.key.to_bytes() {
                return Err(ProgramError::MissingRequiredSignature);
            }
            d[OFF_SEED_SLOT..OFF_SEED_SLOT + 8].copy_from_slice(&(slot - 1).to_le_bytes());
            Ok(())
        }
        MOCK_REVEAL_DISCRIMINATOR => {
            if data.len() != 40 {
                return Err(ProgramError::InvalidInstructionData);
            }
            d[OFF_REVEAL_SLOT..OFF_REVEAL_SLOT + 8].copy_from_slice(&slot.to_le_bytes());
            d[OFF_VALUE..OFF_VALUE + 32].copy_from_slice(&data[8..40]);
            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
