//! LiteSVM harness for hybrid_vault: real hybrid_launch + hybrid_vault SBF builds, the real Metaplex
//! Core binary (fixtures/mpl_core.so), and the TEST-ONLY mock Switchboard at the Switchboard devnet id.
#![allow(dead_code)]

pub use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::{AccountMeta, Instruction}, system_instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{
            get_associated_token_address, spl_associated_token_account::instruction::create_associated_token_account,
            ID as ATA_PROGRAM_ID,
        },
        token::{spl_token, Mint, TokenAccount, ID as SPL_TOKEN_ID},
    },
    hybrid_launch::{LaunchParams, FEE_DESTINATION_BURN},
    hybrid_vault::{
        error::VaultError, merkle, pool::{pool_account_size, PoolView}, selection, Request, Vault, MPL_CORE_ID,
        SLOT_HASHES_SYSVAR_ID, SWITCHBOARD_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

pub const TOKEN_2022_ID: Pubkey = anchor_lang::prelude::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const DECIMALS: u8 = 6;
pub const USER_TOKENS: u64 = 50_000_000 * 1_000_000; // 50M whole tokens each

pub fn pda(seeds: &[&[u8]], program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program).0
}

pub struct Env {
    pub svm: LiteSVM,
    pub creator: Keypair,
    pub guardian: Keypair,
    pub mint: Pubkey,
    pub launch_config: Pubkey,
    pub vault: Pubkey,
    pub vault_authority: Pubkey,
    pub randomness_authority: Pubkey,
    pub vault_tokens: Pubkey,
    pub fee_escrow: Pubkey,
    pub pool: Pubkey,
    pub collection: Pubkey,
    pub creator_ata: Pubkey,
    pub n: u32,
    pub ratio_base: u64,
    pub capture_fee: u64,
    pub reroll_fee: u64,
    pub names: Vec<(String, String)>,
    pub proofs: Vec<Vec<[u8; 32]>>,
    pub root: [u8; 32],
    pub sb_queue: Pubkey,
    pub sb_oracle: Pubkey,
}

pub struct User {
    pub kp: Keypair,
    pub ata: Pubkey,
}

