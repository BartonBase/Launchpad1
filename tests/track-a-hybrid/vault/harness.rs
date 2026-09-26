//! LiteSVM harness for hybrid_vault: real hybrid_launch SBF build (target/deploy), the TEST build
//! of hybrid_vault (target/test-sbf, feature `test-mock-graduation`), the real Metaplex Core binary
//! (fixtures/mpl_core.so) and the TEST-ONLY mock Switchboard at the Switchboard program id.
//! Build the test binaries with `scripts/build-test-sbf.sh` (./scripts/test.sh does it).
#![allow(dead_code, unused_imports)]

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
    hybrid_launch::{LaunchParams, PLATFORM_FEE_RECIPIENT},
    hybrid_vault::{
        asset_source::LeafPreimage, error::VaultError, merkle, pool::{pool_account_size, PoolView}, randomness::RevealArgs, selection, Request, Vault,
        MPL_CORE_ID, SLOT_HASHES_SYSVAR_ID, SWITCHBOARD_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

pub const TOKEN_2022_ID: Pubkey = anchor_lang::prelude::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
/// Must match hybrid_vault::graduation::MOCK_GRADUATION_OWNER (only compiled with the TEST feature).
pub const MOCK_GRADUATION_OWNER: Pubkey = anchor_lang::prelude::pubkey!("grADWKnwMo64gj6DYGQEwuiEv9g5KkofdUT64WtWP2W");
pub const WRAPPED_SOL_MINT: Pubkey = anchor_lang::prelude::pubkey!("So11111111111111111111111111111111111111112");
pub const ALT_PROGRAM: Pubkey = anchor_lang::prelude::pubkey!("AddressLookupTab1e1111111111111111111111111");
pub const DECIMALS: u8 = 6;
pub const RATIO_WHOLE: u64 = 1_000_000;
pub const N: u32 = 100; // hybrid_launch MIN_COLLECTION_SIZE
pub const USER_TOKENS: u64 = 50_000_000 * 1_000_000; // 50M whole tokens each
/// Flat SOL fee for the 1M ratio tier (0.01 SOL), same on capture, release and re-roll.
pub const FEE: u64 = 10_000_000;
pub const TOTAL_SUPPLY: u64 = 1_000_000_000 * 1_000_000;

pub fn pda(seeds: &[&[u8]], program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program).0
}

/// Every address belonging to one launch + vault.
#[derive(Clone)]
pub struct Ids {
    pub mint: Pubkey,
    pub launch_config: Pubkey,
    pub launch_vault: Pubkey,
    pub launch_destination: Pubkey,
    pub fee_recipient: Pubkey,
    pub vault: Pubkey,
    pub vault_authority: Pubkey,
    pub randomness_authority: Pubkey,
    pub vault_tokens: Pubkey,
    pub pool: Pubkey,
    pub collection: Pubkey,
}

impl Ids {
    pub fn for_mint(mint: Pubkey) -> Self {
        let launch_config = pda(&[b"launch_config", mint.as_ref()], &hybrid_launch::ID);
        let launch_vault = pda(&[b"launch_vault", mint.as_ref(), launch_config.as_ref()], &hybrid_launch::ID);
        let vault = pda(&[b"vault", launch_config.as_ref()], &hybrid_vault::ID);
        Ids {
            mint,
            launch_config,
            launch_vault,
            launch_destination: get_associated_token_address(&launch_vault, &mint),
            fee_recipient: PLATFORM_FEE_RECIPIENT,
            vault,
            vault_authority: pda(&[b"vault_authority", vault.as_ref()], &hybrid_vault::ID),
            randomness_authority: pda(&[b"randomness_authority", vault.as_ref()], &hybrid_vault::ID),
            vault_tokens: pda(&[b"vault_tokens", vault.as_ref()], &hybrid_vault::ID),
            pool: Pubkey::default(),
            collection: pda(&[b"collection", vault.as_ref()], &hybrid_vault::ID),
        }
    }
}

pub struct Env {
    pub svm: LiteSVM,
    pub creator: Keypair,
    pub ids: Ids,
    pub n: u32,
    pub ratio_base: u64,
    pub fee: u64,
    pub schema_hash: [u8; 32],
    pub leaves: Vec<LeafPreimage>,
    pub proofs: Vec<Vec<[u8; 32]>>,
    pub root: [u8; 32],
    pub sb_queue: Pubkey,
    /// Oracles listed on the mock queue account (the program picks among them).
    pub sb_oracles: Vec<Pubkey>,
    pub graduation_proof: Pubkey,
    pub users: Vec<Pubkey>,
    /// Test hook: pre-fund the collection PDA (T-GRAD-01 griefing) before init_vault.
    pub prefund_collection: u64,
    /// REAL Switchboard On-Demand devnet program + dumped devnet accounts instead of the mock.
    pub real_sb: Option<RealSb>,
}

/// Dumped devnet Switchboard state (tests/track-a-hybrid/fixtures/switchboard-devnet).
#[derive(Clone)]
pub struct RealSb {
    pub state: Pubkey,
    pub fetched_slot: u64,
    pub unix_time: i64,
}

pub struct User {
    pub kp: Keypair,
    pub ata: Pubkey,
}
impl User {
    pub fn clone_user(&self) -> User {
        User { kp: self.kp.insecure_clone(), ata: self.ata }
    }
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
    pub fn send_as_creator(&mut self, ixs: &[Instruction]) -> Result<(), String> {
        let c = self.creator.insecure_clone();
        self.send(ixs, &[&c])
    }

    pub fn warp(&mut self, slots: u64) {
        let now = self.slot();
        self.svm.warp_to_slot(now + slots);
        if self.real_sb.is_some() {
            refresh_slot_hashes(&mut self.svm);
        }
    }
    pub fn slot(&self) -> u64 {
        self.svm.get_sysvar::<anchor_lang::prelude::Clock>().slot
    }
    pub fn vault_state(&self) -> Vault {
        Vault::try_deserialize(&mut self.svm.get_account(&self.ids.vault).unwrap().data.as_slice()).unwrap()
    }
    pub fn request_state(&self, seq: u64) -> Request {
        let r = self.request_pda(seq);
        Request::try_deserialize(&mut self.svm.get_account(&r).unwrap().data.as_slice()).unwrap()
    }
    pub fn exists(&self, k: &Pubkey) -> bool {
        self.svm.get_account(k).map(|a| a.lamports > 0).unwrap_or(false)
    }
    pub fn lamports(&self, k: &Pubkey) -> u64 {
        self.svm.get_account(k).map(|a| a.lamports).unwrap_or(0)
    }
    pub fn token_amount(&self, acct: &Pubkey) -> u64 {
        match self.svm.get_account(acct) {
            Some(a) if !a.data.is_empty() => TokenAccount::try_deserialize(&mut a.data.as_slice()).unwrap().amount,
            _ => 0,
        }
    }
    pub fn supply(&self) -> u64 {
        Mint::try_deserialize(&mut self.svm.get_account(&self.ids.mint).unwrap().data.as_slice()).unwrap().supply
    }
    pub fn asset_pda(&self, index: u32) -> Pubkey {
        pda(&[b"asset", self.ids.vault.as_ref(), &index.to_le_bytes()], &hybrid_vault::ID)
    }
    pub fn request_pda(&self, seq: u64) -> Pubkey {
        pda(&[b"request", self.ids.vault.as_ref(), &seq.to_le_bytes()], &hybrid_vault::ID)
    }
    pub fn rand_lock_pda(&self, randomness: &Pubkey) -> Pubkey {
        pda(&[b"rand_lock", randomness.as_ref()], &hybrid_vault::ID)
    }
    /// Owner of a Core asset (BaseAssetV1 layout: key u8, owner Pubkey).
    /// Owner of asset `index`, or Pubkey::default() if it hasn't been (lazily) minted yet.
    pub fn asset_owner(&self, index: u32) -> Pubkey {
        match self.svm.get_account(&self.asset_pda(index)) {
            Some(a) if a.data.len() >= 33 => Pubkey::new_from_array(a.data[1..33].try_into().unwrap()),
            _ => Pubkey::default(),
        }
    }

