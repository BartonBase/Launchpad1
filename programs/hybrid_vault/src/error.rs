use anchor_lang::prelude::*;

/// APPEND-ONLY (QA-FEE-01): a variant's position is its on-chain code (6000 + index) and is never
/// reused or renumbered. Removed variants stay as `Reserved*` placeholders; new ones go at the end.
/// `tests::codes_are_pinned` fails if any committed code moves.
#[error_code]
pub enum VaultError {
    #[msg("Signer is not the LaunchConfig creator")]
    NotCreator,
    #[msg("Pool account is malformed or belongs to another vault")]
    InvalidPoolAccount,
    #[msg("Pool accounting mismatch")]
    PoolMismatch,
    #[msg("Name or URI too long")]
    MetadataTooLong,
    #[msg("Asset index out of range")]
    IndexOutOfRange,
    #[msg("Merkle proof does not match the committed trait root")]
    InvalidMerkleProof,
    #[msg("reserved (removed: deposit_asset)")]
    ReservedAssetAlreadyDeposited,
    #[msg("reserved (removed: seal model)")]
    ReservedVaultSealed,
    #[msg("reserved (removed: seal model)")]
    ReservedVaultNotSealed,
    #[msg("reserved (removed: guardian pause, ADR-015)")]
    ReservedPaused,
    #[msg("No NFT is available to draw")]
    NoAssetAvailable,
    #[msg("User token account aliases the vault token account")]
    AliasedTokenAccount,
    #[msg("Randomness account is not a Switchboard randomness account")]
    InvalidRandomnessAccount,
    #[msg("Randomness commit is not fresh")]
    RandomnessNotFresh,
    #[msg("Randomness account does not match the request's commit")]
    RandomnessMismatch,
    #[msg("Randomness not revealed yet")]
    RandomnessNotRevealed,
    #[msg("Randomness already revealed for this request")]
    RandomnessAlreadyRevealed,
    #[msg("Only the head request (FIFO) can be settled or expired")]
    OutOfOrder,
    #[msg("Request kind does not match the instruction")]
    WrongRequestKind,
    #[msg("Too many unmerged incoming NFTs; call merge_incoming first")]
    MergeBacklog,
    #[msg("Asset is not the one selected by the randomness")]
    WrongAsset,
    #[msg("Reveal deadline not reached")]
    DeadlineNotReached,
    #[msg("reserved (removed)")]
    ReservedMissingAsset,
    #[msg("Asset state after CPI does not match expectations")]
    AssetStateMismatch,
    #[msg("reserved (removed: guardian pause, ADR-015)")]
    ReservedNotGuardian,
    #[msg("reserved (removed: guardian pause, ADR-015)")]
    ReservedInvalidPauseDuration,
    #[msg("reserved (removed: guardian pause, ADR-015)")]
    ReservedPauseNotAllowed,
    #[msg("Vault token balance below ratio * (NFTs outside + pending captures + pending re-rolls)")]
    InsolventVault,
    #[msg("reserved (removed: token fee escrow)")]
    ReservedFeeEscrowShort,
    #[msg("NFT accounting invariant broken")]
    AssetAccountingBroken,
    #[msg("reserved (removed: burn fee destination)")]
    ReservedFeeDestinationNotBurn,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("URI must be content-addressed (ipfs:// or ar://)")]
    UriNotContentAddressed,
    #[msg("Converting is closed until the vault is opened (graduation + full mint)")]
    VaultNotOpen,
    #[msg("Vault is already open")]
    VaultAlreadyOpen,
    #[msg("Reserved (was CollectionNotFullyMinted; batch pre-mint removed, lazy mint ADR-016)")]
    ReservedCollectionNotFullyMinted,
    #[msg("Graduation could not be verified on-chain")]
    GraduationNotVerified,
    #[msg("No graduation verifier is compiled into this build (curve program not chosen yet)")]
    GraduationCheckUnavailable,
    #[msg("Randomness authority is not this vault's randomness PDA")]
    RandomnessAuthorityMismatch,
    #[msg("Switchboard queue is not the vault's pinned queue")]
    WrongQueue,
    #[msg("Oracle already used for this request; a re-commit needs a different oracle")]
    OracleReused,
    #[msg("Maximum number of re-commits reached; only the principal-only expire remains")]
    RecommitsExhausted,
    #[msg("Re-commits remain; expire is only possible after MAX_RECOMMITS unrevealed re-commits")]
    RecommitsRemaining,
    #[msg("Expiry grace period not reached")]
    ExpiryNotReached,
    #[msg("LaunchConfig version/fields not supported by this vault")]
    UnsupportedLaunchConfig,
    #[msg("Mint does not match the LaunchConfig mint")]
    MintMismatch,
    #[msg("Fee recipient is not the recipient fixed in the LaunchConfig")]
    FeeRecipientMismatch,
    #[msg("LaunchConfig fee recipient is not the platform fee recipient")]
    FeeRecipientNotPlatform,
    #[msg("LaunchConfig SOL fee is not the ratio tier value or is outside [MIN_FEE_LAMPORTS, MAX_FEE_LAMPORTS]")]
    FeeAboveHardCap,
    #[msg("Reserved (was NothingToSweep; FeeVault/sweep removed when release became free)")]
    ReservedNothingToSweep,
    #[msg("Oracle is not the one the program selected for this commit")]
    WrongOracle,
    #[msg("No oracle on the queue that this request hasn't already used")]
    NoFreshOracle,
    #[msg("Reserved (was GraduationFundShortfall; graduation mint fund removed, lazy mint ADR-016)")]
    ReservedGraduationFundShortfall,
    #[msg("Selected oracle has not heartbeated recently; pass it as a stale proof and retry")]
    OracleStale,
    #[msg("LaunchConfig collection size above MAX_COLLECTION_SIZE (10,000)")]
    CollectionAboveCap,
    #[msg("Picked asset is not minted yet: settle must supply its leaf preimage and Merkle proof")]
    MintArgsMissing,
    #[msg("Lazy mint cost exceeded the request's mint escrow")]
    MintEscrowShort,
    #[msg("Asset index already minted")]
    AlreadyMinted,
    #[msg("Live rent for a Core asset + create fee exceeds the mint escrow constant")]
    MintCostConstantStale,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_pinned() {
        let pinned: &[(VaultError, u32)] = &[
            (VaultError::NotCreator, 6000),
            (VaultError::InvalidPoolAccount, 6001),
            (VaultError::PoolMismatch, 6002),
            (VaultError::MetadataTooLong, 6003),
            (VaultError::IndexOutOfRange, 6004),
            (VaultError::InvalidMerkleProof, 6005),
            (VaultError::ReservedAssetAlreadyDeposited, 6006),
            (VaultError::ReservedVaultSealed, 6007),
            (VaultError::ReservedVaultNotSealed, 6008),
            (VaultError::ReservedPaused, 6009),
            (VaultError::NoAssetAvailable, 6010),
            (VaultError::AliasedTokenAccount, 6011),
            (VaultError::InvalidRandomnessAccount, 6012),
            (VaultError::RandomnessNotFresh, 6013),
            (VaultError::RandomnessMismatch, 6014),
            (VaultError::RandomnessNotRevealed, 6015),
            (VaultError::RandomnessAlreadyRevealed, 6016),
            (VaultError::OutOfOrder, 6017),
            (VaultError::WrongRequestKind, 6018),
            (VaultError::MergeBacklog, 6019),
            (VaultError::WrongAsset, 6020),
            (VaultError::DeadlineNotReached, 6021),
            (VaultError::ReservedMissingAsset, 6022),
            (VaultError::AssetStateMismatch, 6023),
            (VaultError::ReservedNotGuardian, 6024),
            (VaultError::ReservedInvalidPauseDuration, 6025),
            (VaultError::ReservedPauseNotAllowed, 6026),
            (VaultError::InsolventVault, 6027),
            (VaultError::ReservedFeeEscrowShort, 6028),
            (VaultError::AssetAccountingBroken, 6029),
            (VaultError::ReservedFeeDestinationNotBurn, 6030),
            (VaultError::MathOverflow, 6031),
            (VaultError::UriNotContentAddressed, 6032),
            (VaultError::VaultNotOpen, 6033),
            (VaultError::VaultAlreadyOpen, 6034),
            (VaultError::ReservedCollectionNotFullyMinted, 6035),
            (VaultError::GraduationNotVerified, 6036),
            (VaultError::GraduationCheckUnavailable, 6037),
            (VaultError::RandomnessAuthorityMismatch, 6038),
            (VaultError::WrongQueue, 6039),
            (VaultError::OracleReused, 6040),
            (VaultError::RecommitsExhausted, 6041),
            (VaultError::RecommitsRemaining, 6042),
            (VaultError::ExpiryNotReached, 6043),
            (VaultError::UnsupportedLaunchConfig, 6044),
            (VaultError::MintMismatch, 6045),
            (VaultError::FeeRecipientMismatch, 6046),
            (VaultError::FeeRecipientNotPlatform, 6047),
            (VaultError::FeeAboveHardCap, 6048),
            (VaultError::ReservedNothingToSweep, 6049),
            (VaultError::WrongOracle, 6050),
            (VaultError::NoFreshOracle, 6051),
            (VaultError::ReservedGraduationFundShortfall, 6052),
            (VaultError::OracleStale, 6053),
            (VaultError::CollectionAboveCap, 6054),
            (VaultError::MintArgsMissing, 6055),
            (VaultError::MintEscrowShort, 6056),
            (VaultError::AlreadyMinted, 6057),
            (VaultError::MintCostConstantStale, 6058),
        ];
        for (e, code) in pinned {
            assert_eq!(u32::from(*e), *code, "{e:?}");
        }
    }
}
