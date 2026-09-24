//! Uniform selection from the VRF value. Never uses SlotHashes, Clock or blockhash.

use anchor_lang::prelude::Pubkey;
use solana_sha256_hasher::hashv;

/// Domain-separated per-request randomness: the same VRF value can't be replayed for another
/// request or vault.
pub fn request_randomness(vrf_value: &[u8; 32], vault: &Pubkey, seq: u64) -> [u8; 32] {
    hashv(&[b"hybrid_vault:select", vrf_value, vault.as_ref(), &seq.to_le_bytes()]).to_bytes()
}

/// Uniform index in [0, n) by rejection sampling on 64-bit words (re-hashing when all four words of
/// a block are rejected). Bias-free. `n` must be > 0.
pub fn uniform_below(r: &[u8; 32], n: u32) -> u32 {
    assert!(n > 0);
    let n64 = n as u64;
    // largest multiple of n that fits in u64 range: accept x < zone
    let zone = u64::MAX - (u64::MAX % n64 + 1) % n64;
    let mut block = *r;
    loop {
        for chunk in block.chunks_exact(8) {
            let x = u64::from_le_bytes(chunk.try_into().unwrap());
            if x <= zone {
                return (x % n64) as u32;
            }
        }
        block = hashv(&[b"hybrid_vault:resample", &block]).to_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_below_stays_in_range_and_is_roughly_uniform() {
        let n = 7u32;
        let mut counts = [0u32; 7];
        for i in 0..70_000u64 {
            let r = hashv(&[&i.to_le_bytes()]).to_bytes();
            let k = uniform_below(&r, n);
            assert!(k < n);
            counts[k as usize] += 1;
        }
        for c in counts {
            assert!((9_300..10_700).contains(&c), "{counts:?}");
        }
    }

    #[test]
    fn request_randomness_is_domain_separated() {
        let v = [7u8; 32];
        let a = Pubkey::new_unique();
        assert_ne!(request_randomness(&v, &a, 1), request_randomness(&v, &a, 2));
        assert_ne!(request_randomness(&v, &a, 1), request_randomness(&v, &Pubkey::new_unique(), 1));
    }
}
