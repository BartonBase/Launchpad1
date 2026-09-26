//! ADR-014 against the REAL Meteora DBC program (mainnet dump) on LiteSVM, with a real devnet
//! PoolConfig as the template (fixtures/dbc). DBC creates the pool and mint; hybrid_launch's
//! `register_dbc_launch` verifies them; hybrid_vault's `open_vault` accepts only the recorded DBC pool
//! once it has migrated. The unsold "25% buffer" goes (via DBC's own `withdraw_leftover`) to
//! ATA(["dbc_buffer"] PDA, mint), which nothing can move out of.
//!
//! LIMIT: graduation itself is SIMULATED (`dbc_mark_migrated` flips `is_migrated` and
//! `migration_progress`), because a real migration needs swaps up to the threshold plus the DAMM v2
//! program. The real migrated devnet pool is checked by `real_devnet_migrated_pool_parses_as_graduated`.

mod harness;
use anchor_lang::solana_program::program_pack::Pack;
use harness::*;
use hybrid_launch::{dbc, error::LaunchError};

fn launch_err(r: Result<(), String>, e: LaunchError) {
    let code = u32::from(e);
    let err = r.expect_err("must fail");
    assert!(err.contains(&format!("Custom({code})")), "expected {e:?} ({code}), got {}", &err[..err.len().min(3000)]);
}

fn mint_state(env: &Env, mint: &Pubkey) -> spl_token::state::Mint {
    spl_token::state::Mint::unpack(&env.svm.get_account(mint).unwrap().data).unwrap()
}

fn token_state(env: &Env, a: &Pubkey) -> spl_token::state::Account {
    spl_token::state::Account::unpack(&env.svm.get_account(a).unwrap().data).unwrap()
}

/// A second DBC pool (same config) for a fresh mint and creator, not registered.
fn fresh_pool(env: &mut Env) -> (Keypair, Keypair, DbcIds) {
    let creator = Keypair::new();
    env.svm.airdrop(&creator.pubkey(), 100_000_000_000).unwrap();
    let mint = Keypair::new();
    let config: Pubkey = DBC_DEVNET_CONFIG.parse().unwrap();
    let d = env.dbc_create_pool(config, &creator, &mint).expect("DBC pool");
    (creator, mint, d)
}

#[test]
fn research_dbc_mint_is_created_by_dbc_with_authorities_revoked() {
    // What DBC itself leaves behind (research item for ADR-014): classic Token mint, mint authority
    // revoked in the same ix, no freeze authority, fixed 1B supply sitting in DBC's base vault.
    let (env, d) = setup_dbc_closed(100);
    let mint = env.ids.mint;
    let acct = env.svm.get_account(&mint).unwrap();
    assert_eq!(acct.owner, SPL_TOKEN_ID);
    let m = mint_state(&env, &mint);
    assert!(m.mint_authority.is_none() && m.freeze_authority.is_none());
    assert_eq!(m.supply, 1_000_000_000 * 10u64.pow(DECIMALS as u32));
    assert_eq!(m.decimals, DECIMALS);
    let bv = token_state(&env, &d.base_vault);
    assert_eq!(bv.owner, dbc::DBC_POOL_AUTHORITY);
    assert_eq!(bv.amount, m.supply);
    let pool = dbc::parse_pool(&env.svm.get_account(&d.pool).unwrap().data).unwrap();
    assert_eq!((pool.base_mint, pool.config, pool.base_vault), (mint, d.config, d.base_vault));
    assert!(!pool.graduated());
}

