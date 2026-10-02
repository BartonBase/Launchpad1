# Launchpad project brief (v0.1, maintained by Grok Bot, coordinator)

Owner: Barton. Name: **Armory**, ticker **ARMS** (Barton, 2026-10-01 5:13 PM MT; final, chosen regardless of name-check results). Earlier working names: Mintmark, Fuze, Fuse, Hearth, Hawk.fun.
Repo: public GitHub repo BartonBase/Launchpad1 (https://github.com/BartonBase/Launchpad1). Cursor's GitHub connector cannot see it yet; public clone/API reads work.
Chosen visual direction: A "Obsidian" (dark premium fintech), see design/directions/a-obsidian/.

## Status of prior work
- /workspace/launchpad/reference/claude-prototype-v1.html is an earlier Claude-built prototype.
  Barton finds it childish. It is REFERENCE ONLY. Redesign everything from scratch: brand, visuals, UX, copy, structure.
  Reuse only ideas that genuinely hold up (e.g. plain-language "what nobody can change" guarantees, bonding-curve progress to graduation).

## Hard requirements
1. Every token launches with a fixed total supply of exactly 1,000,000,000 (1B), like other Solana launchpads. Mint authority revoked at launch; no one can mint more.
2. Hybrid token/NFT launches (SPL-404 / MPL-404 style via Metaplex MPL-Hybrid): tokens can be swapped into an NFT at a fixed ratio and the NFT unwrapped back into the exact tokens. Ratios must divide 1B cleanly.
3. Token-2022 launches with a transfer tax. The tax buys NFTs from collections and gives them to holders through a lottery.
4. Lottery fairness: tickets = floor(balance / threshold); remainders discarded, so splitting across wallets gains nothing. Minimum holding period / anti-snapshot-sniping. Verifiable randomness (Switchboard or ORAO VRF). No predictable randomness.
5. Security is the top priority. Past incident to learn from: Stonk.fun. Per Bitquery (docs/stonkfun-lessons.md) the 'drain bot' was the platform's own reward wallet selling transfer-tax proceeds into pools, plus $1.41M+ in off-ledger payments to linked wallets and a retained power to raise the tax. It was a design and trust failure, not an external exploit. The distribution/lottery/prize layer gets the heaviest scrutiny.
6. Site: very sleek, modern, premium, easy for newcomers. Mission: revive NFT culture and bridge memes into NFTs.
7. Testnets and throwaway keys only. Professional third-party audit required before mainnet.

## Shared folders
design/ programs/ app/ docs/ tests/ qa/ security/auditor-a/ security/auditor-b/ security/merged/ reference/

## Open issues
- [RESOLVED + APPROVED BY BARTON 2026-09-24, ADR-004 Accepted, see docs/transfer-tax-vs-wrap.md] Two launch types fixed at creation: HYBRID = classic SPL token, no tax, exact unwrap via NFT converter; REWARDS = Token-2022 taxed token funding the NFT lottery, no NFT converter. MPL-Hybrid does not support Token-2022 and is unaudited. Pending auditor review.
  (original issue) Token-2022 transfer fee vs. exact unwrap.
  The transfer-fee extension applies to every transfer, including into and out of the MPL-Hybrid escrow, which breaks
  "convert back for exactly 1,000,000 tokens". Mockup copy "Transfer tax on converts: None*" cannot ship until resolved.
  Options to evaluate: (a) hybrid collections use a non-taxed mint, taxed T22 mints are a separate launch type;
  (b) custom escrow/wrap program that grosses up or compensates for the fee; (c) transfer hook with an exempt-escrow
  allowlist instead of the fee extension; (d) confirm whether MPL-Hybrid supports Token-2022 mints at all.
  Deliverable: written recommendation in docs/ with trade-offs and exploit surface for each option.

## Decisions (2026-09-24)
- Visual direction A (Obsidian) approved; "Mintmark" kept as working name; Creative Director producing more name options.
- Wrap ratio is a creator choice at launch: 10k, 50k, 100k, 200k, 1M tokens per NFT. Collection size (NFT count) is a separate creator setting. Max tokens in NFT form = collection size x ratio, must be <= 1B.
- RARITY MODEL (Barton's decision): cosmetic rarity only. Every NFT in a collection redeems for exactly the same number of tokens (the ratio). Token price sets the NFT floor; rares trade at a market premium. The protocol never promises rares are worth more.
  - Rarity assigned via verifiable randomness (Switchboard/ORAO VRF) or pre-committed/revealed metadata; no predictable randomness.
  - Re-rolling (swap an NFT for a different random one) costs a fee, so farming rares has a real cost. Fee destination TBD.
  - Explicitly REJECTED: random rarity on wrap + larger unwrap payout for rares (bot mint/unwrap loop creates tokens from nothing).
  - To assess: rare cherry-picking from escrow, metadata reveal timing (no peeking at the next NFT before paying), reroll fee sizing.
- [OPEN, owner: Solana Program Engineer] MPL-Hybrid risks: escrow authority can change swap amounts with no timelock (need per-collection multisig + timelock or our own wrapper); capturers can pick which NFT they get; built-in reroll uses predictable randomness. Design in progress: docs/hybrid-rarity-and-assignment.md.
- [OPEN, Barton] Re-roll fee destination for Hybrid launches. Hybrid launches have no lottery (lottery is Rewards-only, assumed, unconfirmed), so the earlier "prize pool" default needs rethinking.

## SCOPE CHANGE (Barton, 2026-09-24 2:51 PM MT) — supersedes anything above that conflicts
- Token-2022 is DROPPED ENTIRELY for now. No Rewards launch type, no transfer tax, no tax-funded NFT buys, no holder ticket lottery.
  Hard requirements #3 and #4 and the Rewards half of ADR-004 are shelved (keep notes; do not build or design further).
- Focus is 100% on SPL-404 hybrid launches: classic SPL token (1B fixed supply, mint authority revoked), token<>NFT conversion (custom hybrid_vault program, Metaplex Core NFTs, ADR-008) with exact unwrap, creator-chosen ratio (10k/50k/100k/200k/1M) and collection size, cosmetic rarity,
  blind NFT assignment, VRF-based re-roll, multisig + timelock on any changeable setting.
- Re-roll fee: default is BURN (Barton skipped the question; burning means the program never custodies a fee pile). Revisit if Barton says otherwise.
- Stonk.fun reference: https://bitquery.io/investigations/is-stonkfun-dumping-on-holders (provided by Barton).
- Re-roll fee is paid in the collection's token and burned. Supply copy: "Fixed at 1,000,000,000 at launch. No one can mint more; re-roll burns can only reduce it." Engineering/QA to verify burns never affect escrow or exact unwrap.
- Token-2022 track / support for existing NFT collections is DEFERRED (not dropped for good). Keep prior design work on file, marked deferred. Supersedes the earlier two-product decision relayed via QA (qa/TEST_PLAN.md INV-16, Q1).
- [DECIDED 2026-09-24, ADR-008 Accepted, reversible if Barton objects] Conversion engine = custom `hybrid_vault` program minting Metaplex Core NFTs. Upstream MPL-Hybrid is NOT in the swap path.
- [OPEN, Barton] Bonding curve: build our own curve program vs. use an existing audited launch venue. Needed before anti-sniping design.
- Copy questions (CD, 5 items) routed to engineer for answers in ADR-008/DECISIONS.md.
- Consistency rules (from Auditor B round 1): re-roll fee = BURNED in tokens (hybrid doc must match); fee destinations and ratio should be IMMUTABLE at init unless Barton decides otherwise; no admin power (e.g. pause) may be listed in ARCHITECTURE without a spec'd instruction, scope, and timelock.

## Decisions (2026-09-24, afternoon)
- Positioning: the "revive NFT culture" mission line is retired. Site focus is SPL-404 tech and showcasing memes and collections.
- Wrap ratio options: 10k, 50k, 100k, 200k, 500k, 1M, 2.5M (CD addition, Barton may drop), 5M tokens per NFT. Max collections 100,000 down to 200. Minimum collection size 100 NFTs.
- hybrid_vault answers (engineer): token fees (capture, re-roll) always burned by the program via SPL burn, never sent to a wallet; ratio locked with no update/close instruction; release returns exactly N tokens with no token fee; creator fee (if any) in SOL, fixed at launch, capped, charged only at capture (amount TBD by Barton). Until post-audit freeze, program upgrades require a 3-of-5 multisig with a 7-day delay, disclosed in copy.
- Secondary NFT trading: Tensor and Magic Eden at launch; no in-house marketplace. Engineering to confirm full Metaplex Core support (list, buy, traits) and real collection URL patterns for trade buttons.
- Repo: on-chain work pushed to BartonBase/Launchpad1 branch onchain/hybrid-launch (commit c02be68), currently the default branch.
- GRADUATION MODEL (Barton, 3:22 PM MT): ONE graduation event. During the bonding curve the token trades as a plain SPL token and converting is closed. When the curve fills: (a) liquidity migrates to a DEX, (b) the full NFT collection (Metaplex Core assets via hybrid_vault, not stock MPL-Hybrid) is minted into the vault escrow, funded from a slice of graduation proceeds, not by the creator up front, (c) converting opens, (d) the collection goes live on Tensor/Magic Eden. Traits/rarity previewable during the curve; art revealed at graduation.
  - Open questions routed to engineer + QA: per-NFT Core mint cost (rent + tx) to size the graduation fee; whether collection mint + escrow deposit for N NFTs can be done reliably at graduation (it cannot be one transaction for large N, so batching/crank design and "converting opens only when fully minted" are required) and a safe max collection size; the graduation threshold (market cap or SOL raised).
  - Pending Barton: whether to drop the 10k/50k ratios or cap collection size (e.g. 10,000), since at a ~500 SOL cap the 10k ratio leaves ~0.005 SOL per NFT, about the per-mint cost.
  - Security note: art revealed at graduation must match the pre-graduation metadata commitment (ties to auditor finding on commitments excluding image bytes).

## Decisions (2026-09-25)
- FEE CHANGE (Barton, 4:27 PM MT), supersedes "re-roll fee = BURN": a 2% fee on capture (token-to-NFT mint) and on every re-roll, paid to Barton. Implementation rules so it cannot repeat Stonk.fun:
  - 2% of the collection ratio, paid in the collection's tokens, on top of the N tokens deposited, so the escrow still holds exactly N per NFT and release still returns exactly N with no fee.
  - The program transfers it directly to ONE fee address fixed at launch and shown on every page. No intermediate wallet, no routing, no selling by the program.
  - Rate hard-capped at 2% in code; can never be raised, not even by the multisig. Destination cannot be changed after launch (or only via multisig + 7-day timelock, TBD).
  - Re-roll also carries a small fixed SOL minimum so re-rolls are not nearly free while the token price is low (Auditor B finding).
  - Recommended: the fee address is a multisig Barton controls, not a personal hot wallet (Auditor B-07).
- PRIORITY (Barton): fix all audit findings, criticals first, and do everything possible to prevent financial exploits. Product must be functional end to end on devnet.
- Bonding curve: Barton skipped the question, so default is an existing audited curve (Meteora or Raydium). Engineer evaluates which fits graduation, anti-sniping, and LP custody.
- FEE CHANGE v2 (Barton, 4:49 PM MT), SUPERSEDES the 4:27 PM 2% token fee: NO token fee at all. A flat SOL fee paid to Barton's fee wallet on every capture (token to NFT), release (NFT to token), and re-roll. Proposed 0.01 SOL each (Barton can adjust).
  - Re-roll (0.01) < release+capture (0.02), closing the M-06 bypass. Flat per-attempt fee is the anti-grind floor (M-19).
  - Release still returns exactly N tokens; fee is SOL only. Escrow stays exactly N per NFT.
  - Fee fixed per launch in LaunchConfig with a hard cap in code (M-08). "Can't be raised" copy only with the beta caveat (3-of-5 multisig + 7-day delay until post-audit freeze).
  - Fee wallet should be a multisig; published policy that the fee wallet never converts or re-rolls (M-16). Fee account must be checked against LaunchConfig (F-06).
  - Charging a SOL fee on release must never be able to block release (M-26).
  - Open for engineering: VRF cost per request (paid from the fee or separately?); whether any Metaplex protocol fee applies per capture/release with hybrid_vault + Core.
- Pending Barton: fee amount (default 0.01 SOL); drop 10k (and maybe 50k) ratios and cap collections at 10,000 NFTs (recommended by CD and engineer); curve = Meteora DBC (engineer recommendation); handling of DBC's 25% unsold buffer.
- Merged audit list: security/merged/round1-merged.md (35 findings, M-01..M-35; criticals M-01 admin changes, M-02 NFT picking/predictable re-roll, M-03 curve sniping). Engineer works from it.
- Graduation/marketplace answers: docs/graduation-design.md and docs/marketplaces-and-ratios.md.
- FEE REFINEMENT (Barton, 4:52 PM MT), refines FEE CHANGE v2: the flat SOL fee on capture, release and re-roll is TIERED BY RATIO, fixed per collection at launch in LaunchConfig, immutable, hard cap 0.01 SOL in code. Tiers: 10k/50k = 0.002 SOL; 100k/200k = 0.005 SOL; 500k/1M/2.5M/5M = 0.01 SOL. Not tied to live market cap (rejected: gameable, needs an oracle). Engineering to confirm 0.002 SOL covers per-request VRF cost and that re-roll <= release+capture in every tier. Dropping 10k still undecided.
- GRADUATION DECISIONS (Barton approved, 4:55 PM MT):
  1. Bonding curve = Meteora DBC.
  2. DBC's 25% unsold buffer is locked in a program-owned account, not burned; supply stays exactly 1B.
  3. 10k ratio DROPPED. Ratios: 50k, 100k, 200k, 500k, 1M, 2.5M, 5M. Fee tiers: 50k = 0.002 SOL; 100k/200k = 0.005 SOL; 500k+ = 0.01 SOL.
  4. Collections capped at 10,000 NFTs (minimum 100).
  5. Launches whose graduation target can't fund minting the full collection (with 25% margin) are refused at launch.
- PAUSE REMOVED (Barton, 4:57 PM MT): no pause/guardian switch. Nobody can halt capture, re-roll or release. Upgrades still via 3-of-5 multisig + 7-day timelock. Resolves merged M-26.
- RELEASE IS FREE (Barton, 5:04 PM MT): release charges no SOL fee and returns exactly N tokens. Tiered SOL fee (50k = 0.002; 100k/200k = 0.005; 500k-5M = 0.01; cap 0.01) applies only to capture and re-roll; re-roll fee = capture fee. VRF cost paid by the requester at cost, separate from the tier fee.
- Merged audit list v2: security/merged/round1-merged.md, M-01..M-40, both auditors agree. Auditor B fee stress test: security/auditor-b/flat-fee-stress-test.md.
- Obsidian pages updated for 4:55 PM decisions (CD). Launch fee, 1% curve trade fee and 0.25% DEX fee are PLACEHOLDERS pending Barton.
- LAZY MINTING (Barton, 5:13 PM MT), REPLACES the 4:55 PM batch pre-mint: nothing is minted at graduation. Each Core NFT is minted the first time a settle picks it; the capturer pays rent + Core fee (~0.005 SOL) on top of the tier fee, escrowed at request. The refuse-unfundable-launch rule (4:55 item 5) is DROPPED. The 10,000 cap and 100 minimum stay. Tradeoff: at graduation, marketplaces show only NFTs captured so far. Design: remove the mint-reserve line and the "collection too large for target" blocking error; show the per-capture mint cost instead.

## Decisions 2026-10-01 (Barton)
- NAME: Armory, ticker ARMS (5:13 PM MT). Replaces Mintmark/Fuze/Fuse. Final regardless of the name check. Domain and handle picks pending the check.
- MASCOT: a comic-looking knight (5:13 PM MT). Replaces the comic bomb (4:51 PM), which replaced Claude's fused coin/card creature. Comic-book ink style, dark palette, exactly one bright accent. Logo, favicon and app icon derive from the knight. Still images only, no animation (4:55 PM). 5:14 PM: NO mascot imagery from the team. Barton will make the mascot in Higgsfield. All mascot-related visuals (sheet, logo, favicon, icon) are on hold until he supplies them; pages use a placeholder.
- FRONTEND PLAN: docs/ITINERARY.md (12 steps), based on Claude's "Frontend design brief & itinerary" PDF, with animation dropped.
- LAUNCH TYPES: plain, hybrid and burn are live in the design. Tax split and raffle are shelved (ADR-020) until there's a public buy path, the stuck-funds fixes, and a raffle legal check, but the site is built to accept them later.
- modes-1-5 fixes on local branch fix/modes-1-5. Round cancel and pot rollover approved. Push as its own branch only after QA's suites are green, never into main.

