# Admin powers, multisig + timelock, upgrade authority (Track A)

> **UPDATE 2026-09-25: there is NO pause (ADR-015).** §3's `pause_new_requests` proposal is rejected and kept for the record. The only admin power left is the program upgrade (Squads + timelock until freeze; M-16 accepted). Devnet deploys go through `scripts/deploy-devnet.sh`, which refuses to deploy if a test-only artifact (`mock_switchboard.so`, the mock-graduation marker) is in `target/deploy`.

Owner: Solana Program Engineer. Status: **Proposed** (2026-09-24, after ADR-009). Nothing here is deployed; localnet
and devnet only. Constraint source: ADR-009 §C (Stonk.fun lessons 2–3: no operator wallet custodying value, no mutable
economics after launch; if anything must stay adjustable, use a multisig plus a long, visible timelock with hard caps).

## 1. Principle

**Prefer no admin power at all.** Where one exists, it must be (a) behind a Squads multisig, (b) behind a timelock or
an automatic expiry, (c) able only to **reduce** risk, and (d) never able to block a holder's exit (unwrap/release),
move funds, or change economics.

## 2. Inventory: which admin powers exist

| Surface | Power | Status | Why |
|---|---|---|---|
| Token mint | Mint authority | **None** (revoked in `hybrid_launch::launch`, same tx) | ✅ tested |
| Token mint | Freeze authority | **None** (never set) | ✅ tested |
| `hybrid_launch::LaunchConfig` | Change ratio, collection size, fees, fee destination, mint | **None**: no update/close instruction | ✅ `config_has_no_mutation_or_close_instruction_in_idl` |
| Fee destination | Redirect fees | **None**: the recipient is the `PLATFORM_FEE_RECIPIENT` code constant; changing it needs a program upgrade (multisig + timelock) | ADR-013 |
| Vault / escrow (engine) | Withdraw, sweep, change backing ratio | **None**. Only `release` moves backing, and only exactly `ratio` to an NFT holder | ADR-008, T-HV-10 |
| NFT collection (engine) | Metadata/plugin updates | **None** after the trait root is committed (PDA update authority, no update ix) | T-HV-06 |
| Engine | **Pause new captures/re-rolls** | **Only candidate power (open, N4).** See §3 | Risk-reducing only |
| Programs | Upgrade | Squads multisig + timelock, then frozen | §5 |

Anything not in this table does not exist. The BRIEF consistency rule applies: no admin power may be listed in
ARCHITECTURE without a spec'd instruction, scope and timelock.

## 3. The one candidate power: `pause_new_requests`

- **Scope.** Blocks only `request_capture` and `request_reroll` (new money coming in). It **never** blocks `release`
  (unwrap), `settle` of already-paid requests, or `expire`. A holder can always get exactly `ratio` tokens back for an
  NFT, and a paid request always completes or refunds.
- **Why keep it at all.** If a bug is found in selection or VRF handling, pausing new requests limits how many users
  are exposed while a fix goes through the upgrade timelock. It can't hurt existing holders.
- **Why maybe not.** Every power is a trust assumption, and the simplest honest claim is "nobody can change
  anything". If Barton prefers that, delete it (N4).
- **Anti-abuse.**
  - Auto-expiry: a pause lapses after `MAX_PAUSE_SLOTS` (proposal: ~7 days) and can't be renewed back-to-back without
    a new timelocked proposal, so it can't become a permanent freeze of new captures.
  - Emits an event and is shown on the token page ("New captures paused until …; unwrapping is unaffected").
  - Stored per collection (engine config), not global.
- **Timelock tension.** A timelocked emergency pause is useless in an emergency. Two options for Barton:
  - (a) **Guardian multisig without timelock**, allowed to call only `pause_new_requests` (auto-expiring), plus the
    governance multisig with a timelock for everything else (upgrades, unpause-before-expiry). Recommended: pause only
    reduces risk and can't touch exits.
  - (b) Everything through the timelocked governance multisig (simpler, but pause arrives days late).

## 4. Multisig and timelock mechanics

- **Squads v4** (program `SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf`). The `Multisig` account has a built-in
  `time_lock: u32`: "How many seconds must pass between transaction voting settlement and execution"
  (https://docs.squads.so/main/development/reference/accounts, https://docs.squads.so/main/development/reference/time-locks).
  It's set at `multisigCreateV2` (`timeLock`) or changed with a `SetTimeLock` config transaction, which itself goes
  through the vote (and makes pending transactions stale).
  - Use `configAuthority = null` (autonomous multisig), so changing members, threshold or the timelock also needs a
    vote and the existing timelock.
  - The upgrade authority of our programs is the multisig's **vault PDA**, not a member key.
- **Built-in delay vs Squads timelock.** A program-level `propose → wait → execute` (as in the old ADR-008
  `propose_fees`) is only needed for parameters that change. Under ADR-009 no economic parameter changes, so there is
  nothing to delay in-program. Squads' timelock covers upgrades and (option b) pause.
- **Proposed defaults (need Barton, Q8/N5):**
  - Governance: 3-of-5 (members TBD, hardware wallets, at least one outside signer), `time_lock` = 7 days (604,800
    s). That's longer than a typical holder's reaction time and long enough for auditors to review an upgrade.
  - Guardian (option a): 2-of-3, no timelock, can only call `pause_new_requests`.
- **Visibility.** The site shows pending governance transactions (Squads proposals targeting our programs) with their
  execute-after time (QA INV-16, INV-12).

## 5. Upgrade-authority policy

1. **Localnet/devnet:** throwaway keys in `.keys/` (gitignored). No mainnet keys on the box.
2. **Mainnet launch (after the third-party audit, BRIEF #7):** deploy with a verifiable build (`solana-verify` /
   `anchor build --verifiable`), publish the build hash and the audited commit. Set the upgrade authority to the
   governance Squads vault PDA (7-day timelock). `assert_launch_ready` (QA SUP-06) fails if the upgrade authority is
   an EOA.
3. **Stabilization:** a fixed period (proposal: 3 months without critical findings), then set the upgrade authority
   to `None` (`solana program set-upgrade-authority --final`). After that, programs can't change.
4. `hybrid_launch` is small and has no admin; it's the first candidate to be frozen.
5. If MPL-Hybrid is chosen as the engine, its upgrade authority (`mp14o4AQ…`, Metaplex) is outside our control: an
   accepted risk to disclose (T-HY-06).

---

## 6. OPEN DESIGN ITEM: bonding-curve launch-window anti-sniping

> **Status: OPEN.** The curve implementation isn't decided (BRIEF: own curve program vs an existing audited venue,
> N6). This section lists options and trade-offs so the choice can be made; it's not a spec.