    /// Mint-escrow PDA of request `seq` (ADR-016).
    pub fn escrow_pda(&self, seq: u64) -> Pubkey {
        hybrid_vault::asset_source::mint_escrow_address(&self.ids.vault, seq).0
    }

    /// Pool minted bitmap bit for `index` (lazy mint).
    pub fn is_minted(&self, index: u32) -> bool {
        let mut data = self.svm.get_account(&self.ids.pool).unwrap().data;
        PoolView::load(&mut data, &self.ids.vault).unwrap().is_minted(index)
    }
    pub fn minted_popcount(&self) -> u32 {
        let mut data = self.svm.get_account(&self.ids.pool).unwrap().data;
        PoolView::load(&mut data, &self.ids.vault).unwrap().minted_popcount()
    }

    /// Lazy-mint args for `index` of the primary launch (committed leaf + proof).
    pub fn mint_args(&self, index: u32) -> hybrid_vault::asset_source::MintArgs {
        hybrid_vault::asset_source::MintArgs { leaf: self.leaves[index as usize].clone(), proof: self.proofs[index as usize].clone() }
    }

    /// Set a classic token account's amount directly (SPL layout: amount at bytes 64..72).
    pub fn set_token_amount(&mut self, acct: &Pubkey, amount: u64) {
        let mut a = self.svm.get_account(acct).unwrap();
        a.data[64..72].copy_from_slice(&amount.to_le_bytes());
        self.svm.set_account(*acct, a).unwrap();
    }

    /// New user with `tokens` MOVED out of the launch vault's ATA (supply and totals are preserved;
    /// stands in for buying on the curve / market, which is out of scope for these tests).
    pub fn new_user(&mut self, tokens: u64) -> User {
        let kp = Keypair::new();
        self.svm.airdrop(&kp.pubkey(), 100_000_000_000).unwrap();
        let ata = get_associated_token_address(&kp.pubkey(), &self.ids.mint);
        let create = create_associated_token_account(&kp.pubkey(), &kp.pubkey(), &self.ids.mint, &SPL_TOKEN_ID);
        self.send(&[create], &[&kp]).unwrap();
        if tokens > 0 {
            let src = self.token_amount(&self.ids.launch_destination);
            let dest = self.ids.launch_destination;
            self.set_token_amount(&dest, src.checked_sub(tokens).unwrap());
            self.set_token_amount(&ata, tokens);
        }
        self.users.push(ata);
        User { kp, ata }
    }

    /// Randomness account created through the vault's permissionless `init_randomness` (authority =
    /// this vault's randomness PDA, queue = the vault's queue).
    pub fn new_randomness(&mut self) -> Pubkey {
        let kp = Keypair::new();
        let ix = self.init_randomness_ix(&kp.pubkey(), self.sb_queue);
        let c = self.creator.insecure_clone();
        self.send(&[ix], &[&c, &kp]).expect("init_randomness");
        kp.pubkey()
    }

