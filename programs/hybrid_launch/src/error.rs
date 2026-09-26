use anchor_lang::prelude::*;

/// APPEND-ONLY (QA-FEE-01): a variant's position is its on-chain code (6000 + index) and is never
/// reused or renumbered. Removed variants stay as `Reserved*` placeholders; new ones go at the end.
/// `tests::codes_are_pinned` fails if any committed code moves.
#[error_code]
pub enum LaunchError {
    #[msg("decimals must be <= 9")]
    InvalidDecimals,
    #[msg("ratio must be one of 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000")]
    RatioNotAllowed,
    #[msg("collection_size must be >= 1")]
    ZeroCollectionSize,
    #[msg("collection_size * ratio exceeds the 1B supply")]
    CollectionTooLargeForSupply,
    #[msg("reserved (removed: token fee bps cap; there is no token fee)")]
    ReservedFeeAboveCap,
    #[msg("reserved (removed: capture vs re-roll fee ordering)")]
    ReservedCaptureFeeBelowRerollFee,
    #[msg("reserved (removed: burn fee destination)")]
    ReservedFeeDestinationNotBurn,
    #[msg("launch_destination is not the classic-SPL ATA of the launch_vault PDA for this mint")]
    InvalidLaunchDestination,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Post-launch invariant check failed")]
    PostLaunchCheckFailed,
    #[msg("collection_size must be >= MIN_COLLECTION_SIZE (100)")]
    CollectionBelowMinimum,
    #[msg("mint address already holds data or is owned by a program")]
    MintAccountInUse,
    #[msg("tier SOL fee outside [MIN_FEE_LAMPORTS, MAX_FEE_LAMPORTS]")]
    SolFeeOutOfRange,
    #[msg("collection_size above MAX_COLLECTION_SIZE (10,000)")]
    CollectionAboveCap,
    #[msg("graduation threshold outside [10 SOL, 100,000 SOL]")]
    GraduationThresholdOutOfRange,
    #[msg("Reserved (was GraduationUnfundable; affordability rule dropped with lazy minting, ADR-016)")]
    ReservedGraduationUnfundable,
    #[msg("live rent for a Core asset exceeds the conservative constant")]
    MintCostConstantStale,
    #[msg("fee recipient must be PLATFORM_FEE_RECIPIENT, system-owned, data-less and not executable")]
    FeeRecipientInvalid,
    #[msg("not a DBC pool/config account (owner, length or discriminator), or pool/config/mint don't match")]
    DbcAccountInvalid,
    #[msg("DBC config rejected: needs SOL quote, SPL token type, fixed 1B supply with no burn, immutable authority, our buffer as leftover receiver, and the launch's decimals and threshold")]
    DbcConfigRejected,
    #[msg("DBC mint rejected: needs classic Token, mint + freeze authority None, supply exactly 1B, the launch's decimals")]
    DbcMintRejected,
    #[msg("the signer must be the DBC pool's creator")]
    DbcCreatorMismatch,
    #[msg("the DBC pool has already migrated; register before graduation")]
    DbcAlreadyGraduated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_pinned() {
        let pinned: &[(LaunchError, u32)] = &[
            (LaunchError::InvalidDecimals, 6000),
            (LaunchError::RatioNotAllowed, 6001),
            (LaunchError::ZeroCollectionSize, 6002),
            (LaunchError::CollectionTooLargeForSupply, 6003),
            (LaunchError::ReservedFeeAboveCap, 6004),
            (LaunchError::ReservedCaptureFeeBelowRerollFee, 6005),
            (LaunchError::ReservedFeeDestinationNotBurn, 6006),
            (LaunchError::InvalidLaunchDestination, 6007),
            (LaunchError::MathOverflow, 6008),
            (LaunchError::PostLaunchCheckFailed, 6009),
            (LaunchError::CollectionBelowMinimum, 6010),
            (LaunchError::MintAccountInUse, 6011),
            (LaunchError::SolFeeOutOfRange, 6012),
            (LaunchError::CollectionAboveCap, 6013),
            (LaunchError::GraduationThresholdOutOfRange, 6014),
            (LaunchError::ReservedGraduationUnfundable, 6015),
            (LaunchError::MintCostConstantStale, 6016),
            (LaunchError::FeeRecipientInvalid, 6017),
            (LaunchError::DbcAccountInvalid, 6018),
            (LaunchError::DbcConfigRejected, 6019),
            (LaunchError::DbcMintRejected, 6020),
            (LaunchError::DbcCreatorMismatch, 6021),
            (LaunchError::DbcAlreadyGraduated, 6022),
        ];
        for (e, code) in pinned {
            assert_eq!(u32::from(*e), *code, "{e:?}");
        }
    }
}
