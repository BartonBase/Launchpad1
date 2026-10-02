//! LAZY-xx (TEST_PLAN §21): lazy minting (Barton 2026-09-25 5:13 PM MT), aligned to docs/lazy-mint-interface.md
//! ("interface frozen for QA", 17:15 MT; ADR-016). Nothing is pre-minted. Asset i is minted by `settle_capture` /
//! `settle_reroll(mint: Option<MintArgs>)` the first time VRF picks it, straight to the user, paid from the request's
//! `mint_escrow_lamports` (= MINT_ESCROW_LAMPORTS = 6_338_100 = (CORE_ASSET_RENT_LAMPORTS 3_570_480 +
//! CORE_CREATE_FEE_LAMPORTS 1_500_000) x 125%). The settler pays only its tx fee (SETTLE_TIP_LAMPORTS = 0). Pool layout
//! v2 (`hvpool02`, lazy Fisher-Yates slots + incoming ring + minted bitmap). Errors: MintArgsMissing, MintEscrowShort,
//! AlreadyMinted, MintCostConstantStale (6055..6058).
//! Lazy code is IN PROGRESS in the engineer's uncommitted tree (settle takes MintArgs, pool v2) and not in any commit
//! (f4761af = creator deposit). So on-chain tests are ignored with exact steps; the model tests run now (LAZY-00,
//! LAZY-10a uses the real `hybrid_vault::selection` functions on a QA replica of the v2 pool).
//! Merged IDs: M-02, M-04, M-06, M-10, M-14, M-21, M-22, M-37, M-40.
#![allow(dead_code)]
use crate::{common::*, harness::*};

const AHEAD: &str = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)";
const FIRST_MINT_MIN: u64 = 3_463_000;
const FIRST_MINT_MAX: u64 = 6_671_000;
/// docs/lazy-mint-interface.md constants.
const MINT_ESCROW_LAMPORTS: u64 = 6_338_100;
const CORE_ASSET_RENT_LAMPORTS: u64 = 3_570_480;
const CORE_CREATE_FEE_LAMPORTS: u64 = 1_500_000;

/// LAZY-00 (spec model, runs now): the protocol pays 0 at graduation. A 10,000-NFT collection at the 85 SOL default
/// needs nothing from the raise; first capturers pay N x m in total (34.6-66.7 SOL at N = 10,000).
#[test]
fn qa_LAZY00_model_10k_collection_costs_protocol_nothing() {
    let _ = AHEAD;
    let n = 10_000u64;
    let (lo, hi) = (n * FIRST_MINT_MIN, n * FIRST_MINT_MAX);
    println!("QA lazy model: N={n}: protocol 0 SOL; capturers pay {:.1}-{:.1} SOL over first captures", lo as f64 / 1e9, hi as f64 / 1e9);
    assert!(hi < 85_000_000_000);
    assert_eq!((CORE_ASSET_RENT_LAMPORTS + CORE_CREATE_FEE_LAMPORTS) * 125 / 100, MINT_ESCROW_LAMPORTS, "interface escrow constant");
}

/// LAZY-01 / M-02 / INV-3: settle mints EXACTLY the picked index, at most once. Open a lazy vault (N=8); 8 captures ->
/// each settle creates only asset(picked) = ["asset", vault, picked_le_u32] (no other asset account appears), bitmap bit
/// set, minted_count +1. Two requests whose picks resolve to the same index can't both mint: the pick swap-removes it
/// from the pool, so the second settle draws from the rest; forcing it (forged pool bytes with the bit already set) ->
/// AlreadyMinted, atomic. Same-slot: two settles in one tx and in two txs of the same slot -> one mint per index.
/// Re-settling a settled request fails (request closed).
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY01_M02_settle_mints_picked_index_once_even_same_slot() {
    todo!("see doc comment")
}

/// LAZY-02 / M-14: minted_count <= N always; a picked index whose bitmap bit is set is TRANSFERRED vault_authority ->
/// user (settle with mint = None), never re-minted: no Core CreateV2 in the logs, asset pubkey unchanged, and passing
/// MintArgs anyway is ignored.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY02_M14_minted_count_le_n_and_minted_index_transferred_not_reminted() {
    todo!("see doc comment")
}

