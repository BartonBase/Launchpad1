use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Only the launch creator may do this")]
    NotCreator,
    #[msg("Pool account must be pre-created, owned by hybrid_vault, zeroed and sized 64 + 16 * collection_size")]
    InvalidPoolAccount,
    #[msg("Pool account does not belong to this vault")]
    PoolMismatch,
    #[msg("Name or URI too long")]
    MetadataTooLong,
    #[msg("Asset index out of range")]
    IndexOutOfRange,
    #[msg("Merkle proof does not match the committed trait root")]
    InvalidMerkleProof,
    #[msg("Asset already deposited")]
    AssetAlreadyDeposited,
    #[msg("Vault is sealed: all committed assets are deposited")]
    VaultSealed,
    #[msg("Vault is not sealed yet: captures open after every committed asset is deposited")]
    VaultNotSealed,
    #[msg("New requests are paused by the guardian (unwrap, settle and expire are never paused)")]
    Paused,
    #[msg("No unreserved asset is available to draw")]
    NoAssetAvailable,
    #[msg("User token account must not alias a vault-owned token account")]
    AliasedTokenAccount,
    #[msg("Randomness account is not a Switchboard randomness account controlled by this vault")]
    InvalidRandomnessAccount,
    #[msg("Randomness commit did not happen in this instruction")]
    RandomnessNotFresh,
    #[msg("Randomness account does not match the request")]
    RandomnessMismatch,
    #[msg("Randomness not revealed yet")]
    RandomnessNotRevealed,
    #[msg("Randomness already revealed: the request must be settled, not expired")]
    RandomnessAlreadyRevealed,
    #[msg("Requests settle strictly in order: this is not the head of the queue")]
    OutOfOrder,
    #[msg("Wrong request kind for this instruction")]
    WrongRequestKind,
    #[msg("Too many incoming entries to merge in one instruction; call merge_incoming first")]
    MergeBacklog,
    #[msg("Asset account is not the one selected by the randomness")]
    WrongAsset,
    #[msg("Request has not reached its deadline")]
    DeadlineNotReached,
    #[msg("Asset account required for a re-roll request")]
    MissingAsset,
    #[msg("Asset is not owned by the expected owner or not in the vault's collection")]
    AssetStateMismatch,
    #[msg("Not the guardian")]
    NotGuardian,
    #[msg("Pause duration exceeds MAX_PAUSE_SLOTS or is zero")]
    InvalidPauseDuration,
    #[msg("Already paused or pause cooldown active")]
    PauseNotAllowed,
    #[msg("Solvency invariant violated: vault tokens < ratio * (assets outside + pending captures)")]
    InsolventVault,
    #[msg("Fee escrow invariant violated")]
    FeeEscrowShort,
    #[msg("NFT conservation invariant violated")]
    AssetAccountingBroken,
    #[msg("LaunchConfig fee destination must be BURN")]
    FeeDestinationNotBurn,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
