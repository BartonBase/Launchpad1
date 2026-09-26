//! Track A `hybrid_launch::launch` LiteSVM tests (QA layout: tests/track-a-hybrid/launch).
//! Runs the real SBF binary from target/deploy (run `./scripts/test.sh` or `anchor build` first).
//! Every test is named for the property or attack it proves.

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{get_associated_token_address_with_program_id, ID as ATA_PROGRAM_ID},
        token::{Mint, TokenAccount, ID as SPL_TOKEN_ID},
    },
    hybrid_launch::{
        error::LaunchError, LaunchConfig, LaunchParams, ALLOWED_RATIOS, LAUNCH_CONFIG_SEED, LAUNCH_VAULT_SEED,
        MAX_COLLECTION_SIZE, MAX_FEE_LAMPORTS, MINT_AUTHORITY_SEED, MIN_COLLECTION_SIZE, PLATFORM_FEE_RECIPIENT,
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
const ANCHOR_CONSTRAINT_ADDRESS: u32 = 2012;

fn params() -> LaunchParams {
    LaunchParams {
        decimals: 6,
        ratio_whole_tokens: 1_000_000,
        collection_size: 1_000,
        graduation_threshold_lamports: hybrid_launch::DEFAULT_GRADUATION_THRESHOLD_LAMPORTS,
    }
}

/// Raise large enough to fund any N <= 10k (636.25 SOL needed at 10%).
const BIG_RAISE: u64 = 700_000_000_000;

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
    /// The launch-vault PDA (owner of the destination). Attack tests overwrite it.
    launch_vault: Pubkey,
    dest: Pubkey,
    fee_recipient: Pubkey,
    token_program: Pubkey,
}

fn launch_vault_pda(mint: &Pubkey) -> (Pubkey, u8) {
    let pid = hybrid_launch::id();
    let config = Pubkey::find_program_address(&[LAUNCH_CONFIG_SEED, mint.as_ref()], &pid).0;
    Pubkey::find_program_address(&[LAUNCH_VAULT_SEED, mint.as_ref(), config.as_ref()], &pid)
}

fn derive(mint: &Pubkey) -> Accts {
    let pid = hybrid_launch::id();
    let config = Pubkey::find_program_address(&[LAUNCH_CONFIG_SEED, mint.as_ref()], &pid).0;
    let mint_authority = Pubkey::find_program_address(&[MINT_AUTHORITY_SEED, config.as_ref()], &pid).0;
    let launch_vault = launch_vault_pda(mint).0;
    let dest = get_associated_token_address_with_program_id(&launch_vault, mint, &SPL_TOKEN_ID);
    Accts { config, mint_authority, launch_vault, dest, fee_recipient: PLATFORM_FEE_RECIPIENT, token_program: SPL_TOKEN_ID }
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
            launch_vault: a.launch_vault,
            launch_destination: a.dest,
            fee_recipient: a.fee_recipient,
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
    let a = derive(&mint.pubkey());
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, p);
    send(&mut env, ix, &[&mint])
}

// ---------------------------------------------------------------- happy paths

#[test]
fn launch_mints_exactly_1b_revokes_mint_and_freeze_authority_and_records_immutable_config() {
    let mut env = setup();
    let mint = Keypair::new();
    let a = derive(&mint.pubkey());
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
    assert_eq!(dest.owner, a.launch_vault, "supply owned by the program-derived launch vault");
    assert!(dest.delegate.is_none() && dest.close_authority.is_none());

    let cfg_acct = env.svm.get_account(&a.config).unwrap();
    assert_eq!(cfg_acct.owner, hybrid_launch::id());
    let cfg = LaunchConfig::try_deserialize(&mut cfg_acct.data.as_slice()).unwrap();
    assert_eq!(cfg.mint, mint.pubkey());
    assert_eq!(cfg.ratio_whole_tokens, 1_000_000);
    assert_eq!(cfg.ratio_base, 1_000_000 * 10u64.pow(6));
    assert_eq!(cfg.collection_size, 1_000);
    assert_eq!(cfg.max_tokens_in_nft_form, cfg.total_supply_base);
    assert_eq!(cfg.fee_lamports, 10_000_000, "1M ratio tier: 0.01 SOL");
    assert_eq!(cfg.fee_recipient, PLATFORM_FEE_RECIPIENT);
    assert!(env.svm.get_account(&get_associated_token_address_with_program_id(&PLATFORM_FEE_RECIPIENT, &mint.pubkey(), &SPL_TOKEN_ID)).is_none(), "no token fee account exists");
    assert_eq!(cfg.launch_destination, a.dest);
    assert_eq!(cfg.creator, env.creator.pubkey());
    assert_eq!((cfg.launch_vault, cfg.launch_vault_bump), launch_vault_pda(&mint.pubkey()));
}

