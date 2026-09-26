//! hybrid_vault LiteSVM tests (QA layout: tests/track-a-hybrid/vault). Real SBF builds of
//! hybrid_launch + hybrid_vault (TEST build with the mock graduation verifier), real Metaplex Core,
//! TEST-ONLY mock Switchboard. Every test is named for the property or attack it proves; audit
//! finding ids are in docs/audit-fixes-round1.md.

mod harness;
use anchor_lang::error::ErrorCode as AErr;
use harness::*;

fn idl_instruction_names() -> Vec<String> {
    let idl = vault_idl();
    idl["instructions"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap().to_string()).collect()
}

fn kp_funded(env: &mut Env) -> Keypair {
    let k = Keypair::new();
    env.svm.airdrop(&k.pubkey(), 10_000_000_000).unwrap();
    k
}

// ---------------------------------------------------------------- flat SOL fee (ADR-013)

#[test]
fn capture_then_release_returns_exactly_ratio_tokens_and_release_is_free() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let before = env.token_amount(&alice.ata);
    let idx = env.capture(&alice, val(1));
    assert_eq!(env.asset_owner(idx), alice.kp.pubkey());
    assert_eq!(env.token_amount(&alice.ata), before - env.ratio_base, "capture costs exactly N tokens, no token fee");
    env.assert_invariants();

    let (rec0, sol0) = (env.lamports(&env.ids.fee_recipient), env.lamports(&alice.kp.pubkey()));
    env.unwrap(&alice, idx).expect("release");
    assert_eq!(env.asset_owner(idx), env.ids.vault_authority);
    assert_eq!(env.token_amount(&alice.ata), before, "release returns exactly N tokens");
    assert_eq!(env.lamports(&env.ids.fee_recipient), rec0, "release pays no fee (Barton 5:04 PM)");
    assert_eq!(sol0 - env.lamports(&alice.kp.pubkey()), 5_000, "user pays only the 1-signature tx fee on release");
    env.assert_invariants();
}

#[test]
fn release_has_no_fee_account_in_its_interface() {
    let names: Vec<String> = hybrid_vault::accounts::Unwrap {
        user: Pubkey::new_unique(), vault: Pubkey::new_unique(), launch_config: Pubkey::new_unique(), pool: Pubkey::new_unique(),
        mint: Pubkey::new_unique(), vault_authority: Pubkey::new_unique(), vault_tokens: Pubkey::new_unique(), user_token: Pubkey::new_unique(),
        asset: Pubkey::new_unique(), collection: Pubkey::new_unique(), mpl_core_program: Pubkey::new_unique(),
        token_program: Pubkey::new_unique(), system_program: Pubkey::new_unique(),
    }
    .to_account_metas(None)
    .iter()
    .map(|m| m.pubkey.to_string())
    .collect();
    assert_eq!(names.len(), 13, "unwrap takes exactly 13 accounts, none of them a fee account");
}

#[test]
fn capture_fee_is_flat_sol_to_fixed_recipient_no_token_fee_no_burn() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let (vault0, rec0, supply0, user_sol0) =
        (env.token_amount(&env.ids.vault_tokens), env.lamports(&env.ids.fee_recipient), env.supply(), env.lamports(&alice.kp.pubkey()));
    let r = env.new_randomness();
    env.request_capture(&alice, &r).unwrap();
    assert_eq!(env.token_amount(&env.ids.vault_tokens) - vault0, env.ratio_base, "vault receives exactly N");
    assert_eq!(env.lamports(&env.ids.fee_recipient) - rec0, FEE, "0.01 SOL (1M tier) to the fixed recipient");
    assert_eq!(env.supply(), supply0, "nothing burned");
    assert!(env.svm.get_account(&get_associated_token_address(&PLATFORM_FEE_RECIPIENT, &env.ids.mint)).is_none(), "no token fee account");
    let spent = user_sol0 - env.lamports(&alice.kp.pubkey());
    assert!(spent >= FEE + hybrid_vault::MINT_ESCROW_LAMPORTS && spent < FEE + hybrid_vault::MINT_ESCROW_LAMPORTS + 10_000_000, "user paid the fee + mint escrow + rent/tx: {spent}");
    env.assert_invariants();
}

#[test]
fn reroll_charges_same_flat_sol_fee_moves_no_tokens_and_never_returns_handed_in_asset() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    env.capture(&alice, val(2));
    for round in 0..5u8 {
        let current = (0..N).find(|i| env.asset_owner(*i) == alice.kp.pubkey()).unwrap();
        let (rec0, vault0, tok0) = (env.lamports(&env.ids.fee_recipient), env.token_amount(&env.ids.vault_tokens), env.token_amount(&alice.ata));
        let r = env.new_randomness();
        let seq = env.request_reroll(&alice, current, &r).unwrap();
        assert_eq!(env.lamports(&env.ids.fee_recipient) - rec0, FEE, "re-roll fee == capture fee");
        assert_eq!(env.token_amount(&env.ids.vault_tokens), vault0, "re-roll moves no tokens");
        assert_eq!(env.token_amount(&alice.ata), tok0, "no token fee");
        env.assert_invariants();
        env.reveal(seq, val(40 + round)).unwrap();
        let got = env.settle(seq, val(40 + round)).unwrap();
        assert_ne!(got, current, "re-roll never returns the handed-in asset");
        assert_eq!(env.asset_owner(got), alice.kp.pubkey());
        env.assert_invariants();
    }
}

#[test]
fn attack_fee_to_own_wallet_or_other_recipient_is_rejected_fail_closed() {
    // M-05 / F-06: user pays the fee to self (or anyone else) instead of PLATFORM_FEE_RECIPIENT.
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    for bad in [alice.kp.pubkey(), Pubkey::new_unique(), env.ids.vault, env.ids.vault_authority] {
        let mut a = env.request_capture_ix(&alice, &r);
        a.fee_recipient = bad;
        expect_vault_err(env.request_capture_with(&alice, a), VaultError::FeeRecipientMismatch);
    }
    let idx = env.capture(&alice, val(3));
    let r2 = env.new_randomness();
    let mut a = env.request_reroll_accts(&alice, idx, &r2);
    a.fee_recipient = alice.kp.pubkey();
    let ix = env.request_reroll_ix(a, idx);
    let kp = alice.kp.insecure_clone();
    expect_vault_err(env.send(&[ix], &[&kp]), VaultError::FeeRecipientMismatch);
    env.assert_invariants();
}

fn patch_config(env: &mut Env, f: impl Fn(&mut Vec<u8>)) {
    let lc = env.ids.launch_config;
    let mut acct = env.svm.get_account(&lc).unwrap();
    f(&mut acct.data);
    acct.lamports = acct.lamports.max(env.svm.minimum_balance_for_rent_exemption(acct.data.len()));
    env.svm.set_account(lc, acct).unwrap();
}

fn set_stored_fee(env: &mut Env, fee: u64) {
    patch_config(env, |d| d[hybrid_launch::LC_OFF_FEE_LAMPORTS..hybrid_launch::LC_OFF_FEE_LAMPORTS + 8].copy_from_slice(&fee.to_le_bytes()));
}

#[test]
fn m08_request_fee_is_min_of_stored_tier_and_cap_never_an_exact_match_revert() {
    // M-08: a patched/forged or stale stored fee never reverts capture; the charge is min(stored, tier, MAX).
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    for (stored, charged) in [(u64::MAX, FEE), (hybrid_launch::MAX_FEE_LAMPORTS + 1, FEE), (5_000_000, 5_000_000), (3_000_000, 3_000_000)] {
        set_stored_fee(&mut env, stored);
        let rec0 = env.lamports(&env.ids.fee_recipient);
        let r = env.new_randomness();
        let seq = env.request_capture(&alice, &r).unwrap_or_else(|e| panic!("stored {stored}: {e}"));
        assert_eq!(env.lamports(&env.ids.fee_recipient) - rec0, charged, "stored {stored}");
        env.reveal(seq, val(stored as u8)).unwrap();
        env.settle(seq, val(stored as u8)).unwrap();
        env.assert_invariants();
    }
}

// ---------------------------------------------------------------- M-41: no launch upgrade can freeze release

fn release_survives(mutate: impl Fn(&mut Vec<u8>), what: &str) {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(70));
    // A pending capture too, so settle and expire (the other exit paths) are exercised after the change.
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(seq, val(71)).unwrap();
    patch_config(&mut env, &mutate);
    let before = env.token_amount(&alice.ata);
    env.unwrap(&alice, idx).unwrap_or_else(|e| panic!("release must succeed after {what}: {e}"));
    assert_eq!(env.token_amount(&alice.ata) - before, env.ratio_base, "exactly N after {what}");
    env.settle(seq, val(71)).unwrap_or_else(|e| panic!("settle must succeed after {what}: {e}"));
    env.assert_invariants();
}