#[test]
fn register_records_dbc_pool_and_mint() {
    let (env, d) = setup_dbc_closed(100);
    let a = env.svm.get_account(&env.ids.launch_config).unwrap();
    let cfg = <hybrid_launch::LaunchConfig as anchor_lang::AccountDeserialize>::try_deserialize(&mut &a.data[..]).unwrap();
    assert_eq!(cfg.version, 4);
    assert_eq!(cfg.dbc_pool, d.pool);
    assert_eq!(cfg.dbc_config, d.config);
    assert_eq!(cfg.mint, env.ids.mint);
    assert_eq!(cfg.launch_destination, d.base_vault);
    assert_eq!(cfg.launch_vault, dbc::DBC_POOL_AUTHORITY);
    assert_eq!(cfg.graduation_threshold_lamports, hybrid_launch::DEFAULT_GRADUATION_THRESHOLD_LAMPORTS);
    let buf = token_state(&env, &d.buffer_tokens);
    assert_eq!((buf.owner, buf.mint, buf.amount), (buffer_authority(), env.ids.mint, 0));
    assert!(buf.delegate.is_none() && buf.close_authority.is_none());
}

#[test]
fn register_rejects_wrong_signer_wrong_config_fake_pool_and_repeat() {
    let (mut env, _) = setup_dbc_closed(100);
    let (creator, mint, d) = fresh_pool(&mut env);

    // Not the pool's creator.
    let stranger = Keypair::new();
    env.svm.airdrop(&stranger.pubkey(), 10_000_000_000).unwrap();
    let ix = env.register_dbc_ix(stranger.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    launch_err(env.send(&[ix], &[&stranger]), LaunchError::DbcCreatorMismatch);

    // Fake pool: identical bytes, but owned by another program.
    let mut fake = env.svm.get_account(&d.pool).unwrap();
    fake.owner = Pubkey::new_unique();
    let fake_key = Pubkey::new_unique();
    env.svm.set_account(fake_key, fake).unwrap();
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &DbcIds { pool: fake_key, ..d }, RATIO_WHOLE, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::DbcAccountInvalid);

    // Pool of ANOTHER mint.
    let (_, _, other) = fresh_pool(&mut env);
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &DbcIds { pool: other.pool, ..d }, RATIO_WHOLE, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::DbcAccountInvalid);

    // Ratio not allowed is still validated.
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, 123, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::RatioNotAllowed);

    // Happy path, then a second registration of the same mint is impossible (init).
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    env.send(&[ix], &[&creator]).expect("register");
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 200);
    assert!(env.send(&[ix], &[&creator]).is_err(), "one registration per mint");
}

#[test]
fn register_rejects_config_without_our_buffer_or_with_burn_or_low_threshold() {
    let (mut env, _) = setup_dbc_closed(100);
    let config: Pubkey = DBC_DEVNET_CONFIG.parse().unwrap();
    let thr = hybrid_launch::DEFAULT_GRADUATION_THRESHOLD_LAMPORTS;
    let cases: Vec<(&str, Box<dyn Fn(&mut Vec<u8>)>, LaunchError)> = vec![
        ("leftover to someone else", Box::new(|d: &mut Vec<u8>| d[dbc::CFG_OFF_LEFTOVER_RECEIVER] ^= 1), LaunchError::DbcConfigRejected),
        (
            "post-migration supply below pre (DBC would burn the buffer)",
            Box::new(|d: &mut Vec<u8>| {
                let post = 900_000_000u64 * 1_000_000;
                d[dbc::CFG_OFF_POST_MIGRATION_SUPPLY..dbc::CFG_OFF_POST_MIGRATION_SUPPLY + 8].copy_from_slice(&post.to_le_bytes())
            }),
            LaunchError::DbcConfigRejected,
        ),
        ("metadata authority kept by creator", Box::new(|d: &mut Vec<u8>| d[dbc::CFG_OFF_TOKEN_UPDATE_AUTHORITY] = 0), LaunchError::DbcConfigRejected),
        (
            "threshold below the floor (10 SOL; 0.1 SOL on the devnet-e2e build)",
            Box::new(|d: &mut Vec<u8>| {
                d[dbc::CFG_OFF_MIGRATION_QUOTE_THRESHOLD..dbc::CFG_OFF_MIGRATION_QUOTE_THRESHOLD + 8]
                    .copy_from_slice(&(hybrid_launch::MIN_GRADUATION_THRESHOLD_LAMPORTS - 1).to_le_bytes())
            }),
            LaunchError::GraduationThresholdOutOfRange,
        ),
    ];
    for (what, patch, e) in cases {
        let mut a = dbc_config_account(buffer_authority(), thr);
        patch(&mut a.data);
        // Install the patched bytes at the APPROVED key so only the structural check can fail.
        let key = config;
        env.svm.set_account(key, a).unwrap();
        let creator = Keypair::new();
        env.svm.airdrop(&creator.pubkey(), 100_000_000_000).unwrap();
        let mint = Keypair::new();
        let d = match env.dbc_create_pool(key, &creator, &mint) {
            Ok(d) => d,
            Err(err) => panic!("{what}: DBC refused the patched config: {}", &err[..err.len().min(2000)]),
        };
        let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
        let r = env.send(&[ix], &[&creator]);
        eprintln!("case: {what}");
        launch_err(r, e);
    }
    env.svm.set_account(config, dbc_config_account(buffer_authority(), thr)).unwrap();
}

