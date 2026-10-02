//! INV-ESC / FEE-06 / M-14 (fee must never be backing; release carries no fee), M-12 (per-collection backing), M-23
//! (FIFO/seq correctness, partial). Random sequences of capture / re-roll / unwrap(release) / expire against the real
//! hybrid_vault. After EVERY step:
//!   - escrow == ratio x (NFTs outside + pending captures + pending re-rolls), exactly;
//!   - every release returns exactly ratio tokens (no token deduction, ever);
//!   - an expired request returns the user's tokens / handed-in NFT in full;
//!   - NFT ownership matches the model and the harness conservation invariants hold.
//! Token fees are deliberately NOT modelled: BRIEF 2026-09-25 4:49 PM removed every token fee. (The committed wip still
//! burns a token fee on capture/re-roll; that behaviour is obsolete and is not asserted here.) SOL fee totals
//! (ops x fee) are checked by `qa_M05_M06_M36_sol_fee::qa_FEE09_per_tier_sol_fee_totals_equal_ops_times_tier_fee` once the vault
//! charges SOL fees.
use crate::{common::*, harness::*};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
enum Op {
    Capture,
    Reroll,
    Release,
    ExpireCapture,
    ExpireReroll,
}

pub fn run(seed: u64, steps: usize) -> BTreeMap<&'static str, u32> {
    let n = 10u32;
    let mut env = setup(n);
    let users: Vec<User> = (0..3).map(|_| env.new_user(USER_TOKENS)).collect();
    let mut owned: Vec<Vec<u32>> = vec![vec![]; users.len()];
    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut rng = Rng(seed);
    let ops = [Op::Capture, Op::Capture, Op::Reroll, Op::Release, Op::ExpireCapture, Op::ExpireReroll];
    for step in 0..steps {
        let op = ops[rng.below(ops.len() as u64) as usize];
        let ui = rng.below(users.len() as u64) as usize;
        let u = &users[ui];
        let v = env.vault_state();
        let in_pool = n as u64 - v.assets_outside;
        let value = val((rng.next() & 0xff) as u8);
        let ctx = format!("seed {seed:#x} step {step} {op:?} user {ui}");
        let before = env.token_amount(&u.ata);
        match op {
            Op::Capture if in_pool > 0 => {
                let r = env.new_randomness(None, None);
                let idx = env.capture(u, &r, value);
                owned[ui].push(idx);
                assert!(before - env.token_amount(&u.ata) >= env.ratio_base, "{ctx}: capture escrows at least ratio");
                *counts.entry("capture").or_default() += 1;
            }
            Op::Reroll if !owned[ui].is_empty() && in_pool > 0 => {
                let k = rng.below(owned[ui].len() as u64) as usize;
                let x = owned[ui][k];
                let esc = env.token_amount(&env.vault_tokens);
                let r = env.new_randomness(None, None);
                let seq = env.request_reroll(u, x, &r).expect(&ctx);
                assert_escrow_exact(&env, &format!("{ctx} (pending re-roll)"));
                env.reveal(&r, value);
                let got = env.settle(seq, value).expect(&ctx);
                assert_ne!(got, x, "{ctx}: re-roll returned the handed-in NFT");
                assert_eq!(env.token_amount(&env.vault_tokens), esc, "{ctx}: a re-roll never changes escrow backing");
                owned[ui][k] = got;
                *counts.entry("reroll").or_default() += 1;
            }
            Op::Release if !owned[ui].is_empty() => {
                let k = rng.below(owned[ui].len() as u64) as usize;
                let x = owned[ui].swap_remove(k);
                env.unwrap(u, x).expect(&ctx);
                assert_eq!(env.token_amount(&u.ata) - before, env.ratio_base, "{ctx}: release returns EXACTLY ratio");
                *counts.entry("release").or_default() += 1;
            }
            Op::ExpireCapture if in_pool > 0 => {
                let r = env.new_randomness(None, None);
                let seq = env.request_capture(u, &r).expect(&ctx);
                assert_escrow_exact(&env, &format!("{ctx} (pending capture)"));
                env.warp(1_600);
                env.expire(seq).expect(&ctx);
                assert_eq!(env.token_amount(&u.ata), before, "{ctx}: expired capture refunds principal in full (wip semantics)");
                *counts.entry("expire_capture").or_default() += 1;
            }
            Op::ExpireReroll if !owned[ui].is_empty() && in_pool > 0 => {
                let x = owned[ui][0];
                let r = env.new_randomness(None, None);
                let seq = env.request_reroll(u, x, &r).expect(&ctx);
                env.warp(1_600);
                env.expire(seq).expect(&ctx);
                assert_eq!(env.asset_owner(x), u.kp.pubkey(), "{ctx}: handed-in NFT returned");
                *counts.entry("expire_reroll").or_default() += 1;
            }
            _ => continue,
        }
        assert_escrow_exact(&env, &ctx);
        let refs: Vec<&User> = users.iter().collect();
        env.assert_invariants(&refs);
        for (i, uu) in users.iter().enumerate() {
            for idx in &owned[i] {
                assert_eq!(env.asset_owner(*idx), uu.kp.pubkey(), "{ctx}: ownership model");
            }
        }
        let v = env.vault_state();
        assert_eq!(v.pending_captures + v.pending_rerolls, 0, "{ctx}: nothing left pending");
        let outside: u64 = owned.iter().map(|o| o.len() as u64).sum();
        assert_eq!(v.assets_outside, outside, "{ctx}: NFTs outside == model");
        assert_eq!(env.token_amount(&env.vault_tokens), v.ratio_base * outside, "{ctx}: escrow == ratio x NFTs outside");
    }
    println!("QA escrow sequence seed {seed:#x}: {counts:?}");
    assert!(counts.len() >= 4, "sequence exercised too few op kinds: {counts:?}");
    counts
}

#[test]
fn qa_M14_INVESC_escrow_exact_random_sequence_seed_1() {
    run(0xFEE1, 60);
}

#[test]
fn qa_M14_INVESC_escrow_exact_random_sequence_seed_2() {
    run(0xFEE2_2026, 60);
}

#[test]
fn qa_M14_INVESC_escrow_exact_random_sequence_seed_3() {
    run(0x5EED_0925, 80);
}