#[test]
fn m41_release_and_settle_survive_a_launch_version_bump() {
    release_survives(|d| d[hybrid_launch::LC_OFF_VERSION] = 99, "version bump");
}

#[test]
fn m41_release_and_settle_survive_a_tier_or_bounds_change() {
    // A tier change or new MIN/MAX looks, to an old config, like a stored fee that no longer matches: any value.
    for fee in [0u64, 1, u64::MAX] {
        release_survives(move |d| d[hybrid_launch::LC_OFF_FEE_LAMPORTS..hybrid_launch::LC_OFF_FEE_LAMPORTS + 8].copy_from_slice(&fee.to_le_bytes()), "fee/tier/bounds change");
    }
}

#[test]
fn m41_release_and_settle_survive_a_fee_wallet_rotation() {
    release_survives(|d| d[hybrid_launch::LC_OFF_FEE_RECIPIENT..hybrid_launch::LC_OFF_FEE_RECIPIENT + 32].copy_from_slice(&[7u8; 32]), "wallet rotation");
}

#[test]
fn m41_release_and_settle_survive_an_appended_layout() {
    release_survives(|d| d.extend_from_slice(&[0xAB; 96]), "appended fields");
}

#[test]
fn m41_release_and_settle_survive_everything_at_once() {
    release_survives(
        |d| {
            d[hybrid_launch::LC_OFF_VERSION] = 42;
            d[hybrid_launch::LC_OFF_FEE_LAMPORTS..hybrid_launch::LC_OFF_FEE_LAMPORTS + 8].copy_from_slice(&u64::MAX.to_le_bytes());
            d[hybrid_launch::LC_OFF_FEE_RECIPIENT..hybrid_launch::LC_OFF_FEE_RECIPIENT + 32].copy_from_slice(&[9u8; 32]);
            d.extend_from_slice(&[1; 200]);
        },
        "all changes",
    );
}

#[test]
fn m41_capture_still_fails_closed_on_an_unknown_version() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    patch_config(&mut env, |d| d[hybrid_launch::LC_OFF_VERSION] = 99);
    let r = env.new_randomness();
    expect_vault_err(env.request_capture(&alice, &r), VaultError::UnsupportedLaunchConfig);
}

#[test]
fn m22_vault_rejects_collection_size_above_cap() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    patch_config(&mut env, |d| d[hybrid_launch::LC_OFF_COLLECTION_SIZE..hybrid_launch::LC_OFF_COLLECTION_SIZE + 8].copy_from_slice(&10_001u64.to_le_bytes()));
    let r = env.new_randomness();
    expect_vault_err(env.request_capture(&alice, &r), VaultError::CollectionAboveCap);
}

// ---------------------------------------------------------------- M-26: no fee wallet state can block release

fn release_ok_with_recipient_state(state: &str) {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(60));
    let rec = env.ids.fee_recipient;
    match state {
        "missing" => env.svm.set_account(rec, Account::default()).unwrap(),
        "program_owned" => env.svm.set_account(rec, Account { lamports: 5_000_000, data: vec![1; 16], owner: Pubkey::new_unique(), executable: false, rent_epoch: 0 }).unwrap(),
        "executable" => {
            let prog = env.svm.get_account(&hybrid_launch::ID).unwrap();
            env.svm.set_account(rec, prog).unwrap()
        }
        _ => unreachable!(),
    }
    let before = env.token_amount(&alice.ata);
    env.unwrap(&alice, idx).unwrap_or_else(|e| panic!("release must succeed with recipient {state}: {e}"));
    assert_eq!(env.token_amount(&alice.ata) - before, env.ratio_base, "exactly N");
    env.assert_invariants();
}

fn solana_sdk_ids_bpf_loader() -> Pubkey {
    anchor_lang::prelude::pubkey!("BPFLoader2111111111111111111111111111111111")
}

#[test]
fn release_succeeds_when_fee_recipient_does_not_exist_or_was_emptied() {
    release_ok_with_recipient_state("missing");
}

#[test]
fn release_succeeds_when_fee_recipient_is_program_owned() {
    release_ok_with_recipient_state("program_owned");
}

#[test]
fn release_succeeds_when_fee_recipient_is_executable() {
    release_ok_with_recipient_state("executable");
}

// ---------------------------------------------------------------- M-04 staleness filter

#[test]
fn m04_stale_oracle_is_skipped_only_with_proof_and_live_oracle_cannot_be_skipped() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let seq = env.vault_state().next_seq;
    let first = env.select_oracle(seq, &[]);
    // Make the program's first choice stale.
    let old = env.now() - hybrid_vault::MAX_ORACLE_HEARTBEAT_AGE_SECS - 1;
    env.set_oracle_heartbeat(first, old);
    // Without the proof, the stale first choice is refused (OracleStale).
    let r = env.new_randomness();
    let mut a = env.request_capture_ix(&alice, &r);
    a.sb_oracle = first;
    env.warp(1);
    let kp = alice.kp.insecure_clone();
    expect_vault_err(env.send(&[Env::ix_capture(a)], &[&kp]), VaultError::OracleStale);
    // With the proof, the program moves to the next oracle; the harness passes proofs automatically.
    let next = env.select_oracle(seq, &[]);
    assert_ne!(next, first);
    let seq2 = env.request_capture(&alice, &r).expect("capture with stale proof");
    assert_eq!(env.request_state(seq2).oracles[0], next);
    // A caller can't skip a LIVE oracle by passing it as a "proof": it's fresh, so it isn't skipped.
    let r2 = env.new_randomness();
    let seq3 = env.vault_state().next_seq;
    let live = env.select_oracle(seq3, &[]);
    let mut a = env.request_capture_ix(&alice, &r2);
    let mut ix = Env::ix_capture({ a.sb_oracle = env.sb_oracles.iter().copied().find(|o| *o != live && *o != first).unwrap(); a });
    ix.accounts.push(AccountMeta::new_readonly(live, false));
    ix.accounts.push(AccountMeta::new_readonly(first, false));
    env.warp(1);
    expect_vault_err(env.send(&[ix], &[&kp]), VaultError::WrongOracle);
}

#[test]
fn pool_floor_rejects_before_any_fee_or_token_is_charged() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS * 3);
    let floor = hybrid_vault::instructions::request::pool_floor(N);
    assert_eq!(floor, 5, "max(5, 2% of 100)");
    for i in 0..(N as u64 - floor) {
        env.capture(&alice, val((i % 250) as u8));
    }
    let (tok, rec, sol) = (env.token_amount(&alice.ata), env.lamports(&env.ids.fee_recipient), env.lamports(&alice.kp.pubkey()));
    let r = env.new_randomness();
    let sol_after_rand = env.lamports(&alice.kp.pubkey());
    expect_vault_err(env.request_capture(&alice, &r), VaultError::NoAssetAvailable);
    let owned = (0..N).find(|i| env.asset_owner(*i) == alice.kp.pubkey()).unwrap();
    expect_vault_err(env.request_reroll(&alice, owned, &r), VaultError::NoAssetAvailable);
    assert_eq!(env.token_amount(&alice.ata), tok, "no tokens taken");
    assert_eq!(env.lamports(&env.ids.fee_recipient), rec, "no fee taken");
    // A failed tx costs only its signature fee (5,000 lamports each); no fee, escrow or rent is kept.
    assert!(sol_after_rand <= sol && sol_after_rand - env.lamports(&alice.kp.pubkey()) <= 10_000, "failed txs charge only the tx fees");
    env.assert_invariants();
}

#[test]
fn pool_floor_math_is_max_5_or_2_percent_never_below_2() {
    use hybrid_vault::instructions::request::pool_floor;
    assert_eq!(pool_floor(100), 5);
    assert_eq!(pool_floor(250), 5);
    assert_eq!(pool_floor(251), 6);
    assert_eq!(pool_floor(1_000), 20);
    assert_eq!(pool_floor(10_000), 200);
    for n in [1u32, 2, 100, 10_000] {
        assert!(pool_floor(n) >= 2);
    }
}

// ---------------------------------------------------------------- blind VRF assignment (A-03/A-04/A-05, R1-B Switchboard)

#[test]
fn attack_capturer_cannot_choose_asset_settle_with_other_asset_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(seq, val(5)).unwrap();
    let pick = env.expected_pick(seq, val(5));
    let other = (pick + 1) % N;
    let crank = kp_funded(&mut env);
    let ix = env.settle_ix(seq, env.asset_pda(other), crank.pubkey());
    expect_vault_err(env.send(&[ix], &[&crank]), VaultError::WrongAsset);
    assert_eq!(env.settle(seq, val(5)).unwrap(), pick);
}