#[test]
fn register_rejects_config_not_on_platform_allowlist() {
    let (mut env, _) = setup_dbc_closed(100);
    assert_eq!(
        hybrid_launch::APPROVED_DBC_CONFIGS,
        &[DBC_DEVNET_CONFIG.parse::<Pubkey>().unwrap(), DBC_PLATFORM_DEVNET_CONFIG.parse::<Pubkey>().unwrap()]
    );
    // Byte-for-byte the approved (valid) config, but at an address that isn't on the list.
    let unlisted = Pubkey::new_unique();
    env.svm
        .set_account(unlisted, dbc_config_account(buffer_authority(), hybrid_launch::DEFAULT_GRADUATION_THRESHOLD_LAMPORTS))
        .unwrap();
    let creator = Keypair::new();
    env.svm.airdrop(&creator.pubkey(), 100_000_000_000).unwrap();
    let mint = Keypair::new();
    let d = env.dbc_create_pool(unlisted, &creator, &mint).expect("DBC accepts any config");
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::DbcConfigNotApproved);
    assert!(env.svm.get_account(&pda(&[b"launch_config", mint.pubkey().as_ref()], &hybrid_launch::ID)).is_none());
}

#[test]
fn register_rejects_already_migrated_pool() {
    let (mut env, _) = setup_dbc_closed(100);
    let (creator, mint, d) = fresh_pool(&mut env);
    env.dbc_mark_migrated(d.pool);
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::DbcAlreadyGraduated);
}

#[test]
fn open_vault_needs_the_recorded_dbc_pool_to_have_migrated() {
    let (mut env, d) = setup_dbc_closed(100);
    // Before graduation.
    expect_vault_err(env.open(d.pool), VaultError::GraduationNotVerified);
    // Another DBC pool that HAS migrated is not ours.
    let (_, _, other) = fresh_pool(&mut env);
    env.dbc_mark_migrated(other.pool);
    expect_vault_err(env.open(other.pool), VaultError::GraduationNotVerified);
    // A byte-identical copy of our migrated pool owned by someone else.
    env.dbc_mark_migrated(d.pool);
    let mut fake = env.svm.get_account(&d.pool).unwrap();
    fake.owner = Pubkey::new_unique();
    let fk = Pubkey::new_unique();
    env.svm.set_account(fk, fake).unwrap();
    expect_vault_err(env.open(fk), VaultError::GraduationNotVerified);
    // is_migrated alone (progress not CreatedPool) is not enough.
    let mut a = env.svm.get_account(&d.pool).unwrap();
    a.data[dbc::POOL_OFF_MIGRATION_PROGRESS] = 2;
    env.svm.set_account(d.pool, a).unwrap();
    expect_vault_err(env.open(d.pool), VaultError::GraduationNotVerified);
    // Migrated: opens with the REAL verifier (no mock account involved).
    env.dbc_mark_migrated(d.pool);
    env.open(d.pool).expect("open_vault after DBC migration");
    assert!(env.vault_state().open);
    // And the vault works: capture via the normal flow.
    let alice = env.new_user(USER_TOKENS);
    let r = env.new_randomness();
    env.request_capture(&alice, &r).expect("capture on a DBC-launched vault");
}

