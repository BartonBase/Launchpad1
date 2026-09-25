//! Raw-layout pool account (too large for Borsh/Anchor `init`; pre-created top-level by the creator,
//! owned by hybrid_vault, then initialised by `init_vault`). LAZY MINT layout v2 (ADR-016,
//! docs/lazy-mint-interface.md): every index 0..N is drawable from the start, minted or not.
//!
//! Layout (little endian):
//!   [0..8)   discriminator  POOL_DISCRIMINATOR ("hvpool02")
//!   [8..40)  vault pubkey
//!   [40..44) capacity (= collection_size)
//!   [44..48) pool_len
//!   [48..52) incoming_head (ring index)
//!   [52..56) incoming_len
//!   [56..64) reserved
//!   [64 .. 64+4*cap)            pool slots: u32 (asset index + 1); 0 = "the slot's own position"
//!                               (lazy Fisher-Yates: O(1) init with pool_len = cap), swap_remove
//!   [64+4*cap .. 64+16*cap)     incoming ring: (u32 asset index, u64 tag), FIFO, tags non-decreasing
//!   [64+16*cap .. +ceil(cap/8)) minted bitmap (bit i = asset i exists; set once, never cleared)
//!
//! Selection fairness (hybrid-rarity doc §3.3): only `pool` entries are drawable. An unwrapped NFT
//! enters `incoming` tagged with the vault's `next_seq` at that time, and is merged into `pool` only
//! when settling a request whose seq >= tag. So NFTs deposited after a request was made (i.e. after
//! its randomness could possibly be known) are never candidates for it.

use crate::error::VaultError;
use anchor_lang::prelude::*;

pub const POOL_DISCRIMINATOR: [u8; 8] = *b"hvpool02";
pub const POOL_HEADER: usize = 64;
const INCOMING_ENTRY: usize = 12;

pub fn pool_account_size(capacity: u32) -> usize {
    POOL_HEADER + 16 * capacity as usize + (capacity as usize).div_ceil(8)
}

fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn wr_u32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn wr_u64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}

/// Initialise a zeroed, correctly-sized buffer.
pub fn init(d: &mut [u8], vault: &Pubkey, capacity: u32) -> Result<()> {
    require!(d.len() == pool_account_size(capacity), VaultError::InvalidPoolAccount);
    require!(d[..8] == [0u8; 8], VaultError::InvalidPoolAccount);
    d[..8].copy_from_slice(&POOL_DISCRIMINATOR);
    d[8..40].copy_from_slice(vault.as_ref());
    wr_u32(d, 40, capacity);
    // Every index is drawable from the start (slots are 0 = identity, so no per-index writes).
    wr_u32(d, 44, capacity);
    wr_u32(d, 48, 0);
    wr_u32(d, 52, 0);
    Ok(())
}

pub struct PoolView<'a> {
    d: &'a mut [u8],
    cap: u32,
}

impl<'a> PoolView<'a> {
    /// Validate discriminator, owning vault and size.
    pub fn load(d: &'a mut [u8], vault: &Pubkey) -> Result<Self> {
        require!(d.len() >= POOL_HEADER, VaultError::InvalidPoolAccount);
        require!(d[..8] == POOL_DISCRIMINATOR, VaultError::InvalidPoolAccount);
        require!(&d[8..40] == vault.as_ref(), VaultError::PoolMismatch);
        let cap = rd_u32(d, 40);
        require!(d.len() == pool_account_size(cap), VaultError::InvalidPoolAccount);
        Ok(Self { d, cap })
    }
    pub fn capacity(&self) -> u32 {
        self.cap
    }
    pub fn pool_len(&self) -> u32 {
        rd_u32(self.d, 44)
    }
    pub fn incoming_len(&self) -> u32 {
        rd_u32(self.d, 52)
    }
    fn incoming_head(&self) -> u32 {
        rd_u32(self.d, 48)
    }
    fn pool_off(&self, i: u32) -> usize {
        POOL_HEADER + 4 * i as usize
    }
    fn inc_off(&self, ring_i: u32) -> usize {
        POOL_HEADER + 4 * self.cap as usize + INCOMING_ENTRY * ring_i as usize
    }
    fn total(&self) -> Result<u32> {
        self.pool_len().checked_add(self.incoming_len()).ok_or_else(|| error!(VaultError::MathOverflow))
    }

