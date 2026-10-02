//! Tiered flat SOL fee, v0.7 (BRIEF 4:49 / 4:52 / 4:55 PM + RELEASE IS FREE 5:04 PM MT): NO token fee; the tier fee
//! (50k = 0.002 SOL, 100k/200k = 0.005, 500k..5M = 0.01; cap 0.01) applies to CAPTURE and RE-ROLL only, re-roll fee ==
//! capture fee; RELEASE IS FREE (no fee account, no holding account, no sweep). Fee derived from the ratio by
//! hybrid_launch, fixed in LaunchConfig. Lazy minting (5:13 PM) adds a first-mint cost paid by the capturer: see
//! qa_M22_lazy_mint.rs. Merged IDs are v2 (M-01..M-40).
//! OBSOLETE v0.7 (deleted): qa_FEE15_M36_sweep_only_to_fixed_fee_wallet_and_permissionless,
//! qa_FEE16_M36_fee_vault_lamports_exit_only_via_sweep, and the FeeVault cases of FEE-08.
//! Committed wip f4761af predates all of this (no SOL fee; its unwrap still takes the token `fee_escrow`).
use crate::{common::*, harness::*};

/// Lamports + tokens a user spends, including rent deposits (and any refunds), across a list of steps.
fn spend(env: &mut Env, u: &User, f: impl FnOnce(&mut Env)) -> (i128, i128) {
    let (l0, t0) = (env.svm.get_balance(&u.kp.pubkey()).unwrap() as i128, env.token_amount(&u.ata) as i128);
    f(env);
    (l0 - env.svm.get_balance(&u.kp.pubkey()).unwrap() as i128, t0 - env.token_amount(&u.ata) as i128)
}

/// FEE-02 / M-14: release (unwrap) returns exactly ratio tokens, with no token deduction, for every NFT, repeatedly.
#[test]
fn qa_FEE02_M14_release_returns_exactly_ratio_no_token_deduction() {
    let mut env = setup(6);
    let u = env.new_user(USER_TOKENS);
    for round in 0..3u8 {
        let mut got = vec![];
        for k in 0..3u8 {
            let r = env.new_randomness(None, None);
            got.push(env.capture(&u, &r, val(40 + round * 3 + k)));
        }
        for idx in got {
            let (_, tokens) = spend(&mut env, &u, |e| e.unwrap(&u, idx).expect("release"));
            assert_eq!(tokens, -(env.ratio_base as i128), "release #{idx}: user receives exactly ratio (spend = -ratio)");
            assert_escrow_exact(&env, "after release");
        }
    }
}

/// FEE-07 / M-06 (measured, tier-independent on f4761af): a re-roll never costs more than release + capture, in lamports (tx fees, rent deposits net of
/// refunds, SOL fees) AND in tokens. Measured end-to-end on the real vault (the crank's settle is paid by the crank).
/// Under the tiered model the expected gap is one tier fee plus one tx fee (see the per-tier tests below).
#[test]
fn qa_FEE07_M06_reroll_cost_le_release_plus_capture() {
    let mut env = setup(8);
    let u = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let x = env.capture(&u, &r, val(60));
    // path A: re-roll x
    let (la, ta) = spend(&mut env, &u, |e| {
        let r = e.new_randomness(None, None);
        let seq = e.request_reroll(&u, x, &r).expect("reroll");
        e.reveal(&r, val(61));
        e.settle(seq, val(61)).expect("settle");
    });
    let y = (0..8u32).find(|i| env.asset_owner(*i) == u.kp.pubkey()).unwrap();
    // path B: release y, then capture again
    let (lb, tb) = spend(&mut env, &u, |e| {
        e.unwrap(&u, y).expect("release");
        let r = e.new_randomness(None, None);
        e.capture(&u, &r, val(62));
    });
    println!("QA M-06: re-roll costs {la} lamports + {ta} tokens; release+capture costs {lb} lamports + {tb} tokens");
    assert!(la <= lb, "re-roll lamports {la} > release+capture {lb} (M-06 bypass)");
    assert!(ta <= tb, "re-roll tokens {ta} > release+capture {tb} (M-06 bypass)");
    assert_escrow_exact(&env, "end");
}