#[test]
fn buffer_receives_leftover_from_real_dbc_and_can_never_be_withdrawn() {
    let (mut env, d) = setup_dbc_closed(100);
    let mint = env.ids.mint;
    env.dbc_mark_migrated(d.pool);
    let before = token_state(&env, &d.base_vault).amount;
    let crank = Keypair::new();
    env.svm.airdrop(&crank.pubkey(), 1_000_000_000).unwrap();
    // Anyone can trigger DBC's withdraw_leftover; it can only pay the config's leftover_receiver = our PDA.
    let ix = env.dbc_withdraw_leftover_ix(&d, mint, d.buffer_authority, d.buffer_tokens);
    env.send(&[ix], &[&crank]).expect("real DBC withdraw_leftover into the buffer");
    let locked = token_state(&env, &d.buffer_tokens).amount;
    assert!(locked > 0 && locked <= before);
    eprintln!("DBC leftover locked in buffer: {locked}");
    // Pointing withdraw_leftover at a different receiver is refused by DBC.
    let thief = Keypair::new();
    env.svm.airdrop(&thief.pubkey(), 1_000_000_000).unwrap();
    let thief_ata = get_associated_token_address(&thief.pubkey(), &mint);
    let create = create_associated_token_account(&thief.pubkey(), &thief.pubkey(), &mint, &SPL_TOKEN_ID);
    env.send(&[create], &[&thief]).unwrap();
    let ix = env.dbc_withdraw_leftover_ix(&d, mint, thief.pubkey(), thief_ata);
    assert!(env.send(&[ix], &[&thief]).is_err());

    // The buffer ATA's owner is a hybrid_launch PDA: no key exists, and no instruction signs for it.
    let tries: Vec<Instruction> = vec![
        spl_token::instruction::transfer(&SPL_TOKEN_ID, &d.buffer_tokens, &thief_ata, &thief.pubkey(), &[], 1).unwrap(),
        spl_token::instruction::approve(&SPL_TOKEN_ID, &d.buffer_tokens, &thief.pubkey(), &thief.pubkey(), &[], 1).unwrap(),
        spl_token::instruction::burn(&SPL_TOKEN_ID, &d.buffer_tokens, &mint, &thief.pubkey(), &[], 1).unwrap(),
        spl_token::instruction::set_authority(
            &SPL_TOKEN_ID,
            &d.buffer_tokens,
            Some(&thief.pubkey()),
            spl_token::instruction::AuthorityType::AccountOwner,
            &thief.pubkey(),
            &[],
        )
        .unwrap(),
        spl_token::instruction::close_account(&SPL_TOKEN_ID, &d.buffer_tokens, &thief.pubkey(), &thief.pubkey(), &[]).unwrap(),
    ];
    for ix in tries {
        assert!(env.send(&[ix], &[&thief]).is_err());
    }
    // The creator (who registered the launch) has no power over it either.
    let creator = env.creator.insecure_clone();
    let ix = spl_token::instruction::transfer(&SPL_TOKEN_ID, &d.buffer_tokens, &thief_ata, &creator.pubkey(), &[], 1).unwrap();
    assert!(env.send(&[ix], &[&creator]).is_err());
    let after = token_state(&env, &d.buffer_tokens);
    assert_eq!(after.amount, locked);
    assert_eq!(after.owner, buffer_authority());
    assert!(after.delegate.is_none() && after.close_authority.is_none());
    // DBC won't pay the leftover twice.
    let ix = env.dbc_withdraw_leftover_ix(&d, mint, d.buffer_authority, d.buffer_tokens);
    assert!(env.send(&[ix], &[&crank]).is_err());
    // Supply is untouched: the buffer still counts in the 1B (nothing burned).
    assert_eq!(mint_state(&env, &mint).supply, 1_000_000_000 * 10u64.pow(DECIMALS as u32));
}

