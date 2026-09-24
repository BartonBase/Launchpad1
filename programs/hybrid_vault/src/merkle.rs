//! Committed trait list: positional Merkle tree over (index, name, uri), SHA-256, domain-separated
//! leaves/nodes, padded with zero leaves to the next power of two. Depth = ceil(log2(collection_size)).
//! The creator commits the root in `init_vault`; every `deposit_asset` must prove its (index, name, uri).

use solana_sha256_hasher::hashv;

pub fn leaf_hash(index: u32, name: &str, uri: &str) -> [u8; 32] {
    hashv(&[
        &[0u8],
        b"hybrid_vault:leaf",
        &index.to_le_bytes(),
        &(name.len() as u16).to_le_bytes(),
        name.as_bytes(),
        &(uri.len() as u16).to_le_bytes(),
        uri.as_bytes(),
    ])
    .to_bytes()
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

/// Off-chain helper (tests, scripts): root and all proofs for a list of leaves.
#[cfg(not(target_os = "solana"))]
pub fn build(leaves: &[[u8; 32]]) -> ([u8; 32], Vec<Vec<[u8; 32]>>) {
    let depth = depth_for(leaves.len() as u32);
    let width = 1usize << depth;
    let mut levels: Vec<Vec<[u8; 32]>> = Vec::new();
    let mut cur: Vec<[u8; 32]> = leaves.to_vec();
    cur.resize(width, [0u8; 32]);
    levels.push(cur.clone());
    while cur.len() > 1 {
        cur = cur.chunks(2).map(|c| node_hash(&c[0], &c[1])).collect();
        levels.push(cur.clone());
    }
    let root = cur[0];
    let proofs = (0..leaves.len())
        .map(|mut i| {
            let mut p = Vec::with_capacity(depth);
            for level in levels.iter().take(depth) {
                p.push(level[i ^ 1]);
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

    #[test]
    fn every_leaf_verifies_and_tampering_fails() {
        for n in [1u32, 2, 3, 5, 8, 13] {
            let leaves: Vec<_> = (0..n).map(|i| leaf_hash(i, &format!("NFT #{i}"), &format!("ipfs://x/{i}"))).collect();
            let (root, proofs) = build(&leaves);
            for i in 0..n {
                assert!(verify(leaves[i as usize], i, n, &proofs[i as usize], &root));
                // same traits claimed at another position must fail
                if n > 1 {
                    let j = (i + 1) % n;
                    assert!(!verify(leaves[i as usize], j, n, &proofs[i as usize], &root));
                }
                // swapped metadata must fail
                assert!(!verify(leaf_hash(i, "fake", "ipfs://rare"), i, n, &proofs[i as usize], &root));
            }
            assert!(!verify(leaves[0], n, n, &proofs[0], &root), "index out of range");
        }
    }
}