#[test]
fn launch_at_max_collection_size_cap_succeeds_and_above_is_rejected() {
    let p = LaunchParams { ratio_whole_tokens: 50_000, collection_size: MAX_COLLECTION_SIZE, graduation_threshold_lamports: BIG_RAISE, ..params() };
    try_params(p).expect("10k NFTs");
    expect_launch_error(try_params(LaunchParams { collection_size: MAX_COLLECTION_SIZE + 1, ..p }), LaunchError::CollectionAboveCap);
    // Lower bound 100.
    try_params(LaunchParams { collection_size: MIN_COLLECTION_SIZE, ..p }).expect("100 NFTs");
    expect_launch_error(try_params(LaunchParams { collection_size: MIN_COLLECTION_SIZE - 1, ..p }), LaunchError::CollectionBelowMinimum);
}

#[test]
fn config_has_no_mutation_or_close_instruction_in_idl() {
    // LaunchConfig is immutable by construction: the program exposes only two CREATE instructions
    // (`launch`, and `register_dbc_launch` for the DBC curve, ADR-014), both `init` on the config PDA.
    let idl: serde_json::Value =
        serde_json::from_str(include_str!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../idl/hybrid_launch.json"))).unwrap();
    let names: Vec<&str> = idl["instructions"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["launch", "register_dbc_launch"], "no update/close/admin instruction may exist");
    for ix in idl["instructions"].as_array().unwrap() {
        let lc = ix["accounts"].as_array().unwrap().iter().find(|a| a["name"] == "launch_config").expect("launch_config");
        assert!(lc["pda"].is_object() && lc["writable"] == true, "{}: launch_config must be the init'd PDA", ix["name"]);
    }
}

// ---------------------------------------------------------------- attacks / rejections

#[test]
fn attack_token_2022_program_substituted_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey());
    a.token_program = TOKEN_2022_ID;
    a.dest = get_associated_token_address_with_program_id(&a.launch_vault, &mint.pubkey(), &TOKEN_2022_ID);
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
    let a = derive(&mint.pubkey());
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_launch_error(send(&mut env, ix, &[&mint]), LaunchError::MintAccountInUse);
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
    let a = derive(&mint.pubkey());
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
    for r in [0u64, 1, 20_000, 30_000, 250_000, 2_000_000, 10_000_000, 1_000_000_000] {
        expect_launch_error(try_params(LaunchParams { ratio_whole_tokens: r, collection_size: 100, ..params() }), LaunchError::RatioNotAllowed);
    }
}

#[test]
fn attack_collection_size_times_ratio_above_1b_is_rejected() {
    expect_launch_error(try_params(LaunchParams { collection_size: 1_001, ..params() }), LaunchError::CollectionTooLargeForSupply);
    expect_launch_error(
        try_params(LaunchParams { ratio_whole_tokens: 5_000_000, collection_size: 201, ..params() }),
        LaunchError::CollectionTooLargeForSupply,
    );
}

#[test]
fn attack_collection_size_overflow_is_rejected() {
    // The size cap rejects it before any multiplication; the checked_mul path is unit-tested.
    expect_launch_error(try_params(LaunchParams { collection_size: u64::MAX, ..params() }), LaunchError::CollectionAboveCap);
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
fn fee_tier_is_derived_from_ratio_for_all_7_ratios_and_stored_immutably() {
    // Barton 2026-09-25 4:52 PM MT: 50k 0.002, 100k/200k 0.005, 500k..5M 0.01 SOL (10k dropped 4:55 PM MT).
    let tiers = [
        (50_000u64, 2_000_000u64),
        (100_000, 5_000_000),
        (200_000, 5_000_000),
        (500_000, 10_000_000),
        (1_000_000, 10_000_000),
        (2_500_000, 10_000_000),
        (5_000_000, 10_000_000),
    ];
    assert_eq!(tiers.map(|t| t.0), ALLOWED_RATIOS);
    for (r, fee) in tiers {
        for d in [0u8, 6, 9] {
            let mut env = setup();
            let mint = Keypair::new();
            let a = derive(&mint.pubkey());
            let p = LaunchParams { ratio_whole_tokens: r, decimals: d, collection_size: MIN_COLLECTION_SIZE, ..params() };
            let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, p);
            send(&mut env, ix, &[&mint]).unwrap_or_else(|e| panic!("r={r} d={d}: {e}"));
            let cfg = LaunchConfig::try_deserialize(&mut env.svm.get_account(&a.config).unwrap().data.as_slice()).unwrap();
            assert_eq!(cfg.fee_lamports, fee, "r={r} d={d}");
            assert!(cfg.fee_lamports <= MAX_FEE_LAMPORTS);
            assert_eq!(cfg.fee_recipient, PLATFORM_FEE_RECIPIENT);
        }
    }
}

