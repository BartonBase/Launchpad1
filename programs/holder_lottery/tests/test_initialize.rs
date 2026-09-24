//! LiteSVM tests for `holder_lottery::initialize` (runs the real SBF binary
//! from `target/deploy`, so run `anchor build` first; `anchor test` does).

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    holder_lottery::{
        constants::{LOTTERY_CONFIG_SEED, PRIZE_VAULT_SEED, TOKEN_2022_PROGRAM_ID},
        InitializeLotteryParams, LotteryConfig, RandomnessProvider,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const LEGACY_TOKEN_PROGRAM_ID: Pubkey =
    anchor_lang::prelude::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

const PARAMS: InitializeLotteryParams = InitializeLotteryParams {
    ticket_threshold: 1_000_000,
    min_holding_seconds: 7 * 24 * 60 * 60,
    randomness_provider: RandomnessProvider::SwitchboardOnDemand,
};

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/holder_lottery.so"));
    svm.add_program(holder_lottery::id(), bytes).unwrap();
    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    (svm, admin)
}

fn fake_mint(svm: &mut LiteSVM, owner: Pubkey) -> Pubkey {
    let mint = Pubkey::new_unique();
    let len = 82;
    svm.set_account(
        mint,
        Account {
            lamports: svm.minimum_balance_for_rent_exemption(len),
            data: vec![0u8; len],
            owner,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    mint
}

fn pdas(mint: &Pubkey) -> (Pubkey, Pubkey) {
    let pid = holder_lottery::id();
    let config = Pubkey::find_program_address(&[LOTTERY_CONFIG_SEED, mint.as_ref()], &pid).0;
    let vault = Pubkey::find_program_address(&[PRIZE_VAULT_SEED, config.as_ref()], &pid).0;
    (config, vault)
}

fn init_ix(admin: &Pubkey, mint: &Pubkey, config: &Pubkey, vault: &Pubkey, params: InitializeLotteryParams) -> Instruction {
    Instruction::new_with_bytes(
        holder_lottery::id(),
        &holder_lottery::instruction::Initialize { params }.data(),
        holder_lottery::accounts::Initialize {
            admin: *admin,
            ticket_mint: *mint,
            config: *config,
            prize_vault: *vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn send(svm: &mut LiteSVM, ix: Instruction, payer: &Keypair) -> Result<(), String> {
    svm.expire_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).map(|_| ()).map_err(|e| e.meta.logs.join("\n"))
}

#[test]
fn initialize_happy_path() {
    let (mut svm, admin) = setup();
    let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
    let (config, vault) = pdas(&mint);

    send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &vault, PARAMS), &admin).unwrap();

    let acct = svm.get_account(&config).unwrap();
    assert_eq!(acct.owner, holder_lottery::id());
    let state = LotteryConfig::try_deserialize(&mut acct.data.as_slice()).unwrap();
    assert_eq!(state.admin, admin.pubkey());
    assert_eq!(state.ticket_mint, mint);
    assert_eq!(state.ticket_threshold, PARAMS.ticket_threshold);
    assert_eq!(state.min_holding_seconds, PARAMS.min_holding_seconds);
    assert_eq!(state.randomness_provider, RandomnessProvider::SwitchboardOnDemand);
    assert_eq!(state.current_round, 0);
    assert!(!state.paused);
}

#[test]
fn reinitialize_fails() {
    let (mut svm, admin) = setup();
    let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
    let (config, vault) = pdas(&mint);
    send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &vault, PARAMS), &admin).unwrap();

    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();
    let hostile = InitializeLotteryParams { ticket_threshold: 1, ..PARAMS };
    assert!(send(&mut svm, init_ix(&attacker.pubkey(), &mint, &config, &vault, hostile), &attacker).is_err());
    let state = LotteryConfig::try_deserialize(&mut svm.get_account(&config).unwrap().data.as_slice()).unwrap();
    assert_eq!(state.admin, admin.pubkey());
    assert_eq!(state.ticket_threshold, PARAMS.ticket_threshold);
}

#[test]
fn legacy_token_program_mint_is_rejected() {
    let (mut svm, admin) = setup();
    let mint = fake_mint(&mut svm, LEGACY_TOKEN_PROGRAM_ID);
    let (config, vault) = pdas(&mint);
    let logs = send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &vault, PARAMS), &admin).unwrap_err();
    assert!(logs.contains("MintNotToken2022"), "{logs}");
}

#[test]
fn spoofed_prize_vault_is_rejected() {
    let (mut svm, admin) = setup();
    let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
    let (config, _) = pdas(&mint);
    let logs = send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &Pubkey::new_unique(), PARAMS), &admin).unwrap_err();
    assert!(logs.contains("ConstraintSeeds"), "{logs}");
}

#[test]
fn invalid_fairness_params_are_rejected() {
    let (mut svm, admin) = setup();
    let cases = [
        (InitializeLotteryParams { ticket_threshold: 0, ..PARAMS }, "ZeroTicketThreshold"),
        (InitializeLotteryParams { min_holding_seconds: 60, ..PARAMS }, "InvalidHoldingPeriod"),
        (InitializeLotteryParams { min_holding_seconds: -1, ..PARAMS }, "InvalidHoldingPeriod"),
        (InitializeLotteryParams { min_holding_seconds: i64::MAX, ..PARAMS }, "InvalidHoldingPeriod"),
    ];
    for (params, expected) in cases {
        let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
        let (config, vault) = pdas(&mint);
        let logs = send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &vault, params), &admin).unwrap_err();
        assert!(logs.contains(expected), "expected {expected}: {logs}");
    }
}
