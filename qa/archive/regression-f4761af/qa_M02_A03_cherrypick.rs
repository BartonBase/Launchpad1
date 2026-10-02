//! M-02 (merged, Critical) / A-03 (Auditor A, High), also R1-B-01: MPL-Hybrid capture takes a caller-chosen asset account, so anyone can capture a
//! specific (rare) NFT from escrow (security/auditor-a/poc/11_cherrypick.out: a non-authority bot captured the
//! chosen rare asset). Also the R1-B-01 cherry-pick / predict-and-abort class.
//!
//! Port to hybrid_vault: request_capture takes no asset; settle must deliver exactly the VRF-selected asset to
//! the requesting user. Each test tries to steer the asset or the recipient and asserts rejection.
use crate::{common::*, harness::*};

#[test]
fn qa_M02_A03_request_capture_takes_no_asset_or_index() {
    let j = ix_json("request_capture");
    assert!(j["args"].as_array().unwrap().is_empty(), "request_capture must take no args");
    for a in j["accounts"].as_array().unwrap() {
        let n = a["name"].as_str().unwrap();
        assert!(n != "asset" && !n.contains("index"), "request_capture must not take `{n}`");
    }
}

/// The bot's move: settle a pending capture with a DIFFERENT (rare) asset than the VRF picked.
#[test]
fn qa_M02_A03_settle_with_non_selected_asset_rejected() {
    let n = 8;
    let mut env = setup(n);
    let bot = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let seq = env.request_capture(&bot, &r).unwrap();
    env.reveal(&r, val(11));
    let pick = env.expected_pick(seq, val(11));
    let creator = env.creator.insecure_clone();
    for rare in (0..n).filter(|i| *i != pick) {
        let ix = env.settle_ix(seq, env.asset_pda(rare), bot.kp.pubkey(), r, true);
        expect_vault_err(env.send(&[ix], &[&creator]), VaultError::WrongAsset);
        assert_eq!(env.asset_owner(rare), env.vault_authority, "rare stays in escrow");
    }
    assert_eq!(env.settle(seq, val(11)).unwrap(), pick);
    assert_eq!(env.asset_owner(pick), bot.kp.pubkey());
    env.assert_invariants(&[&bot]);
}

/// A crank can't redirect someone else's selected NFT to itself.
#[test]
fn qa_M02_A03_settle_to_different_recipient_rejected() {
    let mut env = setup(4);
    let alice = env.new_user(USER_TOKENS);
    let bot = env.new_user(0);
    let r = env.new_randomness(None, None);
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(&r, val(12));
    let pick = env.expected_pick(seq, val(12));
    let ix = env.settle_ix(seq, env.asset_pda(pick), bot.kp.pubkey(), r, true);
    let creator = env.creator.insecure_clone();
    assert!(env.send(&[ix], &[&creator]).is_err(), "settle to a different user");
    assert_eq!(env.asset_owner(pick), env.vault_authority);
    env.settle(seq, val(12)).unwrap();
    assert_eq!(env.asset_owner(pick), alice.kp.pubkey());
}

/// No direct pull: unwrap of a vault-held NFT, or of another user's NFT, fails (user must own + sign).
#[test]
fn qa_M02_A03_unwrap_of_vault_held_or_foreign_asset_rejected() {
    let mut env = setup(4);
    let alice = env.new_user(USER_TOKENS);
    let bot = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let idx = env.capture(&alice, &r, val(13));
    let bot_kp = bot.kp.insecure_clone();
    for target in 0..4u32 {
        let before = env.token_amount(&bot.ata);
        let ix = env.unwrap_ix(&bot, target);
        assert!(env.send(&[ix], &[&bot_kp]).is_err(), "bot unwrap of #{target}");
        assert_eq!(env.token_amount(&bot.ata), before);
    }
    assert_eq!(env.asset_owner(idx), alice.kp.pubkey());
    assert_escrow_exact(&env, "after bot attempts");
}

/// Re-roll: the user hands in X; settle must not return X itself or any non-selected asset.
#[test]
fn qa_M02_A03_reroll_settle_with_handed_in_or_non_selected_asset_rejected() {
    let n = 6;
    let mut env = setup(n);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let x = env.capture(&alice, &r, val(14));
    let r2 = env.new_randomness(None, None);
    let seq = env.request_reroll(&alice, x, &r2).unwrap();
    env.reveal(&r2, val(15));
    let pick = env.expected_pick(seq, val(15));
    assert_ne!(pick, x, "re-roll never returns the handed-in NFT");
    let creator = env.creator.insecure_clone();
    for other in (0..n).filter(|i| *i != pick) {
        let ix = env.settle_ix(seq, env.asset_pda(other), alice.kp.pubkey(), r2, false);
        assert!(env.send(&[ix], &[&creator]).is_err(), "reroll settle with #{other}");
    }
    assert_eq!(env.settle(seq, val(15)).unwrap(), pick);
    env.assert_invariants(&[&alice]);
}

/// Predict-and-abort (R1-B-01 flavour): after the randomness is visible the user cannot cancel a request they
/// don't like. There is no cancel instruction; expire only works after the deadline AND only if unrevealed.
#[test]
fn qa_M02_A03_no_abort_after_reveal() {
    let mut env = setup(4);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(&r, val(16));
    assert!(!ix_names().iter().any(|n| n.contains("cancel")), "no cancel instruction");
    assert!(env.expire(seq).is_err(), "expire before deadline");
    env.warp(REQUEST_TIMEOUT_SLOTS_QA + 10);
    assert!(env.expire(seq).is_err(), "expire of a revealed request must fail even after the deadline");
    env.settle(seq, val(16)).unwrap();
}

pub const REQUEST_TIMEOUT_SLOTS_QA: u64 = 1_500;