/// Static half of the can't-withdraw proof: the buffer seed is never used to sign. The only uses are the
/// constant, the address derivation, and an `#[account(seeds = ...)]` address check in register_dbc.
#[test]
fn buffer_seed_is_never_used_for_signing() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../programs");
    let mut hits = vec![];
    fn walk(dir: &std::path::Path, hits: &mut Vec<(String, String)>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, hits);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let s = std::fs::read_to_string(&p).unwrap();
                if s.contains("DBC_BUFFER_SEED") || s.contains("dbc_buffer") || s.contains("buffer_authority") {
                    hits.push((p.display().to_string(), s));
                }
            }
        }
    }
    walk(std::path::Path::new(root), &mut hits);
    let files: Vec<&str> = hits.iter().map(|(p, _)| p.rsplit("programs/").next().unwrap()).collect();
    let mut sorted = files.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        vec!["hybrid_launch/src/constants.rs", "hybrid_launch/src/dbc.rs", "hybrid_launch/src/instructions/register_dbc.rs"],
        "buffer seed/authority referenced in an unexpected file"
    );
    for (p, s) in &hits {
        for bad in ["invoke_signed", "new_with_signer", "with_signer(", "signer_seeds"] {
            assert!(!s.contains(bad), "{p} contains {bad}: the buffer PDA must never sign");
        }
    }
}

#[test]
fn real_devnet_migrated_pool_parses_as_graduated() {
    let a = dbc_fixture_account(DBC_DEVNET_MIGRATED_POOL);
    assert_eq!(a.owner, DBC_ID);
    let p = dbc::parse_pool(&a.data).expect("real devnet VirtualPool");
    assert!(p.graduated(), "{p:?}");
    let c = dbc::parse_config(&dbc_fixture_account(DBC_DEVNET_CONFIG).data).expect("real devnet PoolConfig");
    assert_eq!(p.config, DBC_DEVNET_CONFIG.parse::<Pubkey>().unwrap());
    assert!(c.fixed_token_supply && c.pre_migration_token_supply == c.post_migration_token_supply);
    assert_eq!((c.token_decimal, c.token_type, c.token_update_authority), (6, 0, 1));
    let unmigrated = dbc::parse_pool(&dbc_fixture_account("9beobQVqGsYNCa66XWfP25yXw5BsW7tTUaPoQm1cfy35").data).unwrap();
    assert!(!unmigrated.graduated());
}

/// The live platform devnet config (created with Meteora's DBC SDK), byte-for-byte from devnet, at its
/// real address, with the real DBC program creating the pool.
fn platform_config_pool(env: &mut Env, threshold: Option<u64>) -> (Keypair, Keypair, DbcIds) {
    let key: Pubkey = DBC_PLATFORM_DEVNET_CONFIG.parse().unwrap();
    let mut a = dbc_fixture_account(DBC_PLATFORM_DEVNET_CONFIG);
    if let Some(t) = threshold {
        a.data[dbc::CFG_OFF_MIGRATION_QUOTE_THRESHOLD..dbc::CFG_OFF_MIGRATION_QUOTE_THRESHOLD + 8].copy_from_slice(&t.to_le_bytes());
    }
    env.svm.set_account(key, a).unwrap();
    let creator = Keypair::new();
    env.svm.airdrop(&creator.pubkey(), 100_000_000_000).unwrap();
    let mint = Keypair::new();
    let d = env.dbc_create_pool(key, &creator, &mint).expect("real DBC creates a pool on the platform config");
    (creator, mint, d)
}

