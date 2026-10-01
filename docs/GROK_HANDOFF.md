# Handoff for the next Grok — Fuze / Launchpad1

Written 2026-10-01 for Barton (@BartonClips). This file is the source of truth for work done **after** git HEAD. Read it before editing.

## Hard rules

- Repo: `BartonBase/Launchpad1`. Local path: `/workspace/Launchpad1`.
- Branch: `onchain/hybrid-launch`. **HEAD is still `df0d1d9dad9b5c1d832c1ab76c417144bd69b94e`.**
- Modes 1–5 exist only as a **dirty local tree** (54 paths: modified + untracked). Do **not** commit, push, open a PR, or change GitHub unless Barton says so in that chat.
- Do **not** deploy. Do **not** start an audit. Do **not** build a frontend.
- Upgrade authority stays the throwaway deployer, **not** the Squads timelock.
- Do **not** change Mode 2 (`launch` / `LaunchConfig` / unwrap). Unwrap still returns the NFT to inventory.
- Do **not** add a $20 holder minimum, a live USD price, a 5-minute clock, or a caller-chosen payout amount, NFT, order, or wallet.
- Do **not** add Mode 5 recommit-on-stuck-oracle unless Barton asks. A Switchboard commit that never reveals can stall that round's pot. Mode 2 already has `recommit_randomness`. Mode 5 does not.
- Product name in conversation: Fuze. Code names: `hybrid_launch`, `hybrid_vault`.

## Programs

| Program | Id | Notes |
|---|---|---|
| `hybrid_launch` | `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8` | Creates the mint and the immutable launch record |
| `hybrid_vault` | `BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG` | NFTs, tax, buyback, split, raffle, Mode 2 draws |

Devnet copies of these ids exist from **2026-09-25/26** and do **not** contain Modes 1–5. Do not treat devnet as this tree. `docs/STATUS.md` still describes that older devnet deploy in its Open section.

## What each mode is

Supply is always 1_000_000_000 whole tokens. Ratios that divide 1B: 50k, 100k, 200k, 500k, 1M, 2.5M, 5M. Minimum collection size 100. The 10k ratio was dropped.

### Mode 1 — plain SPL (`launch_mode = 1`)

- `launch_plain` writes `PlainLaunchConfig`, not `LaunchConfig`. Classic SPL. Mint and freeze revoked. No update instruction.
- `register_plain_dbc` is the Meteora curve path. Config must be in `APPROVED_DBC_CONFIGS`. The mint is typed as classic SPL, so Token-2022 is rejected.
- No ratio, collection, wrap, unwrap, or NFT. `init_vault` still loads `LaunchConfig` only, so a Mode 1 mint cannot open a Mode 2 vault.

### Mode 2 — reversible hybrid (unchanged)

- `launch` / `register_dbc_launch` / `LaunchConfig` v4.
- Wrap locks tokens and mints an NFT. Unwrap returns the NFT and the tokens. Switchboard draws for capture and reroll. Flat SOL fee. This mode was already on HEAD. Leave it alone.

### Mode 3 — burn hybrid (`launch_mode = 3`)

- `launch_burn`, `register_burn_dbc`, `init_permanent_vault`, `open_permanent_vault`, `wrap_permanent`.
- Wrap burns exactly the ratio and mints the next NFT. No unwrap. Index is sequential, not a Switchboard draw.
- Account type is `BurnLaunchConfig`, not `LaunchConfig`.

### Mode 4 — Token-2022 tax, equal split (`launch_mode = 4`)

Barton rejected burning. This mode does **not** burn.

- `launch_token22` creates a Token-2022 mint. The only extension is a transfer fee. `tax_bps` is chosen at launch, range **1..=1000**, then the fee-config authority is `None`. The rate cannot change.
- Withdraw authority is the vault PDA `["tax_authority", mint]`. Only `hybrid_vault` can sign it.
- `init_token22_vault` requires mode 4. A mode 5 mint cannot open a split vault.
- `harvest_tax` pulls withheld fees onto the tax treasury. `buyback` spends SOL **already on that same PDA** to buy tokens from this launch's own inventory at `buyback_lamports_per_whole` (1 lamport ..= 1 SOL per whole token, locked at launch). This is **not** a DEX or Meteora swap. There is **no** Token-2022 curve register. DBC registration was removed because a Meteora curve cannot be forced to use the PDA as withdraw authority.
- `wrap_token22` locks the ratio and mints the next NFT. Token supply does not decrease.
- Distribution is **per minted NFT**, not per wallet and not per collection size. 10 minted NFTs get 10 shares. Unminted NFTs get nothing. An NFT minted after a round freezes gets none of that round.
- The program pays the round itself: NFT index `payout_cursor`, then cursor + 1, and so on. Destination is the derived ATA of the **current Core owner** of that asset. `claim_tax` pays only that NFT. Out of order returns `OutOfOrder`. A closed round pays nothing. `kind` must be `KIND_SPLIT` (0) or `claim_tax` returns `WrongRequestKind`.
- Dust that does not divide evenly stays in `pending_base`. `paid_base` cannot exceed `credited_base`.
- No admin withdraw. No caller-chosen amount, index, or destination.

