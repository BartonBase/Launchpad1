//! End-to-end smoke test against `solana-test-validator` (started by `anchor test`).
//!
//! 1. Creates a *real* Token-2022 mint with the TransferFee extension
//!    (hand-built instructions, 1% fee, max fee 5_000 base units).
//! 2. Calls `fee_treasury::initialize` and `holder_lottery::initialize`
//!    against the deployed programs and checks the resulting PDAs.
//!
//! Throwaway keypairs only; funded from the local test-validator faucet.

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::{AccountMeta, Instruction}, system_instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    solana_commitment_config::CommitmentConfig,
    solana_keypair::Keypair,
    solana_rpc_client::rpc_client::RpcClient,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::{thread::sleep, time::Duration},
};

const TOKEN_2022: Pubkey = anchor_lang::prelude::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// Returns an RPC client only for a local validator; `None` => skip test.
fn local_rpc() -> Option<RpcClient> {
    let url = std::env::var("ANCHOR_PROVIDER_URL").ok()?;
    let host_ok = url.starts_with("http://127.0.0.1") || url.starts_with("http://localhost");
    assert!(host_ok, "SAFETY: localnet-smoke refuses non-local RPC URL {url}");
    Some(RpcClient::new_with_commitment(url, CommitmentConfig::confirmed()))
}

fn funded_keypair(rpc: &RpcClient) -> Keypair {
    let kp = Keypair::new();
    let sig = rpc.request_airdrop(&kp.pubkey(), 10_000_000_000).expect("local airdrop");
    for _ in 0..60 {
        if rpc.confirm_transaction(&sig).unwrap_or(false) {
            return kp;
        }
        sleep(Duration::from_millis(250));
    }
    panic!("local airdrop not confirmed");
}

fn send(rpc: &RpcClient, ixs: &[Instruction], signers: &[&Keypair]) {
    let bh = rpc.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(ixs, Some(&signers[0].pubkey()), signers, bh);
    rpc.send_and_confirm_transaction(&tx).expect("transaction failed");
}

/// Creates a Token-2022 mint with TransferFeeConfig (hand-encoded instructions).
fn create_transfer_fee_mint(rpc: &RpcClient, payer: &Keypair, fee_bps: u16, max_fee: u64) -> Pubkey {
    let mint = Keypair::new();
    // 165 (base account len padding) + 1 (AccountType) + 4 (TLV header) + 108 (TransferFeeConfig)
    let space: u64 = 165 + 1 + 4 + 108;
    let lamports = rpc.get_minimum_balance_for_rent_exemption(space as usize).unwrap();
    let create = system_instruction::create_account(&payer.pubkey(), &mint.pubkey(), lamports, space, &TOKEN_2022);

    // TokenInstruction::TransferFeeExtension (26) / InitializeTransferFeeConfig (0)
    let mut d = vec![26u8, 0u8];
    d.push(1); d.extend_from_slice(payer.pubkey().as_ref()); // transfer_fee_config_authority: Some
    d.push(1); d.extend_from_slice(payer.pubkey().as_ref()); // withdraw_withheld_authority: Some
    d.extend_from_slice(&fee_bps.to_le_bytes());
    d.extend_from_slice(&max_fee.to_le_bytes());
    let init_fee = Instruction::new_with_bytes(TOKEN_2022, &d, vec![AccountMeta::new(mint.pubkey(), false)]);

    // TokenInstruction::InitializeMint2 (20): decimals, mint_authority, freeze_authority: None
    let mut m = vec![20u8, 6u8];
    m.extend_from_slice(payer.pubkey().as_ref());
    m.push(0);
    let init_mint = Instruction::new_with_bytes(TOKEN_2022, &m, vec![AccountMeta::new(mint.pubkey(), false)]);

    send(rpc, &[create, init_fee, init_mint], &[payer, &mint]);
    let acct = rpc.get_account(&mint.pubkey()).unwrap();
    assert_eq!(acct.owner, TOKEN_2022);
    assert_eq!(acct.data.len() as u64, space);
    assert_eq!(acct.data[165], 1, "AccountType::Mint");
    assert_eq!(u16::from_le_bytes([acct.data[166], acct.data[167]]), 1, "ExtensionType::TransferFeeConfig");
    mint.pubkey()
}

#[test]
fn localnet_initialize_both_programs() {
    let Some(rpc) = local_rpc() else {
        eprintln!("ANCHOR_PROVIDER_URL not set: skipping localnet smoke test (run via `anchor test`)");
        return;
    };
    assert!(rpc.get_account(&fee_treasury::id()).map(|a| a.executable).unwrap_or(false), "fee_treasury not deployed");
    assert!(rpc.get_account(&holder_lottery::id()).map(|a| a.executable).unwrap_or(false), "holder_lottery not deployed");

    let admin = funded_keypair(&rpc);
    let mint = create_transfer_fee_mint(&rpc, &admin, 100, 5_000);

    // fee_treasury::initialize
    let ft = fee_treasury::id();
    let t_config = Pubkey::find_program_address(&[fee_treasury::TREASURY_CONFIG_SEED, mint.as_ref()], &ft).0;
    let t_vault = Pubkey::find_program_address(&[fee_treasury::VAULT_AUTHORITY_SEED, t_config.as_ref()], &ft).0;
    let params = fee_treasury::InitializeTreasuryParams {
        max_spend_per_purchase: 1_000_000,
        max_spend_per_window: 10_000_000,
        spend_window_seconds: 86_400,
    };
    let ix = Instruction::new_with_bytes(
        ft,
        &fee_treasury::instruction::Initialize { params }.data(),
        fee_treasury::accounts::Initialize {
            admin: admin.pubkey(),
            fee_mint: mint,
            config: t_config,
            vault_authority: t_vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&rpc, &[ix], &[&admin]);
    let data = rpc.get_account(&t_config).unwrap().data;
    let cfg = fee_treasury::TreasuryConfig::try_deserialize(&mut data.as_slice()).unwrap();
    assert_eq!(cfg.fee_mint, mint);
    assert_eq!(cfg.admin, admin.pubkey());

    // holder_lottery::initialize
    let hl = holder_lottery::id();
    let l_config = Pubkey::find_program_address(&[holder_lottery::LOTTERY_CONFIG_SEED, mint.as_ref()], &hl).0;
    let l_vault = Pubkey::find_program_address(&[holder_lottery::PRIZE_VAULT_SEED, l_config.as_ref()], &hl).0;
    let params = holder_lottery::InitializeLotteryParams {
        ticket_threshold: 1_000_000,
        min_holding_seconds: 7 * 86_400,
        randomness_provider: holder_lottery::RandomnessProvider::SwitchboardOnDemand,
    };
    let ix = Instruction::new_with_bytes(
        hl,
        &holder_lottery::instruction::Initialize { params }.data(),
        holder_lottery::accounts::Initialize {
            admin: admin.pubkey(),
            ticket_mint: mint,
            config: l_config,
            prize_vault: l_vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&rpc, &[ix], &[&admin]);
    let data = rpc.get_account(&l_config).unwrap().data;
    let cfg = holder_lottery::LotteryConfig::try_deserialize(&mut data.as_slice()).unwrap();
    assert_eq!(cfg.ticket_mint, mint);
    assert_eq!(cfg.ticket_threshold, 1_000_000);
    println!("localnet smoke OK: mint={mint} treasury_config={t_config} lottery_config={l_config}");
}
