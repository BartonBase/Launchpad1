# Code Review: BartonBase/Launchpad1 (Auditor B)

**Prepared for:** Barton
**Date:** 2026-09-24, checked 14:29-14:31 MDT
**Repo:** https://github.com/BartonBase/Launchpad1 (public, description "SPL 404 Launchpad")
**Commit reviewed:** **none. The repository is empty and has no commits**, so there is no SHA. This file is named `code-review-no-commits.md` instead of `code-review-<shortsha>.md` for that reason. Re-run this review and write `code-review-<shortsha>.md` once code is pushed.
**Method:** Public, read-only, unauthenticated access with `curl` only (GitHub REST API, github.com HTML, raw.githubusercontent.com). No clone, no build, no deployment, no keys.

## 1. Evidence that the repo is empty

| Request | Result |
|---|---|
| `GET api.github.com/repos/BartonBase/Launchpad1` | 200. `size: 0`, `language: null`, `default_branch: main`, `fork: false`, `visibility: public`. Created 2026-09-24T20:20:29Z (14:20 MDT), pushed_at 20:20:30Z (14:20 MDT), updated_at 20:23:07Z (14:23 MDT). |
| `GET .../git/trees/HEAD?recursive=1` | **409 "Git Repository is empty."** (checked twice, at 14:29 and 14:31 MDT) |
| `GET .../commits/HEAD` | **409 "Git Repository is empty."** |
| `GET .../branches` | `[]` (no branches) |
| `GET .../tags` | `[]` (no tags) |
| `GET .../contents/` | 404 "This repository is empty." |
| `https://github.com/BartonBase/Launchpad1` (HTML) | Shows "This repository is empty" |
| `raw.githubusercontent.com/.../main/README.md`, `.../main/Anchor.toml` | 404 |

The owner's only other public repo is `BartonBase/Fishnu-Arcade` (a Flash-style games web page, last pushed 2025-03-16). It is unrelated and was not reviewed.

## 2. Structure

None. No files, no Anchor workspace (`Anchor.toml`, `programs/`), no frontend, no tests, no `Cargo.lock`.

## 3. Status of threat-model findings (B-01 to B-16)

Every item is **Not yet implemented**: no code exists to be Vulnerable or Mitigated. No file:line evidence is possible. The "When code lands, check" column lists what evidence to look for.

| ID | Sev. | Status | When code lands, check |
|---|---|---|---|
| B-01 Predictable reroll pick | Critical | Not yet implemented | Any read of `SlotHashes`, `RecentBlockhashes`, `Clock`, `count` in a selection path; any use of stock MPL-Hybrid capture with reroll on; a two-phase VRF request/settle (R-01) |
| B-02 Lottery randomness | Critical | Not yet implemented | Draw state machine, VRF account pinned at request, permissionless unconditional settle (R-02) |
| B-03 Authority rug | Critical | Not yet implemented | `assert_launch_ready`, mint/freeze authority `None`, extension allowlist excludes PermanentDelegate, DefaultAccountState, MintCloseAuthority (R-03) |
| B-04 T22 fee authorities | Critical | Not yet implemented | Withdraw-withheld authority = PDA paying only the purchase vault; fee-config authority `None` or capped PDA plus program timelock (R-04) |
| B-05 Vault drain / account validation | High | Not yet implemented | `address =` / `Program<>` pins, per-pot PDA seeds, derived destinations, one-shot claim flags (R-05) |
| B-06 T22 fee bypass / insolvency | High | Not yet implemented | Fork uses `Interface<TokenInterface>` + `transfer_checked`; balance-delta accounting; no `BurnOnCapture` on the fee mint (R-06) |
| B-07 Fee-funded buy manipulation | High | Not yet implemented | Buys only at fixed escrow rate, or allowlisted venue with price cap; TWAP `min_out` (R-07) |
| B-08 Flash snapshot | High | Not yet implemented | TWAB/stake-time accrual, minimum hold (R-08) |
| B-09 Sybil ticket formula | High | Not yet implemented | `tickets = floor(twab / TICKET_UNIT)` only (R-09) |
| B-10 Launch sniping | High | Not yet implemented | Commit window + batch clearing or per-slot caps; `open_slot` fixed in config (R-10) |
| B-11 Sandwich / MEV | Medium | Not yet implemented | `min_out`/`max_in`, `expected_*` args on value-moving ixs (R-11) |
| B-12 Reroll economics | Medium | Not yet implemented | Surcharge/cooldown, no caller-chosen NFT (R-12) |
| B-13 DoS / unbounded iteration | Medium | Not yet implemented | No holder loops, O(log N) winner lookup, paginated cranks (R-13) |
| B-14 Draw liveness | Low | Not yet implemented | `deadline_slot`, permissionless cancel with no re-request, claim expiry (R-14) |
| B-15 Dependency pinning | Info | Not yet implemented | `Cargo.lock`, pinned MPL-Hybrid commit (`aacf1a53` or fork base), program-ID constants (R-15) |
| B-16 MPL-Hybrid `[min, max)` range | Low/Info | Not yet implemented | Inclusive range + reachability test for every metadata index |

**Summary:** Vulnerable 0, Mitigated 0, Not yet implemented 16, Not applicable 0.

## 4. Status of design requirements (R-01 to R-15)

R-01 to R-15: **all Not yet implemented** (no code, tests, or CI exist). R-15.3 (resolve every [needs source]) is partly done in the documents: threat-model.md rev. 2 resolved everything research-sources.md covers. Still open: the Switchboard/ORAO API (B-02) and Solana compute/account limits (B-13).

