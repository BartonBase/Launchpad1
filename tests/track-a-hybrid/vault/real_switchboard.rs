//! hybrid_vault against the REAL Switchboard On-Demand devnet program (dumped binary + dumped devnet
//! queue/state/oracles, fixtures/switchboard-devnet) on LiteSVM. Proves our hand-built CPIs
//! (randomness_init, randomness_commit, randomness_reveal) match the real program's interface.
//! A real reveal needs an oracle's secp256k1 signature from the Switchboard gateway, so it can
//! only happen on devnet; here we prove a FORGED reveal is rejected by the real program.

mod harness;
use harness::*;

#[test]
fn real_switchboard_init_and_commit_via_hybrid_vault_cpi() {
    let mut env = setup_real_sb(100);
    let kp = Keypair::new();
    let ix = env.init_randomness_ix(&kp.pubkey(), env.sb_queue);
    let c = env.creator.insecure_clone();
    env.send(&[ix], &[&c, &kp]).expect("real randomness_init via init_randomness CPI");
    let acct = env.svm.get_account(&kp.pubkey()).unwrap();
    assert_eq!(acct.owner, hybrid_vault::SWITCHBOARD_PROGRAM_ID);
    eprintln!("REAL-SB randomness account: {} bytes, {} lamports", acct.data.len(), acct.lamports);
    let alice = env.new_user(USER_TOKENS);
    let seq = env.request_capture(&alice, &kp.pubkey()).expect("real randomness_commit via request_capture CPI");
    let req = env.request_state(seq);
    eprintln!("REAL-SB committed seq {seq} seed_slot {} oracle {}", req.seed_slot, req.oracles[0]);
}

#[test]
fn real_switchboard_init_cost_breakdown() {
    let mut env = setup_real_sb(100);
    let kp = Keypair::new();
    let payer = env.creator.pubkey();
    let before = env.lamports(&payer);
    let ix = env.init_randomness_ix(&kp.pubkey(), env.sb_queue);
    let c = env.creator.insecure_clone();
    env.send(&[ix], &[&c, &kp]).unwrap();
    let spent = before - env.lamports(&payer);
    let rand = env.lamports(&kp.pubkey());
    let escrow = env.lamports(&get_associated_token_address(&kp.pubkey(), &WRAPPED_SOL_MINT));
    eprintln!("REAL-SB init: payer spent {spent} (randomness acct {rand}, wSOL reward escrow {escrow}, rest = LUT + tx fee {})", spent - rand - escrow);
    assert!(spent >= rand + escrow);
}

#[test]
fn real_switchboard_rejects_forged_reveal() {
    let mut env = setup_real_sb(100);
    let kp = Keypair::new();
    let ix = env.init_randomness_ix(&kp.pubkey(), env.sb_queue);
    let c = env.creator.insecure_clone();
    env.send(&[ix], &[&c, &kp]).unwrap();
    let alice = env.new_user(USER_TOKENS);
    let seq = env.request_capture(&alice, &kp.pubkey()).unwrap();
    let data_before = env.svm.get_account(&kp.pubkey()).unwrap().data;
    let err = env.reveal(seq, [42u8; 32]).expect_err("forged oracle signature must be rejected");
    eprintln!("REAL-SB forged reveal error: {}", &err[..err.len().min(1500)]);
    assert!(err.contains(&SWITCHBOARD_PROGRAM_ID.to_string()), "rejection must come from the Switchboard program");
    assert_eq!(env.svm.get_account(&kp.pubkey()).unwrap().data, data_before, "randomness account untouched");
    assert!(err.contains("InvalidSecpSignature"), "accounts accepted; the oracle signature check is what fails");
    assert!(!env.request_state(seq).revealed);
}

#[test]
fn real_switchboard_recommit_after_timeout_uses_new_oracle() {
    let mut env = setup_real_sb(100);
    let kp = Keypair::new();
    let ix = env.init_randomness_ix(&kp.pubkey(), env.sb_queue);
    let c = env.creator.insecure_clone();
    env.send(&[ix], &[&c, &kp]).unwrap();
    let alice = env.new_user(USER_TOKENS);
    let seq = env.request_capture(&alice, &kp.pubkey()).unwrap();
    let first = env.request_state(seq);
    env.warp(hybrid_vault::REVEAL_TIMEOUT_SLOTS + 1);
    let crank = Keypair::new();
    env.svm.airdrop(&crank.pubkey(), 1_000_000_000).unwrap();
    env.recommit(seq, &crank).expect("real randomness_commit via recommit CPI");
    let second = env.request_state(seq);
    assert_eq!(second.commits, 2);
    assert!(second.seed_slot > first.seed_slot);
    assert_ne!(second.oracles[1], first.oracles[0]);
    let data = env.svm.get_account(&kp.pubkey()).unwrap().data;
    assert_eq!(Pubkey::try_from(&data[112..144]).unwrap(), second.oracles[1], "Switchboard recorded the new oracle");
    eprintln!("REAL-SB recommit seed_slot {} -> {}, oracle {} -> {}", first.seed_slot, second.seed_slot, first.oracles[0], second.oracles[1]);
}