    pub fn pool_get(&self, i: u32) -> u32 {
        match rd_u32(self.d, self.pool_off(i)) {
            0 => i,
            v => v - 1,
        }
    }
    fn pool_set(&mut self, i: u32, asset_index: u32) {
        let o = self.pool_off(i);
        wr_u32(self.d, o, asset_index + 1);
    }

    fn bitmap_off(&self) -> usize {
        POOL_HEADER + 16 * self.cap as usize
    }
    pub fn is_minted(&self, index: u32) -> bool {
        index < self.cap && self.d[self.bitmap_off() + (index / 8) as usize] & (1 << (index % 8)) != 0
    }
    /// Set the minted bit for `index`; errors if it was already set (each index is minted at most once).
    pub fn mark_minted(&mut self, index: u32) -> Result<()> {
        require!(index < self.cap, VaultError::IndexOutOfRange);
        require!(!self.is_minted(index), VaultError::AlreadyMinted);
        let o = self.bitmap_off() + (index / 8) as usize;
        self.d[o] |= 1 << (index % 8);
        Ok(())
    }
    /// Population count of the bitmap (tests / off-chain audits; O(N/8)).
    pub fn minted_popcount(&self) -> u32 {
        let o = self.bitmap_off();
        self.d[o..o + (self.cap as usize).div_ceil(8)].iter().map(|b| b.count_ones()).sum()
    }

    pub fn pool_push(&mut self, asset_index: u32) -> Result<()> {
        require!(self.total()? < self.cap, VaultError::AssetAccountingBroken);
        require!(asset_index < self.cap, VaultError::IndexOutOfRange);
        let len = self.pool_len();
        self.pool_set(len, asset_index);
        wr_u32(self.d, 44, len + 1);
        Ok(())
    }

    /// Remove and return the entry at position `i` (swap with last).
    pub fn pool_swap_remove(&mut self, i: u32) -> Result<u32> {
        let len = self.pool_len();
        require!(i < len, VaultError::NoAssetAvailable);
        let picked = self.pool_get(i);
        let last = self.pool_get(len - 1);
        self.pool_set(i, last);
        wr_u32(self.d, 44, len - 1);
        Ok(picked)
    }

    /// Append to the incoming FIFO. Tags must be non-decreasing (callers pass vault.next_seq).
    pub fn incoming_push(&mut self, asset_index: u32, tag: u64) -> Result<()> {
        require!(self.total()? < self.cap, VaultError::AssetAccountingBroken);
        let len = self.incoming_len();
        if len > 0 {
            let back = (self.incoming_head() + len - 1) % self.cap;
            let back_tag = rd_u64(self.d, self.inc_off(back) + 4);
            require!(tag >= back_tag, VaultError::AssetAccountingBroken);
        }
        let slot = (self.incoming_head() + len) % self.cap;
        let o = self.inc_off(slot);
        wr_u32(self.d, o, asset_index);
        wr_u64(self.d, o + 4, tag);
        wr_u32(self.d, 52, len + 1);
        Ok(())
    }

    /// Tag of the oldest incoming entry, if any.
    pub fn incoming_front_tag(&self) -> Option<u64> {
        if self.incoming_len() == 0 {
            None
        } else {
            Some(rd_u64(self.d, self.inc_off(self.incoming_head()) + 4))
        }
    }

