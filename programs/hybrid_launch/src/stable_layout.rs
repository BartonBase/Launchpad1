//! FROZEN byte offsets into a serialized `LaunchConfig` account (audit M-41).
//!
//! hybrid_vault's user-exit paths (release, settle, expire, sweep) read ONLY these fields as raw bytes
//! so a hybrid_launch upgrade (new fee tiers, a rotated fee wallet, a version bump, new MIN/MAX
//! bounds, or new fields) can never freeze them. RULES, enforced by the `offsets_are_frozen` test:
//! - `LaunchConfig` is APPEND-ONLY. Never reorder, resize, or remove a field that precedes the
//!   last offset below; new fields go at the END of the struct.
//! - These constants are never changed. If a field has to move, that's a new account type, not an edit.
//! Offsets include the 8-byte Anchor discriminator.

pub const LC_DISCRIMINATOR_LEN: usize = 8;
pub const LC_OFF_VERSION: usize = 8;
pub const LC_OFF_MINT: usize = 43;
pub const LC_OFF_DECIMALS: usize = 140;
pub const LC_OFF_RATIO_WHOLE_TOKENS: usize = 149;
pub const LC_OFF_RATIO_BASE: usize = 157;
pub const LC_OFF_COLLECTION_SIZE: usize = 165;
pub const LC_OFF_FEE_LAMPORTS: usize = 181;
pub const LC_OFF_FEE_RECIPIENT: usize = 189;
/// Every stable field ends at or before this byte.
pub const LC_STABLE_PREFIX_LEN: usize = 221;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::LaunchConfig;
    use anchor_lang::prelude::*;
    use anchor_lang::{AccountSerialize, Discriminator};

    fn sample() -> LaunchConfig {
        LaunchConfig {
            version: 0xA1,
            bump: 1,
            mint_authority_bump: 2,
            creator: Pubkey::new_from_array([3; 32]),
            mint: Pubkey::new_from_array([4; 32]),
            launch_destination: Pubkey::new_from_array([5; 32]),
            launch_vault: Pubkey::new_from_array([6; 32]),
            launch_vault_bump: 7,
            decimals: 6,
            total_supply_base: 0x0808_0808_0808_0808,
            ratio_whole_tokens: 0x0909_0909_0909_0909,
            ratio_base: 0x0A0A_0A0A_0A0A_0A0A,
            collection_size: 0x0B0B_0B0B_0B0B_0B0B,
            max_tokens_in_nft_form: 0x0C0C_0C0C_0C0C_0C0C,
            fee_lamports: 0x0D0D_0D0D_0D0D_0D0D,
            fee_recipient: Pubkey::new_from_array([0x0E; 32]),
            graduation_threshold_lamports: 15,
            graduation_slice_pct: 16,
            launched_at: 17,
            dbc_config: Pubkey::new_from_array([8; 32]),
            dbc_pool: Pubkey::new_from_array([9; 32]),
        }
    }

    #[test]
    fn offsets_are_frozen() {
        let c = sample();
        let mut buf = Vec::new();
        c.try_serialize(&mut buf).unwrap();
        assert_eq!(&buf[..LC_DISCRIMINATOR_LEN], LaunchConfig::DISCRIMINATOR);
        let u64_at = |o: usize| u64::from_le_bytes(buf[o..o + 8].try_into().unwrap());
        assert_eq!(buf[LC_OFF_VERSION], 0xA1);
        assert_eq!(&buf[LC_OFF_MINT..LC_OFF_MINT + 32], c.mint.as_ref());
        assert_eq!(buf[LC_OFF_DECIMALS], 6);
        assert_eq!(u64_at(LC_OFF_RATIO_WHOLE_TOKENS), c.ratio_whole_tokens);
        assert_eq!(u64_at(LC_OFF_RATIO_BASE), c.ratio_base);
        assert_eq!(u64_at(LC_OFF_COLLECTION_SIZE), c.collection_size);
        assert_eq!(u64_at(LC_OFF_FEE_LAMPORTS), c.fee_lamports);
        assert_eq!(&buf[LC_OFF_FEE_RECIPIENT..LC_OFF_FEE_RECIPIENT + 32], c.fee_recipient.as_ref());
        assert_eq!(LC_OFF_FEE_RECIPIENT + 32, LC_STABLE_PREFIX_LEN);
        assert!(buf.len() >= LC_STABLE_PREFIX_LEN);
    }

    /// Simulates a future upgrade that APPENDS a field: the stable prefix bytes are unchanged, so an
    /// old reader still gets the right values from a newer, longer account.
    #[test]
    fn appended_field_keeps_prefix() {
        let mut buf = Vec::new();
        sample().try_serialize(&mut buf).unwrap();
        let mut newer = buf.clone();
        newer.extend_from_slice(&[0xFF; 64]); // appended fields
        assert_eq!(&newer[..LC_STABLE_PREFIX_LEN], &buf[..LC_STABLE_PREFIX_LEN]);
    }
}
