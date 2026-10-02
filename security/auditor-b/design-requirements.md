# Launchpad Design Requirements (from Auditor B Threat Model)

Each requirement maps one-to-one to a finding in `threat-model.md`. Each one is written so it can be tested with Anchor/Bankrun/LiteSVM tests or a fuzzer. "MUST" is mandatory before mainnet. Parameter values in brackets are examples for the team to finalize.

| Req | Finding | Severity |
|---|---|---|
| R-01 | B-01 | Critical |
| R-02 | B-02 | Critical |
| R-03 | B-03 | Critical |
| R-04 | B-04 | Critical |
| R-05 | B-05 | High |
| R-06 | B-06 | High |
| R-07 | B-07 | High |
| R-08 | B-08 | High |
| R-09 | B-09 | High |
| R-10 | B-10 | High |
| R-11 | B-11 | Medium |
| R-12 | B-12 | Medium |
| R-13 | B-13 | Medium |
| R-14 | B-14 | Low |
| R-15 | B-15 | Info |

---

### R-01: Unpredictable, committed reroll/capture selection (B-01)
- R-01.1 Metadata or NFT selection for capture and reroll MUST NOT read SlotHashes, RecentBlockhashes, Clock, `escrow.count`, or any other value readable before the transaction.
- R-01.2 Selection MUST be two-phase: `request_*` locks the payment and NFT in a request PDA that stores the VRF account pubkey and seed. `settle_*` is permissionless and applies the VRF result unconditionally. There MUST be no cancel after the VRF result is fulfilled.
- R-01.3 `settle_*` MUST reject a VRF account whose pubkey, owner program, or seed doesn't match the request PDA.
- R-01.4 Index derivation MUST use rejection sampling (no bare `% n` bias).
- **Tests:** (a) A PoC wrapper program that reads all on-chain state and aborts unless the pick is rare must have a success rate statistically indistinguishable from the base rate (p=0.1%, ≥ 100k simulated attempts, 99% CI). (b) Settling with a foreign fulfilled VRF account fails. (c) Cancel after fulfilment fails. (d) A chi-square test over 1M picks with `n` not a power of two passes.

### R-02: Lottery randomness integrity (B-02)
- R-02.1 Each draw MUST be a PDA with one-way states `Open → SnapshotFrozen → RandomnessRequested → Fulfilled → Settled`, plus `Cancelled`. Any backwards transition fails.
- R-02.2 `ticket_root`/`total_tickets` MUST be frozen before randomness is requested. The VRF seed MUST include `draw_pda || draw_id || ticket_root`.
- R-02.3 `settle_draw` MUST be permissionless, MUST require the VRF account stored at request time (pinned owner program, fulfilled), and MUST write the winner and move the prize to a claim PDA in the same instruction.
- R-02.4 `draw_slot` and the snapshot period are fixed at draw creation. There MUST be no admin instruction that changes them afterward.
- R-02.5 A draw MUST NOT be able to request randomness more than once.
- **Tests:** (a) Settling with any other fulfilled VRF account fails. (b) A second `request_randomness` on the same draw fails. (c) Non-admin settle succeeds. (d) A CPI wrapper that calls settle and then reverts on "loss" can't produce a different recorded winner on retry, because state is only written by a successful settle and the result is deterministic. (e) Admin attempts to change `draw_slot` fail.

### R-03: No single-key rug (B-03)
- R-03.1 The program MUST expose `assert_launch_ready` (called by `open_sale`), which fails unless: the upgrade authority is the configured multisig (or none); the fungible mint's `mint_authority == None` and `freeze_authority == None`; the mint extensions are a subset of the allowlist {TransferFeeConfig, MetadataPointer, TokenMetadata} with **no** PermanentDelegate, DefaultAccountState, or NonTransferable; and the NFT collection update authority is the multisig.
- R-03.2 Escrow config changes (`amount`, `fee_amount`, `sol_fee_amount`, URI range) MUST go through a queue/execute timelock (≥ [72h]) and fall within hard-coded bounds.
- **Tests:** `open_sale` fails for each of: mint authority set, freeze authority set, PermanentDelegate present, upgrade authority = single EOA. A config change executed before the timelock fails. Out-of-bounds values fail.