### Mode 5 — raffle (`launch_mode = 5`)

Same mint, same locked tax, same locked buyback price, same market-cap pot tiers as Mode 4. **Separate launch.** Mode 4 still splits.

- `launch_raffle` writes the same account type as Mode 4 (`T22BurnLaunchConfig`) with `launch_mode = 5`.
- `init_raffle_vault` sets `kind = KIND_RAFFLE` (1). It also stores the Switchboard queue.
- When the pot crosses the tier and at least one NFT is already minted, the whole booked amount becomes `round_pot` (not a per-NFT share). `round_id` increments **before** seats are written. Phase becomes `RAFFLE_SNAPSHOT` (1).
- `snapshot_raffle` walks `payout_cursor` from 0 to `round_minted - 1`. Each seat is PDA `["raffle_seat", vault, round_id_le, index_le]` and stores the Core owner **before any randomness exists**. Phase becomes `RAFFLE_READY` (2) only when every existing NFT has a seat. You cannot skip ahead. You cannot commit early.
- `init_raffle_randomness` then `commit_raffle`. The program selects the oracle (`select_oracle`, sequence = `round_id`). Commit is allowed only in READY. A `RandLock` seeded `["rand_lock", randomness]` with `seq = round_id` stops the same randomness account being reused.
- `reveal_raffle` stores the value and sets `RAFFLE_REVEALED` (4). A second reveal is rejected. There is **no** second draw because someone dislikes the result.
- `settle_raffle` re-reads Switchboard, requires the stored value, derives the index with `request_randomness(revealed, vault, round_id)` then `uniform_below` (rejection sampling), and pays `round_pot` only to that seat's snapshotted owner. Buying the winning NFT after the number is public does not move the prize. No instruction argument chooses the index, the amount, or the wallet.
- `claim_tax` on a raffle vault returns `WrongRequestKind`.
- More NFTs means more tickets, not a larger share. One NFT that existed when the round froze wins the **whole** pot.
- Token-2022 transfer fee uses **ceiling** division. A test that does `pot * bps / 10_000` is wrong by a few atoms. The Mode 5 test uses ceiling. Also, `pending_base % minted_count` is **not** part of `round_pot` and stays on the treasury. Do not assert the treasury is zero after settle.

### Pot tiers (Modes 4 and 5)

Implemented in `pot_threshold_lamports` (`programs/hybrid_vault/src/instructions/token22.rs`).

Market cap and pot are both valued at the **locked buyback price**. There is no USD oracle. Dollar labels assume a fixed **1 SOL = $100** and are comments, not an input.

| Market cap (at that price) | Pot must reach |
|---|---|
| under 500 SOL ($50k) | 0.5 SOL |
| 500 SOL up to 1000 SOL | 2 SOL |
| 1000 SOL up to 1250 SOL | 2.5 SOL |
| 1250 SOL and up | 0.1% of market cap (`mcap/1000`), capped at 500 SOL |

No $20 minimum. At the usual test price (`1_000_000` lamports per whole token, 6 decimals, 1B supply) market cap is huge, so the **500 SOL cap** applies. A 10-whole-token buyback does not open a round. A 600_000-whole-token buyback does (5% fee → 570_000 net = 570 SOL).

Stonk.fun, for contrast only: an operator wallet sweeps a 1% or 3% tax, sells into the pool, and pays holders in the pair asset if they hold about $20. Their thresholds were published and can be changed. This program does not sell tax into a pool and has no operator wallet.

## Raffle phases

On `TaxVault`: `kind` 0 = split, 1 = raffle.

`raffle_phase`: 1 snapshot, 2 ready, 3 committed, 4 revealed. After a successful settle the phase and `round_open` and `round_pot` go back to 0.

## Pump.fun (decided 2026-10-01, not built)

Barton asked if this launch and graduation curve can be pointed at Pump.fun because he may try to sell the product to them. Answer already given: **no drop-in.**

