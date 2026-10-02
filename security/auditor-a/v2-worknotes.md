# Auditor A — v2 merge work notes (2026-09-25 ~16:50 MT, saved early)
Repo: /workspace/launchpad (.git). HEAD = wip/hybrid-vault 977f8f2; main 658fb95; large UNCOMMITTED working tree (WT), files touched 16:31-16:44 MT today (engineer active). Read-only inspection only.

## hybrid_launch WT (uncommitted) vs main 658fb95
- FEE_BPS=200 const, MAX_FEE_BPS=200, compile assert (constants.rs WT); LaunchParams has no fee bps/owner/destination; PLATFORM_FEE_OWNER program const (devnet throwaway; mainnet compile_error); LaunchConfig v2 fee_bps, fee_amount (ceil, >0, u128), fee_owner, fee_account, capture/reroll SOL fees.
- MIN_REROLL_SOL_FEE_LAMPORTS = 1_000_000 (0.001), DEFAULT 5_000_000, MAX 50_000_000; reroll SOL fee is a CREATOR launch param in [0.001,0.05]; capture SOL fee in [0,0.05]; no capture>=reroll rule (state.rs comment claims "capture >= re-roll >= MIN" -> comment/code mismatch).
- SOL fees paid to fee_owner (M-16 sink "SKIPPED by Barton" per constants.rs comment).
=> M-07 largely FIXED in WT (uncommitted). M-29 fixed. M-30 fixed (floor >= 890,880).

## hybrid_vault
- committed 977f8f2: expire.rs refunds escrowed fee (fee_escrow PDA, expire.rs:62-63,109,113); mock reveal MOCKREVL with no authority & no oracle sig (mock lib.rs @977f8f2).
- WT: expire.rs + deposit_asset.rs DELETED; fee paid at request directly to fee_account (request.rs:225,273) and SOL to fee_owner (request.rs:226,274); randomness authority = PDA (request.rs:63-65; randomness_ix.rs); reveal permissionless separate ix recording value in SB account (randomness_ix.rs:128-156); recommit permissionless after deadline, refused once revealed, UNCAPPED (randomness_ix.rs:184-212, commits counter only); no expire at all; REVEAL_TIMEOUT_SLOTS=9_000 (constants.rs:36); sb_oracle caller-supplied (request.rs:68-70,145-146; randomness_ix.rs:173-175).
- settle strict FIFO head only (settle.rs:72), reads stored value via revealed_value (randomness.rs:167-171) — not get_value(); MergeBacklog if >32 incoming (settle.rs:82-84), merge_incoming permissionless.
- ensure_fee_account recreates closed ATA (vault_token_ops.rs:60-83) — M-15 fixed; fee_account address pinned to vault.fee_account (request.rs:48) but NO token-account owner check (SetAuthority reassign) — M-05 partial.
- no FeeAboveHardCap check in vault (grep) — M-08 part open.
- guardian pause: guardian chosen by CREATOR in init_vault params (init_vault.rs:17-18,118), single key, up to MAX_PAUSE_SLOTS 1,512,000 (~7d) then 216,000 cooldown (~1d), repeatable; blocks only requests (request.rs:161; admin.rs:20-29).
- invariants.rs:14-20 includes pending_captures (B's "ignores pending" note NOT reproduced on WT as of 16:36).
- WT mock-switchboard: authority must sign on commit/reveal; oracle secp256k1 signature NOT verified (lib.rs:10-13,102-112).
- test-mock-graduation feature (Cargo.toml:20-23; graduation.rs) — compile_error with mainnet; build-test-sbf.sh guards marker; graduation.rs:12-13 claims scripts/deploy-devnet.sh refuses marker but that script does NOT exist (scripts/: build-test-sbf.sh build.sh devnet-airdrop-once.sh env.sh test.sh).
- mint_assets permissionless, payer pays rent (mint_assets.rs:1-5) — graduation-proceeds carve-out not implemented.
- open_vault: fail-closed until curve verifier exists (graduation.rs:39-42).

## 16:55 MT update
- WT moved to ADR-013 during review: FEE_TIERS (0.002/0.005/0.01), MIN 2,000,000, MAX 10,000,000, PLATFORM_FEE_RECIPIENT const, same fee capture/release/reroll, release fee into per-vault FeeVault PDA, permissionless sweep_fees -> recipient only; expire.rs re-added (principal-only, commits > MAX_RECOMMITS=3, +216,000 grace); oracles[4] + OracleReused; sb_oracle still caller-supplied.
- INV-1 excludes pending_rerolls (request_reroll decrements assets_outside) -> B's M-B3 CONFIRMED (earlier note wrong).
- Venue: Meteora DBC base_mint `init, signer` (ix_initialize_virtual_pool_with_spl_token.rs:62-70 @f552f20); Raydium LaunchLab base_mint `init` (raydium-cpi launch-cpi/src/context.rs:69-78 @115df27). Neither takes a pre-existing mint.
- PoC 13 on-chain predict-and-abort PASS (18 attempts, 15 aborts, 3 rare commits). PoC 04 flat/tiered SOL fee model 10/10.
- Merged v2 written: security/merged/round1-merged.md (M-01..M-40). v1+B notes kept at round1-merged.v1-with-B-notes.md.

## v2.1 (2026-09-25 ~17:00–17:05 MT): B flat-fee notes folded in
- WT checked: HEAD 977f8f2 plus 53–54 uncommitted paths. Programs unchanged since 16:58:22. Last recheck at 17:02, when only design/a-obsidian had changed (clean).
- FF-14 → M-41 High (B: Medium). Evidence:
  - `unwrap.rs:70` calls `config::econ()`, and `:110` runs `invariants::check(econ)`.
  - `config.rs:23` checks version==3, `:25` MIN≤fee≤MAX, `:26` `fee_for_ratio==fee`, and `:27` recipient==PLATFORM_FEE_RECIPIENT.
  - `unwrap.rs:24-25` uses a typed `Account<LaunchConfig>`.
- M-04 WT, implemented:
  - `randomness.rs:108-109, 198-218`;
  - `randomness_ix.rs:195-224`;
  - `state.rs:78-80`;
  - `settle.rs:80`;
  - `constants.rs:38, 57`;
  - `expire.rs:78-79`.
- M-04 WT, still open: heartbeat filter and batch expire.
- M-06 closed: `config.rs:22-37` and `validation.rs:13, 45-59`.
- M-40 closed: `invariants.rs:14-20`.
- M-26: pause removed (`lib.rs:22`, `admin.rs:1-2`). ADR-015 has not been written yet.
- VRF ~0.002 SOL comes from a QuickNode guide (third party). Official docs give no price, so it's unverified on devnet.
