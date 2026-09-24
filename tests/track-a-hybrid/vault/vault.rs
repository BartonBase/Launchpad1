//! hybrid_vault LiteSVM tests (QA layout: tests/track-a-hybrid/vault). Real SBF builds of
//! hybrid_launch + hybrid_vault, real Metaplex Core, TEST-ONLY mock Switchboard.
//! Every test is named for the property or attack it proves.

mod harness;
use harness::*;

#[test]
fn capture_then_unwrap_returns_exactly_ratio() {
    let mut env = setup(4);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let before = env.token_amount(&alice.ata);
    let idx = env.capture(&alice, &r, val(1));
    assert_eq!(env.asset_owner(idx), alice.kp.pubkey());
    assert_eq!(env.token_amount(&alice.ata), before - env.ratio_base - env.capture_fee);
    env.assert_invariants(&[&alice]);

    env.unwrap(&alice, idx).expect("unwrap");
    assert_eq!(env.asset_owner(idx), env.vault_authority);
    assert_eq!(env.token_amount(&alice.ata), before - env.capture_fee, "exactly ratio back, no unwrap fee");
    env.assert_invariants(&[&alice]);
}