#[test]
fn settle_and_reveal_are_permissionless_any_crank() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    // `reveal` and `settle` in the harness sign with fresh random keypairs, never the user.
    env.reveal(seq, val(6)).unwrap();
    let got = env.settle(seq, val(6)).unwrap();
    assert_eq!(env.asset_owner(got), alice.kp.pubkey());
}

#[test]
fn attack_randomness_not_owned_by_switchboard_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let (auth, q) = (env.ids.randomness_authority, env.sb_queue);
    let r = env.raw_randomness(auth, Pubkey::new_unique(), q);
    expect_vault_err(env.request_capture(&alice, &r), VaultError::InvalidRandomnessAccount);
}

#[test]
fn attack_randomness_with_foreign_authority_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let q = env.sb_queue;
    let r = env.raw_randomness(alice.kp.pubkey(), SWITCHBOARD_PROGRAM_ID, q);
    expect_vault_err(env.request_capture(&alice, &r), VaultError::RandomnessAuthorityMismatch);
}

#[test]
fn attack_randomness_on_other_queue_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let mut a = env.request_capture_ix(&alice, &r);
    a.sb_queue = Pubkey::new_unique();
    expect_vault_err(env.request_capture_with(&alice, a), VaultError::WrongQueue);
    // init_randomness on a foreign queue is refused too.
    let k = Keypair::new();
    let ix = env.init_randomness_ix(&k.pubkey(), Pubkey::new_unique());
    let c = env.creator.insecure_clone();
    expect_vault_err(env.send(&[ix], &[&c, &k]), VaultError::WrongQueue);
}

#[test]
fn attack_randomness_reused_while_pending_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let bob = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    env.request_capture(&alice, &r).unwrap();
    // rand_lock PDA for `r` already exists: the second request can't commit the same account.
    assert!(env.request_capture(&bob, &r).is_err());
}

#[test]
fn attack_stale_value_after_reuse_needs_fresh_reveal() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(seq, val(7)).unwrap();
    env.settle(seq, val(7)).unwrap();
    // The same account (lock closed) can be committed again, but its OLD revealed value is refused.
    let seq2 = env.request_capture(&alice, &r).unwrap();
    let crank = kp_funded(&mut env);
    let pick = env.expected_pick(seq2, val(7));
    let ix = env.settle_ix(seq2, env.asset_pda(pick), crank.pubkey());
    expect_vault_err(env.send(&[ix], &[&crank]), VaultError::RandomnessNotRevealed);
    env.reveal(seq2, val(8)).unwrap();
    env.settle(seq2, val(8)).unwrap();
    env.assert_invariants();
}

#[test]
fn attack_settle_before_reveal_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    expect_vault_err(env.settle(seq, val(9)), VaultError::RandomnessNotRevealed);
}

#[test]
fn attack_double_settle_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(seq, val(10)).unwrap();
    let pick = env.expected_pick(seq, val(10));
    let crank = kp_funded(&mut env);
    let ix = env.settle_ix(seq, env.asset_pda(pick), crank.pubkey());
    env.send(&[ix.clone()], &[&crank]).unwrap();
    expect_code(env.send(&[ix], &[&crank]), AErr::AccountNotInitialized as u32);
}

#[test]
fn attack_settle_out_of_fifo_order_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let bob = env.new_user(USER_TOKENS);
    let (ra, rb) = (env.new_randomness(), env.new_randomness());
    let sa = env.request_capture(&alice, &ra).unwrap();
    let sb = env.request_capture(&bob, &rb).unwrap();
    env.reveal(sa, val(11)).unwrap();
    env.reveal(sb, val(12)).unwrap();
    expect_vault_err(env.settle(sb, val(12)), VaultError::OutOfOrder);
    env.settle(sa, val(11)).unwrap();
    env.settle(sb, val(12)).unwrap();
    env.assert_invariants();
}

#[test]
fn attack_second_reveal_of_same_commit_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(seq, val(13)).unwrap();
    expect_vault_err(env.reveal(seq, val(14)), VaultError::RandomnessAlreadyRevealed);
}

// ---------------------------------------------------------------- no refund, no abort (ADR-012)

#[test]
fn idl_has_no_cancel_refund_update_close_withdraw_or_burn_instruction() {
    let idl = vault_idl();
    let mut names: Vec<String> = idl["instructions"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap().to_string()).collect();
    names.sort();
    let mut expected = vec![
        "expire_request", "expire_requests", "init_randomness", "init_vault", "merge_incoming", "open_vault", "recommit_randomness",
        "request_capture", "request_reroll", "reveal_randomness", "settle_capture", "settle_reroll", "unwrap",
    ];
    expected.sort();
    assert_eq!(names, expected, "exact instruction set");
    for bad in ["cancel", "refund", "update", "set_", "close", "withdraw", "burn", "migrate", "pause", "freeze", "halt", "guardian"] {
        assert!(!names.iter().any(|n| n.contains(bad)), "forbidden instruction containing {bad}");
    }
}

#[test]
fn attack_user_cannot_abort_after_reveal_recommit_refused_once_revealed() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.reveal(seq, val(15)).unwrap();
    // The user dislikes the (public) outcome and waits past the deadline hoping for a redraw.
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 10);
    let kp = alice.kp.insecure_clone();
    expect_vault_err(env.recommit(seq, &kp), VaultError::RandomnessAlreadyRevealed);
    // Anyone settles; the user gets exactly the revealed pick.
    let pick = env.expected_pick(seq, val(15));
    assert_eq!(env.settle(seq, val(15)).unwrap(), pick);
}

#[test]
fn attack_recommit_before_deadline_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS - 5);
    let kp = alice.kp.insecure_clone();
    expect_vault_err(env.recommit(seq, &kp), VaultError::DeadlineNotReached);
}

#[test]
fn randomness_never_arrives_anyone_recommits_same_request_payment_stays_locked_then_settles() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    let bal = env.token_amount(&alice.ata);
    let old_seed = env.request_state(seq).seed_slot;
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
    let stranger = kp_funded(&mut env);
    env.recommit(seq, &stranger).expect("permissionless recommit");
    let req = env.request_state(seq);
    assert!(req.seed_slot > old_seed);
    assert_eq!(req.commits, 2);
    assert_eq!(env.token_amount(&alice.ata), bal, "no refund");
    env.assert_invariants();
    env.reveal(seq, val(16)).unwrap();
    let got = env.settle(seq, val(16)).unwrap();
    assert_eq!(env.asset_owner(got), alice.kp.pubkey());
    assert_eq!(env.vault_state().total_recommits, 1);
    env.assert_invariants();
}

// ---------------------------------------------------------------- graduation gate + LAZY MINT (ADR-016)

#[test]
fn attack_request_before_open_rejected() {
    let mut env = setup_closed(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    expect_vault_err(env.request_capture(&alice, &r), VaultError::VaultNotOpen);
}

#[test]
fn opens_after_graduation_with_nothing_minted() {
    let mut env = setup_closed(N);
    assert_eq!(env.vault_state().minted_count, 0);
    let mint = env.ids.mint;
    let g = env.set_graduation(MOCK_GRADUATION_OWNER, mint, 1);
    env.open(g).expect("lazy mint: no pre-mint needed to open");
    assert!(env.vault_state().open);
    env.assert_invariants();
}

#[test]
fn attack_open_without_verified_graduation_rejected() {
    let mut env = setup_closed(N);
    let mint = env.ids.mint;
    let fake_owner = env.set_graduation(Pubkey::new_unique(), mint, 1);
    expect_vault_err(env.open(fake_owner), VaultError::GraduationNotVerified);
    let not_graduated = env.set_graduation(MOCK_GRADUATION_OWNER, mint, 0);
    expect_vault_err(env.open(not_graduated), VaultError::GraduationNotVerified);
    let other_mint = env.set_graduation(MOCK_GRADUATION_OWNER, Pubkey::new_unique(), 1);
    expect_vault_err(env.open(other_mint), VaultError::GraduationNotVerified);
    assert!(!env.vault_state().open);
}

#[test]
fn open_is_permissionless_and_one_way() {
    let mut env = setup(N);
    let g = env.graduation_proof;
    expect_vault_err(env.open(g), VaultError::VaultAlreadyOpen);
    assert!(env.vault_state().opened_at_slot > 0);
}

#[test]
fn mint_crank_instruction_no_longer_exists() {
    let names = idl_instruction_names();
    assert!(!names.iter().any(|n| n == "mint_assets"), "batch pre-mint crank removed (lazy mint)");
}

/// Request + reveal a capture; returns (seq, value).
fn pending_capture(env: &mut Env, user: &User, v: [u8; 32]) -> u64 {
    let r = env.new_randomness();
    let seq = env.request_capture(user, &r).unwrap();
    env.reveal(seq, v).unwrap();
    seq
}

#[test]
fn request_escrows_worst_case_mint_cost_in_a_per_request_escrow_pda() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    let req = env.request_state(seq);
    assert_eq!(req.mint_escrow_lamports, hybrid_vault::MINT_ESCROW_LAMPORTS);
    assert_eq!(hybrid_vault::MINT_ESCROW_LAMPORTS, 6_338_100, "(3_570_480 rent + 1_500_000 Core fee) x 125%");
    let pda = env.request_pda(seq);
    let rent = env.svm.minimum_balance_for_rent_exemption(env.svm.get_account(&pda).unwrap().data.len());
    assert_eq!(env.lamports(&pda), rent, "request PDA holds only its rent");
    let esc = env.svm.get_account(&env.escrow_pda(seq)).unwrap();
    assert_eq!(esc.lamports, hybrid_vault::MINT_ESCROW_LAMPORTS, "escrow PDA holds the mint escrow");
    assert!(esc.data.is_empty() && esc.owner == system_program::ID, "data-less system-owned escrow PDA");
    env.assert_invariants();
}