impl Env {
    pub fn send(&mut self, ixs: &[Instruction], signers: &[&Keypair]) -> Result<(), String> {
        let payer = signers[0].pubkey();
        let msg = Message::new_with_blockhash(ixs, Some(&payer), &self.svm.latest_blockhash());
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).map_err(|e| e.to_string())?;
        let res = self.svm.send_transaction(tx).map(|_| ()).map_err(|e| format!("{:?} logs={:#?}", e.err, e.meta.logs));
        self.svm.expire_blockhash();
        res
    }

    pub fn warp(&mut self, slots: u64) {
        let now = self.svm.get_sysvar::<anchor_lang::prelude::Clock>().slot;
        self.svm.warp_to_slot(now + slots);
    }
    pub fn slot(&self) -> u64 {
        self.svm.get_sysvar::<anchor_lang::prelude::Clock>().slot
    }

    pub fn vault_state(&self) -> Vault {
        Vault::try_deserialize(&mut self.svm.get_account(&self.vault).unwrap().data.as_slice()).unwrap()
    }
    pub fn request_state(&self, seq: u64) -> Request {
        let r = self.request_pda(seq);
        Request::try_deserialize(&mut self.svm.get_account(&r).unwrap().data.as_slice()).unwrap()
    }
    pub fn token_amount(&self, acct: &Pubkey) -> u64 {
        TokenAccount::try_deserialize(&mut self.svm.get_account(acct).unwrap().data.as_slice()).unwrap().amount
    }
    pub fn supply(&self) -> u64 {
        Mint::try_deserialize(&mut self.svm.get_account(&self.mint).unwrap().data.as_slice()).unwrap().supply
    }
    pub fn asset_pda(&self, index: u32) -> Pubkey {
        pda(&[b"asset", self.vault.as_ref(), &index.to_le_bytes()], &hybrid_vault::ID)
    }
    pub fn request_pda(&self, seq: u64) -> Pubkey {
        pda(&[b"request", self.vault.as_ref(), &seq.to_le_bytes()], &hybrid_vault::ID)
    }
    pub fn rand_lock_pda(&self, randomness: &Pubkey) -> Pubkey {
        pda(&[b"rand_lock", randomness.as_ref()], &hybrid_vault::ID)
    }
    /// Owner of a Core asset (BaseAssetV1 layout: key u8, owner Pubkey).
    pub fn asset_owner(&self, index: u32) -> Pubkey {
        let d = self.svm.get_account(&self.asset_pda(index)).unwrap().data;
        Pubkey::new_from_array(d[1..33].try_into().unwrap())
    }

    pub fn new_user(&mut self, tokens: u64) -> User {
        let kp = Keypair::new();
        self.svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
        let ata = get_associated_token_address(&kp.pubkey(), &self.mint);
        let create = create_associated_token_account(&kp.pubkey(), &kp.pubkey(), &self.mint, &SPL_TOKEN_ID);
        self.send(&[create], &[&kp]).unwrap();
        if tokens > 0 {
            let t = spl_token::instruction::transfer(&SPL_TOKEN_ID, &self.creator_ata, &ata, &self.creator.pubkey(), &[], tokens).unwrap();
            let creator = self.creator.insecure_clone();
            self.send(&[t], &[&creator]).unwrap();
        }
        User { kp, ata }
    }

    /// A Switchboard-owned randomness account whose authority is this vault's randomness PDA
    /// (or `authority` if given, for attack tests).
    pub fn new_randomness(&mut self, authority: Option<Pubkey>, owner: Option<Pubkey>) -> Pubkey {
        let k = Pubkey::new_unique();
        let mut data = vec![0u8; mock_switchboard::ACCOUNT_SIZE];
        data[..8].copy_from_slice(&mock_switchboard::RANDOMNESS_DISCRIMINATOR);
        data[8..40].copy_from_slice(authority.unwrap_or(self.randomness_authority).as_ref());
        self.svm
            .set_account(
                k,
                Account { lamports: 10_000_000, data, owner: owner.unwrap_or(SWITCHBOARD_PROGRAM_ID), executable: false, rent_epoch: 0 },
            )
            .unwrap();
        k
    }

    /// Stand-in for the Switchboard oracle reveal (TEST-ONLY mock hook).
    pub fn reveal(&mut self, randomness: &Pubkey, value: [u8; 32]) {
        let mut data = mock_switchboard::MOCK_REVEAL_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&value);
        let ix = Instruction::new_with_bytes(SWITCHBOARD_PROGRAM_ID, &data, vec![AccountMeta::new(*randomness, false)]);
        let creator = self.creator.insecure_clone();
        self.send(&[ix], &[&creator]).unwrap();
    }

    pub fn request_capture_ix(&self, user: &User, randomness: &Pubkey, token_program: Pubkey) -> Instruction {
        let seq = self.vault_state().next_seq;
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::RequestCapture {}.data(),
            hybrid_vault::accounts::RequestCapture {
                user: user.kp.pubkey(),
                vault: self.vault,
                pool: self.pool,
                mint: self.mint,
                user_token: user.ata,
                vault_tokens: self.vault_tokens,
                fee_escrow: self.fee_escrow,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(randomness),
                randomness: *randomness,
                randomness_authority: self.randomness_authority,
                sb_queue: self.sb_queue,
                sb_oracle: self.sb_oracle,
                slot_hashes: SLOT_HASHES_SYSVAR_ID,
                switchboard_program: SWITCHBOARD_PROGRAM_ID,
                token_program,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    pub fn request_capture(&mut self, user: &User, randomness: &Pubkey) -> Result<u64, String> {
        self.warp(1);
        let seq = self.vault_state().next_seq;
        let ix = self.request_capture_ix(user, randomness, SPL_TOKEN_ID);
        let kp = user.kp.insecure_clone();
        self.send(&[ix], &[&kp]).map(|_| seq)
    }

    pub fn request_reroll_ix(&self, user: &User, index: u32, randomness: &Pubkey) -> Instruction {
        let seq = self.vault_state().next_seq;
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::RequestReroll { index }.data(),
            hybrid_vault::accounts::RequestReroll {
                user: user.kp.pubkey(),
                vault: self.vault,
                pool: self.pool,
                mint: self.mint,
                user_token: user.ata,
                vault_tokens: self.vault_tokens,
                fee_escrow: self.fee_escrow,
                vault_authority: self.vault_authority,
                asset: self.asset_pda(index),
                collection: self.collection,
                mpl_core_program: MPL_CORE_ID,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(randomness),
                randomness: *randomness,
                randomness_authority: self.randomness_authority,
                sb_queue: self.sb_queue,
                sb_oracle: self.sb_oracle,
                slot_hashes: SLOT_HASHES_SYSVAR_ID,
                switchboard_program: SWITCHBOARD_PROGRAM_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    pub fn request_reroll(&mut self, user: &User, index: u32, randomness: &Pubkey) -> Result<u64, String> {
        self.warp(1);
        let seq = self.vault_state().next_seq;
        let ix = self.request_reroll_ix(user, index, randomness);
        let kp = user.kp.insecure_clone();
        self.send(&[ix], &[&kp]).map(|_| seq)
    }

    /// Off-chain replay of the on-chain selection (same PoolView + selection code) for request `seq`.
    pub fn expected_pick(&self, seq: u64, value: [u8; 32]) -> u32 {
        let mut data = self.svm.get_account(&self.pool).unwrap().data;
        let mut pool = PoolView::load(&mut data, &self.vault).unwrap();
        pool.merge(seq, hybrid_vault::MAX_MERGE_PER_IX).unwrap();
        let r = selection::request_randomness(&value, &self.vault, seq);
        pool.pool_get(selection::uniform_below(&r, pool.pool_len()))
    }

    pub fn settle_ix(&self, seq: u64, asset: Pubkey, user: Pubkey, randomness: Pubkey, capture: bool) -> Instruction {
        let data = if capture {
            hybrid_vault::instruction::SettleCapture {}.data()
        } else {
            hybrid_vault::instruction::SettleReroll {}.data()
        };
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &data,
            hybrid_vault::accounts::Settle {
                settler: self.creator.pubkey(),
                vault: self.vault,
                pool: self.pool,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(&randomness),
                randomness,
                vault_authority: self.vault_authority,
                fee_escrow: self.fee_escrow,
                vault_tokens: self.vault_tokens,
                mint: self.mint,
                asset,
                collection: self.collection,
                user,
                mpl_core_program: MPL_CORE_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    /// Settle request `seq` with the correct (selected) asset. Returns the asset index received.
    pub fn settle(&mut self, seq: u64, value: [u8; 32]) -> Result<u32, String> {
        let req = self.request_state(seq);
        let pick = self.expected_pick(seq, value);
        let ix = self.settle_ix(seq, self.asset_pda(pick), req.user, req.randomness, req.kind == 0);
        let creator = self.creator.insecure_clone();
        self.send(&[ix], &[&creator]).map(|_| pick)
    }

    pub fn capture(&mut self, user: &User, randomness: &Pubkey, value: [u8; 32]) -> u32 {
        let seq = self.request_capture(user, randomness).expect("request_capture");
        self.reveal(randomness, value);
        self.settle(seq, value).expect("settle_capture")
    }

    pub fn unwrap_ix(&self, user: &User, index: u32) -> Instruction {
        self.unwrap_ix_with(user, self.asset_pda(index), index, self.vault_tokens)
    }

    pub fn unwrap_ix_with(&self, user: &User, asset: Pubkey, index: u32, vault_tokens: Pubkey) -> Instruction {
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::Unwrap { index }.data(),
            hybrid_vault::accounts::Unwrap {
                user: user.kp.pubkey(),
                vault: self.vault,
                pool: self.pool,
                mint: self.mint,
                vault_authority: self.vault_authority,
                vault_tokens,
                fee_escrow: self.fee_escrow,
                user_token: user.ata,
                asset,
                collection: self.collection,
                mpl_core_program: MPL_CORE_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    pub fn unwrap(&mut self, user: &User, index: u32) -> Result<(), String> {
        let ix = self.unwrap_ix(user, index);
        let kp = user.kp.insecure_clone();
        self.send(&[ix], &[&kp])
    }

    pub fn expire_ix(&self, seq: u64, caller: Pubkey) -> Instruction {
        let req = self.request_state(seq);
        let asset = if req.kind == 1 { Some(self.asset_pda(req.handed_in_index)) } else { None };
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::ExpireRequest {}.data(),
            hybrid_vault::accounts::ExpireRequest {
                caller,
                vault: self.vault,
                pool: self.pool,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(&req.randomness),
                randomness: req.randomness,
                user: req.user,
                user_token: req.user_token,
                mint: self.mint,
                vault_authority: self.vault_authority,
                vault_tokens: self.vault_tokens,
                fee_escrow: self.fee_escrow,
                asset,
                collection: self.collection,
                mpl_core_program: MPL_CORE_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    pub fn expire(&mut self, seq: u64) -> Result<(), String> {
        let ix = self.expire_ix(seq, self.creator.pubkey());
        let creator = self.creator.insecure_clone();
        self.send(&[ix], &[&creator])
    }

    pub fn guardian_ix(&self, pause: Option<u64>) -> Instruction {
        let data = match pause {
            Some(d) => hybrid_vault::instruction::Pause { duration_slots: d }.data(),
            None => hybrid_vault::instruction::Unpause {}.data(),
        };
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &data,
            hybrid_vault::accounts::GuardianAction { guardian: self.guardian.pubkey(), vault: self.vault }.to_account_metas(None),
        )
    }

    /// Solvency + conservation, checked from the outside after every step.
    pub fn assert_invariants(&self, users: &[&User]) {
        let v = self.vault_state();
        let vault_bal = self.token_amount(&self.vault_tokens);
        let escrow = self.token_amount(&self.fee_escrow);
        assert!(vault_bal >= v.ratio_base * (v.assets_outside + v.pending_captures), "vault insolvent");
        assert_eq!(escrow, v.pending_fee_total, "fee escrow == pending fees");
        let s0 = 1_000_000_000u64 * 10u64.pow(DECIMALS as u32);
        assert_eq!(self.supply(), s0 - v.total_burned, "supply == 1B - burned");
        let mut total = vault_bal + escrow + self.token_amount(&self.creator_ata);
        for u in users {
            total += self.token_amount(&u.ata);
        }
        assert_eq!(total, self.supply(), "every token accounted for");
        let in_vault = (0..self.n).filter(|i| self.asset_owner(*i) == self.vault_authority).count() as u64;
        assert_eq!(in_vault + v.assets_outside, self.n as u64, "NFT conservation");
    }
}

pub fn launch_params(n: u64) -> LaunchParams {
    LaunchParams {
        decimals: DECIMALS,
        ratio_whole_tokens: 1_000_000,
        collection_size: n,
        capture_fee_bps: 200,
        reroll_fee_bps: 200,
        fee_destination: FEE_DESTINATION_BURN,
    }
}

/// Full setup: launch (hybrid_launch), init_vault, deposit all `n` committed assets (sealed).
pub fn setup(n: u32) -> Env {
    let mut env = setup_unsealed(n);
    for i in 0..n {
        env.deposit(i).expect("deposit");
    }
    assert!(env.vault_state().sealed);
    env
}

pub fn setup_unsealed(n: u32) -> Env {
    let mut svm = LiteSVM::new();
    svm.add_program(hybrid_launch::id(), include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/hybrid_launch.so")))
        .unwrap();
    svm.add_program(hybrid_vault::id(), include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/hybrid_vault.so")))
        .unwrap();
    svm.add_program(MPL_CORE_ID, include_bytes!("../fixtures/mpl_core.so")).unwrap();
    svm.add_program(
        SWITCHBOARD_PROGRAM_ID,
        include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/mock_switchboard.so")),
    )
    .unwrap();
    svm.warp_to_slot(1_000);
    let creator = Keypair::new();
    svm.airdrop(&creator.pubkey(), 100_000_000_000).unwrap();

    // 1. hybrid_launch::launch
    let mint = Keypair::new();
    let launch_config = pda(&[b"launch_config", mint.pubkey().as_ref()], &hybrid_launch::ID);
    let mint_authority = pda(&[b"mint_authority", launch_config.as_ref()], &hybrid_launch::ID);
    let creator_ata = get_associated_token_address(&creator.pubkey(), &mint.pubkey());
    let launch_ix = Instruction::new_with_bytes(
        hybrid_launch::ID,
        &hybrid_launch::instruction::Launch { params: launch_params(n as u64) }.data(),
        hybrid_launch::accounts::Launch {
            creator: creator.pubkey(),
            mint: mint.pubkey(),
            launch_config,
            mint_authority,
            launch_destination_owner: creator.pubkey(),
            launch_destination: creator_ata,
            token_program: SPL_TOKEN_ID,
            associated_token_program: ATA_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let vault = pda(&[b"vault", launch_config.as_ref()], &hybrid_vault::ID);
    let names: Vec<(String, String)> = (0..n).map(|i| (format!("NFT #{i}"), format!("ipfs://traits/{i}.json"))).collect();
    let leaves: Vec<[u8; 32]> = names.iter().enumerate().map(|(i, (a, b))| merkle::leaf_hash(i as u32, a, b)).collect();
    let (root, proofs) = merkle::build(&leaves);

    let mut env = Env {
        svm,
        guardian: Keypair::new(),
        mint: mint.pubkey(),
        launch_config,
        vault,
        vault_authority: pda(&[b"vault_authority", vault.as_ref()], &hybrid_vault::ID),
        randomness_authority: pda(&[b"randomness_authority", vault.as_ref()], &hybrid_vault::ID),
        vault_tokens: pda(&[b"vault_tokens", vault.as_ref()], &hybrid_vault::ID),
        fee_escrow: pda(&[b"fee_escrow", vault.as_ref()], &hybrid_vault::ID),
        pool: Pubkey::default(),
        collection: pda(&[b"collection", vault.as_ref()], &hybrid_vault::ID),
        creator_ata,
        n,
        ratio_base: 1_000_000 * 10u64.pow(DECIMALS as u32),
        capture_fee: 1_000_000 * 10u64.pow(DECIMALS as u32) / 50,
        reroll_fee: 1_000_000 * 10u64.pow(DECIMALS as u32) / 50,
        names,
        proofs,
        root,
        sb_queue: Pubkey::new_unique(),
        sb_oracle: Pubkey::new_unique(),
        creator: creator.insecure_clone(),
    };
    env.send(&[launch_ix], &[&creator, &mint]).expect("launch");

    // 2. init_vault (pool pre-created top-level in the same tx)
    let pool_kp = Keypair::new();
    env.pool = pool_kp.pubkey();
    let init = env.init_vault_ix(env.launch_config, env.mint);
    let size = pool_account_size(n);
    let lamports = env.svm.minimum_balance_for_rent_exemption(size);
    let create_pool = system_instruction::create_account(&creator.pubkey(), &pool_kp.pubkey(), lamports, size as u64, &hybrid_vault::ID);
    env.send(&[create_pool, init], &[&creator, &pool_kp]).expect("init_vault");
    env
}

impl Env {
    pub fn init_vault_ix(&self, launch_config: Pubkey, mint: Pubkey) -> Instruction {
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::InitVault {
                params: hybrid_vault::InitVaultParams {
                    trait_root: self.root,
                    collection_name: "Test Collection".into(),
                    collection_uri: "ipfs://collection.json".into(),
                    guardian: self.guardian.pubkey(),
                },
            }
            .data(),
            hybrid_vault::accounts::InitVault {
                creator: self.creator.pubkey(),
                launch_config,
                mint,
                vault: self.vault,
                vault_authority: self.vault_authority,
                randomness_authority: self.randomness_authority,
                vault_tokens: self.vault_tokens,
                fee_escrow: self.fee_escrow,
                pool: self.pool,
                collection: self.collection,
                mpl_core_program: MPL_CORE_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    pub fn deposit_ix(&self, index: u32, name: &str, uri: &str, proof: Vec<[u8; 32]>) -> Instruction {
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::DepositAsset { index, name: name.into(), uri: uri.into(), proof }.data(),
            hybrid_vault::accounts::DepositAsset {
                creator: self.creator.pubkey(),
                vault: self.vault,
                pool: self.pool,
                vault_authority: self.vault_authority,
                collection: self.collection,
                asset: self.asset_pda(index),
                mpl_core_program: MPL_CORE_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    pub fn deposit(&mut self, index: u32) -> Result<(), String> {
        let (name, uri) = self.names[index as usize].clone();
        let ix = self.deposit_ix(index, &name, &uri, self.proofs[index as usize].clone());
        let creator = self.creator.insecure_clone();
        self.send(&[ix], &[&creator])
    }
}

pub fn expect_code(res: Result<impl std::fmt::Debug, String>, code: u32) {
    let err = res.expect_err("transaction must fail");
    assert!(err.contains(&format!("Custom({code})")), "expected Custom({code}), got: {err}");
}
pub fn expect_vault_err(res: Result<impl std::fmt::Debug, String>, e: VaultError) {
    expect_code(res, 6000 + e as u32);
}
pub fn anchor_code(e: anchor_lang::error::ErrorCode) -> u32 {
    e as u32
}
pub fn val(seed: u8) -> [u8; 32] {
    let mut v = [seed; 32];
    v[0] = seed.wrapping_mul(31).wrapping_add(7);
    v
}
