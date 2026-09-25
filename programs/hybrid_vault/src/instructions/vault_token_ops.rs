//! Token and SOL movements. Every CPI result is propagated with `?` (A-08); program ids are pinned
//! by the account types (`Program<Token>`, `Program<System>`).

use crate::constants::VAULT_AUTHORITY_SEED;
use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

/// vault_tokens -> `to`, signed by vault_authority (unwrap only). Always exactly `amount`.
#[allow(clippy::too_many_arguments)]
pub fn pay_out<'info>(
    token_program: &Program<'info, Token>,
    from: &Account<'info, TokenAccount>,
    to: &AccountInfo<'info>,
    mint: &Account<'info, Mint>,
    vault_authority: &AccountInfo<'info>,
    vault_key: &Pubkey,
    authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];
    token::transfer_checked(
        CpiContext::new_with_signer(
            token_program.key(),
            TransferChecked { from: from.to_account_info(), mint: mint.to_account_info(), to: to.clone(), authority: vault_authority.clone() },
            &[seeds],
        ),
        amount,
        mint.decimals,
    )
}

/// user -> `to` (ratio into vault_tokens). The user signs.
pub fn user_pays<'info>(
    token_program: &Program<'info, Token>,
    from: &Account<'info, TokenAccount>,
    to: &AccountInfo<'info>,
    user: &Signer<'info>,
    mint: &Account<'info, Mint>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    token::transfer_checked(
        CpiContext::new(
            token_program.key(),
            TransferChecked { from: from.to_account_info(), mint: mint.to_account_info(), to: to.clone(), authority: user.to_account_info() },
        ),
        amount,
        mint.decimals,
    )
}

/// user -> `to` flat SOL fee by system transfer (the user signs). `to` is PLATFORM_FEE_RECIPIENT
/// (capture / re-roll only; release is free).
pub fn sol_fee<'info>(system: &Program<'info, System>, user: &Signer<'info>, to: &AccountInfo<'info>, lamports: u64) -> Result<()> {
    if lamports == 0 {
        return Ok(());
    }
    system_program::transfer(
        CpiContext::new(system.key(), system_program::Transfer { from: user.to_account_info(), to: to.clone() }),
        lamports,
    )
}
