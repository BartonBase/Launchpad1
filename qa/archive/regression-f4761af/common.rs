//! Shared helpers for QA regression tests (tests/regression). Compiled next to the engineer's
//! vault harness (`crate::harness`) by `run-against-wip.sh`; nothing here is compiled on main.
#![allow(dead_code)]
use crate::harness::*;

pub fn idl() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../idl/hybrid_vault.json"))).unwrap()
}

pub fn ix_names() -> Vec<String> {
    idl()["instructions"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap().to_string()).collect()
}

pub fn ix_json(name: &str) -> serde_json::Value {
    idl()["instructions"].as_array().unwrap().iter().find(|i| i["name"] == name).cloned().unwrap_or_else(|| panic!("ix {name}"))
}

pub fn vault_bytes(env: &Env) -> Vec<u8> {
    env.svm.get_account(&env.vault).unwrap().data
}

/// Exact escrow rule (INV-ESC / FEE-05): vault_tokens holds ratio x (NFTs outside + pending captures + pending
/// re-rolls), no more no less. A pending re-roll's handed-in NFT is back in the vault (assets_outside -1) but its
/// ratio stays escrowed for the replacement NFT the user is owed.
pub fn assert_escrow_exact(env: &Env, ctx: &str) {
    let v = env.vault_state();
    let bal = env.token_amount(&env.vault_tokens);
    assert_eq!(
        bal,
        v.ratio_base * (v.assets_outside + v.pending_captures + v.pending_rerolls),
        "{ctx}: escrow == ratio x (outside + pending captures + pending re-rolls)"
    );
}

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Set a transaction's signer flags from a keypair list and send it with `payer` first.
pub fn send_as(env: &mut Env, ix: Instruction, signers: &[&Keypair]) -> Result<(), String> {
    env.send(&[ix], signers)
}

/// Replace one account in an instruction (by position of the old key).
pub fn swap_key(mut ix: Instruction, old: Pubkey, new: Pubkey) -> Instruction {
    let mut hit = false;
    for m in ix.accounts.iter_mut() {
        if m.pubkey == old {
            m.pubkey = new;
            hit = true;
        }
    }
    assert!(hit, "key {old} not in instruction");
    ix
}

/// A fresh SPL mint + funded ATA for `owner` ("junk" token for A-02).
pub fn junk_mint_and_ata(env: &mut Env, owner: &Keypair, amount: u64) -> (Pubkey, Pubkey) {
    let mint = Keypair::new();
    let rent = env.svm.minimum_balance_for_rent_exemption(82);
    let payer = owner.insecure_clone();
    let create = system_instruction::create_account(&payer.pubkey(), &mint.pubkey(), rent, 82, &SPL_TOKEN_ID);
    let init = spl_token::instruction::initialize_mint2(&SPL_TOKEN_ID, &mint.pubkey(), &payer.pubkey(), None, DECIMALS).unwrap();
    let ata = get_associated_token_address(&payer.pubkey(), &mint.pubkey());
    let cata = create_associated_token_account(&payer.pubkey(), &payer.pubkey(), &mint.pubkey(), &SPL_TOKEN_ID);
    let mt = spl_token::instruction::mint_to(&SPL_TOKEN_ID, &mint.pubkey(), &ata, &payer.pubkey(), &[], amount).unwrap();
    env.send(&[create, init, cata, mt], &[&payer, &mint]).expect("junk mint");
    (mint.pubkey(), ata)
}

/// A token account for `mint` owned by `owner` at a random (non-ATA) address.
pub fn token_account_for(env: &mut Env, payer: &Keypair, mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    let acct = Keypair::new();
    let rent = env.svm.minimum_balance_for_rent_exemption(165);
    let create = system_instruction::create_account(&payer.pubkey(), &acct.pubkey(), rent, 165, &SPL_TOKEN_ID);
    let init = spl_token::instruction::initialize_account3(&SPL_TOKEN_ID, &acct.pubkey(), mint, owner).unwrap();
    let p = payer.insecure_clone();
    env.send(&[create, init], &[&p, &acct]).expect("token account");
    acct.pubkey()
}
