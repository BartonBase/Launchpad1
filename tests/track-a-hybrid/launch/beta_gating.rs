//! ADR-021: the mainnet beta compiles out every instruction that is not shipped (native launches, Mode 3 burn,
//! shelved Modes 4/5). These tests run against whichever production build is in $TARGET/deploy:
//! - default build: every instruction is dispatched (none answers 101 InstructionFallbackNotFound);
//! - `--features mainnet-beta` (scripts/build-mainnet-beta.sh): the gated ones answer 101, the shipped ones don't,
//!   and the IDL lists exactly the shipped set.
//! A gated call must move nothing: the payer only pays the tx fee and no account is created.

use {
    anchor_lang::{prelude::Pubkey, solana_program::instruction::{AccountMeta, Instruction}},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const BETA: bool = cfg!(feature = "mainnet-beta");
const FALLBACK_NOT_FOUND: u32 = 101;

const LAUNCH_SHIPPED: &[&str] = &["register_dbc_launch", "register_plain_dbc"];
const LAUNCH_GATED: &[&str] = &[
    "launch", "launch_plain", "launch_burn", "register_burn_dbc", "launch_token22", "launch_raffle",
    "buy_inventory",
];
const VAULT_SHIPPED: &[&str] = &[
    "init_vault", "open_vault", "init_randomness", "request_capture", "request_reroll", "reveal_randomness",
    "recommit_randomness", "settle_capture", "settle_reroll", "unwrap", "expire_request", "expire_requests",
    "merge_incoming",
];
const VAULT_GATED: &[&str] = &[
    "init_permanent_vault", "open_permanent_vault", "wrap_permanent", "init_token22_vault", "wrap_token22",
    "harvest_tax", "claim_tax", "buyback", "init_raffle_vault", "snapshot_raffle", "init_raffle_randomness",
    "commit_raffle", "reveal_raffle", "skip_dead_nft", "retry_raffle", "settle_raffle",
];

fn artifact(rel: &str) -> Vec<u8> {
    let p = format!("{}/../{rel}", env!("CARGO_TARGET_TMPDIR"));
    std::fs::read(&p).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(hybrid_launch::id(), &artifact("deploy/hybrid_launch.so")).unwrap();
    svm.add_program(hybrid_vault::id(), &artifact("deploy/hybrid_vault.so")).unwrap();
    svm
}

fn disc(name: &str) -> [u8; 8] {
    let h = solana_sha256_hasher::hash(format!("global:{name}").as_bytes());
    h.to_bytes()[..8].try_into().unwrap()
}

/// Calls `name` with only a (writable, signing) payer and 64 zero bytes of args. Returns the custom error code
/// (every outcome here is an error: no instruction can succeed without its accounts).
fn call(svm: &mut LiteSVM, program: Pubkey, name: &str) -> u32 {
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    let mut data = disc(name).to_vec();
    data.extend_from_slice(&[0u8; 64]);
    let ix = Instruction { program_id: program, accounts: vec![AccountMeta::new(payer.pubkey(), true)], data };
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let before = svm.get_balance(&payer.pubkey()).unwrap();
    let err = svm.send_transaction(tx).expect_err("must fail without accounts");
    let after = svm.get_balance(&payer.pubkey()).unwrap();
    assert!(before - after <= 5_000, "{name}: only the tx fee may leave the payer");
    let e = format!("{:?}", err.err);
    let i = e.find("Custom(").unwrap_or_else(|| panic!("{name}: unexpected {e}")) + 7;
    e[i..].split(')').next().unwrap().parse().unwrap()
}

fn check(program: Pubkey, shipped: &[&str], gated: &[&str]) {
    let mut svm = svm();
    for n in shipped {
        assert_ne!(call(&mut svm, program, n), FALLBACK_NOT_FOUND, "{n} must be dispatched in every build");
    }
    for n in gated {
        let c = call(&mut svm, program, n);
        if BETA {
            assert_eq!(c, FALLBACK_NOT_FOUND, "{n} must be compiled out of the mainnet-beta build");
        } else {
            assert_ne!(c, FALLBACK_NOT_FOUND, "{n} must exist in the default (devnet/localnet) build");
        }
    }
}

#[test]
fn launch_program_dispatches_exactly_the_shipped_instructions_for_this_build() {
    check(hybrid_launch::id(), LAUNCH_SHIPPED, LAUNCH_GATED);
}

#[test]
fn vault_program_dispatches_exactly_the_shipped_instructions_for_this_build() {
    check(hybrid_vault::id(), VAULT_SHIPPED, VAULT_GATED);
}

#[test]
fn an_unknown_discriminator_is_rejected_by_both_programs() {
    let mut svm = svm();
    assert_eq!(call(&mut svm, hybrid_launch::id(), "no_such_ix"), FALLBACK_NOT_FOUND);
    assert_eq!(call(&mut svm, hybrid_vault::id(), "withdraw_all"), FALLBACK_NOT_FOUND);
}

#[test]
fn idl_lists_exactly_the_instructions_of_this_build() {
    for (prog, shipped, gated) in [
        ("hybrid_launch", LAUNCH_SHIPPED, LAUNCH_GATED),
        ("hybrid_vault", VAULT_SHIPPED, VAULT_GATED),
    ] {
        let idl: serde_json::Value =
            serde_json::from_slice(&artifact(&format!("idl/{prog}.json"))).unwrap();
        let mut names: Vec<String> = idl["instructions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["name"].as_str().unwrap().to_string())
            .collect();
        names.sort();
        let mut want: Vec<String> = shipped.iter().map(|s| s.to_string()).collect();
        if !BETA {
            want.extend(gated.iter().map(|s| s.to_string()));
        }
        want.sort();
        assert_eq!(names, want, "{prog} IDL instruction set");
    }
}
