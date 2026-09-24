//! Raw-layout pool account (too large for Borsh/Anchor `init` at 100k NFTs; pre-created top-level
//! by the creator, owned by hybrid_vault, then initialised by `init_vault`).
//!
//! Layout (little endian):
//!   [0..8)   discriminator  POOL_DISCRIMINATOR
//!   [8..40)  vault pubkey
//!   [40..44) capacity (= collection_size)
//!   [44..48) pool_len
//!   [48..52) incoming_head (ring index)
//!   [52..56) incoming_len
//!   [56..64) reserved
//!   [64 .. 64+4*cap)            pool entries: u32 asset index, unordered (swap_remove)
//!   [64+4*cap .. 64+16*cap)     incoming ring: (u32 asset index, u64 tag), FIFO, tags non-decreasing
//!
//! Selection fairness (hybrid-rarity doc §3.3): only `pool` entries are drawable. An unwrapped NFT
//! enters `incoming` tagged with the vault's `next_seq` at that time, and is merged into `pool` only
//! when settling a request whose seq >= tag. So NFTs deposited after a request was made (i.e. after
//! its randomness could possibly be known) are never candidates for it.

use crate::error::VaultError;
use anchor_lang::prelude::*;

pub const POOL_DISCRIMINATOR: [u8; 8] = *b"hvpool01";
pub const POOL_HEADER: usize = 64;
const INCOMING_ENTRY: usize = 12;

pub fn pool_account_size(capacity: u32) -> usize {
    POOL_HEADER + 16 * capacity as usize
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
    wr_u32(d, 44, 0);
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
        rd_u32(self.d, self.pool_off(i))
    }

    pub fn pool_push(&mut self, asset_index: u32) -> Result<()> {
        require!(self.total()? < self.cap, VaultError::AssetAccountingBroken);
        let len = self.pool_len();
        let o = self.pool_off(len);
        wr_u32(self.d, o, asset_index);
        wr_u32(self.d, 44, len + 1);
        Ok(())
    }

    /// Remove and return the entry at position `i` (swap with last).
    pub fn pool_swap_remove(&mut self, i: u32) -> Result<u32> {
        let len = self.pool_len();
        require!(i < len, VaultError::NoAssetAvailable);
        let picked = self.pool_get(i);
        let last = self.pool_get(len - 1);
        let o = self.pool_off(i);
        wr_u32(self.d, o, last);
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

    fn buf(cap: u32) -> (Vec<u8>, Pubkey) {
        let v = Pubkey::new_unique();
        let mut d = vec![0u8; pool_account_size(cap)];
        init(&mut d, &v, cap).unwrap();
        (d, v)
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
