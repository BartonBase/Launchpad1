# Answers to QA's open questions (Q1–Q11, qa/archive/TEST_PLAN.v0.2.md §11)

> **CURRENT MODEL (2026-09-25; supersedes any burn / 2% / bps / token-fee / pause text below, which is kept for the
> record):** capture and re-roll each pay ONE flat SOL tier fee (0.002 / 0.005 / 0.01 SOL by ratio, cap 0.01) at request,
> to the fixed `PLATFORM_FEE_RECIPIENT`, never refunded (ADR-013). Release is free and returns exactly N tokens.
> Nothing is burned. No pause exists (ADR-015). Minting is lazy: the requester escrows the worst-case Core mint cost
> (0.0063381 SOL; actual ≈ 0.00507, the rest refunded at settle), and settle mints a never-minted pick straight to the
> user from that escrow (ADR-016, [lazy-mint-interface.md](lazy-mint-interface.md)). Expire refunds the principal +
> the escrow, never the fee. Core cost per asset is 0.0035–0.0067 SOL depending on plugins (ours ≈ 0.00509 all-in
> with plugins; lean/no plugins ≈ 0.00507 raw at the escrowed 385-byte size). Ratios: {50k … 5M}, 100 ≤ N ≤ 10,000.

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
  unfulfilled requests. **Superseded: there is no pause at all (ADR-015)** and no token fee; the SOL fee is charged at
  request and never refunded, and no fee-withdraw instruction exists.
- The pause auto-expires and is shown on the site. Details: [admin-multisig-timelock.md](admin-multisig-timelock.md) §3.
- Suggested QA tests: while paused, `release`/`settle`/`expire` succeed and `request_*` fails; the pause lapses at
  `MAX_PAUSE_SLOTS`.

## Q6: Who holds the authorities?

**[engineering]** for the structure; **[needs Barton]** for signers, threshold and timelock length (DECISIONS Q8, N5).

| Authority | Holder |
|---|---|
| Mint authority | **None**, revoked in the same `launch` instruction (✅ tested) |
| Freeze authority | **None**, never set (✅ tested) |
| `LaunchConfig` (ratio, size, tier fee, mint; fees go to the `PLATFORM_FEE_RECIPIENT` constant) | **Nobody**: immutable, no update/close instruction (✅ IDL test) |
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
- **Q2 capture fee:** **decided (ADR-013):** one flat SOL tier fee, the same for capture and re-roll (0.002 / 0.005 /
  0.01 SOL by ratio, hard cap 0.01), paid to `PLATFORM_FEE_RECIPIENT`. No token fee, no burn. Release is free.
- **Q3 fee timing / expire:** **decided (ADR-013/016):** the tier fee is charged at **request** and **never refunded**.
  The request also escrows the worst-case lazy-mint cost (`MINT_ESCROW_LAMPORTS` = 0.0063381 SOL) in a per-request PDA;
  settle spends ≈ 0.00507 SOL of it only if the pick is minted for the first time and refunds the rest to the user;
  `expire` refunds the principal (tokens or the handed-in NFT) + the whole escrow. Release returns exactly N tokens,
  free. VRF is paid by the requester at cost, separately (≈ 0.00002 SOL, ADR-012). Core cost per asset is
  0.0035–0.0067 SOL depending on plugins (≈ 0.00509 all-in).
- **Q4 bonding curve:** [needs Barton] (N6); options in admin-multisig-timelock.md §6.
- **Q6 multisig/timelock:** see Q6 above.
- **Q7 pause:** see Q3 above.
- **Q9 supply copy:** nothing is burned (ADR-013), so supply stays exactly 1B: "Supply fixed at 1,000,000,000; nobody
  can mint more. Up to {N×R} $TICKER can be held as NFTs." Every circulating NFT is fully backed.
- **Q10 decimals:** [needs Barton] (N3). `hybrid_launch` accepts 0–9; overflow edges are covered by
  `attack_decimals_above_9_are_rejected` and the `checked_mul` tests.