#[test]
fn first_pick_is_minted_to_user_from_escrow_settler_pays_only_tx_fee_no_tip() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let seq = pending_capture(&mut env, &alice, val(1));
    let pick = env.expected_pick(seq, val(1));
    assert!(!env.is_minted(pick));
    let crank = kp_funded(&mut env);
    let (req_l, lock_l) = (env.lamports(&env.request_pda(seq)), env.lamports(&env.rand_lock_pda(&env.request_state(seq).randomness)));
    let esc_l = env.lamports(&env.escrow_pda(seq));
    let (u0, c0, va0) = (env.lamports(&alice.kp.pubkey()), env.lamports(&crank.pubkey()), env.lamports(&env.ids.vault_authority));
    let ix = env.settle_ix(seq, env.asset_pda(pick), crank.pubkey());
    env.send(&[ix], &[&crank]).expect("settle with lazy mint");
    let asset_l = env.lamports(&env.asset_pda(pick));
    assert_eq!(env.asset_owner(pick), alice.kp.pubkey(), "minted straight to the user");
    assert!(env.is_minted(pick));
    assert_eq!(env.vault_state().minted_count, 1);
    assert_eq!(c0 - env.lamports(&crank.pubkey()), 5_000, "settler pays only its tx fee; no tip (T-HV-16)");
    assert_eq!(env.lamports(&env.ids.vault_authority), va0, "vault_authority ends where it started");
    assert!(asset_l <= hybrid_vault::MINT_ESCROW_LAMPORTS);
    assert_eq!(env.lamports(&alice.kp.pubkey()) - u0, req_l + lock_l + esc_l - asset_l, "user gets rents + unspent escrow back");
    assert_eq!(env.lamports(&env.escrow_pda(seq)), 0, "escrow PDA fully drained");
    env.assert_invariants();
}

#[test]
fn attack_settle_never_minted_pick_without_or_with_wrong_leaf_is_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let seq = pending_capture(&mut env, &alice, val(2));
    let pick = env.expected_pick(seq, val(2));
    let c = kp_funded(&mut env);
    let asset = env.asset_pda(pick);
    let ix = env.settle_ix_with(seq, asset, c.pubkey(), None);
    expect_vault_err(env.send(&[ix], &[&c]), VaultError::MintArgsMissing);
    let good = env.mint_args(pick);
    let other = if pick == 0 { 1 } else { 0 };
    let mut cases: Vec<(hybrid_vault::asset_source::MintArgs, VaultError)> = vec![];
    let mut m = good.clone();
    m.leaf.trait_values[0] ^= 1;
    cases.push((m, VaultError::InvalidMerkleProof));
    let mut m = good.clone();
    m.leaf.image_sha256 = [0xEE; 32];
    cases.push((m, VaultError::InvalidMerkleProof));
    let mut m = good.clone();
    m.leaf.uri = "ipfs://bafyswapped".into();
    cases.push((m, VaultError::InvalidMerkleProof));
    cases.push((env.mint_args(other), VaultError::InvalidMerkleProof));
    let mut m = good.clone();
    m.proof = env.proofs[other as usize].clone();
    cases.push((m, VaultError::InvalidMerkleProof));
    let mut m = good.clone();
    m.leaf.uri = "https://example.com/x.json".into();
    cases.push((m, VaultError::UriNotContentAddressed));
    for (k, (m, e)) in cases.into_iter().enumerate() {
        env.warp(1);
        let ix = env.settle_ix_with(seq, asset, c.pubkey(), Some(m));
        let r = env.send(&[ix], &[&c]);
        assert!(r.is_err(), "case {k} must fail");
        expect_vault_err(r, e);
    }
    assert!(!env.is_minted(pick));
    env.settle(seq, val(2)).expect("correct leaf settles");
    env.assert_invariants();
}

/// Find a value whose pick for the next request (after merging returns) is `target`.
fn value_picking(env: &Env, seq: u64, target: u32) -> [u8; 32] {
    for k in 0u64..100_000 {
        let mut v = [0u8; 32];
        v[..8].copy_from_slice(&k.to_le_bytes());
        v[8] = 0x77;
        if env.expected_pick(seq, v) == target {
            return v;
        }
    }
    panic!("no value picks {target}");
}

#[test]
fn returned_asset_is_transferred_again_never_reminted() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS * 2);
    let idx = env.capture(&alice, val(3));
    env.unwrap(&alice, idx).unwrap();
    assert_eq!(env.asset_owner(idx), env.ids.vault_authority);
    let minted = env.vault_state().minted_count;
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    let v = value_picking(&env, seq, idx);
    env.reveal(seq, v).unwrap();
    let c = kp_funded(&mut env);
    let (u0, req_l) = (env.lamports(&alice.kp.pubkey()), env.lamports(&env.request_pda(seq)));
    let lock_l = env.lamports(&env.rand_lock_pda(&r));
    let esc_l = env.lamports(&env.escrow_pda(seq));
    let ix = env.settle_ix_with(seq, env.asset_pda(idx), c.pubkey(), None);
    env.send(&[ix], &[&c]).expect("transfer path needs no leaf");
    assert_eq!(env.asset_owner(idx), alice.kp.pubkey());
    assert_eq!(env.vault_state().minted_count, minted, "no re-mint");
    assert_eq!(esc_l, hybrid_vault::MINT_ESCROW_LAMPORTS);
    assert_eq!(env.lamports(&alice.kp.pubkey()) - u0, req_l + lock_l + esc_l, "full escrow refunded on the transfer path");
    env.assert_invariants();
}

#[test]
fn reroll_hand_in_returns_to_vault_no_remint_no_burn() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(4));
    let supply0 = env.supply();
    let r = env.new_randomness();
    let seq = env.request_reroll(&alice, idx, &r).unwrap();
    assert_eq!(env.asset_owner(idx), env.ids.vault_authority, "hand-in held by the vault, still exists");
    env.reveal(seq, val(5)).unwrap();
    let before = env.vault_state().minted_count;
    let got = env.settle(seq, val(5)).unwrap();
    assert_ne!(got, idx);
    assert!(env.exists(&env.asset_pda(idx)) && env.asset_owner(idx) == env.ids.vault_authority, "never burned");
    assert!(env.vault_state().minted_count <= before + 1, "at most the new pick is minted");
    assert_eq!(env.supply(), supply0);
    env.assert_invariants();
}

#[test]
fn every_index_is_drawable_from_the_start_minted_or_not() {
    let mut env = setup(N);
    let mut data = env.svm.get_account(&env.ids.pool).unwrap().data;
    let pool = PoolView::load(&mut data, &env.ids.vault).unwrap();
    assert_eq!(pool.pool_len(), N, "all N indices drawable at open (lazy Fisher-Yates init)");
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..pool.pool_len() {
        seen.insert(pool.pool_get(i));
    }
    assert_eq!(seen.len() as u32, N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(6));
    env.unwrap(&alice, idx).unwrap();
    env.assert_invariants();
}

