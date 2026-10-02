//! GRAD-xx (TEST_PLAN §18, v0.7 LAZY MINTING): graduation gating for hybrid_vault. Nothing is pre-minted at graduation
//! (Barton 5:13 PM); converting opens on graduation, assets are minted lazily in settle (qa_M22_lazy_mint.rs).
//! OBSOLETE v0.7 (deleted): qa_M22_GRAD03_open_vault_rejected_until_fully_minted_and_graduated,
//! qa_M22_GRAD04_mint_assets_resumable_idempotent, qa_M10_GRAD05_mint_payer_is_proceeds_slice_only (batch pre-mint,
//! crank, "fully minted before opening", proceeds-funded mint).
//! Committed wip (f4761af) has NO graduation gate (QA-HV-01); the uncommitted WT gates on batch mint (superseded).
use crate::{common::*, harness::*};

fn has_ix(n: &str) -> bool {
    ix_names().iter().any(|x| x == n)
}

/// GRAD-01: an explicit open step (graduation gate) exists; the creator-deposit path is gone; the gate takes a
/// graduation proof account and does NOT require a full mint (lazy).
#[test]
#[ignore = "enable when the lazy graduation gate is committed"]
fn qa_M11_GRAD01_graduation_gate_exists_and_deposit_path_removed() {
    assert!(has_ix("open_vault"), "open_vault (graduation gate) must exist");
    assert!(!has_ix("deposit_asset"), "creator-deposit path must be gone");
    let j = ix_json("open_vault");
    assert!(j["accounts"].as_array().unwrap().iter().any(|a| a["name"].as_str().unwrap().contains("graduation")), "open_vault must take a graduation proof account");
}

/// GRAD-02: before graduation, request_capture / request_reroll / settle are rejected (VaultNotOpen or equivalent).
/// On the committed wip this is the gap QA-HV-01 (opens on seal).
#[test]
#[ignore = "enable when the graduation gate is committed (QA-HV-01)"]
fn qa_M10_GRAD02_capture_rejected_before_graduation() {
    let mut env = setup(4);
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness(None, None);
    assert!(env.request_capture(&alice, &r).is_err(), "capture before open_vault/graduation must fail");
    assert_escrow_exact(&env, "no tokens moved");
}
