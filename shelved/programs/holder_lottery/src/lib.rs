//! # holder_lottery
//!
//! **Status: STUB (scaffold only). Not audited. Localnet/devnet only.**
//!
//! Holder lottery for a Token-2022 taxed token. Prizes (NFTs bought by
//! `fee_treasury`) sit in a PDA-owned prize vault and are distributed to
//! verifiably random winners.
//!
//! ## Fairness rules (design, enforced in future milestones)
//! * **Tickets = floor(eligible_balance / ticket_threshold).** Remainders are
//!   discarded, so splitting a balance across many wallets can never yield more
//!   tickets than holding it in one wallet (see [`math::tickets_for_balance`]
//!   and its tests).
//! * **Anti-snapshot-sniping:** only balance that has been held for at least
//!   `min_holding_seconds` counts (opt-in registration/lock is the current
//!   leading design, see `docs/DECISIONS.md`).
//! * **Randomness:** only an external VRF / oracle randomness provider
//!   (Switchboard On-Demand or ORAO VRF). Slot hashes, recent blockhashes,
//!   timestamps or any other validator-influenceable value are never used.
//! * **No reroll:** once randomness is committed for a round it cannot be
//!   re-requested or cancelled.
//!
//! ## What exists today
//! Only [`holder_lottery::initialize`], which creates [`state::LotteryConfig`]
//! for one Token-2022 ticket mint and validates the fairness parameters.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FJGixnazzvAQv2MFr5sABnKsm8yKqJAvxwBiTXNKmVNg");

#[program]
pub mod holder_lottery {
    use super::*;

    /// Create the lottery config PDA for a Token-2022 ticket mint.
    ///
    /// * `admin` (signer, payer) becomes the lottery admin (should be a
    ///   multisig; admin can never pick winners or touch prizes directly).
    /// * `ticket_mint` must be owned by the Token-2022 program.
    /// * `ticket_threshold > 0`, `min_holding_seconds` within bounds.
    /// * Re-initialization is impossible (`init`).
    pub fn initialize(ctx: Context<Initialize>, params: InitializeLotteryParams) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx, params)
    }
}
