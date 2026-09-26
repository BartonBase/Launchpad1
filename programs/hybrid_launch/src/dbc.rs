//! Read-only parsers for Meteora Dynamic Bonding Curve (DBC) accounts (ADR-014).
//!
//! We don't depend on the DBC crate: it's pinned to an older Anchor. We read the two zero-copy
//! accounts we need at fixed offsets taken from DBC 0.2.1 (`state/config.rs` `PoolConfig`, 1040 B + 8;
//! `state/virtual_pool.rs` `VirtualPool`, 416 B + 8). The offsets were checked against real devnet
//! accounts (tests/track-a-hybrid/fixtures/dbc) and the real DBC program in LiteSVM
//! (`launch/dbc.rs`). Each read checks owner == DBC_PROGRAM_ID, exact length and the Anchor
//! discriminator, so nothing else can be passed off as a DBC account.

use anchor_lang::prelude::*;

/// Meteora DBC program (same ID on mainnet and devnet).
pub const DBC_PROGRAM_ID: Pubkey = pubkey!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");
/// DBC's `pool_authority` const PDA (`["pool_authority"]` under DBC). It owns the curve's base vault.
pub const DBC_POOL_AUTHORITY: Pubkey = pubkey!("FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM");
pub const WRAPPED_SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// sha256("account:PoolConfig")[..8]
pub const DBC_CONFIG_DISCRIMINATOR: [u8; 8] = [26, 108, 14, 123, 116, 230, 129, 43];
/// sha256("account:VirtualPool")[..8]
pub const DBC_POOL_DISCRIMINATOR: [u8; 8] = [213, 224, 5, 209, 98, 69, 119, 92];
pub const DBC_CONFIG_LEN: usize = 8 + 1040;
pub const DBC_POOL_LEN: usize = 8 + 416;

// PoolConfig offsets (including the 8-byte discriminator).
pub const CFG_OFF_QUOTE_MINT: usize = 8;
pub const CFG_OFF_LEFTOVER_RECEIVER: usize = 72;
pub const CFG_OFF_TOKEN_DECIMAL: usize = 235;
pub const CFG_OFF_TOKEN_TYPE: usize = 237;
pub const CFG_OFF_FIXED_SUPPLY_FLAG: usize = 244;
pub const CFG_OFF_TOKEN_UPDATE_AUTHORITY: usize = 246;
pub const CFG_OFF_MIGRATION_QUOTE_THRESHOLD: usize = 264;
pub const CFG_OFF_PRE_MIGRATION_SUPPLY: usize = 344;
pub const CFG_OFF_POST_MIGRATION_SUPPLY: usize = 352;

// VirtualPool offsets (including the 8-byte discriminator).
pub const POOL_OFF_CONFIG: usize = 72;
pub const POOL_OFF_CREATOR: usize = 104;
pub const POOL_OFF_BASE_MINT: usize = 136;
pub const POOL_OFF_BASE_VAULT: usize = 168;
pub const POOL_OFF_POOL_TYPE: usize = 304;
pub const POOL_OFF_IS_MIGRATED: usize = 305;
pub const POOL_OFF_MIGRATION_PROGRESS: usize = 308;

