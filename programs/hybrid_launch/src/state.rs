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
    // ---- v4 (appended; ADR-014) ----
    /// DBC config of the curve (`register_dbc_launch`); default for native `launch` (never opens).
    pub dbc_config: Pubkey,
    /// DBC virtual pool of the curve. `hybrid_vault::graduation::verify` only accepts this pool, and
    /// only once DBC reports it migrated. Default for native `launch` => the vault can never open.
    pub dbc_pool: Pubkey,
}

/// Immutable Mode 1 record: a classic SPL memecoin with no NFT pairing.
///
/// Same PDA seeds as [`LaunchConfig`] (`["launch_config", mint]`), different account type, so a mint
/// is one mode or the other and `hybrid_vault` cannot load this as a hybrid launch. There is no
/// instruction that modifies or closes it, and no field a creator can set after init.
///
/// Native [`crate::launch_plain`] leaves `dbc_config` / `dbc_pool` default and parks the supply in
/// the launch-vault ATA (nothing signs for that PDA). [`crate::register_plain_dbc`] is the curve
/// path: DBC already minted the supply into its base vault; this record only binds an allowlisted
/// config. Neither path has a ratio, a collection, a fee, or a wrap.
#[account]
#[derive(InitSpace, Debug, PartialEq, Eq)]
pub struct PlainLaunchConfig {
    pub version: u8,
    pub bump: u8,
    /// 0 when DBC created the mint (`register_plain_dbc`). The native path stores the mint-authority bump.
    pub mint_authority_bump: u8,
    /// Always [`crate::LAUNCH_MODE_PLAIN`].
    pub launch_mode: u8,
    /// Who paid. Holds no powers.
    pub creator: Pubkey,
    /// Classic SPL Token mint; mint + freeze authority are `None`.
    pub mint: Pubkey,
    /// Native: ATA(launch_vault, mint). DBC: the pool's base vault.
    pub launch_destination: Pubkey,
    /// Native: PDA `["launch_vault", mint, launch_config]`. DBC: DBC's pool authority.
    pub launch_vault: Pubkey,
    pub launch_vault_bump: u8,
    pub decimals: u8,
    /// 1_000_000_000 * 10^decimals.
    pub total_supply_base: u64,
    pub launched_at: i64,
    /// Default for native `launch_plain`. Allowlisted config for `register_plain_dbc`.
    pub dbc_config: Pubkey,
    /// Default for native `launch_plain` (not on a curve). The DBC pool for `register_plain_dbc`.
    pub dbc_pool: Pubkey,
    /// 0 for native `launch_plain`. Copied from the allowlisted DBC config on the curve path.
    pub graduation_threshold_lamports: u64,
}

/// Immutable Mode 3 record: classic SPL, fixed ratio, known collection, burn-on-wrap.
///
/// Same PDA seeds as [`LaunchConfig`] (`["launch_config", mint]`), different account type. Mode 2's
/// vault loads only `LaunchConfig`, so this mint cannot be wrapped reversibly and cannot be released.
/// There is no instruction that modifies or closes it.
#[account]
#[derive(InitSpace, Debug, PartialEq, Eq)]
pub struct BurnLaunchConfig {
    pub version: u8,
    pub bump: u8,
    /// 0 when DBC created the mint (`register_burn_dbc`). The native path stores the mint-authority bump.
    pub mint_authority_bump: u8,
    /// Always [`crate::LAUNCH_MODE_BURN`].
    pub launch_mode: u8,
    /// Who paid. Holds no powers.
    pub creator: Pubkey,
    /// Classic SPL Token mint; mint + freeze authority are `None`.
    pub mint: Pubkey,
    /// Native: ATA(launch_vault, mint). DBC: the pool's base vault.
    pub launch_destination: Pubkey,
    /// Native: PDA `["launch_vault", mint, launch_config]`. DBC: DBC's pool authority.
    pub launch_vault: Pubkey,
    pub launch_vault_bump: u8,
    pub decimals: u8,
    /// 1_000_000_000 * 10^decimals.
    pub total_supply_base: u64,
    /// Whole tokens per NFT (one of ALLOWED_RATIOS). Fixed at creation.
    pub ratio_whole_tokens: u64,
    /// ratio_whole_tokens * 10^decimals.
    pub ratio_base: u64,
    /// Number of NFTs in the collection. Fixed at creation.
    pub collection_size: u64,
    /// collection_size * ratio_base (<= total_supply_base).
    pub max_tokens_in_nft_form: u64,
    /// Flat SOL fee charged on wrap. Exact tier for the ratio. No release fee exists.
    pub fee_lamports: u64,
    /// PLATFORM_FEE_RECIPIENT at launch. The only fee destination.
    pub fee_recipient: Pubkey,
    pub graduation_threshold_lamports: u64,
    /// Always 0 (lazy mint). Kept so the economics match the Mode 2 derivation.
    pub graduation_slice_pct: u8,
    pub launched_at: i64,
    /// Default for native `launch_burn`. Allowlisted config for `register_burn_dbc`.
    pub dbc_config: Pubkey,
    /// Default for native `launch_burn` (the vault can never open). The DBC pool otherwise.
    pub dbc_pool: Pubkey,
}

