# Devnet IDLs: match what is DEPLOYED on devnet (verified 2026-10-01, ~5:20 PM MT)

| Program | Program ID | Deployed (MT) | Source commit | Build | Verified |
|---|---|---|---|---|---|
| hybrid_launch | `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8` | 2026-09-26 07:30 (slot 504423460) | `5cd7a7e` | **devnet-e2e** build (`scripts/build-devnet-e2e.sh`, `--features devnet-e2e`) | rebuilt `.so` (221,096 B, sha256 `a22beb3f…7127`) is byte-identical to the deployed program data; the rest of the 231,216 B program data is zero padding |
| hybrid_vault | `BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG` | 2026-09-25 18:46 (slot 504224933) | `e777ea7` … `5cd7a7e` (hybrid_vault source is identical across these; built from `5cd7a7e`) | default production build (`scripts/build.sh`) | rebuilt `.so` (691,744 B, sha256 `414d450c…04cf`) is byte-identical to the full deployed program data |

Both are **before** `d981011` (QA-FEE-03): the deployed vault has no `FeeNotTier` (6061) check, and the deployed
launch still has the third-party DBC config `5L1MfYm4…` in its devnet allowlist (removed in `df0d1d9`, not deployed).
Upgrade authority for both: throwaway devnet deployer `An3ZmiB4SaA7FCJ4bpad2qDRmqhu4F9d5UqUAKHiAB5Z`.

## Files
- `hybrid_launch.json` / `hybrid_launch.ts`: IDL + Anchor TS type, generated WITH `--features devnet-e2e`
  (`anchor idl build -p hybrid_launch -- --features devnet-e2e`). The only difference from the default-build IDL:
  the constant `MIN_GRADUATION_THRESHOLD_LAMPORTS` is absent (its devnet-e2e value isn't marked `#[constant]`).
  **The deployed floor is 0.1 SOL = 100,000,000 lamports**, not the default build's 10 SOL.
- `hybrid_launch_errors.ts`: error-code map (same for both builds).
- `hybrid_vault.json` / `hybrid_vault.ts` / `hybrid_vault_errors.ts`: IDL, TS type, error-code map (13 instructions).

## Not deployed
The IDLs in the repo's `target/idl/` on branch `fix/modes-1-5` (Modes 1–5 plus review fixes) are **NOT deployed**
anywhere. They describe different, larger programs with the same program IDs. Don't use them against devnet.
The same applies to `df0d1d9` / `d981011` builds (not upgraded on devnet).

## On-chain IDL accounts
None. Neither the legacy Anchor IDL account (`create_with_seed(pda([]), "anchor:idl")`: `J7UtKV2Y…` for launch,
`5MpZT6Yu…` for vault) nor any Program Metadata (`ProgM6JC…`) account referencing either program exists on devnet.
These files are the only source; `anchor idl fetch` will not work.

## How this was produced
`git worktree` of `5cd7a7e` in `/workspace/scratch`, separate `CARGO_TARGET_DIR`, platform-tools v1.54
(`scripts/env.sh`), Anchor CLI 1.2.0. Deployed programs fetched with `solana -ud program dump` and compared by sha256.
The worktree was removed afterwards; the shared checkout `/workspace/launchpad` was not touched.