#[test]
fn attack_fee_cap_cannot_be_exceeded_and_params_carry_no_fee_or_recipient() {
    // M-05 / M-07 / M-08: LaunchParams has no fee or recipient field at all (IDL assertion), so no
    // creator can pick a fee; the hard cap is 0.01 SOL and every tier is within it.
    assert_eq!(MAX_FEE_LAMPORTS, 10_000_000);
    let idl: serde_json::Value =
        serde_json::from_str(include_str!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../idl/hybrid_launch.json"))).unwrap();
    let t = idl["types"].as_array().unwrap().iter().find(|t| t["name"] == "LaunchParams").unwrap();
    let fields: Vec<&str> = t["type"]["fields"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(fields, vec!["decimals", "ratio_whole_tokens", "collection_size", "graduation_threshold_lamports"]);
    for r in ALLOWED_RATIOS {
        assert!(hybrid_launch::fee_for_ratio(r).unwrap() <= MAX_FEE_LAMPORTS);
    }
    // A ratio outside the table (the only way to reach another fee) is refused.
    expect_launch_error(try_params(LaunchParams { ratio_whole_tokens: 20_000, ..params() }), LaunchError::RatioNotAllowed);
}

#[test]
fn attack_launch_destination_not_the_derived_ata_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey());
    a.dest = Pubkey::new_unique();
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_launch_error(send(&mut env, ix, &[&mint]), LaunchError::InvalidLaunchDestination);
}

#[test]
fn attack_spoofed_mint_authority_pda_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey());
    a.mint_authority = env.creator.pubkey(); // attacker tries to keep mint authority
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_custom(send(&mut env, ix, &[&mint]), ANCHOR_CONSTRAINT_SEEDS);
}

#[test]
fn attack_spoofed_launch_config_pda_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let other = Keypair::new();
    let mut a = derive(&mint.pubkey());
    a.config = derive(&other.pubkey()).config;
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_custom(send(&mut env, ix, &[&mint]), ANCHOR_CONSTRAINT_SEEDS);
}

#[test]
fn attack_mint_keypair_not_signing_is_rejected() {
    let mut env = setup();
    let mint = Keypair::new();
    let a = derive(&mint.pubkey());
    let mut ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    for m in ix.accounts.iter_mut() {
        if m.pubkey == mint.pubkey() {
            m.is_signer = false;
        }
    }
    expect_custom(send(&mut env, ix, &[]), ANCHOR_ACCOUNT_NOT_SIGNER);
}

// ---------------------------------------------------------------- ratio set + min size (Barton 2026-09-24)

#[test]
fn every_ratio_launches_at_min_100_and_at_max_size() {
    // max = min(1B / ratio, MAX_COLLECTION_SIZE = 10k, pending Barton).
    let table = [
        (50_000u64, 10_000u64),
        (100_000, 10_000),
        (200_000, 5_000),
        (500_000, 2_000),
        (1_000_000, 1_000),
        (2_500_000, 400),
        (5_000_000, 200),
    ];
    assert_eq!(table.map(|t| t.0), ALLOWED_RATIOS);
    for (r, max) in table {
        try_params(LaunchParams { ratio_whole_tokens: r, collection_size: MIN_COLLECTION_SIZE, ..params() })
            .unwrap_or_else(|e| panic!("r={r} N=100: {e}"));
        try_params(LaunchParams { ratio_whole_tokens: r, collection_size: max, graduation_threshold_lamports: BIG_RAISE, ..params() })
            .unwrap_or_else(|e| panic!("r={r} N={max}: {e}"));
        let want = if max == MAX_COLLECTION_SIZE { LaunchError::CollectionAboveCap } else { LaunchError::CollectionTooLargeForSupply };
        expect_launch_error(try_params(LaunchParams { ratio_whole_tokens: r, collection_size: max + 1, graduation_threshold_lamports: BIG_RAISE, ..params() }), want);
    }
}

