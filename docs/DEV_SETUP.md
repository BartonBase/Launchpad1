# Developer setup (on-chain)

Localnet/devnet only. **Never** point these scripts at mainnet. Keys in `.keys/` are throwaway, devnet-only, and
gitignored.

## Versions (verified 2026-09-24 on the shared box)

| Tool | Version | Notes |
|---|---|---|
| Anchor CLI | `anchor-cli 1.2.0` | via `avm 1.2.0`. Repo moved to github.com/otter-sec/anchor (solana-foundation/anchor redirects) |
| Solana / Agave CLI | `solana-cli 4.1.2 (src:182084b8; feat:c763ae0a, client:Agave)` | Anchor 1.2's recommended version. Stable channel 4.2.2 is also installed but not active |
| solana-test-validator | 4.1.2 | |
| cargo-build-sbf | 4.1.0 (Anchor passes platform-tools v1.57) | |
| Rust (workspace) | 1.89.0 (`rust-toolchain.toml`) | rustup stable 1.98.1 also installed. System rustc 1.85 is too old for avm |
| Node | v20.19.2 | only needed for future TS clients |
| anchor-lang (programs) | =1.2.0 | |
| litesvm (tests) | =0.10.0 | needs SBPF v2 builds (see below) |

## One-time setup (already done on the box)

```bash
# Agave (Anza installer), then Anchor via avm
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
cargo install --git https://github.com/otter-sec/anchor avm --locked
avm install 1.2.0 && avm use 1.2.0
agave-install init 4.1.2            # match Anchor 1.2's recommended Solana
# PATH: ~/.solana-dev-env.sh (sourced from ~/.bashrc and ~/.profile)
solana config set --url localhost --keypair /workspace/launchpad/.keys/devnet-only-deployer.json
```

## Daily commands

```bash
cd /workspace/launchpad
source scripts/env.sh                # PATH + ANCHOR_BUILD_SBF_ARCH=v2
./scripts/build.sh                   # anchor build (SBPF v2) → target/deploy/*.so, target/idl/*.json
./scripts/test.sh                    # builds (SBPF v2 + IDL), starts an ISOLATED solana-test-validator on a random
                                     # free port block with a unique temp ledger (programs loaded at genesis), then
                                     # `anchor test --skip-local-validator` → `cargo test --workspace --locked`
./scripts/test.sh --skip-build       # reuse existing build
cargo test -p hybrid_launch --lib    # fast pure-Rust unit tests (validation math)
cargo test -p track-a-hybrid-tests   # LiteSVM tests (needs a prior build: target/deploy + target/idl)
./scripts/build-test-sbf.sh          # REQUIRED before vault tests: test build (mock graduation + mock Switchboard) → target/test-sbf
                                     # after any program change, re-run BOTH anchor build and build-test-sbf.sh
                                     # QA's regression suite: cargo test -p track-a-hybrid-tests --features qa-regression
./scripts/devnet-airdrop-once.sh     # ONE devnet airdrop attempt (rate-limited; failure is fine)
```

## Layout

- `programs/hybrid_launch`: Track A launch program (ADR-010). Unit tests in `src/validation.rs`.
- `tests/track-a-hybrid`: crate `track-a-hybrid-tests`. LiteSVM tests that load `target/deploy/hybrid_launch.so`;
  one `[[test]]` per instruction area (`launch/launch.rs`). Test names state the attack they prove.
- `tests/_shelved/`, `tests/.keys/`: QA's; not part of the Cargo workspace.
- `shelved/`: **SHELVED** Track B code (`fee_treasury`, `holder_lottery`, `localnet-smoke`), excluded from the
  workspace (`exclude = ["shelved"]`) and from `Anchor.toml`. It uses `edition.workspace`, so it doesn't build
  standalone; the fuller WIP is on local branch `shelved/track-b-t22`. See `shelved/README.md`.
- `scripts/`: env/build/test helpers.
- Other top-level folders (`app/`, `design/`, `qa/`, `security/`, `reference/`, `docs/BRIEF.md`) belong to other
  teams. The Anchor tooling doesn't touch them: `.prettierignore` excludes them, and there's no root `package.json`
  lint script.

## Gotchas (all hit and resolved while scaffolding)

1. **`anchor test` defaults to surfpool in Anchor 1.2, and its legacy validator uses the default ports** (8899/8900,
   gossip 8000+), which collide with other agents on the shared box. `scripts/test.sh` starts its own
   `solana-test-validator` on a random free 64-port block (rpc, faucet, gossip, `--dynamic-port-range`) with a
   unique `mktemp` ledger, loads programs with `--bpf-program` (the 4.x loader refuses to *deploy* SBPF v2), and
   runs `anchor test --skip-local-validator --skip-deploy`. It also puts `~/.cargo/bin` first in PATH.
2. **SBPF v3 vs litesvm 0.10.** Anchor 1.2 builds v3 by default, and litesvm 0.10 fails to load it
   (`InvalidAccountData`). `scripts/env.sh` exports `ANCHOR_BUILD_SBF_ARCH=v2`. litesvm 0.16 needs rustc > 1.89.
3. **Pin `solana_version` in Anchor.toml.** Without it, Anchor 1.2 infers the version from `Cargo.lock` and runs
   agave-install to switch the global toolchain for everyone on the shared box.
4. **Anchor needs rustup.** avm deps need rustc ≥ 1.86. The system rustc 1.85 won't work.
5. **`anchor new` with an empty `programs/`** fails on the `programs/*` glob. Temporarily set `members = []`.
6. **Standalone `cargo build-sbf`** uses platform-tools v1.54 and overwrites `target/deploy/*.so`. Rebuild with
   `./scripts/build.sh` before testing.
7. **Anchor rewrites Anchor.toml** (it drops comments) on some commands. Keep notes here instead.
8. **anchor-spl 1.2.0 feature quirks.** The `token_2022` feature also needs `token`. And `idl-build` references
   `token_interface` unconditionally, so a program's `idl-build` feature must also enable `anchor-spl/token_2022`
   (IDL generation only; `hybrid_launch` itself never uses Token-2022).
9. **Switchboard / ORAO crates vs Anchor 1.2.**
   - `switchboard-on-demand` 0.13.0 with `default-features=false, features=["solana-v3"]` passes host `cargo check`,
     but the SBF build fails (getrandom via k256). The `anchor` feature forces solana-v2 and fails.
   - `orao-solana-vrf` 0.7.0 pulls anchor-lang 0.32.2, and its CPI types don't unify with 1.2.
   - Both need a spike (DECISIONS Q1).

## Keys

- `.keys/devnet-only-deployer.json`: throwaway deployer, pubkey `An3ZmiB4SaA7FCJ4bpad2qDRmqhu4F9d5UqUAKHiAB5Z`
  (mode 600, dir 700).
- Program keypairs: `target/deploy/*-keypair.json`, with copies in `.keys/programs/`. These define the localnet
  program IDs in Anchor.toml.
- Devnet airdrop: one attempt on 2026-09-24 was rate-limited ("airdrop request failed"), so there's no devnet deploy
  yet. Use https://faucet.solana.com if needed.

## Shared-box caveat: SBF toolchain collisions

QA builds with cargo-build-sbf platform-tools v1.57; this repo pins v1.54. Both flip the same rustup `solana` toolchain
link, so concurrent builds fail with "No such file" or an apparently uninstalled toolchain. Don't build at the same
time as another team; if it happens, rerun `./scripts/build.sh`. Also, `anchor build` leaves
`target/deploy/mock_switchboard.so` (a workspace member). `scripts/deploy-devnet.sh` refuses to deploy while it's present.