/// Immutable Mode 4 record: Token-2022 with a transfer tax, fixed at launch.
///
/// Same PDA seeds as [`LaunchConfig`], different account type. The tax rate cannot be changed
/// (`transfer_fee_config_authority` is None). Withheld tokens withdraw only to the vault program's
/// tax PDA, which pays them to holders of this launch's NFTs. This mode does not burn.
#[account]
#[derive(InitSpace, Debug, PartialEq, Eq)]
pub struct T22BurnLaunchConfig {
    pub version: u8,
    pub bump: u8,
    pub mint_authority_bump: u8,
    /// Always [`crate::LAUNCH_MODE_TOKEN22`].
    pub launch_mode: u8,
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub launch_destination: Pubkey,
    pub launch_vault: Pubkey,
    pub launch_vault_bump: u8,
    pub decimals: u8,
    pub total_supply_base: u64,
    pub ratio_whole_tokens: u64,
    pub ratio_base: u64,
    pub collection_size: u64,
    pub max_tokens_in_nft_form: u64,
    pub fee_lamports: u64,
    pub fee_recipient: Pubkey,
    pub graduation_threshold_lamports: u64,
    pub graduation_slice_pct: u8,
    pub launched_at: i64,
    pub dbc_config: Pubkey,
    pub dbc_pool: Pubkey,
    /// Transfer-tax rate in basis points. Chosen at launch. Not updatable.
    pub tax_bps: u16,
    /// `hybrid_vault` PDA `["tax_authority", mint]`. The only withdraw-withheld authority.
    pub tax_authority: Pubkey,
    /// Lamports of SOL per whole token. Chosen at launch. Not updatable.
    pub buyback_lamports_per_whole: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::Discriminator;

    #[test]
    fn plain_account_is_not_a_hybrid_launch_config() {
        assert_ne!(
            PlainLaunchConfig::DISCRIMINATOR,
            LaunchConfig::DISCRIMINATOR
        );
        assert_eq!(crate::constants::LAUNCH_MODE_PLAIN, 1);
        assert_eq!(crate::constants::LAUNCH_MODE_HYBRID, 2);
        assert_eq!(crate::constants::LAUNCH_MODE_BURN, 3);
        assert_ne!(BurnLaunchConfig::DISCRIMINATOR, LaunchConfig::DISCRIMINATOR);
        assert_ne!(
            BurnLaunchConfig::DISCRIMINATOR,
            PlainLaunchConfig::DISCRIMINATOR
        );
        assert_eq!(crate::constants::LAUNCH_MODE_TOKEN22, 4);
        assert_ne!(
            T22BurnLaunchConfig::DISCRIMINATOR,
            BurnLaunchConfig::DISCRIMINATOR
        );
        assert_ne!(
            T22BurnLaunchConfig::DISCRIMINATOR,
            LaunchConfig::DISCRIMINATOR
        );
    }
}