### R-04: Fee authorities constrained (B-04)
- R-04.1 `withdraw_withheld_authority` MUST be the PDA `["fee_vault_auth", config]`. The only instruction that signs with it withdraws to the purchase-vault PDA. The destination is not an instruction argument.
- R-04.2 `transfer_fee_config_authority` MUST be `None`, or a PDA whose `update_fee` enforces `bps ≤ [300]`, `maximum_fee ≤ [cap]`, multisig signer, and a timelock ≥ 2 epochs + [72h].
- R-04.3 `assert_launch_ready` MUST verify R-04.1 and R-04.2 on-chain.
- **Tests:** Withdrawing with any destination other than the purchase vault fails. `update_fee` with bps = [301] fails. `open_sale` fails if either authority is an EOA.

### R-05: Vault safety (B-05)
- R-05.1 Every external program account MUST be pinned by address (Token-2022, ATA, System, MPL-Hybrid, MPL Core/Token Metadata, VRF, any marketplace).
- R-05.2 Each pot (fee, purchase, prize per draw, escrow) MUST have its own PDA authority seeds. No PDA signer may be passed to an unpinned CPI.
- R-05.3 Outflow destinations MUST be derived or stored, never free arguments. Claims MUST set a one-shot flag before transferring.
- R-05.4 Invariant: after every instruction, `Σ vault balances ≥ Σ recorded liabilities`.
- **Tests:** Substituting a malicious program for each program account fails. A double claim fails. Cross-pot signing fails. A fuzz run (≥ [1M] random instruction sequences) never violates R-05.4.

### R-06: Token-2022 fee-correct swap accounting (B-06)
- R-06.1 All fungible transfers MUST use `transfer_checked` on the Token-2022 fee mint. There MUST be no mint/burn legs in capture or release.
- R-06.2 The net amount received MUST be measured (balance delta or current-epoch fee calculation) and only the net amount credited.
- R-06.3 Release MUST NOT pay out more than the backing recorded for that NFT. Invariant: `escrow_token_balance ≥ nfts_outstanding × backing_per_nft`.
- R-06.4 A permissionless `harvest_withheld` crank MUST sweep withheld fees from protocol-owned accounts into the fee pot.
- **Tests:** 10,000 capture→release loops at 100 bps leave the escrow solvent (R-06.3 holds). The last NFT holder can always release. Fees charged per round trip equal 2 × bps × amount within rounding.

### R-07: Manipulation-resistant fee-funded buys (B-07)
- R-07.1 Fee-funded NFT acquisition MUST default to capture from the escrow at the fixed rate. Open-market buys (if enabled) MUST go to an allowlisted marketplace, with an on-chain `price ≤ swap_rate × (1 + [10%])`.
- R-07.2 Any token→SOL conversion MUST enforce on-chain `min_out` derived from a TWAP (≥ [30 min]) and a per-epoch spend cap.
- **Tests:** Buying a listing above the cap fails. A conversion whose output is below the TWAP-derived minimum fails. Spend over the epoch cap fails.

### R-08: Snapshot-proof ticket accrual (B-08)
- R-08.1 Tickets MUST accrue from time-weighted stake in a program vault (or a TWAB checkpoint updated on every balance change). Point-in-time balances MUST NOT determine tickets.
- R-08.2 Eligibility MUST require stake held for ≥ [1 epoch] before `draw_slot`.
- **Tests:** A wallet that stakes 50M tokens for 2 slots before `draw_slot` gets ≤ (2 / period_slots) of the tickets of an equal full-period stake, and 0 if under the minimum hold. Sim target: share ≤ 0.0001% at period 216,000 slots.