- Modes 1–3 graduate through Meteora DBC (`APPROVED_DBC_CONFIGS`, pool must have migrated, leftover goes to a locked `["dbc_buffer"]` PDA). Pump's program is `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`. Graduation is their migration into PumpSwap `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`. Their global config sets the threshold and fees. `APPROVED_DBC_CONFIGS` cannot name them.
- Pump creates the mint. A third party cannot put a locked Token-2022 transfer fee on it or set this vault PDA as withdraw authority. A sidecar that watches Pump trades can be bypassed.
- Do not pitch a replacement curve. The sellable piece is Modes 4 and 5: a locked cut that buys the coin back and pays NFT holders (equal split, or one snapshotted NFT wins the pot) with no operator wallet. That only works if Pump puts it **inside their fee path** and locks the recipient. Their creator-fee share is paid to recipients they configure; if the creator can change that list later, it is the Stonk hole this design closed.
- Do not start a Pump integration unless Barton asks.

## Toolchain and how to build

- Anchor 1.2.0, Rust 1.89.0, SBPF v2, `cargo-build-sbf --tools-version v1.54 --arch v2`.
- PATH must include `/root/.local/share/solana/install/active_release/bin` and `/root/.cargo/bin`.
- Do not run a bare `anchor build`. It wants platform-tools v1.57.
- Production launch `.so`: `/workspace/Launchpad1/target/deploy/hybrid_launch.so` (no test feature).
- Vault test `.so`: `/workspace/Launchpad1/target/test-sbf/hybrid_vault.so` **with** `--features test-mock-graduation`. Both `--locked`.
- `test-mock-graduation` must never ship. Production builds refuse it. The Switchboard stand-in is `tests/track-a-hybrid/mock-switchboard` loaded **at the real Switchboard program id in tests only**. The mock does not check secp256k1. Tests use signature `[7u8; 64]`, recovery id 0.
- `init_randomness` loads a Mode 2 `Vault`, not a `TaxVault`. Mode 5 uses `init_raffle_randomness`.

IDL regen (env var is per command, not sticky):

```bash
export PATH="/root/.cargo/bin:/root/.local/share/solana/install/active_release/bin:$PATH"
cd /workspace/Launchpad1
ANCHOR_IDL_BUILD_SKIP_LINT=TRUE cargo test -p hybrid_launch --features idl-build __anchor_private_print_idl -- --exact
ANCHOR_IDL_BUILD_SKIP_LINT=TRUE cargo test -p hybrid_vault --features idl-build __anchor_private_print_idl -- --exact
```

Then assemble stdout into `target/idl/hybrid_launch.json` and `target/idl/hybrid_vault.json` with `/tmp/assemble_idl.py` if it is still there. Without `ANCHOR_IDL_BUILD_SKIP_LINT=TRUE` the vault IDL build fails a CHECK lint on `raffle.rs` (`mint` is unsafe) and the error is reported against the wrong crate.

Current IDL instruction names:

- launch (9): `buy_inventory`, `launch`, `launch_burn`, `launch_plain`, `launch_raffle`, `launch_token22`, `register_burn_dbc`, `register_dbc_launch`, `register_plain_dbc`
- vault (27): `buyback`, `claim_tax`, `commit_raffle`, `expire_request`, `expire_requests`, `harvest_tax`, `init_permanent_vault`, `init_raffle_randomness`, `init_raffle_vault`, `init_randomness`, `init_token22_vault`, `init_vault`, `merge_incoming`, `open_permanent_vault`, `open_vault`, `recommit_randomness`, `request_capture`, `request_reroll`, `reveal_raffle`, `reveal_randomness`, `settle_capture`, `settle_raffle`, `settle_reroll`, `snapshot_raffle`, `unwrap`, `wrap_permanent`, `wrap_token22`

Tests assert those name lists. If you add an instruction, update both IDLs and the expected vectors in `tests/track-a-hybrid/launch/launch.rs` and `tests/track-a-hybrid/vault/vault.rs`.

## Tests last green (2026-10-01, after Mode 5)

```bash
cargo test -p track-a-hybrid-tests --test vault --offline -- --test-threads=8
```

- Vault LiteSVM: **83 passed**, including `mode5_one_snapshotted_nft_wins_the_whole_pot_and_the_draw_cannot_be_aimed`.
- Before Mode 5, also green on this tree: launch 30, dbc graduation 16, real Switchboard 4.
- Host units: `hybrid_launch` 19, `hybrid_vault` 15 (includes `pot_threshold` unit tests).
- The full launch suite was **not** re-run after the Mode 5 vault test fix. Launch program code did not change in that fix. Do not claim launch 30 is freshly green unless you run it.
- `qa_launch` and `qa_regression` are not on this branch. Do not expect those counts from `docs/STATUS.md`'s older "at HEAD" paragraph. That paragraph describes `df0d1d9` before Modes 1–5.
- `docs/STATUS.md` top section was updated for Modes 4 and 5 and the 83 vault count. Its later "Open / devnet / QA" sections still describe the pre-mode-1 HEAD. Do not "fix" them by deleting history.

