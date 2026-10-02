//! PoC 13 attacker "guard" program (LOCAL VALIDATOR ONLY). Predict-and-abort against upstream
//! MPL-Hybrid capture_v2 reroll (A-04). It CPIs capture_v2 with the caller's accounts, then reads
//! the captured asset's post-CPI URI inside the same transaction and returns an error unless the
//! rolled index is below `rare_below`. A failed attempt reverts the capture entirely (tokens and NFT
//! untouched); the attacker pays only the tx fee.
//! ix data: rare_below u64 LE | nonce u64 LE (nonce only makes signatures unique).
//! accounts: [0] mpl_hybrid program, [1..=17] capture_v2 accounts in order (asset = index 1+4).
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, instruction::{AccountMeta, Instruction},
    msg, program::invoke, program_error::ProgramError, pubkey::Pubkey,
};

entrypoint!(process);

fn read_str(d: &[u8], off: &mut usize) -> Result<String, ProgramError> {
    let len = u32::from_le_bytes(d.get(*off..*off + 4).ok_or(ProgramError::InvalidAccountData)?.try_into().unwrap()) as usize;
    *off