#[test]
fn platform_devnet_config_fields_match_every_register_check() {
    let c = dbc::parse_config(&dbc_fixture_account(DBC_PLATFORM_DEVNET_CONFIG).data).expect("PoolConfig");
    assert_eq!(c.leftover_receiver, buffer_authority());
    assert_eq!(c.quote_mint, dbc::WRAPPED_SOL_MINT);
    assert_eq!((c.token_type, c.token_decimal, c.token_update_authority), (dbc::DBC_TOKEN_TYPE_SPL, 6, dbc::DBC_TOKEN_AUTHORITY_IMMUTABLE));
    assert!(c.fixed_token_supply);
    assert_eq!((c.pre_migration_token_supply, c.post_migration_token_supply), (1_000_000_000_000_000, 1_000_000_000_000_000));
    assert_eq!(c.migration_quote_threshold, 100_000_000, "0.1 SOL: graduates on faucet SOL");
}

#[test]
#[cfg(not(feature = "devnet-e2e"))]
fn platform_devnet_config_is_below_the_default_10_sol_floor() {
    // Default (non devnet-e2e) build: the 0.1 SOL threshold is refused, so only the devnet-e2e build
    // can register against it.
    let (mut env, _) = setup_dbc_closed(100);
    let (creator, mint, d) = platform_config_pool(&mut env, None);
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::GraduationThresholdOutOfRange);
}

#[test]
fn platform_devnet_config_registers_when_the_threshold_meets_the_floor() {
    // Same live bytes with only the threshold raised to the default floor: every other check passes.
    let (mut env, _) = setup_dbc_closed(100);
    let (creator, mint, d) = platform_config_pool(&mut env, Some(hybrid_launch::MIN_GRADUATION_THRESHOLD_LAMPORTS));
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    env.send(&[ix], &[&creator]).expect("platform devnet config registers");
    let lc = env.svm.get_account(&pda(&[b"launch_config", mint.pubkey().as_ref()], &hybrid_launch::ID)).expect("LaunchConfig");
    assert_eq!(lc.owner, hybrid_launch::ID);
}

#[test]
fn qa_fee03_register_stores_exactly_the_tier_for_every_ratio() {
    // register_dbc_launch derives the fee with the shared tier function and requires an exact,
    // non-zero tier before storing it (0 / off-tier: validation::qa_fee03 unit test; FeeNotTier 6024).
    let (mut env, _) = setup_dbc_closed(100);
    for &(ratio, tier) in hybrid_launch::FEE_TIERS.iter() {
        let (creator, mint, d) = fresh_pool(&mut env);
        let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, ratio, 100);
        env.send(&[ix], &[&creator]).unwrap_or_else(|e| panic!("ratio {ratio}: {e}"));
        let lc = env.svm.get_account(&pda(&[b"launch_config", mint.pubkey().as_ref()], &hybrid_launch::ID)).unwrap();
        let o = hybrid_launch::LC_OFF_FEE_LAMPORTS;
        assert_eq!(u64::from_le_bytes(lc.data[o..o + 8].try_into().unwrap()), tier, "ratio {ratio}");
    }
    let (creator, mint, d) = fresh_pool(&mut env);
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, 10_000, 100);
    launch_err(env.send(&[ix], &[&creator]), LaunchError::RatioNotAllowed);
}

#[test]
#[cfg(feature = "devnet-e2e")]
fn devnet_e2e_build_registers_the_platform_config_at_0_1_sol() {
    let (mut env, _) = setup_dbc_closed(100);
    let (creator, mint, d) = platform_config_pool(&mut env, None);
    let ix = env.register_dbc_ix(creator.pubkey(), mint.pubkey(), &d, RATIO_WHOLE, 100);
    env.send(&[ix], &[&creator]).expect("devnet-e2e build accepts the 0.1 SOL platform config");
}
