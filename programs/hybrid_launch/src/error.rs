use anchor_lang::prelude::*;

#[error_code]
pub enum LaunchError {
    #[msg("decimals must be <= 9")]
    InvalidDecimals,
    #[msg("ratio must be one of 10_000, 50_000, 100_000, 200_000, 1_000_000")]
    RatioNotAllowed,
    #[msg("collection_size must be >= 1")]
    ZeroCollectionSize,
    #[msg("collection_size * ratio exceeds the 1B supply")]
    CollectionTooLargeForSupply,
    #[msg("token fee exceeds MAX_TOKEN_FEE_BPS")]
    FeeAboveCap,
    #[msg("capture fee must be >= re-roll fee (else unwrap+rewrap is a cheaper re-roll)")]
    CaptureFeeBelowRerollFee,
    #[msg("fee destination must be BURN")]
    FeeDestinationNotBurn,
    #[msg("launch_destination is not the classic-SPL ATA of launch_destination_owner for this mint")]
    InvalidLaunchDestination,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Post-launch invariant check failed")]
    PostLaunchCheckFailed,
}