    pub fn init_randomness_ix(&self, randomness: &Pubkey, queue: Pubkey) -> Instruction {
        if let Some(r) = &self.real_sb {
            // Real Switchboard derivations (switchboard-on-demand 0.13.0): reward escrow = wSOL ATA of the
            // randomness account; LUT signer = PDA ["LutSigner", randomness]; LUT = ALT address(lut_signer, recent_slot).
            let recent_slot = self.slot() - 1;
            let lut_signer = pda(&[b"LutSigner", randomness.as_ref()], &SWITCHBOARD_PROGRAM_ID);
            let lut = pda(&[lut_signer.as_ref(), &recent_slot.to_le_bytes()], &ALT_PROGRAM);
            return Instruction::new_with_bytes(
                hybrid_vault::ID,
                &hybrid_vault::instruction::InitRandomness { recent_slot }.data(),
                hybrid_vault::accounts::InitRandomness {
                    payer: self.creator.pubkey(),
                    vault: self.ids.vault,
                    randomness: *randomness,
                    randomness_authority: self.ids.randomness_authority,
                    sb_reward_escrow: get_associated_token_address(randomness, &WRAPPED_SOL_MINT),
                    sb_queue: queue,
                    system_program: system_program::ID,
                    token_program: SPL_TOKEN_ID,
                    associated_token_program: ATA_PROGRAM_ID,
                    wrapped_sol_mint: WRAPPED_SOL_MINT,
                    sb_program_state: r.state,
                    sb_lut_signer: lut_signer,
                    sb_lut: lut,
                    address_lookup_table_program: ALT_PROGRAM,
                    switchboard_program: SWITCHBOARD_PROGRAM_ID,
                }
                .to_account_metas(None),
            );
        }
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::InitRandomness { recent_slot: self.slot() }.data(),
            hybrid_vault::accounts::InitRandomness {
                payer: self.creator.pubkey(),
                vault: self.ids.vault,
                randomness: *randomness,
                randomness_authority: self.ids.randomness_authority,
                sb_reward_escrow: Pubkey::new_unique(),
                sb_queue: queue,
                system_program: system_program::ID,
                token_program: SPL_TOKEN_ID,
                associated_token_program: ATA_PROGRAM_ID,
                wrapped_sol_mint: WRAPPED_SOL_MINT,
                sb_program_state: Pubkey::new_unique(),
                sb_lut_signer: Pubkey::new_unique(),
                sb_lut: Pubkey::new_unique(),
                address_lookup_table_program: ALT_PROGRAM,
                switchboard_program: SWITCHBOARD_PROGRAM_ID,
            }
            .to_account_metas(None),
        )
    }

    /// Raw randomness account (attack tests): arbitrary authority / owner / queue.
    pub fn raw_randomness(&mut self, authority: Pubkey, owner: Pubkey, queue: Pubkey) -> Pubkey {
        let k = Pubkey::new_unique();
        let mut data = vec![0u8; mock_switchboard::ACCOUNT_SIZE];
        data[..8].copy_from_slice(&mock_switchboard::RANDOMNESS_DISCRIMINATOR);
        data[8..40].copy_from_slice(authority.as_ref());
        data[40..72].copy_from_slice(queue.as_ref());
        self.svm.set_account(k, Account { lamports: 10_000_000, data, owner, executable: false, rent_epoch: 0 }).unwrap();
        k
    }

    pub fn reveal_ix(&self, seq: u64, payer: Pubkey, value: [u8; 32]) -> Instruction {
        let req = self.request_state(seq);
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::RevealRandomness { args: RevealArgs { signature: [7u8; 64], recovery_id: 0, value } }.data(),
            hybrid_vault::accounts::RevealRandomness {
                payer,
                vault: self.ids.vault,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(&req.randomness),
                randomness: req.randomness,
                randomness_authority: self.ids.randomness_authority,
                // The oracle of the CURRENT commit (a recommit picks a new one).
                sb_oracle: req.oracles[(req.commits as usize).saturating_sub(1)],
                sb_queue: self.sb_queue,
                sb_stats: match &self.real_sb {
                    Some(_) => pda(&[b"OracleRandomnessStats", req.oracles[(req.commits as usize).saturating_sub(1)].as_ref()], &SWITCHBOARD_PROGRAM_ID),
                    None => Pubkey::new_unique(),
                },
                slot_hashes: SLOT_HASHES_SYSVAR_ID,
                system_program: system_program::ID,
                sb_reward_escrow: match &self.real_sb {
                    Some(_) => get_associated_token_address(&req.randomness, &WRAPPED_SOL_MINT),
                    None => Pubkey::new_unique(),
                },
                token_program: SPL_TOKEN_ID,
                wrapped_sol_mint: WRAPPED_SOL_MINT,
                sb_program_state: self.real_sb.as_ref().map(|r| r.state).unwrap_or_else(Pubkey::new_unique),
                switchboard_program: SWITCHBOARD_PROGRAM_ID,
            }
            .to_account_metas(None),
        )
    }

    /// Oracle reveal submitted through the vault's permissionless `reveal_randomness` (by a crank).
    pub fn reveal(&mut self, seq: u64, value: [u8; 32]) -> Result<(), String> {
        self.warp(1);
        let crank = Keypair::new();
        self.svm.airdrop(&crank.pubkey(), 1_000_000_000).unwrap();
        let ix = self.reveal_ix(seq, crank.pubkey(), value);
        self.send(&[ix], &[&crank])
    }

    pub fn recommit_ix(&self, seq: u64, caller: Pubkey) -> Instruction {
        let req = self.request_state(seq);
        let o = self.select_oracle(seq, &req.oracles[..req.commits as usize]);
        self.recommit_ix_with_oracle(seq, caller, o)
    }

    /// Same selection the program makes (hybrid_vault::randomness::select_oracle_in).
    pub fn select_oracle(&self, seq: u64, used: &[Pubkey]) -> Pubkey {
        let q = self.svm.get_account(&self.sb_queue).unwrap();
        hybrid_vault::randomness::select_oracle_in(&q.data, &self.ids.vault, seq, used, &self.stale_oracles()).unwrap_or_default()
    }

    /// Mock Switchboard queue account with `oracles` in its oracle list (real QueueAccountData layout).
    pub fn install_queue(&mut self, oracles: &[Pubkey], curr_idx: u32) {
        let (size, keys_off, len_off, curr_off) = hybrid_vault::randomness::queue_layout();
        let mut data = vec![0u8; 8 + size];
        data[..8].copy_from_slice(&hybrid_vault::randomness::QUEUE_ACCOUNT_DISCRIMINATOR);
        for (i, o) in oracles.iter().enumerate() {
            data[8 + keys_off + 32 * i..8 + keys_off + 32 * (i + 1)].copy_from_slice(o.as_ref());
        }
        data[8 + len_off..8 + len_off + 4].copy_from_slice(&(oracles.len() as u32).to_le_bytes());
        data[8 + curr_off..8 + curr_off + 4].copy_from_slice(&curr_idx.to_le_bytes());
        let q = self.sb_queue;
        self.svm.set_account(q, Account { lamports: 1_000_000_000, data, owner: SWITCHBOARD_PROGRAM_ID, executable: false, rent_epoch: 0 }).unwrap();
        self.sb_oracles = oracles.to_vec();
        let now = self.now();
        for o in oracles {
            self.set_oracle_heartbeat(*o, now);
        }
    }

    pub fn now(&self) -> i64 {
        self.svm.get_sysvar::<anchor_lang::prelude::Clock>().unix_timestamp
    }

    /// Mock Switchboard oracle account (real OracleAccountData size, discriminator, last_heartbeat).
    pub fn set_oracle_heartbeat(&mut self, oracle: Pubkey, last_heartbeat: i64) {
        let (size, hb) = hybrid_vault::randomness::oracle_layout();
        let mut data = vec![0u8; 8 + size];
        data[..8].copy_from_slice(&hybrid_vault::randomness::ORACLE_ACCOUNT_DISCRIMINATOR);
        data[8 + hb..8 + hb + 8].copy_from_slice(&last_heartbeat.to_le_bytes());
        let lamports = self.svm.minimum_balance_for_rent_exemption(data.len());
        self.svm.set_account(oracle, Account { lamports, data, owner: SWITCHBOARD_PROGRAM_ID, executable: false, rent_epoch: 0 }).unwrap();
    }

    /// Oracles whose installed heartbeat is stale at the current clock (what a client would pass as proofs).
    pub fn stale_oracles(&self) -> Vec<Pubkey> {
        let now = self.now();
        let (_, hb) = hybrid_vault::randomness::oracle_layout();
        self.sb_oracles
            .iter()
            .filter(|o| {
                self.svm.get_account(o).is_some_and(|a| {
                    a.data.len() >= 8 + hb + 8
                        && !hybrid_vault::randomness::heartbeat_is_fresh(i64::from_le_bytes(a.data[8 + hb..8 + hb + 8].try_into().unwrap()), now)
                })
            })
            .copied()
            .collect()
    }

    /// Stale-proof remaining accounts for a commit.
    pub fn stale_proof_metas(&self) -> Vec<AccountMeta> {
        self.stale_oracles().into_iter().map(|k| AccountMeta::new_readonly(k, false)).collect()
    }

    pub fn recommit_ix_with_oracle(&self, seq: u64, caller: Pubkey, oracle: Pubkey) -> Instruction {
        let req = self.request_state(seq);
        let mut ix = Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::RecommitRandomness {}.data(),
            hybrid_vault::accounts::RecommitRandomness {
                caller,
                vault: self.ids.vault,
                request: self.request_pda(seq),
                randomness: req.randomness,
                randomness_authority: self.ids.randomness_authority,
                sb_queue: self.sb_queue,
                sb_oracle: oracle,
                slot_hashes: SLOT_HASHES_SYSVAR_ID,
                switchboard_program: SWITCHBOARD_PROGRAM_ID,
            }
            .to_account_metas(None),
        );
        ix.accounts.extend(self.stale_proof_metas());
        ix
    }

    pub fn recommit(&mut self, seq: u64, caller: &Keypair) -> Result<(), String> {
        let ix = self.recommit_ix(seq, caller.pubkey());
        self.send(&[ix], &[caller])
    }

    pub fn request_capture_ix(&self, user: &User, randomness: &Pubkey) -> hybrid_vault::accounts::RequestCapture {
        let seq = self.vault_state().next_seq;
        hybrid_vault::accounts::RequestCapture {
            user: user.kp.pubkey(),
            vault: self.ids.vault,
            launch_config: self.ids.launch_config,
            pool: self.ids.pool,
            mint: self.ids.mint,
            user_token: user.ata,
            vault_tokens: self.ids.vault_tokens,
            fee_recipient: self.ids.fee_recipient,
            request: self.request_pda(seq),
            mint_escrow: self.escrow_pda(seq),
            rand_lock: self.rand_lock_pda(randomness),
            randomness: *randomness,
            randomness_authority: self.ids.randomness_authority,
            sb_queue: self.sb_queue,
            sb_oracle: self.select_oracle(seq, &[]),
            slot_hashes: SLOT_HASHES_SYSVAR_ID,
            switchboard_program: SWITCHBOARD_PROGRAM_ID,
            token_program: SPL_TOKEN_ID,
            system_program: system_program::ID,
        }
    }

    pub fn ix_capture(accts: hybrid_vault::accounts::RequestCapture) -> Instruction {
        Instruction::new_with_bytes(hybrid_vault::ID, &hybrid_vault::instruction::RequestCapture {}.data(), accts.to_account_metas(None))
    }

    pub fn ix_capture_with_proofs(&self, accts: hybrid_vault::accounts::RequestCapture) -> Instruction {
        let mut ix = Self::ix_capture(accts);
        ix.accounts.extend(self.stale_proof_metas());
        ix
    }

    pub fn request_capture_with(&mut self, user: &User, accts: hybrid_vault::accounts::RequestCapture) -> Result<u64, String> {
        self.warp(1);
        let seq = self.vault_state().next_seq;
        let kp = user.kp.insecure_clone();
        let ix = self.ix_capture_with_proofs(accts);
        self.send(&[ix], &[&kp]).map(|_| seq)
    }

    pub fn request_capture(&mut self, user: &User, randomness: &Pubkey) -> Result<u64, String> {
        let a = self.request_capture_ix(user, randomness);
        self.request_capture_with(user, a)
    }

    pub fn request_reroll_accts(&self, user: &User, index: u32, randomness: &Pubkey) -> hybrid_vault::accounts::RequestReroll {
        let seq = self.vault_state().next_seq;
        hybrid_vault::accounts::RequestReroll {
            user: user.kp.pubkey(),
            vault: self.ids.vault,
            launch_config: self.ids.launch_config,
            pool: self.ids.pool,
            vault_tokens: self.ids.vault_tokens,
            fee_recipient: self.ids.fee_recipient,
            vault_authority: self.ids.vault_authority,
            asset: self.asset_pda(index),
            collection: self.ids.collection,
            mpl_core_program: MPL_CORE_ID,
            request: self.request_pda(seq),
            mint_escrow: self.escrow_pda(seq),
            rand_lock: self.rand_lock_pda(randomness),
            randomness: *randomness,
            randomness_authority: self.ids.randomness_authority,
            sb_queue: self.sb_queue,
            sb_oracle: self.select_oracle(seq, &[]),
            slot_hashes: SLOT_HASHES_SYSVAR_ID,
            switchboard_program: SWITCHBOARD_PROGRAM_ID,
            system_program: system_program::ID,
        }
    }

    pub fn request_reroll_ix(&self, a: hybrid_vault::accounts::RequestReroll, index: u32) -> Instruction {
        Instruction::new_with_bytes(hybrid_vault::ID, &hybrid_vault::instruction::RequestReroll { index }.data(), a.to_account_metas(None))
    }

    pub fn request_reroll(&mut self, user: &User, index: u32, randomness: &Pubkey) -> Result<u64, String> {
        self.warp(1);
        let seq = self.vault_state().next_seq;
        let a = self.request_reroll_accts(user, index, randomness);
        let mut ix = Instruction::new_with_bytes(hybrid_vault::ID, &hybrid_vault::instruction::RequestReroll { index }.data(), a.to_account_metas(None));
        ix.accounts.extend(self.stale_proof_metas());
        let kp = user.kp.insecure_clone();
        self.send(&[ix], &[&kp]).map(|_| seq)
    }

    /// Off-chain replay of the on-chain selection (same PoolView + selection code) for request `seq`.
    pub fn expected_pick(&self, seq: u64, value: [u8; 32]) -> u32 {
        let mut data = self.svm.get_account(&self.ids.pool).unwrap().data;
        let mut pool = PoolView::load(&mut data, &self.ids.vault).unwrap();
        pool.merge(seq, hybrid_vault::MAX_MERGE_PER_IX).unwrap();
        let r = selection::request_randomness(&value, &self.ids.vault, seq);
        pool.pool_get(selection::uniform_below(&r, pool.pool_len()))
    }

    /// Settle ix for `asset`; supplies the committed leaf + proof automatically when `asset` is a
    /// never-minted index of this vault (the lazy-mint path), None otherwise.
    pub fn settle_ix(&self, seq: u64, asset: Pubkey, settler: Pubkey) -> Instruction {
        let mint = (0..self.n).find(|i| self.asset_pda(*i) == asset).filter(|i| !self.is_minted(*i)).map(|i| self.mint_args(i));
        self.settle_ix_with(seq, asset, settler, mint)
    }

    pub fn settle_ix_with(&self, seq: u64, asset: Pubkey, settler: Pubkey, mint: Option<hybrid_vault::asset_source::MintArgs>) -> Instruction {
        let req = self.request_state(seq);
        let data = if req.kind == 0 { hybrid_vault::instruction::SettleCapture { mint }.data() } else { hybrid_vault::instruction::SettleReroll { mint }.data() };
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &data,
            hybrid_vault::accounts::Settle {
                settler,
                vault: self.ids.vault,
                launch_config: self.ids.launch_config,
                pool: self.ids.pool,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(&req.randomness),
                randomness: req.randomness,
                vault_authority: self.ids.vault_authority,
                mint_escrow: self.escrow_pda(seq),
                vault_tokens: self.ids.vault_tokens,
                asset,
                collection: self.ids.collection,
                user: req.user,
                mpl_core_program: MPL_CORE_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    /// Settle request `seq` with the correct (selected) asset, by an unrelated crank.
    pub fn settle(&mut self, seq: u64, value: [u8; 32]) -> Result<u32, String> {
        let pick = self.expected_pick(seq, value);
        let crank = Keypair::new();
        self.svm.airdrop(&crank.pubkey(), 1_000_000_000).unwrap();
        let ix = self.settle_ix(seq, self.asset_pda(pick), crank.pubkey());
        self.send(&[ix], &[&crank]).map(|_| pick)
    }

    pub fn capture(&mut self, user: &User, value: [u8; 32]) -> u32 {
        let r = self.new_randomness();
        let seq = self.request_capture(user, &r).expect("request_capture");
        self.reveal(seq, value).expect("reveal");
        self.settle(seq, value).expect("settle_capture")
    }

    pub fn unwrap_accts(&self, user: &User, index: u32) -> hybrid_vault::accounts::Unwrap {
        hybrid_vault::accounts::Unwrap {
            user: user.kp.pubkey(),
            vault: self.ids.vault,
            launch_config: self.ids.launch_config,
            pool: self.ids.pool,
            mint: self.ids.mint,
            vault_authority: self.ids.vault_authority,
            vault_tokens: self.ids.vault_tokens,
            user_token: user.ata,
            asset: self.asset_pda(index),
            collection: self.ids.collection,
            mpl_core_program: MPL_CORE_ID,
            token_program: SPL_TOKEN_ID,
            system_program: system_program::ID,
        }
    }

    pub fn unwrap_with(&mut self, user: &User, index: u32, a: hybrid_vault::accounts::Unwrap) -> Result<(), String> {
        let ix = Instruction::new_with_bytes(hybrid_vault::ID, &hybrid_vault::instruction::Unwrap { index }.data(), a.to_account_metas(None));
        let kp = user.kp.insecure_clone();
        self.send(&[ix], &[&kp])
    }

    pub fn unwrap(&mut self, user: &User, index: u32) -> Result<(), String> {
        let a = self.unwrap_accts(user, index);
        self.unwrap_with(user, index, a)
    }

    pub fn expire_ix(&self, seq: u64, caller: Pubkey, user_token: Pubkey) -> Instruction {
        let req = self.request_state(seq);
        let asset = if req.kind == 0 { self.asset_pda(0) } else { self.asset_pda(req.handed_in_index) };
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::ExpireRequest {}.data(),
            hybrid_vault::accounts::ExpireRequest {
                caller,
                vault: self.ids.vault,
                launch_config: self.ids.launch_config,
                pool: self.ids.pool,
                request: self.request_pda(seq),
                rand_lock: self.rand_lock_pda(&req.randomness),
                randomness: req.randomness,
                user: req.user,
                user_token,
                mint: self.ids.mint,
                vault_authority: self.ids.vault_authority,
                mint_escrow: self.escrow_pda(seq),
                vault_tokens: self.ids.vault_tokens,
                asset,
                collection: self.ids.collection,
                mpl_core_program: MPL_CORE_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    /// Per-request remaining accounts for `expire_requests` (stride 7, in queue order).
    pub fn expire_group(&self, seq: u64, user_token: Pubkey) -> Vec<AccountMeta> {
        let req = self.request_state(seq);
        let asset = if req.kind == 0 { self.asset_pda(0) } else { self.asset_pda(req.handed_in_index) };
        vec![
            AccountMeta::new(self.request_pda(seq), false),
            AccountMeta::new(self.rand_lock_pda(&req.randomness), false),
            AccountMeta::new_readonly(req.randomness, false),
            AccountMeta::new(req.user, false),
            AccountMeta::new(user_token, false),
            AccountMeta::new(self.escrow_pda(seq), false),
            AccountMeta::new(asset, false),
        ]
    }
    /// `expire_requests(count)` with explicit remaining accounts.
    pub fn expire_batch_ix_raw(&self, count: u8, caller: Pubkey, remaining: Vec<AccountMeta>) -> Instruction {
        let mut metas = hybrid_vault::accounts::ExpireRequests {
            caller,
            vault: self.ids.vault,
            launch_config: self.ids.launch_config,
            pool: self.ids.pool,
            mint: self.ids.mint,
            vault_authority: self.ids.vault_authority,
            vault_tokens: self.ids.vault_tokens,
            collection: self.ids.collection,
            mpl_core_program: MPL_CORE_ID,
            token_program: SPL_TOKEN_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None);
        metas.extend(remaining);
        Instruction::new_with_bytes(hybrid_vault::ID, &hybrid_vault::instruction::ExpireRequests { count }.data(), metas)
    }
    /// `expire_requests` over `heads` = [(seq, user_token)] in queue order.
    pub fn expire_batch_ix(&self, heads: &[(u64, Pubkey)], caller: Pubkey) -> Instruction {
        let rem = heads.iter().flat_map(|(s, t)| self.expire_group(*s, *t)).collect();
        self.expire_batch_ix_raw(heads.len() as u8, caller, rem)
    }
    /// Send with a 1.4M CU limit; returns compute units consumed.
    pub fn send_max_cu(&mut self, ixs: &[Instruction], signers: &[&Keypair]) -> Result<u64, String> {
        let mut data = vec![2u8];
        data.extend_from_slice(&1_400_000u32.to_le_bytes());
        let cb = Instruction::new_with_bytes(solana_sdk_ids_compute_budget(), &data, vec![]);
        let mut all = vec![cb];
        all.extend_from_slice(ixs);
        let payer = signers[0].pubkey();
        let msg = Message::new_with_blockhash(&all, Some(&payer), &self.svm.latest_blockhash());
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).map_err(|e| e.to_string())?;
        let res = self.svm.send_transaction(tx).map(|m| m.compute_units_consumed).map_err(|e| format!("{:?} logs={:#?}", e.err, e.meta.logs));
        self.svm.expire_blockhash();
        res
    }

    /// Create an address lookup table holding `addresses` (raw account: 56-byte LookupTableMeta,
    /// active, never deactivated, extended at slot 0) and return its key.
    pub fn create_alt(&mut self, addresses: &[Pubkey]) -> Pubkey {
        let key = Pubkey::new_unique();
        let mut data = vec![0u8; 56];
        data[0..4].copy_from_slice(&1u32.to_le_bytes());
        data[4..12].copy_from_slice(&u64::MAX.to_le_bytes());
        data[21] = 1;
        data[22..54].copy_from_slice(self.creator.pubkey().as_ref());
        for a in addresses {
            data.extend_from_slice(a.as_ref());
        }
        let lamports = self.svm.minimum_balance_for_rent_exemption(data.len());
        let alt_program: Pubkey = "AddressLookupTab1e1111111111111111111111111".parse().unwrap();
        self.svm
            .set_account(key, solana_account::Account { lamports, data, owner: alt_program, executable: false, rent_epoch: 0 })
            .unwrap();
        key
    }
    /// Send a v0 transaction using lookup table `alt` (+ a 1.4M CU limit); returns (CU, tx bytes).
    pub fn send_v0(&mut self, ixs: &[Instruction], signers: &[&Keypair], alt: Pubkey, alt_addrs: Vec<Pubkey>) -> Result<(u64, usize), String> {
        let mut data = vec![2u8];
        data.extend_from_slice(&1_400_000u32.to_le_bytes());
        let mut all = vec![Instruction::new_with_bytes(solana_sdk_ids_compute_budget(), &data, vec![])];
        all.extend_from_slice(ixs);
        let table = solana_message::AddressLookupTableAccount { key: alt, addresses: alt_addrs };
        let msg = solana_message::v0::Message::try_compile(&signers[0].pubkey(), &all, &[table], self.svm.latest_blockhash())
            .map_err(|e| e.to_string())?;
        let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), signers).map_err(|e| e.to_string())?;
        let size = bincode_len(&tx);
        let res = self.svm.send_transaction(tx).map(|m| (m.compute_units_consumed, size)).map_err(|e| format!("{:?} logs={:#?}", e.err, e.meta.logs));
        self.svm.expire_blockhash();
        res
    }

    /// TEST-ONLY mock graduation record for `mint` (owner = MOCK_GRADUATION_OWNER).
    pub fn set_graduation(&mut self, owner: Pubkey, mint: Pubkey, graduated: u8) -> Pubkey {
        let k = Pubkey::new_unique();
        let mut data = b"MOCKGRAD".to_vec();
        data.extend_from_slice(mint.as_ref());
        data.push(graduated);
        self.svm.set_account(k, Account { lamports: 1_000_000_000, data, owner, executable: false, rent_epoch: 0 }).unwrap();
        k
    }

    pub fn open_ix(&self, caller: Pubkey, proof: Pubkey) -> Instruction {
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::OpenVault {}.data(),
            hybrid_vault::accounts::OpenVault {
                caller,
                vault: self.ids.vault,
                launch_config: self.ids.launch_config,
                pool: self.ids.pool,
                vault_tokens: self.ids.vault_tokens,
                collection: self.ids.collection,
                graduation_proof: proof,
            }
            .to_account_metas(None),
        )
    }

    pub fn open(&mut self, proof: Pubkey) -> Result<(), String> {
        let anyone = Keypair::new();
        self.svm.airdrop(&anyone.pubkey(), 1_000_000_000).unwrap();
        let ix = self.open_ix(anyone.pubkey(), proof);
        self.send(&[ix], &[&anyone])
    }

    /// Solvency + conservation, checked from the outside after every step (mirrors invariants.rs,
    /// with EQUALITY because tests make no donations).
    pub fn assert_invariants(&self) {
        let v = self.vault_state();
        let vault_bal = self.token_amount(&self.ids.vault_tokens);
        assert_eq!(
            vault_bal,
            self.ratio_base * (v.assets_outside + v.pending_captures + v.pending_rerolls),
            "vault tokens == N * (outside + pending captures + pending re-rolls)"
        );
        assert_eq!(self.supply(), TOTAL_SUPPLY, "supply never changes (no burn)");
        let mut total = vault_bal + self.token_amount(&self.ids.launch_destination);
        for u in &self.users {
            total += self.token_amount(u);
        }
        assert_eq!(total, TOTAL_SUPPLY, "every token accounted for");
        let minted: Vec<u32> = (0..self.n).filter(|i| self.is_minted(*i)).collect();
        assert_eq!(minted.len() as u32, v.minted_count, "bitmap popcount == minted_count");
        assert!(v.minted_count <= self.n, "minted_count <= N");
        for i in 0..self.n {
            assert_eq!(self.is_minted(i), self.asset_owner(i) != Pubkey::default(), "asset {i} exists iff its bit is set");
        }
        let in_vault = minted.iter().filter(|i| self.asset_owner(**i) == self.ids.vault_authority).count() as u64;
        assert_eq!(in_vault + v.assets_outside, v.minted_count as u64, "NFT conservation");
        let mut data = self.svm.get_account(&self.ids.pool).unwrap().data;
        let pool = PoolView::load(&mut data, &self.ids.vault).unwrap();
        assert_eq!(
            pool.pool_len() as u64 + pool.incoming_len() as u64 + v.pending_rerolls + v.assets_outside,
            self.n as u64,
            "every index is drawable, returning, held, or outside"
        );
    }
}

