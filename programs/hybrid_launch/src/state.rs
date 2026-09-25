use anchor_lang::prelude::*;

/// Immutable launch record for one Track A mint. PDA `["launch_config", mint]`.
///
/// There is NO instruction that modifies or closes this account: ratio,
/// collection size, fees, fee destination and mint are fixed at launch
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
    /// Capture (wrap) fee, bps of the ratio, and the exact base-unit amount.
    pub capture_fee_bps: u16,
    pub capture_fee_amount: u64,
    /// Re-roll fee, bps of the ratio, and the exact base-unit amount.
    pub reroll_fee_bps: u16,
    pub reroll_fee_amount: u64,
    /// Always FEE_DESTINATION_BURN.
    pub fee_destination: u8,
    pub launched_at: i64,
}