#[test]
fn expire_refunds_principal_and_full_mint_escrow_never_the_fee() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let tok0 = env.token_amount(&alice.ata);
    let rec0 = env.lamports(&env.ids.fee_recipient);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    exhaust_recommits(&mut env, seq);
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + hybrid_vault::EXPIRE_GRACE_SLOTS + 2);
    let (u0, req_l, lock_l) = (env.lamports(&alice.kp.pubkey()), env.lamports(&env.request_pda(seq)), env.lamports(&env.rand_lock_pda(&r)));
    let esc_l = env.lamports(&env.escrow_pda(seq));
    assert_eq!(esc_l, hybrid_vault::MINT_ESCROW_LAMPORTS);
    let s = kp_funded(&mut env);
    let ix = env.expire_ix(seq, s.pubkey(), alice.ata);
    env.send(&[ix], &[&s]).expect("expire");
    assert_eq!(env.token_amount(&alice.ata), tok0, "principal back");
    assert_eq!(env.lamports(&alice.kp.pubkey()) - u0, req_l + lock_l + esc_l, "rent + full mint escrow back");
    assert_eq!(env.lamports(&env.ids.fee_recipient) - rec0, FEE, "tier fee never refunded");
    assert_eq!(env.vault_state().minted_count, 0);
    env.assert_invariants();
}

#[test]
fn many_captures_keep_minted_count_le_n_and_bitmap_consistent() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS * 3);
    for k in 0..40u8 {
        let idx = env.capture(&alice, val(100 + k));
        if k % 3 == 0 {
            env.unwrap(&alice, idx).unwrap();
        }
    }
    let v = env.vault_state();
    assert!(v.minted_count <= N);
    assert_eq!(env.minted_popcount(), v.minted_count);
    env.assert_invariants();
}

// ---------------------------------------------------------------- account substitution (A-06, A-08)

#[test]
fn two_launches_get_distinct_per_launch_escrows_and_cross_vault_accounts_rejected() {
    let mut env = setup(N);
    let other = env.launch_and_init(N).unwrap();
    assert_ne!(other.vault, env.ids.vault);
    assert_ne!(other.vault_tokens, env.ids.vault_tokens);
    assert_ne!(other.vault_authority, env.ids.vault_authority);
    assert_ne!(other.collection, env.ids.collection);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(17));
    // Unwrap into vault A while pointing at vault B's token account / collection.
    let mut a = env.unwrap_accts(&alice, idx);
    a.vault_tokens = other.vault_tokens;
    expect_code(env.unwrap_with(&alice, idx, a), AErr::ConstraintSeeds as u32);
    let mut a = env.unwrap_accts(&alice, idx);
    a.collection = other.collection;
    expect_code(env.unwrap_with(&alice, idx, a), AErr::ConstraintAddress as u32);
}

#[test]
fn attack_unwrap_of_foreign_or_wrong_index_asset_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(18));
    let mut a = env.unwrap_accts(&alice, idx);
    a.asset = Pubkey::new_unique();
    expect_code(env.unwrap_with(&alice, idx, a), AErr::ConstraintSeeds as u32);
    // Right PDA, but the asset isn't the caller's: Core refuses the transfer.
    let bob = env.new_user(USER_TOKENS);
    assert!(env.unwrap(&bob, idx).is_err());
    env.assert_invariants();
}

#[test]
fn attack_fake_program_ids_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(19));
    let mut a = env.unwrap_accts(&alice, idx);
    a.mpl_core_program = Pubkey::new_unique();
    expect_code(env.unwrap_with(&alice, idx, a), AErr::ConstraintAddress as u32);
    let mut a = env.unwrap_accts(&alice, idx);
    a.token_program = TOKEN_2022_ID;
    expect_code(env.unwrap_with(&alice, idx, a), AErr::InvalidProgramId as u32);
    let r = env.new_randomness();
    let mut c = env.request_capture_ix(&alice, &r);
    c.switchboard_program = Pubkey::new_unique();
    expect_code(env.request_capture_with(&alice, c), AErr::ConstraintAddress as u32);
}

#[test]
fn attack_token_2022_or_aliased_user_token_account_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    // Copy alice's account under the Token-2022 owner: typed classic-only, so rejected.
    let mut acct = env.svm.get_account(&alice.ata).unwrap();
    acct.owner = TOKEN_2022_ID;
    let fake = Pubkey::new_unique();
    env.svm.set_account(fake, acct).unwrap();
    let r = env.new_randomness();
    let mut a = env.request_capture_ix(&alice, &r);
    a.user_token = fake;
    expect_code(env.request_capture_with(&alice, a), AErr::AccountOwnedByWrongProgram as u32);
    let mut a = env.request_capture_ix(&alice, &r);
    a.user_token = env.ids.vault_tokens;
    assert!(env.request_capture_with(&alice, a).is_err());
}

// ---------------------------------------------------------------- no pause path (ADR-015, Barton 4:57 PM MT)

#[test]
fn no_pause_path_exists_no_key_can_halt_any_instruction() {
    let idl = vault_idl();
    // Instructions, accounts and types only: `errors` keeps Reserved*Pause* names so codes never renumber.
    // Doc strings are excluded too (e.g. LaunchConfig's "mint + freeze authority are None").
    fn strip(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                m.remove("docs");
                m.values_mut().for_each(strip);
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut surface = idl.clone();
    surface.as_object_mut().unwrap().remove("errors");
    strip(&mut surface);
    let text = surface.to_string().to_lowercase();
    for bad in ["pause", "guardian", "freeze", "halt"] {
        assert!(!text.contains(bad), "IDL mentions {bad}");
    }
    let vault = idl["types"].as_array().unwrap().iter().find(|t| t["name"] == "Vault").unwrap();
    let fields: Vec<&str> = vault["type"]["fields"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(!fields.iter().any(|f| f.contains("paus") || f.contains("guardian")), "{fields:?}");
    // Program sources: no paused check anywhere.
    for f in ["instructions/request.rs", "instructions/unwrap.rs", "instructions/settle.rs", "instructions/expire.rs", "instructions/randomness_ix.rs"] {
        let src = std::fs::read_to_string(format!("{}/../../programs/hybrid_vault/src/{f}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(!src.contains("paused_until") && !src.contains("VaultError::Paused"), "{f}");
    }
}

#[test]
fn expire_several_stuck_requests_in_one_transaction() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let bob = env.new_user(USER_TOKENS);
    let (ra, rb) = (env.new_randomness(), env.new_randomness());
    let sa = env.request_capture(&alice, &ra).unwrap();
    let sb = env.request_capture(&bob, &rb).unwrap();
    for _ in 0..hybrid_vault::MAX_RECOMMITS {
        env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
        let s = kp_funded(&mut env);
        env.recommit(sa, &s).unwrap();
        env.recommit(sb, &s).unwrap();
    }
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + hybrid_vault::EXPIRE_GRACE_SLOTS + 2);
    let s = kp_funded(&mut env);
    let ixs = [env.expire_ix(sa, s.pubkey(), alice.ata), env.expire_ix(sb, s.pubkey(), bob.ata)];
    env.send(&ixs, &[&s]).expect("batch expire (FIFO head advances per instruction)");
    assert_eq!(env.vault_state().total_expired, 2);
    env.assert_invariants();
}

#[test]
fn attack_caller_chosen_oracle_is_rejected_program_selects_it() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.vault_state().next_seq;
    let chosen = env.select_oracle(seq, &[]);
    for bad in env.sb_oracles.clone().into_iter().filter(|o| *o != chosen).chain([Pubkey::new_unique()]) {
        let mut a = env.request_capture_ix(&alice, &r);
        a.sb_oracle = bad;
        expect_vault_err(env.request_capture_with(&alice, a), VaultError::WrongOracle);
    }
    let seq = env.request_capture(&alice, &r).unwrap();
    assert_eq!(env.request_state(seq).oracles[0], chosen);
    // Each recommit gets a different, program-chosen oracle.
    let mut seen = vec![chosen];
    for _ in 0..hybrid_vault::MAX_RECOMMITS {
        env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
        let s = kp_funded(&mut env);
        env.recommit(seq, &s).unwrap();
        let req = env.request_state(seq);
        let o = req.oracles[req.commits as usize - 1];
        assert!(!seen.contains(&o), "fresh oracle each recommit");
        seen.push(o);
    }
}

// ---------------------------------------------------------------- solvency (A-07)

#[test]
fn donation_to_vault_tokens_changes_no_outcome_and_keeps_invariant() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(22));
    let vt = env.ids.vault_tokens;
    let bal = env.token_amount(&vt);
    let src = env.token_amount(&env.ids.launch_destination);
    let ld = env.ids.launch_destination;
    env.set_token_amount(&ld, src - 12345);
    env.set_token_amount(&vt, bal + 12345);
    let before = env.token_amount(&alice.ata);
    env.unwrap(&alice, idx).unwrap();
    assert_eq!(env.token_amount(&alice.ata) - before, env.ratio_base, "still exactly N");
    assert_eq!(env.token_amount(&vt), 12345);
}