/// DBC `TokenType::SplToken` / `PoolType::SplToken`.
pub const DBC_TOKEN_TYPE_SPL: u8 = 0;
/// DBC `TokenAuthorityOption::Immutable` (metadata update authority None, no mint authority).
pub const DBC_TOKEN_AUTHORITY_IMMUTABLE: u8 = 1;
/// DBC `MigrationProgress::CreatedPool`: the DAMM pool exists, the curve is done.
pub const DBC_MIGRATION_CREATED_POOL: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbcConfig {
    pub quote_mint: Pubkey,
    pub leftover_receiver: Pubkey,
    pub token_decimal: u8,
    pub token_type: u8,
    pub fixed_token_supply: bool,
    pub token_update_authority: u8,
    pub migration_quote_threshold: u64,
    pub pre_migration_token_supply: u64,
    pub post_migration_token_supply: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbcPool {
    pub config: Pubkey,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub base_vault: Pubkey,
    pub pool_type: u8,
    pub is_migrated: bool,
    pub migration_progress: u8,
}

impl DbcPool {
    /// Graduated = migrated to the DAMM pool (DBC sets `is_migrated = 1` and progress = CreatedPool in
    /// the same migration instruction).
    pub fn graduated(&self) -> bool {
        self.is_migrated && self.migration_progress == DBC_MIGRATION_CREATED_POOL
    }
}

fn pk(d: &[u8], off: usize) -> Pubkey {
    Pubkey::new_from_array(d[off..off + 32].try_into().unwrap())
}
fn u64_at(d: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(d[off..off + 8].try_into().unwrap())
}

/// Parse raw PoolConfig bytes (no owner check; use `load_config` for accounts).
pub fn parse_config(d: &[u8]) -> Option<DbcConfig> {
    if d.len() != DBC_CONFIG_LEN || d[..8] != DBC_CONFIG_DISCRIMINATOR {
        return None;
    }
    Some(DbcConfig {
        quote_mint: pk(d, CFG_OFF_QUOTE_MINT),
        leftover_receiver: pk(d, CFG_OFF_LEFTOVER_RECEIVER),
        token_decimal: d[CFG_OFF_TOKEN_DECIMAL],
        token_type: d[CFG_OFF_TOKEN_TYPE],
        fixed_token_supply: d[CFG_OFF_FIXED_SUPPLY_FLAG] == 1,
        token_update_authority: d[CFG_OFF_TOKEN_UPDATE_AUTHORITY],
        migration_quote_threshold: u64_at(d, CFG_OFF_MIGRATION_QUOTE_THRESHOLD),
        pre_migration_token_supply: u64_at(d, CFG_OFF_PRE_MIGRATION_SUPPLY),
        post_migration_token_supply: u64_at(d, CFG_OFF_POST_MIGRATION_SUPPLY),
    })
}

/// Parse raw VirtualPool bytes (no owner check; use `load_pool` for accounts).
pub fn parse_pool(d: &[u8]) -> Option<DbcPool> {
    if d.len() != DBC_POOL_LEN || d[..8] != DBC_POOL_DISCRIMINATOR {
        return None;
    }
    Some(DbcPool {
        config: pk(d, POOL_OFF_CONFIG),
        creator: pk(d, POOL_OFF_CREATOR),
        base_mint: pk(d, POOL_OFF_BASE_MINT),
        base_vault: pk(d, POOL_OFF_BASE_VAULT),
        pool_type: d[POOL_OFF_POOL_TYPE],
        is_migrated: d[POOL_OFF_IS_MIGRATED] == 1,
        migration_progress: d[POOL_OFF_MIGRATION_PROGRESS],
    })
}

pub fn load_config(a: &AccountInfo) -> Option<DbcConfig> {
    if *a.owner != DBC_PROGRAM_ID {
        return None;
    }
    parse_config(&a.try_borrow_data().ok()?)
}

pub fn load_pool(a: &AccountInfo) -> Option<DbcPool> {
    if *a.owner != DBC_PROGRAM_ID {
        return None;
    }
    parse_pool(&a.try_borrow_data().ok()?)
}

/// The single hybrid_launch PDA that DBC configs must name as `leftover_receiver` (the "25% buffer").
/// DBC's permissionless `withdraw_leftover` sends the unsold buffer to ATA(this PDA, mint). hybrid_launch
/// has NO instruction that signs with these seeds, so the tokens there can never move (T-GRAD-03).
pub fn buffer_authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[crate::constants::DBC_BUFFER_SEED], &crate::ID)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_consistent_with_dbc_0_2_1() {
        assert_eq!(DBC_CONFIG_LEN, 1048);
        assert_eq!(DBC_POOL_LEN, 424);
        // PoolState: VolatilityTracker (64) then config, creator, base_mint, base_vault.
        assert_eq!(POOL_OFF_CONFIG, 8 + 64);
        assert_eq!(POOL_OFF_BASE_VAULT, POOL_OFF_BASE_MINT + 32);
        // 5 pubkeys + 6 u64 + u128 sqrt_price + u64 activation_point, then pool_type.
        assert_eq!(POOL_OFF_POOL_TYPE, 8 + 64 + 5 * 32 + 6 * 8 + 16 + 8);
        assert_eq!(POOL_OFF_IS_MIGRATED, POOL_OFF_POOL_TYPE + 1);
        // quote_mint, fee_claimer, leftover_receiver, PoolFeesConfig (80), 2 x LiquidityVestingInfo (16), 16 B pad.
        assert_eq!(CFG_OFF_LEFTOVER_RECEIVER, 8 + 64);
        assert_eq!(CFG_OFF_TOKEN_DECIMAL, 8 + 96 + 80 + 32 + 16 + 3);
        assert_eq!(CFG_OFF_MIGRATION_QUOTE_THRESHOLD, 8 + 248 + 8);
        // migration_sqrt_price (16) + LockedVestingConfig (48) after migration_base_threshold.
        assert_eq!(CFG_OFF_PRE_MIGRATION_SUPPLY, 8 + 272 + 16 + 48);
    }

    #[test]
    fn rejects_wrong_length_or_discriminator() {
        let mut d = vec![0u8; DBC_POOL_LEN];
        assert!(parse_pool(&d).is_none());
        d[..8].copy_from_slice(&DBC_POOL_DISCRIMINATOR);
        assert!(parse_pool(&d).is_some());
        d.push(0);
        assert!(parse_pool(&d).is_none());
        let mut c = vec![0u8; DBC_CONFIG_LEN];
        c[..8].copy_from_slice(&DBC_POOL_DISCRIMINATOR);
        assert!(parse_config(&c).is_none());
    }
}