/// LAZY-03 / T-HV-16 / M-37: mint cost comes from the request escrow, never the settler. A settler funded with exactly
/// the tx fee settles a first-mint request and ends at 0 lamports (tip 0). The request PDA's escrow moves to
/// vault_authority, Core takes rent + 1_500_000, and vault_authority's lamports are unchanged after the ix. The user
/// receives request rent + rand_lock rent + (escrow - actual asset rent - Core fee).
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY03_M37_mint_cost_paid_from_request_escrow_not_settler() {
    todo!("see doc comment")
}

/// LAZY-04 / M-04 / M-22: an underfunded request can't be created, can't settle, and can't strand the principal.
/// (a) raise Rent (set the Rent sysvar) so rent(385) + 1_500_000 > MINT_ESCROW_LAMPORTS -> request_capture /
///     request_reroll fail with MintCostConstantStale, and nothing moves.
/// (b) forge a pending request's lamports below rent + escrow (INV-4), with a pick on an unminted index -> settle fails
///     with MintEscrowShort (or an INV-4 error), atomically: no asset, no bitmap bit, no token move.
/// (c) EXPIRE, EXACT REFUND: never-revealed capture request, MAX_RECOMMITS recommits, warp past deadline + grace,
///     expire_request by a third party -> user tokens +ratio_base exactly; user lamports + (request rent + rand_lock
///     rent + mint_escrow_lamports [= 6_338_100, fully unspent]) exactly; fee_recipient unchanged (tier fee NOT
///     refunded); caller pays only the tx fee. Re-roll variant: the handed-in NFT goes back to the user, same lamport
///     refund. Escrow rule holds.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY04_M22_underfunded_request_cannot_settle_or_strand_principal_exact_refund() {
    let expected_expire_lamport_refund = |request_rent: u64, rand_lock_rent: u64| request_rent + rand_lock_rent + MINT_ESCROW_LAMPORTS;
    let _ = expected_expire_lamport_refund;
    todo!("see doc comment")
}

/// LAZY-14 / M-37: a request whose pick lands on an ALREADY-MINTED index gets its whole unused mint escrow back at
/// settle. Per interface §3 the request closes to the user, who gets request rent + rand_lock rent + the full 6_338_100
/// escrow; the settler gets 0; no Core create. Also first-mint picks: user gets escrow - actual rent - Core fee
/// (+ any drained pre-fund at asset(picked)). PROVISIONAL: exact refund routing per docs/lazy-mint-interface.md
/// (17:15 MT); re-check when the code lands.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY14_M37_minted_pick_refunds_unused_mint_escrow_at_settle() {
    todo!("see doc comment")
}

/// LAZY-05 / M-21: every lazily minted asset is in vault.collection (Core collection field = vault.collection; the
/// collection's update authority = vault_authority), owner = the user after settle (vault_authority after release or a
/// re-roll hand-in), name == "#<picked>", uri == leaf.uri (content-addressed, <= 200 chars), and no plugins. Attributes
/// are NOT on-chain (no plugins): they're bound through the leaf (trait_values + json_sha256); see LAZY-13.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY05_M21_minted_asset_in_collection_owned_named_as_committed() {
    todo!("see doc comment")
}

/// LAZY-13 / M-21: metadata comes from the committed Merkle leaf (leaf v2 binds launch_config, index, trait hash, art
/// hash) + proof, verified against vault.trait_root in settle. Each of these is rejected, atomically (no asset, no
/// bitmap bit, the request stays pending):
///  (a) wrong proof (one sibling flipped; too short; too long);
///  (b) a valid leaf+proof for ANOTHER index j != picked;
///  (c) a valid leaf from another launch (different launch_config);
///  (d) tampered uri, trait_values, salt, image_sha256 or json_sha256;
///  (e) a proof against a different root (a tree built by the attacker);
///  (f) a non-content-addressed or > 200-char uri;
///  (g) a missing MintArgs for an unminted pick -> MintArgsMissing.
/// Root immutability: no hybrid_vault or hybrid_launch instruction writes trait_root after init (IDL enumeration plus
/// random discriminators); vault bytes at the trait_root offset are unchanged after every instruction in a random
/// sequence; init_vault can't be re-run.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY13_M21_merkle_leaf_proof_verified_and_root_immutable() {
    for n in ix_names() {
        let l = n.to_lowercase();
        assert!(!(l.contains("root") || l.contains("set_") || l.contains("update")), "instruction `{n}` may change trait_root");
    }
    todo!("runtime cases (a)-(g) + root-bytes check: see doc comment")
}

