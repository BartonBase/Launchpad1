//! # hybrid_launch (Track A, SPL-404)
//!
//! **Status: in development. Not audited. Localnet/devnet only.**
//!
//! Engine-independent launch step for hybrid (token <-> NFT) collections. The
//! swap engine (MPL-Hybrid vs. the proposed `hybrid_vault`, ADR-008) is still
//! Barton's decision and is NOT part of this program.
//!
//! [`hybrid_launch::launch`] creates a classic SPL Token mint (Token-2022 is
//! rejected), mints exactly 1,000,000,000 whole tokens, never sets a freeze
//! authority, revokes the mint authority in the same instruction, and writes an
//! immutable [`state::LaunchConfig`]: ratio in {10k, 50k, 100k, 200k, 1M},
//! `collection_size * ratio <= 1B` (checked_mul), token fees within caps,
//! fee destination = BURN. There is no instruction that changes or closes it.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod validation;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use validation::*;

declare_id!("9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8");

#[program]
pub mod hybrid_launch {
    use super::*;

    /// Create the mint, mint the fixed 1B supply, revoke authorities and record
    /// the immutable LaunchConfig (see `instructions::launch`).
    pub fn launch(ctx: Context<Launch>, params: LaunchParams) -> Result<()> {
        instructions::launch::handle_launch(ctx, params)
    }
}
