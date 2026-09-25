# Answers to QA's open questions (Q1–Q11, qa/archive/TEST_PLAN.v0.2.md §11)

From: Solana Program Engineer, 2026-09-24 (after the SCOPE CHANGE, ADR-009). Each answer is labeled
**[engineering]** (decided by engineering within Barton's constraints, can be tested now) or **[needs Barton]**.
Track B (Token-2022 tax / treasury / raffle) is **shelved**, so its questions are marked **OBSOLETE**. Newer QA
questions (qa/TEST_PLAN.md §14) are cross-referenced where they overlap.

| Q | Status |
|---|---|
| Q1 | Resolved earlier, now superseded: Track B shelved, Track A only (ADR-009) |
| Q2 | **OBSOLETE** (Track B treasury buys) |
| Q3 | Answered (Track A part) |
| Q4 | **OBSOLETE** (raffle exclusions) |
| Q5 | **OBSOLETE** for the raffle; VRF for re-rolls is still open (DECISIONS Q1) |
| Q6 | Answered (Track A part) |
| Q7 | **OBSOLETE**: there's no raffle on any track |
| Q8 | **OBSOLETE** (existing-collection NFT standards) |
| Q9 | Answered |
| Q10 | **OBSOLETE** (raffle prizes) |
| Q11 | Answered (Track A part) |

## Q3: What's allowed while paused?

**[engineering]**, with one **[needs Barton]** item (N4: whether a pause exists at all).
- Nothing in `hybrid_launch` is pausable; it has no admin.
- In the engine, the only pausable instructions are **`request_capture` and `request_reroll`** (new money in).
- **Never pausable:** `release` (unwrap, exact `ratio` back), `settle` of already-paid requests, `expire` of
  unfulfilled requests. The token fee is escrowed in the request PDA's token account at request time, **burned at
  `settle`** and refunded with the locked tokens on `expire` (DECISIONS N7, final). There's no separate "burn crank"
  to pause, and no fee-withdraw instruction exists.
- The pause auto-expires and is shown on the site. Details: [admin-multisig-timelock.md](admin-multisig-timelock.md) §3.
- Suggested QA tests: while paused, `release`/`settle`/`expire` succeed and `request_*` fails; the pause lapses at
  `MAX_PAUSE_SLOTS`.

## Q6: Who holds the authorities?

**[engineering]** for the structure; **[needs Barton]** for signers, threshold and timelock length (DECISIONS Q8, N5).

| Authority | Holder |
|---|---|
| Mint authority | **None**, revoked in the same `launch` instruction (✅ tested) |
| Freeze authority | **None**, never set (✅ tested) |
| `LaunchConfig` (ratio, size, fees, fee destination = burn, mint) | **Nobody**: immutable, no update/close instruction (✅ IDL test) |
| Engine config (fees, ratio, destinations) | **Nobody**: fixed at init (ADR-009 C3) |
| Pause new captures (if kept) | Squads guardian multisig, auto-expiring (option a) or the timelocked governance multisig (option b) |
| Program upgrade | Squads governance multisig vault PDA, 7-day timelock proposed; frozen (`--final`) after audit + stabilization |
| Vault / collection authority | Program PDAs; no instruction lets a person move backing or edit metadata |
| Launch supply (1B at launch) | ATA of the program-derived **launch vault PDA** `["launch_vault", mint, launch_config]`; not caller-chosen, no withdraw instruction (QA-HL-02 fix, N2). Distribution (curve) TBD |
| Track B authorities (fee config, withdraw-withheld, treasury/raffle admin) | **OBSOLETE** (shelved) |

## Q9: Shared registry, or separate programs?

**[engineering]** With Track B shelved there's only one track. No registry program is planned. `hybrid_launch` owns
`LaunchConfig` PDAs seeded by the mint, and the engine will read `LaunchConfig` (pinned program ID + seeds) rather
than keep its own copy. Separate program IDs namespace all PDAs, so no cross-program confusion is possible. The
"Track B token targeting a Track A collection" part (SEP-13) is **OBSOLETE** for now.

## Q11: Must mints be created by the launchpad?

**[engineering]** **Yes, for Track A v1: launchpad-created mints only.** Enforced on-chain: `launch` requires the mint
as a fresh keypair **signer** and creates the account itself (owner = classic SPL Token). A pre-funded but empty
system-owned address is tolerated (top-up + allocate + assign, QA-HL-01 fix); an address with data or a non-system
owner is rejected (`MintAccountInUse`). Existing classic or Token-2022 mints can't be onboarded (✅ `attack_existing_classic_mint_cannot_be_onboarded`,
`attack_existing_token_2022_mint_cannot_be_onboarded`, `attack_mint_keypair_not_signing_is_rejected`). The Track B
half (onboarding existing Token-2022 mints) is **OBSOLETE**.

## Cross-reference: newer QA questions (qa/TEST_PLAN.md §14)

- **Q1 engine:** **ACCEPTED: `hybrid_vault`** (ADR-008 / Q-H1, decided 2026-09-24 by the Solana Program Engineer
  because MPL-Hybrid can't meet Barton's stated requirements; reversible if Barton objects). MPL-Hybrid is reference
  only.
- **Q2 capture fee:** token bps of R, ≤ 1,000 bps, `capture ≥ re-roll` enforced by `hybrid_launch`, burned like the
  re-roll fee ([engineering] default, DECISIONS N1; needs Barton's confirmation).
- **Q3 fee timing / expire:** **Token fee: decided (final, N7): burn at SETTLE.** Escrowed in the request PDA's token
  account at request time, burned at `settle`, refunded together with the locked tokens on `expire`. **SOL part:**
  [needs Barton]; engineering default is that it pays VRF + rent only, refunded minus VRF cost on `expire`.
- **Q4 bonding curve:** [needs Barton] (N6); options in admin-multisig-timelock.md §6.
- **Q6 multisig/timelock:** see Q6 above.
- **Q7 pause:** see Q3 above.
- **Q9 supply copy after burns:** [engineering] proposal: "Up to {N×R} $TICKER could be held as NFTs at launch" plus
  a live line "Current supply: {supply} ({burned} burned by re-roll fees)". Every circulating NFT stays fully backed
  (ADR-009 burn check).
- **Q10 decimals:** [needs Barton] (N3). `hybrid_launch` accepts 0–9; overflow edges are covered by
  `attack_decimals_above_9_are_rejected` and the `checked_mul` tests.
