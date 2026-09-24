//! Token movements signed by the vault_authority PDA. Destinations are always accounts the caller
//! can't choose freely (user's own ATA-like account validated by constraints, or the stored refund account).

use crate::constants::VAULT_AUTHORITY_SEED;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, TransferChecked};

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
    if amount == 0 {
        return Ok(());
    }
    let seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];
    token::transfer_checked(
        CpiContext::new_with_signer(
            token_program.key(),
            TransferChecked {
                from: from.to_account_info(),
                mint: mint.to_account_info(),
                to: to.clone(),
                authority: vault_authority.clone(),
            },
            &[seeds],
        ),
        amount,
        mint.decimals,
    )
}

/// Burn escrowed fee tokens (SPL Token `burn`): supply goes down; no wallet receives them.
pub fn burn_fee<'info>(
    token_program: &Program<'info, Token>,
    fee_escrow: &Account<'info, TokenAccount>,
    mint: &Account<'info, Mint>,
    vault_authority: &AccountInfo<'info>,
    vault_key: &Pubkey,
    authority_bump: u8,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[authority_bump]];
    token::burn(
        CpiContext::new_with_signer(
            token_program.key(),
            Burn { mint: mint.to_account_info(), from: fee_escrow.to_account_info(), authority: vault_authority.clone() },
            &[seeds],
        ),
        amount,
    )
}
