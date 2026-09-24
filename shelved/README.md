# 💤 SHELVED: Track B (Token-2022 tax / treasury / holder lottery)

**Status: SHELVED/DEFERRED per Barton's SCOPE CHANGE (2026-09-24, 2:51 PM MT) and ADR-009 in `docs/DECISIONS.md`.**
Deferred, not dropped for good. Don't build on this until the scope comes back.

This folder is **excluded from the build**: `Cargo.toml` has `exclude = ["shelved"]`, and `Anchor.toml` lists only
`hybrid_launch`. The crates use `edition.workspace`/`rust-version.workspace`, so they won't build standalone either;
to resume, move them back under `programs/` / `tests/` and re-add them to `Anchor.toml`.

| Path | What it is | State |
|---|---|---|
| `programs/fee_treasury` | Tax treasury (`initialize` only), program ID `9mNyaZ3iDVdZKpdy6ZJvt3oPXTCYBisfPsqqzCaa7vsB` | as committed on `main` at `d29edea` |
| `programs/holder_lottery` | Raffle (`initialize` + sybil-neutral ticket math), program ID `FJGixnazzvAQv2MFr5sABnKsm8yKqJAvxwBiTXNKmVNg` | as committed on `main` at `d29edea` |
| `tests/localnet-smoke` | Token-2022 TransferFee mint smoke test against a local validator | as committed on `main` at `d29edea` |

**The newer work in progress is on local branch `shelved/track-b-t22` (commit `3916467`, WIP, not built for SBF, no
tests wired):** `fee_treasury` renamed to `programs/tax_treasury` with `launch_tax_token`, `harvest_withheld`,
`withdraw_withheld_to_treasury`, fee math and a `SHELVED.md` note; tests regrouped under `tests/track-b-tax-raffle/`.
Host `cargo check -p tax_treasury` passed there.

Design notes (kept, marked SHELVED): `docs/ARCHITECTURE.md` §SHELVED, `docs/THREAT_MODEL.md` 💤 rows,
`docs/DECISIONS.md` ADR-004 (superseded) / ADR-005 / ADR-006, `docs/transfer-tax-vs-wrap.md`. QA's shelved notes:
`tests/_shelved/track-b/README.md`.
