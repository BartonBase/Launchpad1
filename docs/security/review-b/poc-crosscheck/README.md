# B's PoC cross-check (local validator only)
- `a_pocs_10_11_12_rerun.out`: A's PoCs 10/11/12 re-run on B's local validator (port 28899, now stopped). All PASS.
- `b13_token_swap_full_drain.js` / `.out`: completes A's PoC 12. Upstream MPL-Hybrid escrow 1,000,000 → 0 for junk tokens; a third party's fee re-routed.
  To run: copy A's harness (`docs/security/review-a/poc/`, needs its `lib.js` plus the local MPL-Hybrid build @ aacf1a53) to a scratch dir,
  point RPC at a local `solana-test-validator`, and run `node b13_token_swap_full_drain.js`. No devnet/mainnet, no real keys.
