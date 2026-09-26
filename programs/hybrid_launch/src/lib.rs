//! # hybrid_launch (Track A, SPL-404)
//!
//! **Status: in development. Not audited. Localnet/devnet only.**
//!
//! Launch step for hybrid (token <-> NFT) collections. The swap engine is the
//! separate `hybrid_vault` program (ADR-008 ACCEPTED 2026-09-24), which reads
//! the LaunchConfig written here.
//!
//! [`hybrid_launch::launch`] creates a classic SPL Token mint (Token-2022 is
//! rejected), mints exactly 1,000,000,000 whole tokens, never sets a freeze
//! authority, revokes the mint authority in the same instruction, and writes an
//! immutable [`state::LaunchConfig`]: ratio in {10k, 50k, 100k, 200k, 500k, 1M,
//! 2.5M, 5M}, `100 <= collection_size` and `collection_size * ratio <= 1B`
//! (checked_mul), `collection_size <= MAX_COLLECTION_SIZE`, and ONE flat SOL fee for capture,
//! release and re-roll derived from the ratio tier table (0.002 / 0.005 / 0.01 SOL, hard cap
//! 0.01 SOL), paid to the PLATFORM_FEE_RECIPIENT constant (ADR-013). There is NO token fee.
//! There is no instruction that changes or closes it. The full supply goes to the ATA of a
//! program-derived `launch_vault` PDA that no instruction can sign for.

pub mod constants;
pub mod dbc;
pub mod error;
pub mod instructions;
pub mod needs_barton;
pub mod stable_layout;
pub mod state;
pub mod validation;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use stable_layout::*;
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

    /// Register a Meteora DBC-created token as a hybrid launch (ADR-014). Verifies DBC's pool, config
    /// and mint, and creates the locked buffer ATA. Signed by the DBC pool creator, before graduation.
    pub fn register_dbc_launch(ctx: Context<RegisterDbcLaunch>, params: RegisterDbcParams) -> Result<()> {
        instructions::register_dbc::handle_register_dbc_launch(ctx, params)
    }
}
