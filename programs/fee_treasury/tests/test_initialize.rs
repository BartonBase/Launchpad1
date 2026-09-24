//! LiteSVM tests for `fee_treasury::initialize` (runs the real SBF binary
//! from `target/deploy`, so run `anchor build` first; `anchor test` does).

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    fee_treasury::{
        constants::{TOKEN_2022_PROGRAM_ID, TREASURY_CONFIG_SEED, VAULT_AUTHORITY_SEED},
        InitializeTreasuryParams, TreasuryConfig,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

/// Classic SPL Token program id (NOT allowed as fee mint owner).
const LEGACY_TOKEN_PROGRAM_ID: Pubkey =
    anchor_lang::prelude::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

const PARAMS: InitializeTreasuryParams = InitializeTreasuryParams {
    max_spend_per_purchase: 1_000,
    max_spend_per_window: 10_000,
    spend_window_seconds: 24 * 60 * 60,
};

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/fee_treasury.so"));
    svm.add_program(fee_treasury::id(), bytes).unwrap();
    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    (svm, admin)
}

/// Place a dummy account owned by `owner` (only the owner is checked by the stub).
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
    let pid = fee_treasury::id();
    let config = Pubkey::find_program_address(&[TREASURY_CONFIG_SEED, mint.as_ref()], &pid).0;
    let vault = Pubkey::find_program_address(&[VAULT_AUTHORITY_SEED, config.as_ref()], &pid).0;
    (config, vault)
}

fn init_ix(admin: &Pubkey, mint: &Pubkey, config: &Pubkey, vault: &Pubkey, params: InitializeTreasuryParams) -> Instruction {
    Instruction::new_with_bytes(
        fee_treasury::id(),
        &fee_treasury::instruction::Initialize { params }.data(),
        fee_treasury::accounts::Initialize {
            admin: *admin,
            fee_mint: *mint,
            config: *config,
            vault_authority: *vault,
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
    assert_eq!(acct.owner, fee_treasury::id());
    let state = TreasuryConfig::try_deserialize(&mut acct.data.as_slice()).unwrap();
    assert_eq!(state.admin, admin.pubkey());
    assert_eq!(state.fee_mint, mint);
    assert_eq!(state.max_spend_per_purchase, PARAMS.max_spend_per_purchase);
    assert_eq!(state.max_spend_per_window, PARAMS.max_spend_per_window);
    assert!(!state.paused);
    assert_eq!(state.spent_in_window, 0);
    let (_, vault_bump) = Pubkey::find_program_address(&[VAULT_AUTHORITY_SEED, config.as_ref()], &fee_treasury::id());
    assert_eq!(state.vault_authority_bump, vault_bump);
}

#[test]
fn reinitialize_fails() {
    let (mut svm, admin) = setup();
    let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
    let (config, vault) = pdas(&mint);
    send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &vault, PARAMS), &admin).unwrap();

    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();
    let res = send(&mut svm, init_ix(&attacker.pubkey(), &mint, &config, &vault, PARAMS), &attacker);
    assert!(res.is_err(), "second initialize must fail");
    let state = TreasuryConfig::try_deserialize(&mut svm.get_account(&config).unwrap().data.as_slice()).unwrap();
    assert_eq!(state.admin, admin.pubkey(), "admin must not be overwritten");
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
fn spoofed_vault_authority_is_rejected() {
    let (mut svm, admin) = setup();
    let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
    let (config, _) = pdas(&mint);
    let fake_vault = Pubkey::new_unique();
    let logs = send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &fake_vault, PARAMS), &admin).unwrap_err();
    assert!(logs.contains("ConstraintSeeds"), "{logs}");
}

#[test]
fn invalid_spending_params_are_rejected() {
    let (mut svm, admin) = setup();
    let cases = [
        (InitializeTreasuryParams { max_spend_per_purchase: 0, ..PARAMS }, "ZeroPurchaseCap"),
        (InitializeTreasuryParams { max_spend_per_purchase: 20_000, ..PARAMS }, "PurchaseCapExceedsWindowCap"),
        (InitializeTreasuryParams { spend_window_seconds: 59, ..PARAMS }, "InvalidSpendWindow"),
        (InitializeTreasuryParams { spend_window_seconds: i64::MAX, ..PARAMS }, "InvalidSpendWindow"),
    ];
    for (params, expected) in cases {
        let mint = fake_mint(&mut svm, TOKEN_2022_PROGRAM_ID);
        let (config, vault) = pdas(&mint);
        let logs = send(&mut svm, init_ix(&admin.pubkey(), &mint, &config, &vault, params), &admin).unwrap_err();
        assert!(logs.contains(expected), "expected {expected}: {logs}");
    }
}
