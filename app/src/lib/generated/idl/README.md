# Devnet IDLs: match what is DEPLOYED on devnet (upgraded 2026-10-02, ~2:09 PM MT)

| Program | Program ID | Deployed (MT) | Source | Build | .so |
|---|---|---|---|---|---|
| hybrid_launch | `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8` | 2026-10-02 ~14:09 (slot 506750423) | `hackathon/meteora-dbc` (= `fix/modes-1-5` + ADR-021 beta gating + ADR-023 fee recipient) | `--features devnet-e2e,mainnet-beta`, opt-level z | 175,384 B, sha256 `8b340f1900f69dcc…` |
| hybrid_vault | `BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG` | 2026-10-02 ~14:09 (slot 506750498) | same | `--features mainnet-beta`, opt-level z | 532,824 B, sha256 `9bfa365fcae86c43…` |

Program data accounts were not resized (231,216 B and 691,744 B); the rest is zero padding.
Upgrade authority for both: throwaway devnet deployer `An3ZmiB4SaA7FCJ4bpad2qDRmqhu4F9d5UqUAKHiAB5Z`.

What changed versus the previous devnet builds (`5cd7a7e`):
- `PLATFORM_FEE_RECIPIENT` (devnet) = `BVKxZMjuXryATqCeifPgh6Ee9H93BH5UGv8eML66yT3j` (ADR-023).
- Native `launch` compiled out (`no-native-launch`); `register_plain_dbc` present; burn / tax / raffle modes compiled out.
- Vault includes QA-FEE-03 (`FeeNotTier` 6061) and the other `fix/modes-1-5` audit fixes; Vault/Request layouts unchanged.
- Devnet DBC allowlist: only `DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9`.

## Files
- `hybrid_launch.json` / `.ts` / `_errors.ts`: `anchor idl build -p hybrid_launch -- --features devnet-e2e,mainnet-beta`.
  `MIN_GRADUATION_THRESHOLD_LAMPORTS` is absent from the IDL (devnet-e2e value isn't a `#[constant]`): **the deployed floor is 0.1 SOL**.
- `hybrid_vault.json` / `.ts` / `_errors.ts`: `anchor idl build -p hybrid_vault -- --features mainnet-beta`.

## On-chain IDL accounts
None; these files are the only source (`anchor idl fetch` will not work).

## How this was produced
Platform-tools v1.57 (v1.54 isn't installed on this box), Anchor CLI 1.2.0, separate `CARGO_TARGET_DIR`.
Deployed with `solana program deploy --program-id …` (buffer upgrade; fees 0.0035 SOL total, buffer rent refunded).