#[test]
fn capture_down_to_pool_floor_then_next_request_rejected_no_asset_available() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS * 3);
    let max = N - hybrid_vault::instructions::request::pool_floor(N) as u32;
    for i in 0..max {
        env.capture(&alice, val((i % 250) as u8));
    }
    env.assert_invariants();
    assert_eq!(env.vault_state().assets_outside, max as u64);
    assert_eq!(env.token_amount(&env.ids.vault_tokens), env.ratio_base * max as u64);
    let r = env.new_randomness();
    expect_vault_err(env.request_capture(&alice, &r), VaultError::NoAssetAvailable);
}

#[test]
fn unwrapped_nft_is_not_drawable_by_earlier_pending_request() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let bob = env.new_user(USER_TOKENS);
    let x = env.capture(&bob, val(23));
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.unwrap(&bob, x).unwrap(); // tagged after alice's request
    for v in 0..20u8 {
        assert_ne!(env.expected_pick(seq, val(v)), x);
    }
    env.reveal(seq, val(24)).unwrap();
    let got = env.settle(seq, val(24)).unwrap();
    assert_ne!(got, x);
    assert_eq!(env.asset_owner(x), env.ids.vault_authority);
    env.assert_invariants();
}

/// Property test (deterministic xorshift, several seeds): random interleavings of capture,
/// re-roll, unwrap, reveal, recommit and settle by several users. After EVERY transaction the
/// on-chain invariants hold (the program also asserts them itself) and the outside view is exact:
/// vault tokens == N * (NFTs outside + pending captures + pending re-rolls), supply constant, all tokens and NFTs
/// accounted for.
#[test]
fn property_solvency_invariant_holds_under_random_operation_sequences() {
    for seed in [0x9E37_79B9_7F4A_7C15u64, 0xD1B5_4A32_D192_ED03, 42] {
        let mut env = setup(N);
        let users: Vec<User> = (0..3).map(|_| env.new_user(USER_TOKENS)).collect();
        let mut rng = seed;
        let mut next = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        let mut pending: Vec<(u64, bool)> = vec![]; // (seq, revealed)
        for step in 0..60 {
            let u = &users[(next() % 3) as usize];
            match next() % 6 {
                0 | 1 => {
                    let r = env.new_randomness();
                    if let Ok(s) = env.request_capture(u, &r) {
                        pending.push((s, false));
                    }
                }
                2 => {
                    let owned: Vec<u32> = (0..N).filter(|i| env.asset_owner(*i) == u.kp.pubkey()).collect();
                    if let Some(&i) = owned.get((next() as usize) % owned.len().max(1)) {
                        let r = env.new_randomness();
                        if let Ok(s) = env.request_reroll(u, i, &r) {
                            pending.push((s, false));
                        }
                    }
                }
                3 => {
                    let owned: Vec<u32> = (0..N).filter(|i| env.asset_owner(*i) == u.kp.pubkey()).collect();
                    if let Some(&i) = owned.first() {
                        env.unwrap(u, i).expect("unwrap never fails for the owner");
                    }
                }
                4 => {
                    if let Some(p) = pending.iter_mut().find(|p| !p.1) {
                        if next() % 4 == 0 && env.request_state(p.0).commits <= hybrid_vault::MAX_RECOMMITS {
                            env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
                            let s = Keypair::new();
                            env.svm.airdrop(&s.pubkey(), 1_000_000_000).unwrap();
                            env.recommit(p.0, &s).expect("recommit after timeout");
                        } else {
                            env.reveal(p.0, val((next() % 256) as u8)).unwrap();
                            p.1 = true;
                        }
                    }
                }
                _ => {
                    if let Some(&(s, true)) = pending.first() {
                        let v = env.svm.get_account(&env.request_state(s).randomness).unwrap().data[152..184].try_into().unwrap();
                        env.settle(s, v).expect("settle head");
                        pending.remove(0);
                    }
                }
            }
            env.assert_invariants();
            let _ = step;
        }
    }
}

#[test]
fn uniform_below_reaches_every_index_and_is_unbiased_enough() {
    let n = 7u32;
    let mut hits = vec![0u32; n as usize];
    for i in 0..7000u32 {
        let r = selection::request_randomness(&[(i % 251) as u8; 32], &Pubkey::new_from_array([(i / 251) as u8; 32]), i as u64);
        hits[selection::uniform_below(&r, n) as usize] += 1;
    }
    for h in hits {
        assert!((800..1200).contains(&h), "bucket {h}");
    }
}

// ---------------------------------------------------------------- audit PoC regressions (Auditor A 10/11/12)

#[test]
fn regress_poc10_escrow_drain_must_fail() {
    // PoC 10: an authority zeroed the swap amount, pulled NFTs for free, then raised it and drained the
    // escrow. Here: no instruction can change the ratio, and nobody but the vault PDA can move escrow.
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(70));
    let lc_before = env.svm.get_account(&env.ids.launch_config).unwrap().data;
    // Creator (the would-be authority) tries a direct SPL transfer out of the escrow: not the owner.
    let creator = env.creator.insecure_clone();
    let steal = spl_token::instruction::transfer(&SPL_TOKEN_ID, &env.ids.vault_tokens, &alice.ata, &creator.pubkey(), &[], env.ratio_base).unwrap();
    assert!(env.send(&[steal], &[&creator]).is_err(), "escrow can't be moved by the creator");
    // No update/withdraw instruction exists (IDL test) and LaunchConfig is unchanged.
    assert_eq!(env.svm.get_account(&env.ids.launch_config).unwrap().data, lc_before);
    // Every NFT out is still backed 1:1 and a release still pays exactly N.
    assert_eq!(env.token_amount(&env.ids.vault_tokens), env.ratio_base);
    let before = env.token_amount(&alice.ata);
    env.unwrap(&alice, idx).unwrap();
    assert_eq!(env.token_amount(&alice.ata) - before, env.ratio_base);
    env.assert_invariants();
}

#[test]
fn regress_poc11_cherrypick_must_fail() {
    // PoC 11: a bot captured a chosen (rare) NFT. Here: request_capture takes no asset argument, settle
    // only accepts the VRF pick, and pooled assets are owned by the vault PDA.
    let idl = vault_idl();
    let cap = idl["instructions"].as_array().unwrap().iter().find(|i| i["name"] == "request_capture").unwrap();
    assert!(cap["args"].as_array().unwrap().is_empty(), "no asset/index argument on capture");
    let mut env = setup(N);
    let bot = env.new_user(USER_TOKENS);
    let rare = 7u32;
    let r = env.new_randomness();
    let seq = env.request_capture(&bot, &r).unwrap();
    env.reveal(seq, val(71)).unwrap();
    let pick = env.expected_pick(seq, val(71));
    if pick != rare {
        let crank = kp_funded(&mut env);
        let ix = env.settle_ix(seq, env.asset_pda(rare), crank.pubkey());
        expect_vault_err(env.send(&[ix], &[&crank]), VaultError::WrongAsset);
    }
    assert_eq!(env.settle(seq, val(71)).unwrap(), pick);
    // The un-picked index stays with the vault: either never minted (lazy) or owned by the vault PDA.
    let other = if pick == rare { (rare + 1) % N } else { rare };
    assert!(!env.is_minted(other) || env.asset_owner(other) == env.ids.vault_authority);
    assert_ne!(env.asset_owner(other), bot.kp.pubkey());
}

#[test]
fn regress_poc12_token_swap_must_fail() {
    // PoC 12: capture paid with a worthless mint. Here: the mint is pinned to the LaunchConfig mint and
    // the user's token account must be of that mint.
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let junk = Keypair::new();
    let creator = env.creator.insecure_clone();
    let rent = env.svm.minimum_balance_for_rent_exemption(82);
    let mk = system_instruction::create_account(&creator.pubkey(), &junk.pubkey(), rent, 82, &SPL_TOKEN_ID);
    let init = spl_token::instruction::initialize_mint2(&SPL_TOKEN_ID, &junk.pubkey(), &creator.pubkey(), None, DECIMALS).unwrap();
    let ata = get_associated_token_address(&alice.kp.pubkey(), &junk.pubkey());
    let cata = create_associated_token_account(&creator.pubkey(), &alice.kp.pubkey(), &junk.pubkey(), &SPL_TOKEN_ID);
    let mt = spl_token::instruction::mint_to(&SPL_TOKEN_ID, &junk.pubkey(), &ata, &creator.pubkey(), &[], u64::MAX / 2).unwrap();
    env.send(&[mk, init, cata, mt], &[&creator, &junk]).unwrap();
    let r = env.new_randomness();
    let mut a = env.request_capture_ix(&alice, &r);
    a.mint = junk.pubkey();
    a.user_token = ata;
    expect_vault_err(env.request_capture_with(&alice, a), VaultError::MintMismatch);
    let mut a = env.request_capture_ix(&alice, &r);
    a.user_token = ata;
    assert!(env.request_capture_with(&alice, a).is_err(), "junk-mint token account refused (token::mint)");
    assert_eq!(env.vault_state().pending_captures, 0);
    env.assert_invariants();
}