#[test]
fn attack_collection_below_minimum_100_is_rejected() {
    for n in [1u64, 50, 99] {
        for r in [50_000u64, 5_000_000] {
            expect_launch_error(
                try_params(LaunchParams { ratio_whole_tokens: r, collection_size: n, ..params() }),
                LaunchError::CollectionBelowMinimum,
            );
        }
    }
}

// ---------------------------------------------------------------- QA-HL-02: launch vault PDA

#[test]
fn attack_caller_chosen_launch_vault_owner_is_rejected() {
    // A creator tries to route the whole supply to their own wallet (or any non-PDA owner).
    for owner in ["creator", "random", "pda_of_other_program"] {
        let mut env = setup();
        let mint = Keypair::new();
        let mut a = derive(&mint.pubkey());
        a.launch_vault = match owner {
            "creator" => env.creator.pubkey(),
            "random" => Pubkey::new_unique(),
            _ => Pubkey::find_program_address(&[b"curve_vault"], &Pubkey::new_unique()).0,
        };
        a.dest = get_associated_token_address_with_program_id(&a.launch_vault, &mint.pubkey(), &SPL_TOKEN_ID);
        let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
        expect_custom(send(&mut env, ix, &[&mint]), ANCHOR_CONSTRAINT_SEEDS);
        assert!(env.svm.get_account(&mint.pubkey()).is_none(), "{owner}: atomic");
    }
}

#[test]
fn supply_sits_in_launch_vault_pda_with_no_withdraw_instruction() {
    let mut env = setup();
    let mint = Keypair::new();
    let a = derive(&mint.pubkey());
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    send(&mut env, ix, &[&mint]).expect("launch");
    assert!(!a.launch_vault.is_on_curve(), "launch vault is a PDA: no private key exists");
    let dest = TokenAccount::try_deserialize(&mut env.svm.get_account(&a.dest).unwrap().data.as_slice()).unwrap();
    assert_eq!(dest.owner, a.launch_vault);
    assert_eq!(dest.amount, 1_000_000_000 * 10u64.pow(6));
    // The PDA has no account data. hybrid_launch has two instructions (`launch`, `register_dbc_launch`),
    // and neither ever signs with the launch-vault seeds (register_dbc doesn't even take the account):
    // no instruction exists that can move these tokens.
    assert!(env.svm.get_account(&a.launch_vault).is_none());
    let idl: serde_json::Value =
        serde_json::from_str(include_str!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../idl/hybrid_launch.json"))).unwrap();
    let names: Vec<&str> = idl["instructions"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["launch", "register_dbc_launch"]);
    let reg = &idl["instructions"][1]["accounts"];
    assert!(reg.as_array().unwrap().iter().all(|a| a["name"] != "launch_vault" && a["name"] != "launch_destination"));
    // Creator can't move it with a plain SPL transfer either (not the owner, no delegate).
    let creator_ata = Pubkey::new_unique();
    let t = anchor_spl::token::spl_token::instruction::transfer(&SPL_TOKEN_ID, &a.dest, &creator_ata, &env.creator.pubkey(), &[], 1).unwrap();
    assert!(send(&mut env, t, &[]).is_err());
}

// ---------------------------------------------------------------- QA-HL-01: pre-funded mint address

#[test]
fn prefunded_mint_address_does_not_block_launch() {
    for lamports in [1u64, 1_000_000, 50_000_000] {
        // 1 lamport (top-up), ~rent-exempt, and above rent-exempt (no top-up needed)
        let mut env = setup();
        let mint = Keypair::new();
        let t = system_instruction::transfer(&env.creator.pubkey(), &mint.pubkey(), lamports);
        send(&mut env, t, &[]).expect("prefund");
        let a = derive(&mint.pubkey());
        let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
        send(&mut env, ix, &[&mint]).unwrap_or_else(|e| panic!("prefunded {lamports}: {e}"));
        let acct = env.svm.get_account(&mint.pubkey()).unwrap();
        assert_eq!(acct.owner, SPL_TOKEN_ID);
        assert_eq!(acct.data.len(), 82);
        assert!(acct.lamports >= env.svm.minimum_balance_for_rent_exemption(82), "rent-exempt after top-up");
        let m = Mint::try_deserialize(&mut acct.data.as_slice()).unwrap();
        assert!(m.mint_authority.is_none() && m.freeze_authority.is_none());
        assert_eq!(m.supply, 1_000_000_000 * 10u64.pow(6));
    }
}

#[test]
fn attack_mint_address_with_data_or_program_owner_is_rejected() {
    let cases: [(&str, Pubkey, usize); 3] = [
        ("system-owned with data", system_program::ID, 10),
        ("other program, no data", hybrid_launch::id(), 0),
        ("classic token program, no data", SPL_TOKEN_ID, 0),
    ];
    for (name, owner, len) in cases {
        let mut env = setup();
        let mint = Keypair::new();
        env.svm
            .set_account(mint.pubkey(), Account { lamports: 5_000_000, data: vec![0u8; len], owner, executable: false, rent_epoch: 0 })
            .unwrap();
        let a = derive(&mint.pubkey());
        let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
        let res = send(&mut env, ix, &[&mint]);
        assert!(res.is_err(), "{name}: must be rejected");
        expect_launch_error(res, LaunchError::MintAccountInUse);
    }
}

// ---------------------------------------------------------------- Barton 2026-09-25 4:55 PM MT decisions

#[test]
fn attack_10k_ratio_launch_is_rejected() {
    assert!(!ALLOWED_RATIOS.contains(&10_000));
    expect_launch_error(try_params(LaunchParams { ratio_whole_tokens: 10_000, collection_size: 100, ..params() }), LaunchError::RatioNotAllowed);
}

#[test]
fn lazy_mint_no_affordability_rule_any_n_up_to_10k_at_min_threshold() {
    // ADR-016 (Barton 5:13 PM MT): affordability dropped; 10k NFTs at the 10 SOL floor launch fine.
    let mut env = setup();
    for n in [1_336u64, 10_000] {
        let mint = Keypair::new();
        let a = derive(&mint.pubkey());
        let p = LaunchParams { ratio_whole_tokens: 50_000, collection_size: n, graduation_threshold_lamports: 10_000_000_000, ..params() };
        let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, p);
        send(&mut env, ix, &[&mint]).unwrap_or_else(|e| panic!("N={n}: {e}"));
        let cfg = LaunchConfig::try_deserialize(&mut env.svm.get_account(&a.config).unwrap().data.as_slice()).unwrap();
        assert_eq!((cfg.graduation_threshold_lamports, cfg.graduation_slice_pct), (10_000_000_000, 0));
    }
    // Below the floor of the build under test (10 SOL; 0.1 SOL with --features devnet-e2e).
    for t in [0u64, hybrid_launch::MIN_GRADUATION_THRESHOLD_LAMPORTS - 1, u64::MAX] {
        expect_launch_error(
            try_params(LaunchParams { graduation_threshold_lamports: t, collection_size: 100, ..params() }),
            LaunchError::GraduationThresholdOutOfRange,
        );
    }
}