/// LAZY-06 / T-GRAD-01: an attacker can't pre-create or profit from pre-funding asset(i). (a) pre-fund asset(picked)
/// with lamports -> settle drains them into vault_authority, mints, and the net gain is refunded to the USER (never the
/// attacker); (b) asset(i) is a hybrid_vault PDA, so system create_account / assign at that address fails without the
/// PDA signer; (c) a Core asset created elsewhere can't stand in for index i (seeds check).
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY06_asset_address_cannot_be_precreated_or_griefed() {
    todo!("see doc comment")
}

/// LAZY-07 / M-14: release (`unwrap`) of a lazily minted asset returns exactly ratio_base (free); the asset goes back
/// via incoming (tag) and a later capture that picks it transfers it (bit already set, mint = None) with the full
/// escrow refunded (LAZY-14).
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY07_M14_release_of_lazy_asset_exact_and_recapture_not_reminted() {
    todo!("see doc comment")
}

/// LAZY-08 / M-06 per tier: user's net capture cost = tier fee + (asset rent + Core fee if the pick was unminted, else
/// 0) + tx fees (the escrow is refunded down to the actual spend). Re-roll likewise. M-06 holds for both-minted and
/// both-unminted picks; the mixed case is recorded for QA-FEE-04.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY08_M06_capture_cost_tier_fee_plus_first_mint_per_tier() {
    todo!("see doc comment and regression::qa_FEE07_M06_tier_model_reroll_le_release_plus_capture")
}

/// LAZY-09 / M-14 / M-40 / interface INV-1..INV-4: after random sequences on the v2 pool (minted and unminted picks,
/// releases, re-rolls, expires): vault_tokens == ratio x (outside + pending captures + pending re-rolls);
/// pool_len + incoming_len + pending_rerolls + assets_outside == N; minted_count <= N and never decreases; no index is
/// owned by two holders; every pending request holds >= rent + mint_escrow_lamports.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY09_M14_escrow_backing_holds_with_virtual_pool() {
    todo!("extend qa_M14_escrow_sequence::run with the v2 pool + bitmap model")
}

// ---------------------------------------------------------------- LAZY-10: uniform over ALL held indices

/// QA replica of the v2 pool (interface §Pool account): slot s holds index+1, 0 = "index s" (lazy Fisher-Yates).
struct PoolModel {
    slots: Vec<u32>,
    len: u32,
}
impl PoolModel {
    fn new(n: u32) -> Self {
        PoolModel { slots: vec![0; n as usize], len: n }
    }
    fn get(&self, s: u32) -> u32 {
        let v = self.slots[s as usize];
        if v == 0 { s } else { v - 1 }
    }
    fn swap_remove(&mut self, pos: u32) -> u32 {
        let picked = self.get(pos);
        let last = self.get(self.len - 1);
        self.slots[pos as usize] = last + 1;
        self.len -= 1;
        picked
    }
    fn push(&mut self, idx: u32) {
        self.slots[self.len as usize] = idx + 1;
        self.len += 1;
    }
    fn held(&self) -> Vec<u32> {
        (0..self.len).map(|s| self.get(s)).collect()
    }
}

/// Chi-square critical value at p = 0.001 (Wilson-Hilferty approximation), df = k - 1.
fn chi2_crit_p001(df: f64) -> f64 {
    let z = 3.090_232; // one-sided 0.999
    df * (1.0 - 2.0 / (9.0 * df) + z * (2.0 / (9.0 * df)).sqrt()).powi(3)
}