// ---------------------------------------------------------------- T-GRAD-01 pre-funded PDA griefing

#[test]
fn attack_prefund_asset_pda_does_not_block_lazy_mint_lamports_go_to_the_user() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let seq = pending_capture(&mut env, &alice, val(9));
    let pick = env.expected_pick(seq, val(9));
    let griefer = kp_funded(&mut env);
    let target = env.asset_pda(pick);
    let t = system_instruction::transfer(&griefer.pubkey(), &target, 890_880);
    env.send(&[t], &[&griefer]).unwrap();
    let va0 = env.lamports(&env.ids.vault_authority);
    let u0 = env.lamports(&alice.kp.pubkey());
    let (req_l, lock_l) = (env.lamports(&env.request_pda(seq)), env.lamports(&env.rand_lock_pda(&env.request_state(seq).randomness)));
    let esc_l = env.lamports(&env.escrow_pda(seq));
    env.settle(seq, val(9)).expect("mint succeeds despite pre-funding");
    assert_eq!(env.asset_owner(pick), alice.kp.pubkey());
    let asset_l = env.lamports(&target);
    assert_eq!(env.lamports(&env.ids.vault_authority), va0);
    assert_eq!(env.lamports(&alice.kp.pubkey()) - u0, req_l + lock_l + esc_l + 890_880 - asset_l, "drained pre-fund refunded to the user");
    env.assert_invariants();
}

#[test]
fn attack_prefund_collection_pda_does_not_block_init_vault() {
    let mut env = setup_closed_with(N, 1_000_000);
    assert!(env.exists(&env.ids.collection));
    let mint = env.ids.mint;
    let g = env.set_graduation(MOCK_GRADUATION_OWNER, mint, 1);
    env.open(g).expect("opens");
}

#[test]
fn attack_reinitialize_vault_fails() {
    let mut env = setup(N);
    let ids = env.ids.clone();
    let root = env.root;
    let ix = env.init_vault_ix(&ids, root);
    assert!(env.send_as_creator(&[ix]).is_err());
}

// ---------------------------------------------------------------- recommit limits + principal-only expire (M-04)

fn exhaust_recommits(env: &mut Env, seq: u64) {
    for _ in 0..hybrid_vault::MAX_RECOMMITS {
        env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
        let s = kp_funded(env);
        env.recommit(seq, &s).expect("recommit");
    }
}

#[test]
fn attack_recommit_with_reused_oracle_rejected() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
    let s = kp_funded(&mut env);
    let first = env.request_state(seq).oracles[0];
    let ix = env.recommit_ix_with_oracle(seq, s.pubkey(), first);
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::OracleReused);
}

#[test]
fn recommits_are_capped_then_expire_returns_principal_only_fee_kept() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let tok0 = env.token_amount(&alice.ata);
    let rec0 = env.lamports(&env.ids.fee_recipient);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    let s = kp_funded(&mut env);
    // Too early: recommits remain.
    let ix = env.expire_ix(seq, s.pubkey(), alice.ata);
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::RecommitsRemaining);
    exhaust_recommits(&mut env, seq);
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
    let ix = env.recommit_ix(seq, s.pubkey());
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::RecommitsExhausted);
    let ix = env.expire_ix(seq, s.pubkey(), alice.ata);
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::ExpiryNotReached);
    env.warp(hybrid_vault::EXPIRE_GRACE_SLOTS + 1);
    let ix = env.expire_ix(seq, s.pubkey(), alice.ata);
    env.send(&[ix], &[&s]).expect("expire");
    assert_eq!(env.token_amount(&alice.ata), tok0, "principal (exactly N) returned");
    assert_eq!(env.lamports(&env.ids.fee_recipient) - rec0, FEE, "fee never refunded");
    assert_eq!(env.vault_state().total_expired, 1);
    env.assert_invariants();
}

#[test]
fn expired_reroll_returns_the_handed_in_nft() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let idx = env.capture(&alice, val(72));
    let r = env.new_randomness();
    let seq = env.request_reroll(&alice, idx, &r).unwrap();
    exhaust_recommits(&mut env, seq);
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + hybrid_vault::EXPIRE_GRACE_SLOTS + 2);
    let s = kp_funded(&mut env);
    let ix = env.expire_ix(seq, s.pubkey(), alice.ata);
    env.send(&[ix], &[&s]).expect("expire re-roll");
    assert_eq!(env.asset_owner(idx), alice.kp.pubkey());
    env.assert_invariants();
}

#[test]
fn attack_expire_impossible_once_revealed() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let seq = env.request_capture(&alice, &r).unwrap();
    exhaust_recommits(&mut env, seq);
    env.reveal(seq, val(73)).unwrap();
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + hybrid_vault::EXPIRE_GRACE_SLOTS + 2);
    let s = kp_funded(&mut env);
    let ix = env.expire_ix(seq, s.pubkey(), alice.ata);
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::RandomnessAlreadyRevealed);
    env.settle(seq, val(73)).unwrap();
    env.assert_invariants();
}

#[test]
fn no_todo_or_fixme_left_in_program_sources() {
    // M-32: no unfinished markers in shipped program code.
    for dir in ["hybrid_launch", "hybrid_vault"] {
        let root = format!("{}/../../programs/{dir}/src", env!("CARGO_MANIFEST_DIR"));
        let mut stack = vec![std::path::PathBuf::from(root)];
        while let Some(p) = stack.pop() {
            for e in std::fs::read_dir(&p).unwrap() {
                let path = e.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().map(|x| x == "rs").unwrap_or(false) {
                    let src = std::fs::read_to_string(&path).unwrap();
                    for bad in ["TODO", "FIXME", "XXX", "unimplemented!", "todo!"] {
                        assert!(!src.contains(bad), "{bad} in {}", path.display());
                    }
                }
            }
        }
    }
}

/// CD Q1 / QA-FEE-04: the exact lamport flow of one first mint (measured, not modelled).
#[test]
fn first_mint_cost_breakdown_is_exact_and_unspent_deposit_is_refunded() {
    let mut env = setup(N);
    let alice = env.new_user(USER_TOKENS);
    let seq = pending_capture(&mut env, &alice, val(1));
    let pick = env.expected_pick(seq, val(1));
    assert!(!env.is_minted(pick));
    let esc = env.lamports(&env.escrow_pda(seq));
    assert_eq!(esc, hybrid_vault::MINT_ESCROW_LAMPORTS);
    let (req_l, lock_l) = (env.lamports(&env.request_pda(seq)), env.lamports(&env.rand_lock_pda(&env.request_state(seq).randomness)));
    let u0 = env.lamports(&alice.kp.pubkey());
    let crank = kp_funded(&mut env);
    let ix = env.settle_ix(seq, env.asset_pda(pick), crank.pubkey());
    env.send(&[ix], &[&crank]).unwrap();
    let asset = env.svm.get_account(&env.asset_pda(pick)).unwrap();
    let rent = env.svm.minimum_balance_for_rent_exemption(asset.data.len());
    let core_fee_in_asset = asset.lamports - rent;
    let spent = asset.lamports;
    let refunded = env.lamports(&alice.kp.pubkey()) - u0 - req_l - lock_l;
    eprintln!(
        "FIRST-MINT: req_rent={req_l} lock_rent={lock_l} escrow={esc} asset_len={} rent={rent} core_fee_held_in_asset={core_fee_in_asset} spent={spent} refunded_to_user={refunded}",
        asset.data.len()
    );
    assert_eq!(spent + refunded, esc, "escrow = mint spend + exact refund; nothing stranded");
    assert!(core_fee_in_asset == 0 || core_fee_in_asset == hybrid_launch::CORE_CREATE_FEE_LAMPORTS);
    assert!(rent <= hybrid_launch::CORE_ASSET_RENT_LAMPORTS, "actual asset is no bigger than the escrowed worst case");
}

// ---- M-04 batch expire: expire_requests(count) ----