#[test]
fn attack_fee_recipient_not_system_owned_or_executable_is_rejected() {
    for bad in ["program_owned", "executable", "has_data"] {
        let mut env = setup();
        let acct = match bad {
            "program_owned" => Account { lamports: 1_000_000, data: vec![], owner: Pubkey::new_unique(), executable: false, rent_epoch: 0 },
            // A real executable account (LiteSVM rejects executable accounts with non-program data).
            "executable" => env.svm.get_account(&hybrid_launch::ID).unwrap(),
            _ => Account { lamports: 10_000_000, data: vec![0; 8], owner: system_program::ID, executable: false, rent_epoch: 0 },
        };
        env.svm.set_account(PLATFORM_FEE_RECIPIENT, acct).unwrap();
        expect_launch_error(try_params_env(&mut env, params()), LaunchError::FeeRecipientInvalid);
    }
    // Not passing the platform recipient at all.
    let mut env = setup();
    let mint = Keypair::new();
    let mut a = derive(&mint.pubkey());
    a.fee_recipient = env.creator.pubkey();
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, params());
    expect_launch_error(send(&mut env, ix, &[&mint]), LaunchError::FeeRecipientInvalid);
    // Missing (never funded) recipient is fine: it is a system account and fees >= rent create it.
    let mut env = setup();
    assert!(env.svm.get_account(&PLATFORM_FEE_RECIPIENT).map(|a| a.lamports == 0).unwrap_or(true));
    try_params_env(&mut env, params()).expect("missing recipient ok");
}

fn try_params_env(env: &mut Env, p: LaunchParams) -> Result<(), String> {
    let mint = Keypair::new();
    let a = derive(&mint.pubkey());
    let ix = launch_ix(&env.creator.pubkey(), &mint.pubkey(), &a, p);
    send(env, ix, &[&mint]).map(|_| ())
}
