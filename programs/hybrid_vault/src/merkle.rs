//! Art-binding commitment, leaf format v2 (docs/graduation-design.md §5.1; reference implementation
//! scratch/graduation/artbind/leaf.js, byte-identical):
//!
//! ```text
//! traits_hash = sha256(trait_schema_hash ‖ u16le[8] trait_value_indices)
//! H_art       = sha256(salt32 ‖ sha256(image_bytes) ‖ sha256(metadata_json_bytes) ‖ uri_utf8)
//! leaf_i      = sha256(0x00 ‖ "mintmark-leaf-v2" ‖ launch_config ‖ u32le(i) ‖ traits_hash ‖ H_art)
//! node        = sha256(0x01 ‖ left ‖ right)   // positional; an odd last node is paired with itself
//! ```
//! The mint crank supplies the preimage; the program recomputes the leaf, verifies the proof against
//! the committed root, and sets the on-chain URI from the SAME `uri` bytes, so the URI can't differ
//! from the commitment. Image bytes are checked off-chain by the verifier (§5.5).

use solana_sha256_hasher::hashv;

pub const LEAF_DOMAIN: &[u8] = b"mintmark-leaf-v2";

pub fn traits_hash(schema_hash: &[u8; 32], trait_values: &[u16; 8]) -> [u8; 32] {
    let mut b = [0u8; 16];
    for (i, v) in trait_values.iter().enumerate() {
        b[2 * i..2 * i + 2].copy_from_slice(&v.to_le_bytes());
    }
    hashv(&[schema_hash, &b]).to_bytes()
}

pub fn art_hash(salt: &[u8; 32], image_sha256: &[u8; 32], json_sha256: &[u8; 32], uri: &str) -> [u8; 32] {
    hashv(&[salt, image_sha256, json_sha256, uri.as_bytes()]).to_bytes()
}

pub fn leaf_hash(launch_config: &[u8; 32], index: u32, traits_hash: &[u8; 32], art_hash: &[u8; 32]) -> [u8; 32] {
    hashv(&[&[0u8], LEAF_DOMAIN, launch_config, &index.to_le_bytes(), traits_hash, art_hash]).to_bytes()
}

pub fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    hashv(&[&[1u8], left, right]).to_bytes()
}

pub fn depth_for(size: u32) -> usize {
    let mut d = 0usize;
    while (1u64 << d) < size as u64 {
        d += 1;
    }
    d
}

/// Verify a positional proof; `proof.len()` must equal `depth_for(size)` (binds the position).
pub fn verify(leaf: [u8; 32], index: u32, size: u32, proof: &[[u8; 32]], root: &[u8; 32]) -> bool {
    if index >= size || proof.len() != depth_for(size) {
        return false;
    }
    let mut node = leaf;
    let mut i = index;
    for sib in proof {
        node = if i & 1 == 0 { node_hash(&node, sib) } else { node_hash(sib, &node) };
        i >>= 1;
    }
    &node == root
}

/// Off-chain helper (tests, scripts): root and all proofs, same tree shape as leaf.js.
#[cfg(not(target_os = "solana"))]
pub fn build(leaves: &[[u8; 32]]) -> ([u8; 32], Vec<Vec<[u8; 32]>>) {
    let mut levels: Vec<Vec<[u8; 32]>> = vec![leaves.to_vec()];
    while levels.last().unwrap().len() > 1 {
        let c = levels.last().unwrap();
        let next = c.chunks(2).map(|p| node_hash(&p[0], p.get(1).unwrap_or(&p[0]))).collect();
        levels.push(next);
    }
    let root = levels.last().unwrap()[0];
    let proofs = (0..leaves.len())
        .map(|mut i| {
            let mut p = Vec::new();
            for level in levels.iter().take(levels.len() - 1) {
                p.push(*level.get(i ^ 1).unwrap_or(&level[i]));
                i >>= 1;
            }
            p
        })
        .collect();
    (root, proofs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(i: u32, uri: &str) -> [u8; 32] {
        let t = traits_hash(&[7u8; 32], &[1, 2, 3, 4, 5, 6, 7, (i % 9) as u16]);
        leaf_hash(&[9u8; 32], i, &t, &art_hash(&[i as u8; 32], &[1u8; 32], &[2u8; 32], uri))
    }

    #[test]
    fn every_leaf_verifies_and_tampering_fails() {
        for n in [1u32, 2, 3, 5, 8, 13, 100, 1000] {
            let leaves: Vec<_> = (0..n).map(|i| leaf(i, &format!("ipfs://bafy{i}"))).collect();
            let (root, proofs) = build(&leaves);
            for i in 0..n {
                assert!(verify(leaves[i as usize], i, n, &proofs[i as usize], &root));
                if n > 1 {
                    assert!(!verify(leaves[i as usize], (i + 1) % n, n, &proofs[i as usize], &root));
                }
                assert!(!verify(leaf(i, "ipfs://swapped"), i, n, &proofs[i as usize], &root));
            }
            assert!(!verify(leaves[0], n, n, &proofs[0], &root), "index out of range");
        }
    }
}