/// BRIEF 4:52 PM tier table (with the 4:55 PM 10k drop). (ratio whole tokens, flat SOL fee lamports).
pub const FEE_TIERS: [(u64, u64); 7] = [
    (50_000, 2_000_000),
    (100_000, 5_000_000),
    (200_000, 5_000_000),
    (500_000, 10_000_000),
    (1_000_000, 10_000_000),
    (2_500_000, 10_000_000),
    (5_000_000, 10_000_000),
];
pub const FEE_HARD_CAP_LAMPORTS: u64 = 10_000_000;
const TX_FEE: u64 = 5_000;
/// Measured first-mint cost range (rent + 0.0015 SOL Core create fee), qa/reports/2026-09-25-core-mint-cost.md.
const FIRST_MINT_MIN: u64 = 3_463_000; // min-short variant
const FIRST_MINT_MAX: u64 = 6_671_000; // royalty + attributes
const AHEAD: &str = "enable when hybrid_vault charges the tiered SOL fee (BRIEF 4:52 PM; uncommitted WT only)";

/// FEE-07 / M-06 per tier, SPEC MODEL (runs now, no program). Release is free (tx fee only). Re-roll = f + tx;
/// release + capture = tx + (f + tx). Holds for all 7 tiers when both draws land on already-minted indices.
/// LAZY CASE (5:13 PM): a draw landing on an unminted index adds the first-mint cost m (0.0035..0.0067 SOL) to that
/// request. Worst realized case: re-roll lands unminted (f + tx + m) while release + capture lands minted (f + 2tx):
/// re-roll costs MORE by m - tx. In expectation both paths draw from the same pool, so M-06 holds on average but NOT
/// per outcome -> flagged as design question QA-FEE-04 (printed below, not asserted).
#[test]
fn qa_FEE07_M06_tier_model_reroll_le_release_plus_capture() {
    let _ = AHEAD;
    for (r, f) in FEE_TIERS {
        assert!(f <= FEE_HARD_CAP_LAMPORTS, "tier {r}: {f} above 0.01 SOL cap");
        let reroll = f + TX_FEE;
        let release_capture = TX_FEE + (f + TX_FEE);
        assert!(reroll <= release_capture, "M-06 bypass at ratio {r}");
        for m in [FIRST_MINT_MIN, FIRST_MINT_MAX] {
            assert!(reroll + m <= release_capture + m, "M-06 both-unminted case at ratio {r}");
            let worst = (reroll + m) as i64 - release_capture as i64;
            println!("QA M-06 r={r}: re-roll {reroll} <= release+capture {release_capture}; LAZY worst case (re-roll hits unminted, capture hits minted, m={m}): re-roll costs {worst} lamports MORE (QA-FEE-04)");
        }
    }
    assert!(!FEE_TIERS.iter().any(|t| t.0 == 10_000), "10k dropped (BRIEF 4:55 PM)");
}

/// FEE-07 / M-06 per tier, ON-CHAIN, parameterized over all 7 ratios (10k dropped, so 7 not 8). For each tier:
/// launch with that ratio (hybrid_launch derives fee = tier), init + fill + open the vault, capture x; then measure
/// path A = re-roll x (request_reroll + reveal + settle, crank pays settle) vs path B = release y (free: tx only) +
/// capture, from the user's lamports (tx fees, SOL fee, first-mint cost, rent deposits net of refunds) and tokens.
/// Control the draws so both land on already-minted indices, then both on unminted: assert A <= B in lamports and
/// tokens; fee_recipient delta = 1 tier fee on each path (release pays none); escrow exact. Also record the mixed case
/// (A unminted, B minted) for QA-FEE-04.
#[test]
#[ignore = "enable when hybrid_vault charges the tiered SOL fee (BRIEF 4:52 PM; uncommitted WT only)"]
fn qa_FEE07_M06_per_tier_reroll_le_release_plus_capture_all_ratios() {
    for (r, f) in FEE_TIERS {
        todo!("setup_with_ratio({r}) on the tiered harness; expected fee {f}; see doc comment");
    }
}