## 5. New code-level findings

**None.** There is no code to review. The observation below is about the repo metadata, not code.

### CR-00 (Info): repo description says "SPL 404", but the design needs a Token-2022 fee mint
- **Observation:** The repo description is "SPL 404 Launchpad". The design (threat-model.md) uses a Token-2022 transfer-fee mint whose fees fund NFT buys. Stock MPL-Hybrid supports classic SPL Token only (threat-model.md B-06), so "SPL 404 on stock MPL-Hybrid" and "Token-2022 fee mint" can't both be true without a fork.
- **Risk:** If the team starts from stock MPL-Hybrid with a classic SPL mint, the fee-funded-buy design has no on-chain fee source. If the team later bolts on Token-2022, it has to fork the swap (B-06).
- **Fix:** Decide the token program before code is written, and record it in the README. If Token-2022, plan the MPL-Hybrid fork (token interface + `transfer_checked`, burn paths disabled, VRF-based selection per B-01/B-16) from day one.
- **Attacker cost (estimate):** N/A (design clarity).

## 6. Checklist for the first real review (what will be audited once code exists)

Each item gives the severity it would carry if found, the attack, a cost estimate, and the fix, so the team can build to it now.

| # | Check | Sev. if found | Scenario | Attacker cost (estimate) | Fix |
|---|---|---|---|---|---|
| C-1 | Missing `Signer` / `has_one` on admin ixs (config update, fee sweep, draw create) | Critical | Anyone calls `update_config` or `sweep_fees` with their own destination | < 0.001 SOL tx fee | `Signer` + `has_one = admin` (multisig) + timelock; destinations as PDA constraints |
| C-2 | `UncheckedAccount` / `AccountInfo` without owner or address checks (VRF account, token program, MPL-Hybrid, marketplace) | Critical | Attacker passes a fake VRF account with a chosen result, or a fake program that receives a PDA signer (B-05) | < 0.01 SOL | `address =` pins, `Program<>`, owner checks, provider SDK parsing |
| C-3 | PDA seed collisions (e.g. `[b"ticket", user]` without draw id; variable-length seeds concatenated without separators) | High | A ticket or claim PDA is reused across draws; `["vault", a+b]` collides with `["vault", a'+b']` | < 0.01 SOL | Include draw_id/config key in seeds; fixed-length seeds; store and check `bump` |
| C-4 | Unchecked arithmetic on fees, tickets, prize splits (`+`, `*`, `as u64` casts) | High | Overflow or truncation mints tickets or under-charges fees | ~0 | `checked_*` everywhere, `overflow-checks = true` in release profile, u128 intermediates |
| C-5 | Fee accounting uses gross, not net, amounts on Token-2022 transfers | High | Escrow insolvency via capture/release loops (B-06) | ~0.07 SOL per loop | Balance-delta accounting; solvency invariant fuzz test |
| C-6 | Lottery snapshot reads live balances or a caller-supplied account list | High | Flash snapshot (B-08) or selective omission (B-13) | ~1 SOL per snapshot (sim) | TWAB/stake accrual; totals checked against `draw.total_tickets` |
| C-7 | Randomness from SlotHashes/Clock, or VRF not bound to the request | Critical | Predict-and-abort (B-01/B-02) | ~0.05 SOL per rare grab (sim) | Two-phase VRF with stored account pubkey and seed |
| C-8 | Authority handling: admin transfer in one step, no two-step accept; authority stored but never checked | High | Admin set to a wrong or attacker key permanently | ~0 | Two-step `propose/accept`; tests for each admin ix |
| C-9 | Close-account / reinit: `close =` without zeroing, `init_if_needed`, closed claim PDA re-creatable | High | Close a claimed-prize PDA, re-init it and claim again (double claim) | < 0.01 SOL | Avoid `init_if_needed`; keep a tombstone or `claimed` flag in a non-closable account; Anchor discriminator checks |
| C-10 | `remaining_accounts` used without owner, discriminator, and uniqueness checks | High | Duplicate accounts double-count tickets | ~0 | Validate each account; reject duplicates |

## 7. PoC tests to write once code exists (local validator or bankrun/LiteSVM only)

No toolchain was built for this pass because there is nothing to compile.
1. **B-01 predict-and-abort:** a wrapper program (or a trailing top-level instruction) that recomputes the pick and reverts unless it is rare. Pass criterion: success rate equals the base rate over ≥ 100k attempts (R-01 test a). If stock MPL-Hybrid capture is used, this test should *succeed* at 100%, proving the Critical.
2. **B-16 reachability:** for a small range (e.g. min 0, max 7), sweep seeds and assert every index 0..7 is produced; stock MPL-Hybrid math will never produce 7.
3. **B-06 solvency:** 10,000 capture→release loops at 100 bps on a Token-2022 mint; assert `escrow_balance ≥ outstanding × backing` after every loop.
4. **B-04 authority gate:** `open_sale` fails when withdraw-withheld or fee-config authority is an EOA, when MintCloseAuthority/PermanentDelegate is present, or when mint/freeze authority is set.
5. **C-9 double claim:** claim, close, re-init, claim again must fail.

## 8. Next step

Push the Anchor workspace (including `Cargo.lock` and tests) to `BartonBase/Launchpad1`, then re-run this review against the new HEAD SHA. A professional third-party audit is still required before any mainnet deployment.
