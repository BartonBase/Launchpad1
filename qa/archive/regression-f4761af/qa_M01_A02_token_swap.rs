//! M-01 (merged, Critical) / A-02 (Auditor A): MPL-Hybrid `update_recipe_v1` overwrote `recipe.token` / `fee_location` even when
//! the update didn't set them, so the authority could switch the escrow token to a worthless mint, capture NFTs
//! paying junk, then switch back (security/auditor-a/poc/12_token_swap.out).
//!
//! Port to hybrid_vault: the token mint, vault token account and fee account are fixed at init from LaunchConfig,
//! and every flow pins them. Each test substitutes a junk mint / spoofed account and asserts rejection with no
//! state change.
use crate::{common::*, harness::*};

fn setup_with_user() -> (Env, User, Keypair) {
    let mut env = setup(4);
    let u = env.new_user(USER_TOKENS);
    let kp = u.kp.insecure_clone();
    (env, u, kp)
}

#[test]
fn qa_M01_A02_capture_paid_in_junk_mint_rejected() {
    let (mut env, u, kp) = setup_with_user();
    let (junk, junk_ata) = junk_mint_and_ata(&mut env, &kp, USER_TOKENS);
    let before = vault_bytes(&env);
    // (a) junk user_token only
    env.warp(1);
    let r = env.new_randomness(None, None);
    let ix = swap_key(env.request_capture_ix(&u, &r, SPL_TOKEN_ID), u.ata, junk_ata);
    assert!(env.send(&[ix], &[&kp]).is_err(), "junk user_token");
    // (b) junk mint + junk user_token (full swap, as in the PoC)
    let r = env.new_randomness(None, None);
    let ix = swap_key(swap_key(env.request_capture_ix(&u, &r, SPL_TOKEN_ID), u.ata, junk_ata), env.mint, junk);
    assert!(env.send(&[ix], &[&kp]).is_err(), "junk mint");
    assert_eq!(vault_bytes(&env), before, "no request created");
    assert_eq!(env.token_amount(&junk_ata), USER_TOKENS, "junk untouched");
}

#[test]
fn qa_M01_A02_spoofed_vault_tokens_or_fee_escrow_rejected() {
    let (mut env, u, kp) = setup_with_user();
    let mint = env.mint;
    let attacker_tokens = token_account_for(&mut env, &kp, &mint, &kp.pubkey());
    let before = vault_bytes(&env);
    for (name, old) in [("vault_tokens", env.vault_tokens), ("fee_escrow", env.fee_escrow)] {
        env.warp(1);
        let r = env.new_randomness(None, None);
        let ix = swap_key(env.request_capture_ix(&u, &r, SPL_TOKEN_ID), old, attacker_tokens);
        assert!(env.send(&[ix], &[&kp]).is_err(), "request_capture with spoofed {name}");
    }
    assert_eq!(vault_bytes(&env), before);
    assert_eq!(env.token_amount(&attacker_tokens), 0);
}

#[test]
fn qa_M01_A02_unwrap_from_spoofed_vault_tokens_rejected() {
    let (mut env, u, kp) = setup_with_user();
    let r = env.new_randomness(None, None);
    let idx = env.capture(&u, &r, val(4));
    let mint = env.mint;
    // attacker-controlled token account of the real mint, pre-funded, in the vault_tokens slot:
    let fake = token_account_for(&mut env, &kp, &mint, &kp.pubkey());
    let ix = env.unwrap_ix_with(&u, env.asset_pda(idx), idx, fake);
    assert!(env.send(&[ix], &[&kp]).is_err(), "unwrap with spoofed vault_tokens");
    assert_eq!(env.asset_owner(idx), u.kp.pubkey(), "NFT not taken");
    assert_escrow_exact(&env, "after spoofed unwrap");
}

#[test]
fn qa_M01_A02_token2022_program_substitution_rejected() {
    let (mut env, u, kp) = setup_with_user();
    env.warp(1);
    let r = env.new_randomness(None, None);
    let ix = env.request_capture_ix(&u, &r, TOKEN_2022_ID);
    assert!(env.send(&[ix], &[&kp]).is_err(), "Token-2022 program in token_program slot");
}

/// A second init_vault can't bind a different mint / launch_config to the existing vault (the only place the
/// token is chosen), and a non-creator can't init a vault for someone else's launch.
#[test]
fn qa_M01_A02_init_vault_cannot_rebind_mint_or_be_run_by_non_creator() {
    let (mut env, _u, kp) = setup_with_user();
    let (junk, _) = junk_mint_and_ata(&mut env, &kp, 1);
    let before = vault_bytes(&env);
    let creator = env.creator.insecure_clone();
    let ix = env.init_vault_ix(env.launch_config, junk);
    assert!(env.send(&[ix], &[&creator]).is_err(), "init_vault with junk mint");
    // attacker signs as `creator`
    let ix = swap_key(env.init_vault_ix(env.launch_config, env.mint), creator.pubkey(), kp.pubkey());
    assert!(env.send(&[ix], &[&kp]).is_err(), "init_vault by non-creator");
    assert_eq!(vault_bytes(&env), before);
    assert_eq!(env.vault_state().mint, env.mint);
}

/// Unsealed vault (assets not all deposited yet): a non-creator can't re-init it to choose the collection, trait
/// root or guardian. (A true front-run of the first init_vault needs a launch without a vault; the harness always
/// inits in setup, so that case is VAULT-INIT-01 in TEST_PLAN, to add when the harness exposes launch-only setup.)
#[test]
fn qa_M01_A02_init_vault_rebind_on_unsealed_vault_by_non_creator_rejected() {
    let mut env = setup_unsealed(4);
    let attacker = Keypair::new();
    env.svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();
    let before = vault_bytes(&env);
    let ix = swap_key(env.init_vault_ix(env.launch_config, env.mint), env.creator.pubkey(), attacker.pubkey());
    assert!(env.send(&[ix], &[&attacker]).is_err());
    assert_eq!(vault_bytes(&env), before);
}