/// FEE-06 / M-05 (F-06): capture and re-roll pin the fee destination to LaunchConfig.fee_recipient (release has no
/// fee destination at all). Steps: for each of capture / re-roll,
/// replace the destination with (a) the user's own wallet, (b) a random system account, (c) a PDA of another
/// program, (d) the vault PDA, (e) another launch's recipient -> each tx rejected (ConstraintAddress /
/// FeeRecipientMismatch); user lamports, tokens, escrow and pool unchanged; the honest op moves
/// exactly one tier fee.
#[test]
#[ignore = "enable when hybrid_vault charges the tiered SOL fee (BRIEF 4:52 PM; uncommitted WT only)"]
fn qa_FEE06_M05_fee_destination_substitution_rejected_on_every_op() {
    todo!("see doc comment")
}

/// FEE-08 / M-36 (resolved by removing the release fee): release takes NO fee account: the unwrap IDL has no account
/// whose name contains "fee" (f4761af still has the token `fee_escrow`, so ignored there) and no SOL moves except
/// the tx fee. Replaces the v0.6 FeeVault cases (obsolete).
#[test]
#[ignore = "enable when free release is committed (f4761af unwrap still takes the token fee_escrow)"]
fn qa_FEE08_M36_release_takes_no_fee_account() {
    let j = ix_json("unwrap");
    for a in j["accounts"].as_array().unwrap() {
        let n = a["name"].as_str().unwrap();
        assert!(!n.contains("fee"), "release takes fee account `{n}` (release is free, BRIEF 5:04 PM)");
    }
    assert!(!ix_names().iter().any(|n| n.contains("sweep")), "no sweep instruction (holding account removed)");
}

/// FEE-08b / M-36: a release whose payer holds ONLY the tx fee (5,000 lamports) succeeds, returns exactly ratio
/// tokens, and leaves the payer at 0 SOL. Runs now on f4761af (its release charges no SOL either).
#[test]
fn qa_FEE08_M36_release_with_payer_holding_only_tx_fee_succeeds() {
    let mut env = setup(4);
    let u = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    let idx = env.capture(&u, &r, val(80));
    let mut acct = env.svm.get_account(&u.kp.pubkey()).unwrap();
    acct.lamports = TX_FEE;
    env.svm.set_account(u.kp.pubkey(), acct).unwrap();
    let t0 = env.token_amount(&u.ata);
    env.unwrap(&u, idx).expect("release with only the tx fee must succeed (release is free)");
    assert_eq!(env.token_amount(&u.ata) - t0, env.ratio_base, "exactly ratio back");
    assert_eq!(env.svm.get_balance(&u.kp.pubkey()).unwrap_or(0), 0, "payer spent exactly the tx fee");
    assert_escrow_exact(&env, "after free release");
}

/// FEE-18 / M-06: re-roll fee == capture fee in every tier (one LaunchConfig.fee_lamports; vault econ copies it to
/// both). Steps per tier: capture -> fee_recipient +f; re-roll -> fee_recipient +f; release -> fee_recipient +0.
#[test]
#[ignore = "enable when hybrid_vault charges the tiered SOL fee (BRIEF 4:52 PM; uncommitted WT only)"]
fn qa_FEE18_reroll_fee_equals_capture_fee_per_tier() {
    for (r, f) in FEE_TIERS {
        todo!("setup_with_ratio({r}); assert capture delta == reroll delta == {f}, release delta == 0");
    }
}

/// FEE-09 per tier: for each of the 7 tiers run the INV-ESC random sequence; fee_recipient delta ==
/// (#captures + #re-rolls) x tier fee exactly (releases pay 0) (expired requests: fee kept, M-04); users' lamport deltas ==
/// -((captures + re-rolls) x tier fee + first-mint costs + tx fees + net rent); escrow invariant unchanged.
#[test]
#[ignore = "enable when hybrid_vault charges the tiered SOL fee (BRIEF 4:52 PM; uncommitted WT only)"]
fn qa_FEE09_per_tier_sol_fee_totals_equal_ops_times_tier_fee() {
    for (r, f) in FEE_TIERS {
        todo!("extend qa_M14_escrow_sequence::run with ratio {r} and a fee model at {f} lamports/op");
    }
}