### R-09: Sybil-neutral ticket formula (B-09)
- R-09.1 `tickets = floor(twab / TICKET_UNIT)`, linear in stake, with no per-wallet caps, flat, log, or sqrt terms.
- **Tests:** For any split of stake S into k wallets (k ∈ {1, 10, 100, 1000}), total tickets are within k tickets of the single-wallet total (rounding only). The split/unsplit property is fuzz-tested.

### R-10: Launch fairness (B-10)
- R-10.1 The initial sale MUST use a commit window followed by batch clearing at one price, with a per-wallet deposit cap (or an equivalent on-chain anti-snipe mechanism: max buy per wallet and max total per slot for the first [N] slots).
- R-10.2 Trading and sale open MUST be enforced by a slot stored in config at initialization, not by an admin transaction at launch time.
- R-10.3 Initial NFT captures MUST use R-01 selection.
- **Tests:** Deposits in the first slot and the last slot of the window receive the same price. A purchase over the per-slot cap reverts. Opening before `open_slot` fails.

### R-11: Slippage and parameter guards (B-11)
- R-11.1 Every value-moving user instruction MUST take `min_out` or `max_in` and revert when it's violated.
- R-11.2 Capture/release MUST take `expected_amount`, `expected_fee_amount`, `expected_sol_fee` and revert on mismatch.
- **Tests:** Changing a config value between build and execution reverts the user transaction. A simulated sandwich that makes output fall below `min_out` reverts.

### R-12: Reroll economics bounds (B-12)
- R-12.1 An on-chain reroll surcharge and/or cooldown MUST keep `cycle_cost / p_rarest ≥ [2] × published expected premium`.
- R-12.2 Capture MUST NOT let the caller pick a specific escrowed NFT.
- R-12.3 The economic model (`sim/reroll_ev.py`) MUST be re-run with final parameters and attached to the audit package.
- **Tests:** A second reroll inside the cooldown fails. A capture instruction with a caller-supplied NFT mint fails or ignores the input. An automated check computes R-12.1 from config and fails CI if it's violated.

### R-13: Bounded compute everywhere (B-13)
- R-13.1 No instruction may loop over an unbounded account set. Winner selection MUST be O(log N) (prefix-sum tree or merkle proof) and fit within [400k] CU at N = [1M] ticket holders.
- R-13.2 A minimum stake ≥ TICKET_UNIT is needed for eligibility.
- R-13.3 Settlement MUST NOT depend on caller-supplied lists being complete, unless completeness is proven against `draw.total_tickets`.
- R-13.4 All cranks MUST be paginated with a stored cursor and bounded batch size.
- **Tests:** CU benchmark of settle and claim at 1M synthetic holders stays under budget. Omitting accounts from settle either fails or doesn't change the winner. Creating 10,000 dust accounts doesn't change settle cost or outcome.

### R-14: Draw liveness (B-14)
- R-14.1 Each randomness request MUST have `deadline_slot`. After it, a permissionless `cancel_draw` rolls the prize and tickets forward. The same draw can never be re-requested.
- R-14.2 Unclaimed prizes expire after [30 days] into the prize pool through a permissionless sweep. Destination is the prize pool PDA only.
- **Tests:** Cancel before the deadline fails and after the deadline succeeds. Re-request after cancel fails. Sweep before expiry fails, and after expiry goes only to the prize pool.

### R-15: Dependency pinning (B-15)
- R-15.1 Crate versions and git commits MUST be pinned. All external program IDs MUST be hard-coded constants checked with `address =`. VRF accounts MUST be parsed with the provider SDK at a pinned version, with an owner check.
- R-15.2 CI MUST fail if a pinned dependency or program ID changes without a review sign-off.
- R-15.3 Every **[needs source]** item in the threat model MUST be resolved with a primary reference before audit hand-off.
- **Tests:** CI job that diffs `Cargo.lock` and program-ID constants against an approved list.

---

**Gate:** All MUST requirements pass in CI **and** an independent professional third-party audit is complete, with findings fixed, before any mainnet deployment.
