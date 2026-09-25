use anchor_lang::prelude::*;

/// Immutable launch record for one Track A mint. PDA `["launch_config", mint]`.
///
/// There is NO instruction that modifies or closes this account: ratio,
/// collection size, SOL fees, fee recipient and mint are fixed at launch
/// (Stonk.fun lesson: no mutable economics after launch).
#[account]
#[derive(InitSpace, Debug, PartialEq, Eq)]
pub struct LaunchConfig {
    pub version: u8,
    pub bump: u8,
    pub mint_authority_bump: u8,
    /// Who paid for the launch. Holds no powers.
    pub creator: Pubkey,
    /// Classic SPL Token mint; mint + freeze authority are `None`.
    pub mint: Pubkey,
    /// ATA(launch_vault, mint): received the full supply. Owned by the PDA below;
    /// no instruction can move tokens out of it (distribution mechanism TBD).
    pub launch_destination: Pubkey,
    /// PDA `["launch_vault", mint, launch_config]` owning `launch_destination`.
    pub launch_vault: Pubkey,
    pub launch_vault_bump: u8,
    pub decimals: u8,
    /// 1_000_000_000 * 10^decimals.
    pub total_supply_base: u64,
    /// Whole tokens per NFT (one of ALLOWED_RATIOS).
    pub ratio_whole_tokens: u64,
    /// ratio_whole_tokens * 10^decimals.
    pub ratio_base: u64,
    /// Number of NFTs in the collection.
    pub collection_size: u64,
    /// collection_size * ratio_base (<= total_supply_base).
    pub max_tokens_in_nft_form: u64,
    /// The flat SOL fee (lamports) charged on EVERY capture, release and re-roll, fixed forever
    /// (ADR-013) = fee_for_ratio(ratio_whole_tokens) at launch; <= MAX_FEE_LAMPORTS (0.01 SOL).
    /// There is no token fee.
    pub fee_lamports: u64,
    /// PLATFORM_FEE_RECIPIENT at launch time: the only address any fee can reach (F-06 / M-05).
    pub fee_recipient: Pubkey,
    /// Graduation threshold (lamports raised; DBC migration threshold). `graduation_slice_pct` is
    /// always 0 since lazy minting (ADR-016); the field stays for the append-only layout.
    pub graduation_threshold_lamports: u64,
    pub graduation_slice_pct: u8,
    pub launched_at: i64,
}