/// `k` stuck requests (captures, plus one re-roll if `with_reroll`), all past expiry.
/// Returns [(seq, user, user_token)] in queue order.
fn stuck_heads(env: &mut Env, k: usize, with_reroll: bool) -> Vec<(u64, User)> {
    let mut out = vec![];
    let reroller = if with_reroll { Some(env.new_user(USER_TOKENS)) } else { None };
    let idx = reroller.as_ref().map(|u| env.capture(u, val(90)));
    for i in 0..k {
        let r = env.new_randomness();
        if i == 1 && with_reroll {
            let u = reroller.as_ref().unwrap();
            let seq = env.request_reroll(u, idx.unwrap(), &r).unwrap();
            out.push((seq, u.clone_user()));
        } else {
            let u = env.new_user(USER_TOKENS);
            let seq = env.request_capture(&u, &r).unwrap();
            out.push((seq, u));
        }
    }
    for _ in 0..hybrid_vault::MAX_RECOMMITS {
        env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
        let s = kp_funded(env);
        for (seq, _) in &out {
            env.recommit(*seq, &s).unwrap();
        }
    }
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + hybrid_vault::EXPIRE_GRACE_SLOTS + 2);
    out
}

#[test]
fn m04_batch_expire_k_heads_in_one_instruction_refunds_principal_and_escrow_never_fee() {
    let mut env = setup(N);
    let heads = stuck_heads(&mut env, 3, true);
    let before: Vec<(u64, u64)> = heads.iter().map(|(_, u)| (env.token_amount(&u.ata), env.lamports(&u.kp.pubkey()))).collect();
    let owed: Vec<u64> = heads
        .iter()
        .map(|(seq, _)| {
            let r = env.request_state(*seq);
            env.lamports(&env.request_pda(*seq)) + env.lamports(&env.rand_lock_pda(&r.randomness)) + env.lamports(&env.escrow_pda(*seq))
        })
        .collect();
    let handed_in = env.request_state(heads[1].0).handed_in_index;
    let rec0 = env.lamports(&env.ids.fee_recipient);
    let s = kp_funded(&mut env);
    let ix = env.expire_batch_ix(&heads.iter().map(|(q, u)| (*q, u.ata)).collect::<Vec<_>>(), s.pubkey());
    let legacy = VersionedTransaction::try_new(
        VersionedMessage::Legacy(Message::new_with_blockhash(&[ix.clone()], Some(&s.pubkey()), &env.svm.latest_blockhash())),
        &[&s],
    )
    .unwrap();
    assert!(bincode_len(&legacy) <= 1_232, "K = 3 fits a LEGACY tx ({} bytes)", bincode_len(&legacy));
    let cu = env.send_max_cu(&[ix], &[&s]).expect("expire 3 heads in ONE instruction");
    eprintln!("BATCH-EXPIRE k=3 legacy (2 captures + 1 re-roll): {cu} CU, {} bytes", bincode_len(&legacy));
    let v = env.vault_state();
    assert_eq!(v.total_expired, 3);
    assert_eq!(v.next_settle_seq, heads[2].0 + 1);
    assert_eq!(env.lamports(&env.ids.fee_recipient), rec0, "no fee refunded or charged");
    for (i, (seq, u)) in heads.iter().enumerate() {
        assert!(!env.exists(&env.request_pda(*seq)), "request closed");
        assert_eq!(env.lamports(&env.escrow_pda(*seq)), 0, "escrow drained");
        if i == 1 {
            assert_eq!(env.asset_owner(handed_in), u.kp.pubkey(), "handed-in NFT back");
            assert_eq!(env.token_amount(&u.ata), before[i].0);
        } else {
            assert_eq!(env.token_amount(&u.ata), before[i].0 + env.ratio_base, "exactly N tokens back");
        }
        assert_eq!(env.lamports(&u.kp.pubkey()) - before[i].1, owed[i], "rents + FULL escrow back");
    }
    env.assert_invariants();
}

#[test]
fn m04_batch_expire_max_per_call_fits_compute() {
    let mut env = setup(N);
    let k = hybrid_vault::MAX_EXPIRE_PER_CALL as usize;
    let heads = stuck_heads(&mut env, k, true);
    let s = kp_funded(&mut env);
    let ix = env.expire_batch_ix(&heads.iter().map(|(q, u)| (*q, u.ata)).collect::<Vec<_>>(), s.pubkey());
    // K > 3 doesn't fit a legacy tx (1,232 bytes), so send it as a real client must: a v0 tx whose
    // non-signer accounts come from an address lookup table.
    let mut addrs: Vec<Pubkey> = ix.accounts.iter().filter(|m| m.pubkey != s.pubkey()).map(|m| m.pubkey).collect();
    addrs.sort();
    addrs.dedup();
    let alt = env.create_alt(&addrs);
    env.warp(1);
    let (cu, bytes) = env.send_v0(&[ix], &[&s], alt, addrs).expect("MAX_EXPIRE_PER_CALL heads in one v0 tx");
    eprintln!("BATCH-EXPIRE k={k} (v0 + ALT): {cu} CU, {bytes} tx bytes");
    assert!(bytes <= 1_232, "fits a packet with the ALT");
    assert!(cu < 1_400_000);
    assert_eq!(env.vault_state().total_expired, k as u64);
    env.assert_invariants();
}

#[test]
fn m04_batch_expire_rejects_bad_count_and_account_counts() {
    let mut env = setup(N);
    let heads = stuck_heads(&mut env, 2, false);
    let s = kp_funded(&mut env);
    let rem: Vec<AccountMeta> = heads.iter().flat_map(|(q, u)| env.expire_group(*q, u.ata)).collect();
    for (count, r) in [
        (0u8, vec![]),
        (hybrid_vault::MAX_EXPIRE_PER_CALL + 1, rem.clone()),
        (2, rem[..13].to_vec()),
        (1, rem.clone()),
    ] {
        let ix = env.expire_batch_ix_raw(count, s.pubkey(), r);
        expect_vault_err(env.send(&[ix], &[&s]), VaultError::ExpireBatchInvalid);
    }
    assert_eq!(env.vault_state().total_expired, 0);
}

#[test]
fn m04_batch_expire_is_all_or_nothing_and_fifo_only() {
    let mut env = setup(N);
    let heads = stuck_heads(&mut env, 3, false);
    let s = kp_funded(&mut env);
    // Out of order: second head first.
    let ix = env.expire_batch_ix(&[(heads[1].0, heads[1].1.ata), (heads[0].0, heads[0].1.ata)], s.pubkey());
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::ExpireBatchAccountMismatch);
    // A fresh request behind the stuck ones isn't expirable: the whole batch fails, nothing changes.
    let late = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    let late_seq = env.request_capture(&late, &r).unwrap();
    let tok0 = env.token_amount(&heads[0].1.ata);
    let mut all: Vec<(u64, Pubkey)> = heads.iter().map(|(q, u)| (*q, u.ata)).collect();
    all.push((late_seq, late.ata));
    let ix = env.expire_batch_ix(&all, s.pubkey());
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::RecommitsRemaining);
    assert_eq!(env.vault_state().total_expired, 0, "atomic: nothing expired");
    assert_eq!(env.token_amount(&heads[0].1.ata), tok0);
    // The three stuck heads alone expire fine.
    let ix = env.expire_batch_ix(&all[..3], s.pubkey());
    env.send(&[ix], &[&s]).unwrap();
    assert_eq!(env.vault_state().total_expired, 3);
    env.assert_invariants();
}

#[test]
fn m04_batch_expire_rejects_substituted_per_request_accounts() {
    let mut env = setup(N);
    let heads = stuck_heads(&mut env, 2, false);
    let s = kp_funded(&mut env);
    let attacker = env.new_user(USER_TOKENS);
    let good: Vec<AccountMeta> = heads.iter().flat_map(|(q, u)| env.expire_group(*q, u.ata)).collect();
    let other_escrow = env.escrow_pda(heads[1].0);
    let cases: Vec<(usize, Pubkey)> = vec![
        (3, attacker.kp.pubkey()), // user swapped
        (4, attacker.ata),         // refund to the attacker's token account
        (5, other_escrow),         // another request's escrow
        (1, env.rand_lock_pda(&env.request_state(heads[1].0).randomness)), // wrong rand_lock
        (0, env.request_pda(heads[1].0)), // wrong request for the head
    ];
    for (pos, sub) in cases {
        let mut rem = good.clone();
        rem[pos].pubkey = sub;
        let ix = env.expire_batch_ix_raw(2, s.pubkey(), rem);
        assert!(env.send(&[ix], &[&s]).is_err(), "substitution at {pos} must fail");
    }
    // Read-only where writable is required.
    let mut rem = good.clone();
    rem[5].is_writable = false;
    let ix = env.expire_batch_ix_raw(2, s.pubkey(), rem);
    expect_vault_err(env.send(&[ix], &[&s]), VaultError::ExpireBatchAccountMismatch);
    assert_eq!(env.vault_state().total_expired, 0);
    let ix = env.expire_batch_ix_raw(2, s.pubkey(), good);
    env.send(&[ix], &[&s]).unwrap();
    env.assert_invariants();
}