Useful test entry points: `tests/track-a-hybrid/vault/harness.rs` (`launch_tax`, `setup_raffle`, `snapshot_raffle`, `commit_raffle`, `reveal_raffle`, `settle_raffle`). Decimals in those tests are 6. `RATIO_WHOLE = 1_000_000`.

## Files added for Modes 1–5 (untracked)

Launch:

- `programs/hybrid_launch/src/instructions/plain.rs`
- `programs/hybrid_launch/src/instructions/launch_burn.rs`
- `programs/hybrid_launch/src/instructions/launch_token22.rs`
- `programs/hybrid_launch/src/instructions/register_plain_dbc.rs`
- `programs/hybrid_launch/src/instructions/register_burn_dbc.rs`
- `programs/hybrid_launch/src/t22.rs`

Vault:

- `programs/hybrid_vault/src/instructions/permanent.rs`
- `programs/hybrid_vault/src/instructions/token22.rs`
- `programs/hybrid_vault/src/instructions/raffle.rs`

`lib.rs` on both crates exports the new instructions. Many existing files are modified (state, constants, config, errors, tests, `docs/STATUS.md`, `docs/ARCHITECTURE.md`). `git status --short` is the list. `docs/ARCHITECTURE.md` is partly stale (it has said Mode 3 was not built). Trust this handoff and the code over that sentence.

## Trust rules that must survive any edit

- PDA authorities only. Mint authority and freeze authority revoked inside the launch instruction.
- Launch parameters are immutable. No update, no close, no admin withdraw of inventory, tax, or the DBC buffer.
- No caller-chosen outflow. Mode 4 cursor and Mode 5 seat math are the whole policy.
- Mode 4 `init_token22_vault` must keep rejecting mode 5, and mode 5 must keep rejecting `claim_tax`.
- Randomness is only Switchboard On-Demand. Do not use slot hashes, clock, or blockhash as the draw.
- `getrandom` on SBF is registered to always fail. The program must not need OS randomness.

## Owner decisions, in order (2026-10-01)

1. Build Mode 1 only. Do not disturb Mode 2. Do not start Mode 4 or 5. No frontend. No GitHub writes.
2. Then Mode 3 (burn hybrid), after Barton asked for the next mode.
3. Then the next mode was **not** burn. Token-2022 tax, rate chosen at launch and locked. Tax buys the token back. Tokens go to NFT holders of **this project only**.
4. Buyback of launch inventory at a locked price. Not a pool sale.
5. Payout scales with how many NFTs a wallet holds: each minted NFT gets one share, so 10 NFTs get 10x one share. Unminted NFTs earn nothing until they exist. Later mints miss earlier rounds.
6. The program pays. No person, bot, or crank may choose the amount, the destination, or the order.
7. Pot opens on Stonk-style market-cap tiers. **Ignore** Stonk's $20 minimum. No live price the caller can set.
8. Mode 5 was reviewed in chat **before** any Mode 5 code. Barton then said "Build it."
9. Mode 5 spec that was built: one NFT that already existed when the round froze wins that round's whole pot (harvested tax plus buyback tokens). More NFTs = more tickets. Owner snapshotted before randomness. Switchboard commit-reveal pinned to that round. Rejection sampling. No admin. Crank cannot pick the winner. No second draw. Mode 4 stays an equal split.
10. Pump.fun was a question only. Do not build it unless a later message asks.

## Known gaps (do not "helpfully" close)

- Mode 5 pot can stall if the chosen oracle never reveals. Recommit was considered and **not** built.
- Modes 4 and 5 have no bonding curve. Buyback only sells inventory the launch PDA already holds, and only if the SOL is already on the tax PDA.
- Not audited. Not deployed. Local LiteSVM only.
- `docs/DECISIONS.md` ADR-009 still shelves Token-2022 and the holder lottery. The code has now built both, as Modes 4 and 5, with different rules than that shelved ADR. Do not revive Track B from `docs/ARCHITECTURE.md` (the old holder-lottery sketch). Mode 5 is the implementation. Track B stays shelved.
- Full vault suite was re-run after Mode 5 (83). Launch suite was not re-run after the fee-assertion edit.

## If Barton asks for something new

Confirm the spec in chat before writing a new mode. He required that for Mode 5. One sentence of what you will not change (Mode 2, Mode 4 split, no GitHub) belongs in that confirmation.