**Why it matters (Stonk.fun, Bitquery 2026-09-22):** the first minutes are where snipers dominate. On one launch 522
wallets traded 3.4× the supply within a minute, and ZCAT lost 20% of its supply to tax in its first hour. We have no
tax, but the same bots would take the cheapest part of the curve and dump on later buyers.

| Mechanism | How it works | Strength | Weaknesses | Available in |
|---|---|---|---|---|
| **Launch delay after pool creation** | Pool is created, trading opens at a stored `open_slot`/`activation_point` | Stops bots that snipe the creation tx in the same block | Alone it just moves the race to `open_slot` (Jito bundle auctions decide who lands first) | Raydium LaunchLab `open_time` (https://docs.raydium.io/products/launchlab/instructions); Meteora DAMM v2 `activation_point` (https://docs.meteora.ag/developer-guides/damm-v2/pool-fee-configs) |
| **Per-wallet caps** | Max buy per wallet during the window | Simple, easy to explain | **Sybil-able** (wallets are free); only meaningful with per-tx and per-slot caps; per-wallet state costs rent | Own curve; prototypes such as Dripz (per-tx cap + rolling wallet window, https://github.com/gryd-systems/gryd) |
| **Per-tx / per-slot caps** | Max buy per transaction and total per slot in the first N slots | Limits how much supply any one block can take, regardless of wallet count | Legit buyers also capped; per-slot cap can be filled by a bot bundle (needs to be small) | Own curve |
| **Commit-reveal / batch auction** | Buyers commit (hidden amount + bond) during a window; after it closes, everyone clears at one uniform price | Strongest: ordering gives no advantage, so sniping has nothing to win | Two transactions; non-revealers must forfeit a bond; needs our own program and an audit; pro-rata fills | Own program; prototypes such as Veil (https://github.com/priyanshpatel18/veil), SealBid (https://github.com/ytuDevs/private-auction) |
| **Decaying fee in the first N slots** | High "cliff" fee at open that decays linearly or exponentially | Makes the earliest flips expensive; well-understood | Fee proceeds must go somewhere: **must be burned or go to LP, never to an operator wallet** (ADR-009 C2); snipers can wait for the decay | Meteora DBC/DAMM v2 fee scheduler (https://docs.meteora.ag/core-products/dbc/fees/fee-scheduler, https://docs.meteora.ag/core-products/damm-v2/fees/time-scheduler) |
| **Size-based fee (rate limiter)** | Fee rises with buy size during the window, one swap per pool per tx | Targets large early buys | Implementation risk: Code4rena found a `swap2` bypass of the single-swap check in Meteora DBC (https://code4rena.com/reports/2025-08-meteora-dynamic-bonding-curve) | Meteora DBC rate limiter (https://docs.meteora.ag/core-products/dbc/fees/rate-limiter), max 12 h |
| **Alpha vault / pre-activation pro-rata** | Users deposit quote during a window; a vault buys before activation and distributes pro-rata with vesting | Fair to depositors; bots gain nothing from speed | Adds vesting/claim UX; still needs a public-open anti-snipe | Meteora Alpha Vault (https://docs.meteora.ag/helper-products/alpha-vault/what-is-alpha-vault) |
| **Creator/dev buy limit** | Cap and disclose the creator's own buy at launch | Addresses single-wallet launches | Creator can use other wallets; disclosure matters more than the cap | Venue config / our launch wizard |

**Engineering leaning (not a decision):**
- **Third-party venue:** Meteora DBC with `activation_point` delay + exponential fee scheduler (+ optional rate
  limiter), verifying the deployed program includes the audit fixes, with fee proceeds going to LP or burn. The 1B
  supply goes from `hybrid_launch`'s destination straight into the venue's vault (N2).
- **Own curve:** a short commit window with a uniform clearing price (batch) for the opening, then the continuous
  curve with small per-tx and per-slot caps for the next N slots. `open_slot` is stored at init and immutable (QA
  INV-15).
- Either way: parameters fixed at init, shown on the site, and tested (QA SN-xx).

Questions for Barton: venue (own vs Meteora DBC vs Raydium LaunchLab), window length, cap values, and whether the
anti-sniper fee (if any) burns or goes to LP.