/// FEE-03 / M-08: forged LaunchConfig injected into LiteSVM -> vault refuses (FeeAboveHardCap) for fee = 0.01 SOL + 1
/// lamport (10_000_001), for a fee that is not the ratio's tier value (e.g. 50k with 0.01 SOL, 5M with 0.002 SOL), and
/// for fee = 0; the vault copies fee + recipient at init; no vault instruction can change them.
#[test]
#[ignore = "enable when hybrid_vault charges the tiered SOL fee (BRIEF 4:52 PM; uncommitted WT only)"]
fn qa_FEE03_M08_vault_rejects_forged_config_off_tier_or_above_cap() {
    for n in ix_names() {
        assert!(!n.contains("set_") && !n.contains("update") && !n.contains("config"), "fee-editing instruction `{n}`");
    }
    let _cases = [(1_000_000u64, FEE_HARD_CAP_LAMPORTS + 1), (50_000, 10_000_000), (5_000_000, 2_000_000), (1_000_000, 0)];
    todo!("inject each forged LaunchConfig and assert init_vault fails with FeeAboveHardCap")
}

/// FEE-17 / M-04 expire rule (M-04 wins over DECISIONS N7): expire refunds the PRINCIPAL only (N tokens for a capture, the
/// handed-in NFT for a re-roll) plus the request/rand-lock rent; the SOL fee is NOT refunded (fee_recipient keeps it).
/// Steps: capture request with a randomness that never reveals; recommit MAX_RECOMMITS times; warp past deadline +
/// EXPIRE_GRACE_SLOTS; expire (permissionless caller) -> user tokens +ratio, user lamports + rent only (no fee),
/// fee_recipient unchanged, escrow exact. Same for a re-roll (NFT returned). Expire of a revealed request rejected.
/// Committed f4761af still refunds the (token) fee, so this is ignored there.
#[test]
#[ignore = "enable when the principal-only expire is committed (f4761af refunds the fee; WT fixed)"]
fn qa_M04_expire_refunds_principal_not_fee() {
    todo!("see doc comment")
}

/// M-16 ACCEPTED RISK (Barton tabled it 2026-09-25): the fee wallet's own re-rolls cost it only tx + VRF, because it
/// pays the fee to itself. Documentation only: this prints the insider's effective cost per tier and never fails.
#[test]
fn qa_M16_ACCEPTED_RISK_fee_wallet_rerolls_for_tx_cost_only() {
    for (r, f) in FEE_TIERS {
        println!("QA M-16 (accepted risk) r={r}: public re-roll {} lamports; fee wallet's own re-roll {} lamports (+VRF)", f + TX_FEE, TX_FEE);
    }
}

/// M-26 (pause removed, ADR-015): hybrid_vault has no pause / unpause / guardian instruction and no paused/guardian
/// field in any account. Replaces RR-14 and "release works while paused" (obsolete). Committed f4761af still has
/// pause/unpause + guardian fields, so this is ignored until the removal is committed.
#[test]
#[ignore = "enable when pause removal (ADR-015) is committed (f4761af still has pause/unpause)"]
fn qa_M26_no_pause_instruction_or_field_in_vault() {
    for n in ix_names() {
        let l = n.to_lowercase();
        assert!(!l.contains("pause") && !l.contains("guardian"), "pause-like instruction `{n}` exists");
    }
    let idl = idl();
    for t in idl["types"].as_array().unwrap() {
        if let Some(fields) = t["type"]["fields"].as_array() {
            for f in fields {
                let l = f["name"].as_str().unwrap_or("").to_lowercase();
                assert!(!l.contains("pause") && !l.contains("guardian"), "type {} has pause-like field `{l}`", t["name"]);
            }
        }
    }
}
