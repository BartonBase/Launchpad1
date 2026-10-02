//! M-01 (merged, Critical) / A-01 (Auditor A): MPL-Hybrid `update_recipe_v1` let the authority change `amount` instantly with no
//! timelock: capture NFTs at amount=0, raise amount, release -> drain every backing token
//! (security/auditor-a/poc/10_escrow_drain.out, harness/10_escrow_drain.js).
//!
//! Port to hybrid_vault. The same attack needs any path that changes ratio_base / fees / escrow after init, or
//! any path that pays more (or less) than ratio_base per NFT. Each test asserts the exploit FAILS.
//! Exploit steps from the PoC: (1) authority sets amount=0, (2) captures k NFTs paying 0, (3) sets amount=3R,
//! (4) releases the k NFTs, draining 3R backing that belonged to other holders.
use crate::{common::*, harness::*};

const KNOWN_IX: [&str; 11] = [
    "deposit_asset", "expire_request", "init_vault", "merge_incoming", "pause", "request_capture", "request_reroll",
    "settle_capture", "settle_reroll", "unpause", "unwrap",
];

/// Step (1)/(3) of the PoC: there is no instruction that edits economics. Enumerate the IDL: every instruction is
/// a known user/crank flow, none is an update/set/withdraw/close/migrate, and none takes a ratio/fee/mint arg.
#[test]
fn qa_M01_A01_no_instruction_can_change_ratio_fees_or_mint() {
    let names = ix_names();
    for n in &names {
        assert!(KNOWN_IX.contains(&n.as_str()), "new hybrid_vault instruction `{n}`: review for A-01/A-02 (economics/escrow mutation)");
        for bad in ["update", "set_", "withdraw", "close", "migrate", "sweep", "admin", "recipe"] {
            assert!(!n.contains(bad), "`{n}` looks like an economics/escrow mutation path");
        }
        for arg in ix_json(n)["args"].as_array().unwrap() {
            let a = arg["name"].as_str().unwrap();
            for bad in ["ratio", "amount", "fee", "mint", "token", "destination"] {
                assert!(!a.split('_').any(|w| w == bad), "`{n}` takes arg `{a}`");
            }
        }
    }
}

/// Unknown discriminators (a hidden update path) never succeed and never change vault state.
#[test]
fn qa_M01_A01_random_discriminators_cannot_mutate_vault() {
    let mut env = setup(4);
    let before = vault_bytes(&env);
    let creator = env.creator.insecure_clone();
    let mut rng = Rng(0xA01);
    for i in 0..128u64 {
        let mut data = rng.next().to_le_bytes().to_vec();
        data.extend_from_slice(&(env.ratio_base / 1000 * i).to_le_bytes()); // looks like "amount"
        data.extend_from_slice(&[0u8; 24]);
        let metas = vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(env.vault, false),
            AccountMeta::new(env.pool, false),
            AccountMeta::new(env.vault_tokens, false),
            AccountMeta::new(env.fee_escrow, false),
            AccountMeta::new_readonly(env.launch_config, false),
            AccountMeta::new_readonly(env.mint, false),
        ];
        let ix = Instruction::new_with_bytes(hybrid_vault::ID, &data, metas);
        assert!(env.send(&[ix], &[&creator]).is_err(), "random discriminator #{i} succeeded");
    }
    assert_eq!(vault_bytes(&env), before, "vault state unchanged");
}

/// The authority (creator) cannot re-run init_vault to re-derive economics, even after users hold NFTs.
#[test]
fn qa_M01_A01_reinit_vault_rejected_and_state_unchanged() {
    let mut env = setup(4);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    env.capture(&alice, &r, val(3));
    let before = vault_bytes(&env);
    let ix = env.init_vault_ix(env.launch_config, env.mint);
    let creator = env.creator.insecure_clone();
    assert!(env.send(&[ix], &[&creator]).is_err(), "second init_vault must fail");
    assert_eq!(vault_bytes(&env), before);
    let v = env.vault_state();
    assert_eq!(v.ratio_base, env.ratio_base, "ratio fixed at init");
    assert_eq!((v.capture_fee_amount, v.reroll_fee_amount), (env.capture_fee, env.reroll_fee), "economics fields fixed at init (wip still has token-fee fields; obsolete per BRIEF 4:49 PM)");
}

/// Full PoC shape: the authority (creator, who also holds the supply) captures k NFTs, then releases them all.
/// Each release returns exactly ratio, so it can never net a profit, and the
/// backing of other holders (alice's NFT) is untouched at every step.
#[test]
fn qa_M01_A01_authority_capture_release_cycle_cannot_drain_backing() {
    let n = 6;
    let mut env = setup(n);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let alice_idx = env.capture(&alice, &r, val(9));
    let attacker = env.new_user(USER_TOKENS); // stands in for the authority; creator-funded
    let start = env.token_amount(&attacker.ata);
    let mut got = vec![];
    for k in 0..(n - 1) {
        let r = env.new_randomness(None, None);
        got.push(env.capture(&attacker, &r, val(20 + k as u8)));
        assert_escrow_exact(&env, "after attacker capture");
        env.assert_invariants(&[&alice, &attacker]);
    }
    assert!(env.token_amount(&attacker.ata) <= start - (n as u64 - 1) * env.ratio_base, "each capture escrows at least ratio");
    for idx in &got {
        let before = env.token_amount(&attacker.ata);
        env.unwrap(&attacker, *idx).expect("unwrap");
        assert_eq!(env.token_amount(&attacker.ata) - before, env.ratio_base, "unwrap pays exactly ratio");
        assert_escrow_exact(&env, "after attacker unwrap");
        env.assert_invariants(&[&alice, &attacker]);
    }
    let net = env.token_amount(&attacker.ata) as i128 - start as i128;
    assert!(net <= 0, "attacker can't profit from a capture/release cycle (net {net})");
    assert_eq!(env.token_amount(&env.vault_tokens), env.ratio_base, "alice's backing intact");
    // alice can still unwrap for exactly ratio
    let before = env.token_amount(&alice.ata);
    env.unwrap(&alice, alice_idx).expect("alice unwrap");
    assert_eq!(env.token_amount(&alice.ata) - before, env.ratio_base);
    assert_eq!(env.token_amount(&env.vault_tokens), 0);
}

/// Unwrapping the same NFT twice (double release) or unwrapping an NFT that is already back in the vault fails.
#[test]
fn qa_M01_A01_double_unwrap_rejected() {
    let mut env = setup(3);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let idx = env.capture(&alice, &r, val(5));
    env.unwrap(&alice, idx).expect("first unwrap");
    let bal = env.token_amount(&alice.ata);
    assert!(env.unwrap(&alice, idx).is_err(), "second unwrap must fail");
    assert_eq!(env.token_amount(&alice.ata), bal);
    assert_escrow_exact(&env, "after double unwrap attempt");
}
