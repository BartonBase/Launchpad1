//! # fee_treasury
//!
//! **Status: STUB (scaffold only). Not audited. Localnet/devnet only.**
//!
//! Custody program for Token-2022 transfer-fee revenue ("the tax").
//!
//! ## Intended flow (future milestones)
//! 1. The taxed token is a **Token-2022 mint with the `TransferFeeConfig` extension**.
//!    We do NOT write a custom transfer-fee program; Token-2022 withholds fees on
//!    recipient token accounts natively.
//! 2. Anyone may call Token-2022's permissionless `HarvestWithheldTokensToMint`.
//! 3. The mint's `withdraw_withheld_authority` is set to this program's
//!    `vault_authority` PDA, so only this program can call
//!    `WithdrawWithheldTokensFromMint` / `...FromAccounts`, moving fees into a
//!    PDA-owned vault token account.
//! 4. Vault funds may only leave through narrowly scoped instructions
//!    (e.g. "buy NFT from allow-listed collection X, price <= cap, delivered to
//!    the lottery prize vault"), subject to per-purchase and per-window spending
//!    caps enforced on-chain, plus a pause switch.
//!
//! ## What exists today
//! Only [`fee_treasury::initialize`], which creates the [`state::TreasuryConfig`]
//! PDA for one fee mint, validates spending-rule parameters and records the
//! canonical bump of the vault-authority PDA.
//!
//! See `docs/ARCHITECTURE.md` and `docs/THREAT_MODEL.md` at the workspace root.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("9mNyaZ3iDVdZKpdy6ZJvt3oPXTCYBisfPsqqzCaa7vsB");

#[program]
pub mod fee_treasury {
    use super::*;

    /// Create the treasury config PDA for a Token-2022 fee mint.
    ///
    /// * `admin` (signer, payer) becomes the config admin. In production this
    ///   should be a multisig (e.g. Squads) and later be reduced/revoked.
    /// * `fee_mint` must be owned by the Token-2022 program (owner check).
    /// * Re-initialization is impossible: `init` fails if the PDA exists.
    pub fn initialize(ctx: Context<Initialize>, params: InitializeTreasuryParams) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx, params)
    }
}
