//! Track A `hybrid_launch::launch` LiteSVM tests (QA layout: tests/track-a-hybrid/launch).
//! Runs the real SBF binary from target/deploy (run `./scripts/test.sh` or `anchor build` first).
//! Every test is named for the property or attack it proves.

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{get_associated_token_address_with_program_id, ID as ATA_PROGRAM_ID},
        token::{Mint, TokenAccount, ID as SPL_TOKEN_ID},
    },
    hybrid_launch::{
        error::LaunchError, LaunchConfig, LaunchParams, FEE_DESTINATION_BURN, LAUNCH_CONFIG_SEED, MINT_AUTHORITY_SEED,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const TOKEN_2022_ID: Pubkey = anchor_lang::prelude::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const ANCHOR_INVALID_PROGRAM_ID: u32 = 3008;
const ANCHOR_ACCOUNT_NOT_SIGNER: u32 = 3010;
const ANCHOR_CONSTRAINT_SEEDS: u32 = 2006;

fn params() -> LaunchParams {
    LaunchParams {
        decimals: 6,
        ratio_whole_tokens: 1_000_000,
        collection_size: 1_000,
        capture_fee_bps: 200,
        reroll_fee_bps: 200,
        fee_destination: FEE_DESTINATION_BURN,
    }
}

struct Env {
    svm: LiteSVM,
    creator: Keypair,
}

fn setup() -> Env {
    let mut svm = LiteSVM::new();
    let so = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/hybrid_launch.so"));
    svm.add_program(hybrid_launch::id(), so).unwrap();
    let creator = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();
    Env { svm, creator }
}

struct Accts {
    config: Pubkey,
    mint_authority: Pubkey,
    dest_owner: Pubkey,
    dest: Pubkey,
    token_program: Pubkey,
}

fn derive(mint: &Pubkey, dest_owner: Pubkey) -> Accts {
    let pid = hybrid_launch::id();
    let config = Pubkey::find_program_address(&[LAUNCH_CONFIG_SEED, mint.as_ref()], &pid).0;
    let mint_authority = Pubkey::find_program_address(&[MINT_AUTHORITY_SEED, config.as_ref()], &pid).0;
    let dest = get_associated_token_address_with_program_id(&dest_owner, mint, &SPL_TOKEN_ID);
    Accts { config, mint_authority, dest_owner, dest, token_program: SPL_TOKEN_ID }
}

fn launch_ix(creator: &Pubkey, mint: &Pubkey, a: &Accts, p: LaunchParams) -> Instruction {
    Instruction::new_with_bytes(
        hybrid_launch::id(),
        &hybrid_launch::instruction::Launch { params: p }.data(),
        hybrid_launch::accounts::Launch {
            creator: *creator,
            mint: *mint,
            launch_config: a.config,
            mint_authority: a.mint_authority,
            launch_destination_owner: a.dest_owner,
            launch_destination: a.dest,
            token_program: a.token_program,
            associated_token_program: ATA_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn send(env: &mut Env, ix: Instruction, extra: &[&Keypair]) -> Result<(), String> {
    let mut signers: Vec<&Keypair> = vec![&env.creator];
    signers.extend_from_slice(extra);
    let msg = Message::new_with_blockhash(&[ix], Some(&env.creator.pubkey()), &env.svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers).map_err(|e| e.to_string())?;
    let res = env.svm.send_transaction(tx).map(|_| ()).map_err(|e| format!("{:?} logs={:?}", e.err, e.meta.logs));
    env.svm.expire_blockhash();
    res
}

fn expect_custom(res: Result<(), String>, code: u32) {
    let err = res.expect_err("transaction must fail");
    assert!(err.contains(&format!("Custom({code})")), "expected Custom({code}), got: {err}");
}

fn expect_launch_error(res: Result<(), String>, e: LaunchError) {
    expect_custom(res, 6000 + e as u32);
}

/// Try a launch with the given params; returns the result.
fn try_params(p: LaunchParams) -> Result<(), String> {
    let mut env = setup();
    let mint = Keypair::new();
    let a = derive(&mint.pubkey(), env.creator.pubkey());
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, p);
    send(&mut env, ix, &[&mint])
}

// ---------------------------------------------------------------- happy paths

#[test]
fn launch_mints_exactly_1b_revokes_mint_and_freeze_authority_and_records_immutable_config() {
    let mut env = setup();
    let mint = Keypair::new();
    let dest_owner = Keypair::new().pubkey();
    let a = derive(&mint.pubkey(), dest_owner);
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    send(&mut env, ix, &[&mint]).expect("launch");

    let mint_acct = env.svm.get_account(&mint.pubkey()).unwrap();
    assert_eq!(mint_acct.owner, SPL_TOKEN_ID, "classic SPL Token mint");
    let m = Mint::try_deserialize(&mut mint_acct.data.as_slice()).unwrap();
    assert_eq!(m.supply, 1_000_000_000 * 10u64.pow(6), "exactly 1B whole tokens");
    assert_eq!(m.decimals, 6);
    assert!(m.mint_authority.is_none(), "mint authority revoked");
    assert!(m.freeze_authority.is_none(), "no freeze authority");

    let dest = TokenAccount::try_deserialize(&mut env.svm.get_account(&a.dest).unwrap().data.as_slice()).unwrap();
    assert_eq!(dest.amount, m.supply, "whole supply at the launch destination");
    assert_eq!(dest.owner, dest_owner);

    let cfg_acct = env.svm.get_account(&a.config).unwrap();
    assert_eq!(cfg_acct.owner, hybrid_launch::id());
    let cfg = LaunchConfig::try_deserialize(&mut cfg_acct.data.as_slice()).unwrap();
    assert_eq!(cfg.mint, mint.pubkey());
    assert_eq!(cfg.ratio_whole_tokens, 1_000_000);
    assert_eq!(cfg.ratio_base, 1_000_000 * 10u64.pow(6));
    assert_eq!(cfg.collection_size, 1_000);
    assert_eq!(cfg.max_tokens_in_nft_form, cfg.total_supply_base);
    assert_eq!(cfg.capture_fee_amount, cfg.ratio_base / 50, "200 bps of ratio, exact");
    assert_eq!(cfg.reroll_fee_amount, cfg.ratio_base / 50);
    assert_eq!(cfg.fee_destination, FEE_DESTINATION_BURN);
    assert_eq!(cfg.launch_destination, a.dest);
    assert_eq!(cfg.creator, env.creator.pubkey());
}

#[test]
fn launch_at_max_collection_size_for_smallest_ratio_succeeds() {
    try_params(LaunchParams { ratio_whole_tokens: 10_000, collection_size: 100_000, ..params() }).expect("100k NFTs at 10k");
}

#[test]
fn config_has_no_mutation_or_close_instruction_in_idl() {
    // LaunchConfig is immutable by construction: the program exposes exactly one instruction.
    let idl: serde_json::Value =
        serde_json::from_str(include_str!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../idl/hybrid_launch.json"))).unwrap();
    let names: Vec<&str> = idl["instructions"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["launch"], "no update/close/admin instruction may exist");
}

// ---------------------------------------------------------------- attacks / rejections

#[test]
fn attack_token_2022_program_substituted_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey(), env.creator.pubkey());
    a.token_program = TOKEN_2022_ID;
    a.dest = get_associated_token_address_with_program_id(&a.dest_owner, &mint.pubkey(), &TOKEN_2022_ID);
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_custom(send(&mut env, ix, &[&mint]), ANCHOR_INVALID_PROGRAM_ID);
}

fn preexisting_mint_rejected(owner: Pubkey) {
    let mut env = setup();
    let mint = Keypair::new();
    let len = if owner == TOKEN_2022_ID { 234 } else { 82 };
    let mut data = vec![0u8; len];
    data[45] = 1; // is_initialized
    env.svm
        .set_account(mint.pubkey(), Account { lamports: 10_000_000, data, owner, executable: false, rent_epoch: 0 })
        .unwrap();
    let a = derive(&mint.pubkey(), env.creator.pubkey());
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    let err = send(&mut env, ix, &[&mint]).expect_err("existing mint must not be onboarded");
    assert!(err.contains("already in use") || err.contains("Custom(0)"), "unexpected error: {err}");
}

#[test]
fn attack_existing_token_2022_mint_cannot_be_onboarded() {
    preexisting_mint_rejected(TOKEN_2022_ID);
}

#[test]
fn attack_existing_classic_mint_cannot_be_onboarded() {
    preexisting_mint_rejected(SPL_TOKEN_ID);
}

#[test]
fn attack_relaunch_of_same_mint_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let a = derive(&mint.pubkey(), env.creator.pubkey());
    let creator = env.creator.pubkey();
    send(&mut env, launch_ix(&creator, &mint.pubkey(), &a, params()), &[&mint]).expect("first launch");
    let before = env.svm.get_account(&a.config).unwrap();
    let p2 = LaunchParams { ratio_whole_tokens: 10_000, collection_size: 5, ..params() };
    let res = send(&mut env, launch_ix(&creator, &mint.pubkey(), &a, p2), &[&mint]);
    assert!(res.is_err(), "second launch must fail");
    assert_eq!(env.svm.get_account(&a.config).unwrap().data, before.data, "config unchanged");
}

#[test]
fn attack_ratio_outside_allowed_set_is_rejected() {
    for r in [0u64, 1, 30_000, 250_000, 1_000_000_000] {
        expect_launch_error(try_params(LaunchParams { ratio_whole_tokens: r, collection_size: 1, ..params() }), LaunchError::RatioNotAllowed);
    }
}

#[test]
fn attack_collection_size_times_ratio_above_1b_is_rejected() {
    expect_launch_error(try_params(LaunchParams { collection_size: 1_001, ..params() }), LaunchError::CollectionTooLargeForSupply);
    expect_launch_error(
        try_params(LaunchParams { ratio_whole_tokens: 10_000, collection_size: 100_001, ..params() }),
        LaunchError::CollectionTooLargeForSupply,
    );
}

#[test]
fn attack_collection_size_overflow_is_rejected() {
    expect_launch_error(try_params(LaunchParams { collection_size: u64::MAX, ..params() }), LaunchError::CollectionTooLargeForSupply);
}

#[test]
fn attack_zero_collection_size_is_rejected() {
    expect_launch_error(try_params(LaunchParams { collection_size: 0, ..params() }), LaunchError::ZeroCollectionSize);
}

#[test]
fn attack_decimals_above_9_are_rejected() {
    expect_launch_error(try_params(LaunchParams { decimals: 10, ..params() }), LaunchError::InvalidDecimals);
}

#[test]
fn attack_fee_above_cap_is_rejected() {
    expect_launch_error(try_params(LaunchParams { capture_fee_bps: 1_001, ..params() }), LaunchError::FeeAboveCap);
    expect_launch_error(
        try_params(LaunchParams { capture_fee_bps: 1_001, reroll_fee_bps: 1_001, ..params() }),
        LaunchError::FeeAboveCap,
    );
}

#[test]
fn attack_capture_fee_below_reroll_fee_is_rejected() {
    expect_launch_error(
        try_params(LaunchParams { capture_fee_bps: 100, reroll_fee_bps: 200, ..params() }),
        LaunchError::CaptureFeeBelowRerollFee,
    );
}

#[test]
fn attack_fee_destination_other_than_burn_is_rejected() {
    for d in [1u8, 2, 255] {
        expect_launch_error(try_params(LaunchParams { fee_destination: d, ..params() }), LaunchError::FeeDestinationNotBurn);
    }
}

#[test]
fn attack_launch_destination_not_the_derived_ata_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey(), env.creator.pubkey());
    a.dest = Pubkey::new_unique();
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_launch_error(send(&mut env, ix, &[&mint]), LaunchError::InvalidLaunchDestination);
}

#[test]
fn attack_spoofed_mint_authority_pda_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey(), env.creator.pubkey());
    a.mint_authority = env.creator.pubkey(); // attacker tries to keep mint authority
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_custom(send(&mut env, ix, &[&mint]), ANCHOR_CONSTRAINT_SEEDS);
}

#[test]
fn attack_spoofed_launch_config_pda_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let other = Keypair::new();
    let mut a = derive(&mint.pubkey(), env.creator.pubkey());
    a.config = derive(&other.pubkey(), env.creator.pubkey()).config;
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_custom(send(&mut env, ix, &[&mint]), ANCHOR_CONSTRAINT_SEEDS);
}

#[test]
fn attack_mint_keypair_not_signing_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let a = derive(&mint.pubkey(), env.creator.pubkey());
    let mut ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    for m in ix.accounts.iter_mut() {
        if m.pubkey == mint.pubkey() {
            m.is_signer = false;
        }
    }
    expect_custom(send(&mut env, ix, &[]), ANCHOR_ACCOUNT_NOT_SIGNER);
}