/// LAZY-10a (MODEL, runs now) / M-02: the pick is uniform over ALL indices the vault holds: never-minted AND
/// minted-and-returned. N = 24. Rounds of captures (swap-remove, mark minted) and returns (push back) leave a held
/// set mixing both kinds. Then 60,000 seeded draws (real `hybrid_vault::selection::request_randomness` +
/// `uniform_below`, seq and VRF value varied) against that fixed state; chi-square over the held indices < p=0.001
/// critical; the minted-and-returned and never-minted groups each get their proportional share (+/- 2%). Re-roll: the
/// hand-in is pushed AFTER the pick, so its draw count for that request is 0.
#[test]
fn qa_LAZY10a_M02_model_pick_uniform_over_all_held_indices() {
    use hybrid_vault::selection::{request_randomness, uniform_below};
    let n = 24u32;
    let mut pool = PoolModel::new(n);
    let mut minted = vec![false; n as usize];
    let mut rng = Rng(0x1A2B_0925);
    let vault = Pubkey::new_unique();
    let mut outside: Vec<u32> = vec![];
    // 10 captures, then return 4 of them: held = 14 never-minted + 4 minted-and-returned.
    for seq in 0..10u64 {
        let mut v = [0u8; 32];
        for c in v.chunks_mut(8) {
            c.copy_from_slice(&rng.next().to_le_bytes());
        }
        let pos = uniform_below(&request_randomness(&v, &vault, seq), pool.len);
        let idx = pool.swap_remove(pos);
        minted[idx as usize] = true;
        outside.push(idx);
    }
    for idx in outside.drain(..4) {
        pool.push(idx);
    }
    let held = pool.held();
    let k = held.len();
    let returned: Vec<u32> = held.iter().copied().filter(|i| minted[*i as usize]).collect();
    assert_eq!(k, 18);
    assert_eq!(returned.len(), 4, "state mixes minted-and-returned with never-minted");
    let draws = 60_000u64;
    let mut count = vec![0u64; n as usize];
    for d in 0..draws {
        let mut v = [0u8; 32];
        for c in v.chunks_mut(8) {
            c.copy_from_slice(&rng.next().to_le_bytes());
        }
        let pos = uniform_below(&request_randomness(&v, &vault, 1_000 + d), pool.len);
        count[pool.get(pos) as usize] += 1;
    }
    let exp = draws as f64 / k as f64;
    let chi2: f64 = held.iter().map(|i| (count[*i as usize] as f64 - exp).powi(2) / exp).sum();
    let crit = chi2_crit_p001((k - 1) as f64);
    println!("QA LAZY-10a: k={k} held ({} returned), chi2={chi2:.2}, crit(p=0.001)={crit:.2}", returned.len());
    assert!(chi2 < crit, "pick not uniform over held indices: chi2 {chi2:.2} >= {crit:.2}");
    for i in 0..n {
        if !held.contains(&i) {
            assert_eq!(count[i as usize], 0, "index {i} not held but drawn");
        }
    }
    let share = returned.iter().map(|i| count[*i as usize]).sum::<u64>() as f64 / draws as f64;
    let want = returned.len() as f64 / k as f64;
    assert!((share - want).abs() < 0.02, "minted-and-returned share {share:.4} vs {want:.4}");
    // Re-roll: the hand-in is not drawable for its own request (pushed after the pick).
    let hand_in = outside[0];
    let pos = uniform_below(&request_randomness(&[7u8; 32], &vault, 99_999), pool.len);
    let picked = pool.swap_remove(pos);
    assert_ne!(picked, hand_in, "hand-in drawn by its own re-roll");
    pool.push(hand_in);
}

/// LAZY-10b (ON-CHAIN) / M-02: the same property on the real program. Lazy vault N = 16; capture 6, release 3 (merge);
/// then sample >= 3,200 settles through snapshot/restore of the LiteSVM state (the same pool state, varied VRF values
/// from the mock), record `picked`; chi-square over the 13 held indices < p=0.001 critical; returned indices get their
/// share; the program's pick == QA's replay (`PoolView` + `selection`) for every sample.
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY10b_M02_onchain_pick_uniform_over_all_held_indices() {
    todo!("see doc comment")
}

/// LAZY-11 / M-10: no batch pre-mint path: the IDL has no `mint_assets` / `crank_mint` / `deposit_asset`, and
/// `open_vault` doesn't require minted_count == N (CollectionNotFullyMinted is Reserved*).
#[test]
#[ignore = "enable when lazy minting is committed (docs/lazy-mint-interface.md; WT only)"]
fn qa_LAZY11_M10_no_batch_premint_path_and_open_not_gated_on_full_mint() {
    for n in ix_names() {
        assert!(n != "mint_assets" && n != "crank_mint" && n != "deposit_asset", "batch pre-mint / deposit path `{n}` still exists");
    }
    let errs = idl()["errors"].as_array().cloned().unwrap_or_default();
    assert!(!errs.iter().any(|e| e["name"] == "CollectionNotFullyMinted"), "CollectionNotFullyMinted must be Reserved*");
}