pub fn launch_params(n: u64) -> LaunchParams {
    LaunchParams {
        decimals: DECIMALS,
        ratio_whole_tokens: RATIO_WHOLE,
        collection_size: n,
        graduation_threshold_lamports: hybrid_launch::DEFAULT_GRADUATION_THRESHOLD_LAMPORTS,
    }
}

/// LiteSVM with the REAL Switchboard devnet program and its dumped accounts.
fn new_svm_real_sb() -> (LiteSVM, RealSb, Pubkey, Vec<Pubkey>) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/switchboard-devnet");
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/accounts.json")).unwrap()).unwrap();
    let mut svm = LiteSVM::new();
    svm.add_program(hybrid_launch::id(), include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/hybrid_launch.so"))).unwrap();
    svm.add_program(hybrid_vault::id(), include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../test-sbf/hybrid_vault.so"))).unwrap();
    svm.add_program(MPL_CORE_ID, include_bytes!("../fixtures/mpl_core.so")).unwrap();
    svm.add_program(SWITCHBOARD_PROGRAM_ID, &std::fs::read(format!("{dir}/sb_devnet.so")).unwrap()).unwrap();
    for (k, a) in j["accounts"].as_object().unwrap() {
        let acct = Account {
            lamports: a["lamports"].as_u64().unwrap(),
            data: b64decode(a["data"].as_str().unwrap()),
            owner: a["owner"].as_str().unwrap().parse().unwrap(),
            executable: a["executable"].as_bool().unwrap(),
            rent_epoch: 0,
        };
        svm.set_account(k.parse().unwrap(), acct).unwrap();
    }
    let fetched_slot = j["fetched_slot"].as_u64().unwrap();
    let unix_time = j["unix_time"].as_i64().unwrap();
    svm.warp_to_slot(fetched_slot);
    refresh_slot_hashes(&mut svm);
    let mut clock = svm.get_sysvar::<anchor_lang::prelude::Clock>();
    clock.unix_timestamp = unix_time;
    svm.set_sysvar(&clock);
    let queue: Pubkey = j["queue"].as_str().unwrap().parse().unwrap();
    let oracles = j["oracles"].as_array().unwrap().iter().map(|o| o.as_str().unwrap().parse().unwrap()).collect();
    (svm, RealSb { state: j["state"].as_str().unwrap().parse().unwrap(), fetched_slot, unix_time }, queue, oracles)
}

/// LiteSVM doesn't maintain SlotHashes across warps; the real Switchboard (and the ALT program it
/// CPIs) need the recent slots there. Fill the last 512 slots with deterministic hashes.
pub fn refresh_slot_hashes(svm: &mut LiteSVM) {
    let slot = svm.get_sysvar::<anchor_lang::prelude::Clock>().slot;
    let n = 512u64.min(slot);
    // SlotHashes account layout: u64 count, then (u64 slot, [u8; 32] hash), newest first.
    let mut data = Vec::with_capacity(8 + 40 * 512);
    data.extend_from_slice(&n.to_le_bytes());
    for d in 1..=n {
        let s = slot - d;
        data.extend_from_slice(&s.to_le_bytes());
        let mut h = [0u8; 32];
        h[..8].copy_from_slice(&s.to_le_bytes());
        h[8] = 0xAB;
        data.extend_from_slice(&h);
    }
    data.resize(8 + 40 * 512, 0);
    let sysvar_owner: Pubkey = "Sysvar1111111111111111111111111111111111111".parse().unwrap();
    let lamports = svm.minimum_balance_for_rent_exemption(data.len());
    svm.set_account(SLOT_HASHES_SYSVAR_ID, Account { lamports, data, owner: sysvar_owner, executable: false, rent_epoch: 0 }).unwrap();
}

/// Minimal standard base64 decoder (fixtures only).
pub fn b64decode(s: &str) -> Vec<u8> {
    let val = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a' + 26) as u32,
            b'0'..=b'9' => (c - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => 0,
        }
    };
    let b: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(b.len() * 3 / 4);
    for chunk in b.chunks(4) {
        let n = chunk.iter().enumerate().fold(0u32, |acc, (i, c)| acc | (if *c == b'=' { 0 } else { val(*c) }) << (18 - 6 * i));
        let pad = chunk.iter().filter(|c| **c == b'=').count();
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    out
}

fn new_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(hybrid_launch::id(), include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/hybrid_launch.so"))).unwrap();
    svm.add_program(hybrid_vault::id(), include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../test-sbf/hybrid_vault.so"))).unwrap();
    svm.add_program(MPL_CORE_ID, include_bytes!("../fixtures/mpl_core.so")).unwrap();
    svm.add_program(SWITCHBOARD_PROGRAM_ID, include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../test-sbf/mock_switchboard.so"))).unwrap();
    svm.warp_to_slot(1_000);
    svm
}

impl Env {
    /// hybrid_launch::launch + init_vault for a fresh mint, in this SVM. Returns its ids.
    pub fn launch_and_init(&mut self, n: u32) -> Result<Ids, String> {
        let mint = Keypair::new();
        let ids = Ids::for_mint(mint.pubkey());
        let launch_ix = Instruction::new_with_bytes(
            hybrid_launch::ID,
            &hybrid_launch::instruction::Launch { params: launch_params(n as u64) }.data(),
            hybrid_launch::accounts::Launch {
                creator: self.creator.pubkey(),
                mint: mint.pubkey(),
                launch_config: ids.launch_config,
                mint_authority: pda(&[b"mint_authority", ids.launch_config.as_ref()], &hybrid_launch::ID),
                launch_vault: ids.launch_vault,
                launch_destination: ids.launch_destination,
                fee_recipient: PLATFORM_FEE_RECIPIENT,
                token_program: SPL_TOKEN_ID,
                associated_token_program: ATA_PROGRAM_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        let creator = self.creator.insecure_clone();
        self.send(&[launch_ix], &[&creator, &mint])?;
        self.init_vault_for(ids, n)
    }

    /// Pool account + init_vault for an existing LaunchConfig (native `launch` or `register_dbc_launch`).
    pub fn init_vault_for(&mut self, mut ids: Ids, n: u32) -> Result<Ids, String> {
        let creator = self.creator.insecure_clone();
        let pool_kp = Keypair::new();
        ids.pool = pool_kp.pubkey();
        // The committed leaves bind the LaunchConfig key, so they are built per launch.
        let (leaves, root, proofs) = build_leaves(&ids, n, &self.schema_hash);
        if self.leaves.is_empty() {
            (self.leaves, self.root, self.proofs) = (leaves, root, proofs);
        }
        let init = self.init_vault_ix(&ids, root);
        if self.prefund_collection > 0 {
            let griefer = Keypair::new();
            self.svm.airdrop(&griefer.pubkey(), 1_000_000_000).unwrap();
            let t = system_instruction::transfer(&griefer.pubkey(), &ids.collection, self.prefund_collection);
            self.send(&[t], &[&griefer]).expect("pre-fund collection PDA");
        }
        let size = pool_account_size(n);
        let lamports = self.svm.minimum_balance_for_rent_exemption(size);
        let create_pool = system_instruction::create_account(&creator.pubkey(), &pool_kp.pubkey(), lamports, size as u64, &hybrid_vault::ID);
        self.send(&[create_pool, init], &[&creator, &pool_kp])?;
        Ok(ids)
    }

    pub fn init_vault_ix(&self, ids: &Ids, root: [u8; 32]) -> Instruction {
        Instruction::new_with_bytes(
            hybrid_vault::ID,
            &hybrid_vault::instruction::InitVault {
                params: hybrid_vault::InitVaultParams {
                    trait_root: root,
                    trait_schema_hash: self.schema_hash,
                    collection_name: "Test Collection".into(),
                    collection_uri: "ipfs://bafycollection".into(),
                    sb_queue: self.sb_queue,
                },
            }
            .data(),
            hybrid_vault::accounts::InitVault {
                creator: self.creator.pubkey(),
                launch_config: ids.launch_config,
                mint: ids.mint,
                vault: ids.vault,
                vault_authority: ids.vault_authority,
                randomness_authority: ids.randomness_authority,
                vault_tokens: ids.vault_tokens,
                pool: ids.pool,
                collection: ids.collection,
                mpl_core_program: MPL_CORE_ID,
                token_program: SPL_TOKEN_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }
}

/// Launch + init_vault only (vault closed, nothing minted).
pub fn setup_closed(n: u32) -> Env {
    setup_closed_with(n, 0)
}

pub fn setup_closed_with(n: u32, prefund_collection: u64) -> Env {
    setup_closed_inner(n, prefund_collection, false)
}

/// Like `setup` but with the REAL Switchboard On-Demand devnet program and dumped devnet queue,
/// state, oracles and oracle stats (fixtures/switchboard-devnet); the clock is set to the dump time.
pub fn setup_real_sb(n: u32) -> Env {
    let mut env = setup_closed_inner(n, 0, true);
    let mint = env.ids.mint;
    env.graduation_proof = env.set_graduation(MOCK_GRADUATION_OWNER, mint, 1);
    let p = env.graduation_proof;
    env.open(p).expect("open_vault");
    env
}

fn setup_closed_inner(n: u32, prefund_collection: u64, real: bool) -> Env {
    let (svm, real_sb, queue, oracles) = if real {
        let (svm, r, q, o) = new_svm_real_sb();
        (svm, Some(r), q, o)
    } else {
        (new_svm(), None, Pubkey::new_unique(), vec![])
    };
    let creator = Keypair::new();
    let mut env = Env {
        svm,
        ids: Ids::for_mint(Pubkey::default()),
        n,
        ratio_base: RATIO_WHOLE * 10u64.pow(DECIMALS as u32),
        fee: FEE,
        schema_hash: [0x5c; 32],
        leaves: vec![],
        proofs: vec![],
        root: [0; 32],
        sb_queue: queue,
        sb_oracles: oracles,
        graduation_proof: Pubkey::default(),
        users: vec![],
        prefund_collection: 0,
        creator: creator.insecure_clone(),
        real_sb,
    };
    env.svm.airdrop(&creator.pubkey(), 1_000_000_000_000).unwrap();
    env.prefund_collection = prefund_collection;
    env.ids = env.launch_and_init(n).expect("launch + init_vault");
    env.prefund_collection = 0;
    if env.real_sb.is_none() {
        let oracles: Vec<Pubkey> = (0..6).map(|_| Pubkey::new_unique()).collect();
        env.install_queue(&oracles, 3);
    }
    env
}

/// Fully open vault (lazy mint: nothing minted), graduation verified (TEST-ONLY mock), opened.
pub fn setup(n: u32) -> Env {
    let mut env = setup_closed(n);
    let mint = env.ids.mint;
    env.graduation_proof = env.set_graduation(MOCK_GRADUATION_OWNER, mint, 1);
    let p = env.graduation_proof;
    env.open(p).expect("open_vault");
    assert!(env.vault_state().open);
    env
}

/// Leaf-v2 preimages (graduation-design §5.1) for every index of a launch, plus root and proofs.
pub fn leaf_preimage(i: u32) -> LeafPreimage {
    let mut tv = [0u16; 8];
    for (k, t) in tv.iter_mut().enumerate() {
        *t = ((i as usize * 7 + k * 3) % 11) as u16;
    }
    LeafPreimage {
        trait_values: tv,
        salt: [i as u8; 32],
        image_sha256: [(i as u8).wrapping_add(1); 32],
        json_sha256: [(i as u8).wrapping_add(2); 32],
        uri: format!("ipfs://bafytraits{i}"),
    }
}

pub fn leaf_of(ids: &Ids, i: u32, l: &LeafPreimage, schema: &[u8; 32]) -> [u8; 32] {
    let th = merkle::traits_hash(schema, &l.trait_values);
    let ah = merkle::art_hash(&l.salt, &l.image_sha256, &l.json_sha256, &l.uri);
    merkle::leaf_hash(&ids.launch_config.to_bytes(), i, &th, &ah)
}

pub fn build_leaves(ids: &Ids, n: u32, schema: &[u8; 32]) -> (Vec<LeafPreimage>, [u8; 32], Vec<Vec<[u8; 32]>>) {
    let pre: Vec<LeafPreimage> = (0..n).map(leaf_preimage).collect();
    let hashes: Vec<[u8; 32]> = pre.iter().enumerate().map(|(i, l)| leaf_of(ids, i as u32, l, schema)).collect();
    let (root, proofs) = merkle::build(&hashes);
    (pre, root, proofs)
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

pub fn solana_sdk_ids_compute_budget() -> Pubkey {
    "ComputeBudget111111111111111111111111111111".parse().unwrap()
}

/// Wire size of a versioned transaction (signatures + message), without a bincode dependency.
pub fn bincode_len(tx: &VersionedTransaction) -> usize {
    let n = tx.signatures.len();
    let short_vec = if n < 0x80 { 1 } else { 2 };
    short_vec + 64 * n + tx.message.serialize().len()
}


// ---------------------------------------------------------------------------------------------------
// Meteora DBC (ADR-014): the REAL DBC program (mainnet dump) + Token Metadata on LiteSVM, with a real
// devnet PoolConfig as the template (fixtures/dbc). DBC itself creates the pool, the mint and the
// base vault; hybrid_launch::register_dbc_launch verifies them.
// ---------------------------------------------------------------------------------------------------

pub const DBC_ID: Pubkey = hybrid_launch::dbc::DBC_PROGRAM_ID;
pub const TOKEN_METADATA_ID: Pubkey = anchor_lang::prelude::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
/// Real devnet DBC config (SOL quote, SPL, 6 decimals, fixed 1B supply with pre == post, Immutable).
pub const DBC_DEVNET_CONFIG: &str = "5L1MfYm4yqPySiVddKugruYoGyN6an6viSkHzL7MK1Y";
/// Real devnet DBC pool that has migrated (is_migrated 1, progress CreatedPool).
pub const DBC_DEVNET_MIGRATED_POOL: &str = "DGtaRQ9EbxPT9mFYDikyWcsVVNJKoFTA4rzDk9gB4at3";

#[derive(Clone, Copy, Debug)]
pub struct DbcIds {
    pub config: Pubkey,
    pub pool: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub metadata: Pubkey,
    pub buffer_authority: Pubkey,
    pub buffer_tokens: Pubkey,
}

pub fn dbc_fixture_account(key: &str) -> Account {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/dbc");
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/devnet_samples.json")).unwrap()).unwrap();
    let a = &j[key];
    Account {
        lamports: a["lamports"].as_u64().unwrap(),
        data: b64decode(a["data"].as_str().unwrap()),
        owner: a["owner"].as_str().unwrap().parse().unwrap(),
        executable: false,
        rent_epoch: 0,
    }
}

/// The devnet config with `leftover_receiver` and the migration threshold patched. Everything else
/// (fees, curve, fixed supply, decimals, token type, authority option) is the real devnet config.
pub fn dbc_config_account(leftover_receiver: Pubkey, threshold: u64) -> Account {
    use hybrid_launch::dbc::*;
    let mut a = dbc_fixture_account(DBC_DEVNET_CONFIG);
    a.data[CFG_OFF_LEFTOVER_RECEIVER..CFG_OFF_LEFTOVER_RECEIVER + 32].copy_from_slice(leftover_receiver.as_ref());
    a.data[CFG_OFF_MIGRATION_QUOTE_THRESHOLD..CFG_OFF_MIGRATION_QUOTE_THRESHOLD + 8].copy_from_slice(&threshold.to_le_bytes());
    a
}

pub fn buffer_authority() -> Pubkey {
    hybrid_launch::dbc::buffer_authority().0
}

/// LiteSVM (as `new_svm`) plus the real DBC + Token Metadata programs and a wSOL mint.
pub fn new_svm_dbc() -> LiteSVM {
    use anchor_lang::solana_program::program_pack::Pack;
    let mut svm = new_svm();
    svm.add_program(DBC_ID, include_bytes!("../fixtures/dbc/dbc_mainnet.so")).unwrap();
    svm.add_program(TOKEN_METADATA_ID, include_bytes!("../fixtures/dbc/token_metadata_mainnet.so")).unwrap();
    let mut data = vec![0u8; spl_token::state::Mint::LEN];
    spl_token::state::Mint { mint_authority: None.into(), supply: 0, decimals: 9, is_initialized: true, freeze_authority: None.into() }
        .pack_into_slice(&mut data);
    let lamports = svm.minimum_balance_for_rent_exemption(data.len());
    svm.set_account(WRAPPED_SOL_MINT, Account { lamports, data, owner: SPL_TOKEN_ID, executable: false, rent_epoch: 0 }).unwrap();
    svm
}

impl Env {
    /// Install a DBC config (at `config`) and have the REAL DBC program create a pool + mint for `creator`.
    pub fn dbc_create_pool(&mut self, config: Pubkey, creator: &Keypair, mint: &Keypair) -> Result<DbcIds, String> {
        let base = mint.pubkey();
        let (hi, lo) = if base > WRAPPED_SOL_MINT { (base, WRAPPED_SOL_MINT) } else { (WRAPPED_SOL_MINT, base) };
        let pool = pda(&[b"pool", config.as_ref(), hi.as_ref(), lo.as_ref()], &DBC_ID);
        let base_vault = pda(&[b"token_vault", base.as_ref(), pool.as_ref()], &DBC_ID);
        let quote_vault = pda(&[b"token_vault", WRAPPED_SOL_MINT.as_ref(), pool.as_ref()], &DBC_ID);
        let metadata = pda(&[b"metadata", TOKEN_METADATA_ID.as_ref(), base.as_ref()], &TOKEN_METADATA_ID);
        let event_authority = pda(&[b"__event_authority"], &DBC_ID);
        let mut data = vec![140, 85, 215, 176, 102, 54, 104, 79];
        for s in ["Hybrid Test", "HYB", "ipfs://bafytoken"] {
            data.extend_from_slice(&(s.len() as u32).to_le_bytes());
            data.extend_from_slice(s.as_bytes());
        }
        let m = |k: Pubkey, w: bool, sig: bool| if w { AccountMeta::new(k, sig) } else { AccountMeta::new_readonly(k, sig) };
        let ix = Instruction {
            program_id: DBC_ID,
            accounts: vec![
                m(config, false, false),
                m(hybrid_launch::dbc::DBC_POOL_AUTHORITY, false, false),
                m(creator.pubkey(), false, true),
                m(base, true, true),
                m(WRAPPED_SOL_MINT, false, false),
                m(pool, true, false),
                m(base_vault, true, false),
                m(quote_vault, true, false),
                m(metadata, true, false),
                m(TOKEN_METADATA_ID, false, false),
                m(creator.pubkey(), true, true),
                m(SPL_TOKEN_ID, false, false),
                m(SPL_TOKEN_ID, false, false),
                m(system_program::ID, false, false),
                m(event_authority, false, false),
                m(DBC_ID, false, false),
            ],
            data,
        };
        self.send_max_cu(&[ix], &[creator, mint])?;
        let buffer_authority = buffer_authority();
        Ok(DbcIds {
            config,
            pool,
            base_vault,
            quote_vault,
            metadata,
            buffer_authority,
            buffer_tokens: get_associated_token_address(&buffer_authority, &base),
        })
    }

    pub fn register_dbc_ix(&self, signer: Pubkey, mint: Pubkey, d: &DbcIds, ratio_whole_tokens: u64, collection_size: u64) -> Instruction {
        Instruction::new_with_bytes(
            hybrid_launch::ID,
            &hybrid_launch::instruction::RegisterDbcLaunch {
                params: hybrid_launch::RegisterDbcParams { ratio_whole_tokens, collection_size },
            }
            .data(),
            hybrid_launch::accounts::RegisterDbcLaunch {
                creator: signer,
                mint,
                dbc_config: d.config,
                dbc_pool: d.pool,
                launch_config: pda(&[b"launch_config", mint.as_ref()], &hybrid_launch::ID),
                buffer_authority: d.buffer_authority,
                buffer_tokens: d.buffer_tokens,
                fee_recipient: PLATFORM_FEE_RECIPIENT,
                token_program: SPL_TOKEN_ID,
                associated_token_program: ATA_PROGRAM_ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        )
    }

    /// SIMULATED graduation: flip the pool to the state DBC's DAMM v2 migration leaves it in
    /// (`is_migrated = 1`, `migration_progress = CreatedPool`). A real migration needs swaps up to the
    /// threshold plus the DAMM v2 program and its accounts, which these tests don't load.
    pub fn dbc_mark_migrated(&mut self, pool: Pubkey) {
        use hybrid_launch::dbc::*;
        let mut a = self.svm.get_account(&pool).unwrap();
        a.data[POOL_OFF_IS_MIGRATED] = 1;
        a.data[POOL_OFF_MIGRATION_PROGRESS] = DBC_MIGRATION_CREATED_POOL;
        self.svm.set_account(pool, a).unwrap();
    }

    /// DBC's permissionless `withdraw_leftover`: sends the pool's leftover base tokens to
    /// ATA(config.leftover_receiver, mint).
    pub fn dbc_withdraw_leftover_ix(&self, d: &DbcIds, mint: Pubkey, receiver: Pubkey, receiver_ata: Pubkey) -> Instruction {
        let event_authority = pda(&[b"__event_authority"], &DBC_ID);
        Instruction {
            program_id: DBC_ID,
            accounts: vec![
                AccountMeta::new_readonly(hybrid_launch::dbc::DBC_POOL_AUTHORITY, false),
                AccountMeta::new_readonly(d.config, false),
                AccountMeta::new(d.pool, false),
                AccountMeta::new(receiver_ata, false),
                AccountMeta::new(d.base_vault, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(receiver, false),
                AccountMeta::new_readonly(SPL_TOKEN_ID, false),
                AccountMeta::new_readonly(event_authority, false),
                AccountMeta::new_readonly(DBC_ID, false),
            ],
            data: vec![20, 198, 202, 237, 235, 243, 183, 66],
        }
    }
}

/// DBC path end to end up to a CLOSED vault: the real DBC program creates the pool + mint from a real
/// devnet config (leftover receiver = our buffer PDA), `register_dbc_launch` verifies and records it,
/// and `init_vault` binds the vault. `graduation_proof` = the DBC pool.
pub fn setup_dbc_closed(n: u32) -> (Env, DbcIds) {
    let creator = Keypair::new();
    let mut env = Env {
        svm: new_svm_dbc(),
        ids: Ids::for_mint(Pubkey::default()),
        n,
        ratio_base: RATIO_WHOLE * 10u64.pow(DECIMALS as u32),
        fee: FEE,
        schema_hash: [0x5c; 32],
        leaves: vec![],
        proofs: vec![],
        root: [0; 32],
        sb_queue: Pubkey::new_unique(),
        sb_oracles: vec![],
        graduation_proof: Pubkey::default(),
        users: vec![],
        prefund_collection: 0,
        creator: creator.insecure_clone(),
        real_sb: None,
    };
    env.svm.airdrop(&creator.pubkey(), 1_000_000_000_000).unwrap();
    let config: Pubkey = DBC_DEVNET_CONFIG.parse().unwrap();
    env.svm
        .set_account(config, dbc_config_account(buffer_authority(), hybrid_launch::DEFAULT_GRADUATION_THRESHOLD_LAMPORTS))
        .unwrap();
    let mint = Keypair::new();
    let d = env.dbc_create_pool(config, &creator, &mint).expect("real DBC initialize_virtual_pool_with_spl_token");
    let reg = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, n as u64);
    env.send(&[reg], &[&creator]).expect("register_dbc_launch");
    let mut ids = Ids::for_mint(mint.pubkey());
    // Tokens for test users are moved out of DBC's base vault (stands in for buying on the curve).
    ids.launch_destination = d.base_vault;
    env.ids = env.init_vault_for(ids, n).expect("init_vault on a DBC launch");
    env.graduation_proof = d.pool;
    let oracles: Vec<Pubkey> = (0..6).map(|_| Pubkey::new_unique()).collect();
    env.install_queue(&oracles, 3);
    (env, d)
}