    /// Move up to `max` incoming entries with `tag <= max_tag` into the pool. Returns how many moved.
    pub fn merge(&mut self, max_tag: u64, max: u32) -> Result<u32> {
        let mut moved = 0;
        while moved < max {
            match self.incoming_front_tag() {
                Some(t) if t <= max_tag => {
                    let head = self.incoming_head();
                    let idx = rd_u32(self.d, self.inc_off(head));
                    let len = self.incoming_len();
                    wr_u32(self.d, 48, (head + 1) % self.cap);
                    wr_u32(self.d, 52, len - 1);
                    self.pool_push(idx)?;
                    moved += 1;
                }
                _ => break,
            }
        }
        Ok(moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pool with every index drawn out (pool_len 0), for the FIFO tests.
    fn buf(cap: u32) -> (Vec<u8>, Pubkey) {
        let v = Pubkey::new_unique();
        let mut d = vec![0u8; pool_account_size(cap)];
        init(&mut d, &v, cap).unwrap();
        wr_u32(&mut d, 44, 0);
        (d, v)
    }

    #[test]
    fn lazy_init_makes_every_index_drawable_exactly_once() {
        let v = Pubkey::new_unique();
        let cap = 1000;
        let mut d = vec![0u8; pool_account_size(cap)];
        init(&mut d, &v, cap).unwrap();
        let mut p = PoolView::load(&mut d, &v).unwrap();
        assert_eq!(p.pool_len(), cap);
        let mut seen = vec![false; cap as usize];
        let mut x: u64 = 7;
        while p.pool_len() > 0 {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let i = ((x >> 33) % p.pool_len() as u64) as u32;
            let idx = p.pool_swap_remove(i).unwrap();
            assert!(!seen[idx as usize], "index {idx} drawn twice");
            seen[idx as usize] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    fn minted_bitmap_sets_once_and_counts() {
        let v = Pubkey::new_unique();
        let mut d = vec![0u8; pool_account_size(10_000)];
        init(&mut d, &v, 10_000).unwrap();
        assert_eq!(pool_account_size(10_000), 64 + 160_000 + 1_250);
        let mut p = PoolView::load(&mut d, &v).unwrap();
        for i in [0u32, 7, 8, 9_999] {
            assert!(!p.is_minted(i));
            p.mark_minted(i).unwrap();
            assert!(p.is_minted(i));
            assert!(p.mark_minted(i).is_err(), "never minted twice");
        }
        assert_eq!(p.minted_popcount(), 4);
        assert!(p.mark_minted(10_000).is_err());
    }

    #[test]
    fn deposits_after_a_request_are_never_merged_for_it() {
        let (mut d, v) = buf(4);
        let mut p = PoolView::load(&mut d, &v).unwrap();
        p.pool_push(0).unwrap();
        p.incoming_push(1, 5).unwrap(); // unwrapped before request 5 was made
        p.incoming_push(2, 6).unwrap(); // unwrapped after request 5 was made
        assert_eq!(p.merge(5, 32).unwrap(), 1);
        assert_eq!(p.pool_len(), 2);
        assert_eq!(p.incoming_front_tag(), Some(6));
    }

    #[test]
    fn incoming_tags_must_not_decrease() {
        let (mut d, v) = buf(4);
        let mut p = PoolView::load(&mut d, &v).unwrap();
        p.incoming_push(1, 7).unwrap();
        assert!(p.incoming_push(2, 6).is_err());
    }

    #[test]
    fn capacity_is_never_exceeded_and_ring_wraps() {
        let (mut d, v) = buf(3);
        let mut p = PoolView::load(&mut d, &v).unwrap();
        for round in 0..10u64 {
            p.incoming_push(round as u32 % 3, round).unwrap();
            p.merge(round, 32).unwrap();
            let picked = p.pool_swap_remove(0).unwrap();
            assert_eq!(picked, round as u32 % 3);
        }
        p.pool_push(0).unwrap();
        p.pool_push(1).unwrap();
        p.incoming_push(2, 99).unwrap();
        assert!(p.pool_push(0).is_err(), "4th entry in a 3-capacity pool must fail");
    }

    #[test]
    fn foreign_vault_pool_is_rejected() {
        let (mut d, _v) = buf(2);
        assert!(PoolView::load(&mut d, &Pubkey::new_unique()).is_err());
    }
}
