# Research Sources (Auditor B)

**Prepared for:** Barton
**Started:** 2026-09-24 (MDT)
**Rule:** Only sources actually read are cited, with URLs. Anything not found is stated plainly. Relevance notes reference findings B-01..B-15 in `threat-model.md`.

## 5. MPL-Hybrid: how the metadata pick works on capture when reroll is on (most important)

**Sources read:**
- Source: `git clone https://github.com/metaplex-foundation/mpl-hybrid`, HEAD = `aacf1a53c8a43bf395db7b562c02cf016ce604c5` (committed 2026-05-27; the latest commit on `main` when cloned 2026-09-24). This is the same commit the sim docstring cites.
  - capture.rs: https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture.rs
  - capture_v2.rs: https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture_v2.rs
  - release.rs, init_escrow.rs, state/path.rs, constants.rs (same commit, under `programs/mpl-hybrid/src/`)
- Docs. developers.metaplex.com now redirects to metaplex.com/docs:
  - Overview: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid
  - Capture: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/swapping-tokens-to-nfts
  - Create escrow: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/create-escrow
  - FAQ: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/faq
  - Protocol fees: https://www.metaplex.com/docs/protocol-fees. The fee table is filled in client-side from `src/components/products/mpl-hybrid/index.js` in https://github.com/metaplex-foundation/developer-hub (commit `b2bf7ba0`), which I read.
- Solana SlotHashes sysvar: https://docs.anza.xyz/runtime/sysvars

**What the docs say:** "If reroll (path) is enabled in the escrow configuration then the metadata index written to the NFT will be picked at random from the pool of available indexes min, max" (capture page). The overview says metadata "can have its metadata blanked as it enters the escrow wallet and randomly reassigned (rerolled) as it leaves escrow". The docs never say where the randomness comes from.

**What the code does.** From capture.rs, lines 167-183 ([permalink](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture.rs#L167-L183)):
```rust
    //If the path has bit 0 unset, we need to update the metadata onchain
    if !Path::NoRerollMetadata.check(escrow.path) {
        let clock = Clock::get()?;
        // seed for the random number is a combination of the slot_hash - timestamp
        let recent_slothashes = &ctx.accounts.recent_blockhashes;
        let data = recent_slothashes.data.borrow();
        let most_recent = array_ref![data, 12, 8];

        let seed = u64::from_le_bytes(*most_recent)
            .saturating_sub(clock.unix_timestamp as u64)
            .wrapping_mul(escrow.count);

        // remainder is the random number between the min and max
        let remainder = seed
            .checked_rem(escrow.max - escrow.min)
            .ok_or(MplHybridError::RandomnessError)?
            + escrow.min;
```
- The `recent_blockhashes` account is really the SlotHashes sysvar (capture.rs L83-86: `address = SLOT_HASHES`; constants.rs L7: `SLOT_HASHES = "SysvarS1otHashes111111111111111111111111111"`).
- The index is appended to the base URI: `uri.push_str(&remainder.to_string()); uri.push_str(".json")` (L190-191). It is written with an MPL Core `UpdateV1` CPI.
- `escrow.count += 1;` runs at the end of **both** capture (L293) and release (release.rs L273). `init_escrow.rs` L119 sets `count: 1`.
- capture_v2.rs L188-204 uses the same logic with `recipe.count` ([permalink](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture_v2.rs#L188-L204)).
- Release does not pick a random index. When reroll is on, it sets name `"Captured"` and URI `<base>captured.json` (release.rs L171-180).
- `CaptureV1Ctx` has no instructions-sysvar account and no caller check. `owner` is just a `Signer`, so a wrapper program can CPI into capture.
- Anza docs: "The SlotHashes sysvar contains the most recent hashes of the slot's parent banks. It is updated every slot." So everything that goes into the seed (the parent bank hash, the bank clock `unix_timestamp`, `escrow.count`) is fixed before the capture instruction runs, and another instruction in the same transaction can read it.

**Randomness source, in one line:** the slot hash (4 bytes of the parent bank hash), the Clock unix timestamp, and a swap counter. There is no VRF, no commit/reveal, and nothing specific to the user or asset (owner and asset keys are not part of the seed).

**Checking sim Part B (`sim/reroll_ev.py`) against this code:** the model **holds**.
- `mpl_hybrid_pick()` reads `data[12:20]` as little-endian u64, does `max(0, x - ts)` (= `saturating_sub`), `& 2^64-1` after multiplying by count (= `wrapping_mul`), then `% (max-min) + min` (= `checked_rem` + min). That matches L173-183 line for line.
- The layout claim also holds. SlotHashes is a bincode `Vec<(u64, [u8;32])>`, so bytes 12..20 are the high 4 bytes of the newest slot number (zero while slot < 2^32) followed by the first 4 bytes of the newest hash. The Anza page does not show the byte layout. The Vec encoding (8-byte length, then entries) is standard bincode, and I did not re-verify it against the sysvar source in this pass.
- I re-implemented the Rust expression with u64 semantics and checked it independently: range 1024 or 4096 gives 1 distinct pick; 10,000 gives 625; 5,000 gives 625. Same as the sim output.
- Caveats. (a) The sim treats count as fixed across slots. In reality count changes with every capture or release by anyone, which only adds another public input. (b) A PoC against a local validator is still the right confirmation, but the code reading is no longer an assumption.

**Extra observations from the code (not in the sim):**
1. **Off-by-one against the docs.** The create-escrow page says for files `0.json … 4999.json` use "min: 0, max: 4999". The code computes `seed % (max - min) + min`, so the range is `[min, max)` and **index `max` can never be picked**. With min 0 / max 4999 the highest reachable index is 4998 (checked numerically). If the rarest metadata sits at the top index, a reroll can never produce it. If a launchpad follows the docs, one metadata file is silently orphaned.
2. **Token program is classic SPL only.** Every swap context declares `token_program: Program<'info, Token>` (anchor_spl::token), e.g. capture.rs L95 and capture_v2.rs L106, and `token: Account<'info, Mint>`. The FAQ says: "Currently the MPL-Hybrid program only supports Metaplex Core NFT assets and SPL Token fungible assets. We are working to support other token standards including pNFTs & Token Extensions." **So the shipped MPL-Hybrid cannot use a Token-2022 transfer-fee mint.** The design has to fork MPL-Hybrid or write its own swap.
3. **V2 has burn paths.** `Path` has `NoRerollMetadata, BlockCapture, BlockRelease, BurnOnCapture, BurnOnRelease` (state/path.rs). capture_v2 with `BurnOnCapture` **burns** `recipe.amount` tokens instead of transferring them to escrow (capture_v2.rs ~L261-272). That is the mint/burn-leg pattern B-06 warns about.
4. **Ignored transfer result in V1.** capture.rs L236-237 is `let _transfer_nft_result = transfer_nft_ix.invoke_signed(...)`, which discards the NFT-transfer result (V2 uses `?`). My understanding is that a failing CPI aborts the whole transaction in the Solana runtime, which would make this harmless, but I **did not verify** that in this pass. It is a code smell worth a PoC test in any fork.
5. The reroll path can't be toggled after swaps begin: update_escrow.rs L122-125 rejects a path change "if the escrow has a swap count > 1".

**Fees per swap (documented + code):**
- Protocol fee: docs say "There is a .005 sol swap protocol fee on every swap" (FAQ), and the protocol-fees data lists MPL-Hybrid `swap: '0.005 SOL', payer: 'Collector'`. In code, `get_protocol_fee()` = `Rent::minimum_balance(590) + 2_720` lamports (constants.rs L10-18) = (128+590)×3480×2 + 2720 = 5,000,000 lamports = 0.005 SOL at current rent. It is charged on **both** capture (capture.rs L264-274) and release (release.rs L243-254) and goes to `FEE_WALLET_V1` (`GjF4LqmEhV33riVyAwHwiEeAHx4XXFn2yMY3fmMigoP3`). The protocol-fees page also says "may change the fee amounts over time".
- Project fees, set by the creator (create-escrow docs + code). `feeAmount`: tokens paid to `fee_location` on capture. `solFeeAmount`: SOL paid to `fee_location`. In **V1 code, `escrow.sol_fee_amount` is charged on both capture (L277-290) and release (release.rs L256-270)**, although the docs describe it as "Optional fee to pay when swapping from Tokens to NFT". V2 recipes have separate `sol_fee_amount_capture` / `sol_fee_amount_release` and `fee_amount_capture`.
- The sim's Part A cost model (2 × 0.005 SOL protocol + 2 × project SOL fee + capture token fee) matches V1 code.

**Relevance:** confirms **B-01** (Critical stands: the pick is a pure function of public, same-transaction-readable inputs, capture is CPI-callable, and for power-of-two ranges the hash term cancels out). Resolves B-01/B-15 "[needs source]". **B-06** needs rewording: shipped MPL-Hybrid does not support Token-2022, so a fork or custom swap is required, and V2 burn paths exist. **B-12** Part A fee assumptions are confirmed. The new off-by-one (index `max` unreachable) should be added under B-01 or B-12, or as a new Low/Info item.

## 1. Stonk.fun (StonkFun, stonkfun.xyz): exploits, incidents, bot abuse, rugs, controversy

**Bottom line:** I found **no credible report of an exploit, hack, smart-contract incident, or platform rug of StonkFun.** What credible sources do document: (a) sniping / bot and single-wallet-dominated launches, which StonkFun itself acknowledged; (b) phishing domains impersonating the brand; (c) governance and transparency concerns (anonymous team, buyback run at the operator's discretion, off-chain reward distribution, revenue figures that don't reconcile). Searches for "exploit / hack / rug / scam / hacked / drainer" turned up only the items below.

**Sources read:**
- The Coinomist, 2026-09-06: https://thecoinomist.com/news/stonk-surges-250-to-140m-after-raydium-launchlab/
  - "A September 2 post from StonkFun described the changes as targeting sniping, single-wallet launches and deployment costs. The team also temporarily raised maximum developer buy limits to discourage sniper bots." New launches moved to Raydium LaunchLab, which StonkFun described as offering "reduced sniper risk".
  - I did **not** read the original StonkFun X post; this is second-hand.
- Fintrender, "A case for STONK", 2026-09-08: https://fintrender.com/en/reports/a-case-for-stonk
  - Anonymous operator (@LaunchOnSF).
  - STONK mint has no mint or freeze authority.
  - The ~60% buyback is "off-chain discretion". There is "no DAO, no token vote, and no named operator".
  - "multiple impostor tokens trade as 'STONK'/'STONKS'".
  - The revenue dashboard "renders in-browser and returns nothing to a direct query, with reported revenue and reported buybacks failing to reconcile (a ~4× gap in late August)".
  - Thin liquidity: "a market a few wallets can move".
  - This is an analyst opinion piece, not an incident report.
- StakePoint blog (a locker vendor, so commercially interested), dated 2026-09-24: https://stakepoint.app/blog/how-to-lock-stonkfun-tokens
  - StonkFun reward launches are Token-2022 with a transfer fee.
  - "The transfer fee is collected on-chain by the token program, but everything after that is administered by StonkFun … Fees accumulate, StonkFun sweeps them, swaps them into the paired asset, and pays out to a list of eligible addresses."
  - Program accounts (PDAs) "were originally excluded from that payout list entirely"; StonkFun later added whitelisting.
  - Note: the search-engine snippet of this page said locked positions are excluded, but the page I fetched says whitelisting now exists. The page appears to have been updated.
- PhishDestroy record for claim-stonkfun.xyz: https://phishdestroy.io/domain/claim-stonkfun.xyz/
  - First observed 2026-08-23 15:47 UTC (09:47 MDT); registered 2026-08-21.
  - VirusTotal 13/91 detections; listed on MetaMask, ScamSniffer and SEAL blocklists.
  - Targeted brand stored as "Solana". A sibling domain vote-stonkfun.xyz appeared in search results; I did not fetch its page.
  - This is brand-impersonation phishing aimed at users. It is **not** a compromise of the platform.
- AVOID.NET, https://avoid.net/stonk: the page itself is labeled "[AI-DRAFTED · AWAITING VERIFICATION]", provisional score 32/100. Its claims: at least four unrelated "STONK/STONKS" tokens; no audits or doxxed team found; phishing domains flagged. **Low credibility**, used only as a pointer. I found no source for its "mentioned with ZachXBT / Tectonic exploit" entity links and don't rely on them.

**What StonkFun's model implies (from the sources, not an incident):**
- Launches are bundled through Jito. The Sumo docs snippet says every bundle buy goes "in one Jito bundle alongside" the create transaction. I saw that only in a search snippet and did not fetch the page, so it is not cited as read.
- StonkFun's own developer page says launchers can build "your own mint keypair, your own instructions, your own bundle" (https://www.stonkfun.xyz/developers, search-snippet level only; not fetched). So creator-bundled first buys are an explicitly supported pattern.

**Relevance:** backs **B-10**: a live Solana launchpad publicly admitted sniper and single-wallet launch problems and responded with dev-buy limits plus a move to a different curve venue. Also relevant to **B-04 / B-07**: a Token-2022 fee pot swept and distributed off-chain at operator discretion is exactly the trust model B-04 flags. There is no evidence to raise or lower any severity from a StonkFun "incident", because none was found.

## 2. pump.fun incident, 2024-05-16 (ex-employee, flash loans, bonding-curve withdraw authority)

**Sources read:**
- The Block, 2024-05-16 (updated 05-17), quoting pump.fun's X post-mortem: https://www.theblock.co/post/295029/pump-fun-post-mortem
- Cointelegraph, 2024-05-17: https://cointelegraph.com/news/solana-memecoin-tool-pumpfun-claims-ex-employee-exploiter
- Quadriga Initiative case study, which quotes an on-chain analysis and pump.fun's post: https://quadrigainitiative.com/casestudy/pumpfuninsiderflashloanexploit.php
- Decrypt, 2025-12-18, sentencing: https://decrypt.co/352876/former-pump-fun-dev-sentenced-six-years-prison-2-million-solana-fraud
- The Defiant, https://thedefiant.io/news/hacks/attacker-abuses-flashloans-to-exploit-pump-fun: the fetch returned only a headline ("Loses Nearly $2M"), with no body. Not relied on.
- I did **not** read pump.fun's original X post directly. All quotes from it are via The Block / Quadriga.

**What happened (per the sources):**
- pump.fun post-mortem, quoted by The Block: "At 15:21 UTC, a former employee, having illegitimately taken access of the withdraw authority using their privileged position at the company, used flash loans on a Solana lending protocol." That is 09:21 MDT on 2024-05-16.
- The borrowed SOL was used "to buy out as many memecoins until they hit 100% on their bonding curves, which allowed the exploiter to gain liquidity to repay the flash loans" (The Block).
- "By 17:00 UTC, all trading on pump.fun was halted" (11:00 MDT).
- Quadriga, quoting on-chain analysis: the attacker took a 129 SOL flash loan in the analysed transaction. Service account `5PXxuZ` "itself initiated the withdrawal of all liquidity from the bonding curve", then "instead of creating a Raydium pool as expected, 5PXxuZ transferred the remaining SOL to a random account", and "5PXxuZ acted as a cosigner for all the attacker's transactions". The analysis concludes "the evidence heavily favors a scenario where the private key for 5PXxuZ was compromised".
- Cointelegraph reports Wintermute's Igor Igamberdiev attributing it to "an internal private key leak". Cointelegraph names the flash-loan venue as "Solana lending protocol Raydium". Raydium is an AMM, not a lending protocol, so I treat the venue as **unconfirmed**; The Block only says "a Solana lending protocol".

**Loss:** about **12,300 SOL (~$1.9M at the time)**, out of ~$45M of liquidity in the bonding-curve contracts (The Block, Cointelegraph). Remediation: contracts upgraded; affected coins seeded on Raydium with ">= 100% of the liquidity that it previously had" within 24h; trading fees 0% for 7 days (The Block, Quadriga).

**Root cause:** not a bug in the bonding-curve math. It was a **privileged off-chain key / authority** (the bonding-curve "withdraw authority" used to migrate liquidity to Raydium at 100% completion) that an insider misused. The flash loan only made it possible to push many curves to 100% within a single transaction, so the privileged migration/withdraw path could be triggered on them. pump.fun: "the pump.fun contracts are safe. they have always been safe" (Quadriga quoting the post). Decrypt: Jarett Dunn (X: @STACCoverflow), a senior developer for six weeks, pleaded guilty to fraud by abuse of position and transfer of criminal property. He was sentenced 2025-12-18 at Wood Green Crown Court, London, to two concurrent six-year terms.

**Relevance:** direct precedent for **B-03 / B-04 / B-05**. A single privileged "withdraw/migrate" key on a launch curve was the whole attack surface, and flash liquidity amplified it (**B-08**-style atomic capital). It supports keeping B-03 and B-04 at **Critical**. It also supports designing graduation/migration so it is permissionless and the program sends funds to a PDA-derived destination, with no hot operator cosigner.

## 3. Solana sniper bots, Jito bundles and sandwiching around token launches

**Sources read:**
- CoinDesk, 2024-03-08, "Solana Client Developer Jito Ends 'Mempool' Function": https://www.coindesk.com/business/2024/03/08/solana-client-developer-jito-announces-end-of-mempool-function
  - Jito's mempool "would go offline within hours".
  - Jito contributor Lucas Bruder: "we attempted to engineer solutions to reject sandwich bundles, but our solutions became a cat-and-mouse game with MEV searchers … we have made the difficult decision to suspend the mempool."
- CoinDesk, 2024-06-10, "Solana Heavyweights Wage War Against Private Mempool Operators": https://www.coindesk.com/business/2024/06/10/solana-heavyweights-wage-war-against-private-mempool-operators
  - "Rather than completely solve the problem, Jito's move pushed it underground."
  - 32 operators with ~1.5M SOL were removed from the Solana Foundation Delegation Program. Solana Foundation: "Enforcement actions are on going as we detect operators participating in mempools which allow sandwich attacks."
  - A DeezNode private-mempool proposal offered validators 50% of MEV profits.
- Gerzon, Weintraub, In, Mislove, Nita-Rotaru (Northeastern), "Quantifying the Threat of Sandwiching MEV on Jito", ACM IMC 2025: https://ben-weintraub.com/files/solanamev.pdf (DOI 10.1145/3730567.3764493)
  - Data window: Jito bundle data 2025-02-09 to 2025-06-09.
  - **521,903 sandwich attacks**; victims lost **≥ $7,712,138**, attackers gained $9,678,466 (USD at the 2025-09-12 SOL rate). 28% of sandwiches had no SOL leg, so these numbers are **lower bounds**.
  - Per-victim loss: median ≈ $5, some > $100. Daily attacks fell from > 15,000/day to ~1,000/day over the period.
  - Sandwich bundles tip much more: median tip > 2,000,000 lamports, versus a 1,000-lamport median for other length-3 bundles.
  - > 86% of length-1 bundles carry tiny tips ("defensive bundling"); users spent ~$2.42M on this.
  - Only 0.038% of bundles were sandwiches.
  - The paper also notes that which transactions were bundled "is not available on Solana's final ledger".
- Jito docs, "Low Latency Transaction Send": https://docs.jito.wtf/lowlatencytxnsend/
  - Bundles: "up to 5 transactions that execute sequentially and atomically". "Bundles execute within the same slot". "If any transaction in a bundle fails, none of the transactions in the bundle will be committed to the chain."
  - Sandwich mitigation: a transaction that references any `jitodontfront…` account "will be rejected by the block engine unless that transaction appears first (at index 0) in the bundle".
- Hu, Tekin, Xu, Liu (Georgia Tech), "MELT: A Behavioral Trace Dataset for High-Risk Memecoin Launch Detection", arXiv 2602.13480. The search index lists it under its earlier title "MemeTrans". https://arxiv.org/abs/2602.13480
  - Covers 41,470 pump.fun launches that migrated, 2024-12-01 to 2025-03-01.
  - **"36.5% of the total token supply is held by bundled accounts at the point of migration."** By source: Jito bundle ID 15.96% of supply, fund-flow 28.22%, same-transaction co-purchase 9.16%.
  - "98.7% of 'create' events co-occur with developer buys in the same transaction."
  - The first 10 and 20 buyers of high-risk tokens hold 17 and 19 percentage points more supply than those of low-risk tokens.
  - 84.13% of launches were labeled high-risk. 60.26% hit < 0.2× the migration price within 20 minutes of migration.
- Kamat, "Coordinated Sniper Cohorts on Pump.fun", arXiv 2607.02795 v3 (single author, q-fin preprint, not peer-reviewed): https://arxiv.org/abs/2607.02795
  - 166,098 launches, 2026-06-11 to 06-25.
  - **1,012 persistent wallet cohorts** (2,965 addresses) that "systematically co-fire as early buyers".
  - 7.0% of launches with a cohort present had **zero** non-cohort buyers in the first 30 minutes.
  - Causal lift on other buyers is modest: +16.1% buyer count; the SOL-inflow lift is not distinguishable from zero.

**Takeaways for the design:**
- Jito's all-or-nothing, same-slot bundle semantics are documented. That confirms the B-01 premise: a bundle whose wrapper instruction aborts does not land, so the only cost is failed simulation or submission.
  - Caveat: the docs I read don't say whether a failed bundle costs anything. Those semantics are **not** stated there.
- Sandwiching did **not** stop with the public mempool. It moved to private mempools and continued through Jito bundles in 2025 (measured lower bound $7.7M in four months).
- At launches, coordinated or bundled first-buyer supply is large and measured: 36.5% of supply at migration in the MELT sample.

**Relevance:**
- **B-10 (High) is well supported.** Bundled or coordinated first buys are the norm, not an edge case. Consider keeping it High, and treat anti-snipe design as a launch blocker.
- **B-11 (Medium)** is supported: sandwiching still happens, though it is rare per bundle and the median loss is small.
- **B-01**: its "[needs source for Jito revert semantics]" can be closed with the Jito docs quote.
- **B-09 / B-13**: the fund-flow clustering heuristics in MELT show why sybils are cheap and common. They are only partly detectable after the fact.

## 4. Solana raffle/lottery or weak on-chain randomness incidents

**Bottom line:** I found **no credible, independently reported incident** (news, post-mortem, or audit disclosure) of a Solana raffle or lottery being drained through weak randomness. What I did find:
- The official Metaplex docs admit that Candy Machine's slot-hash-based "random" pick is predictable.
- The same slot-hash byte pattern that MPL-Hybrid uses appears in Candy Machine v2 source.
- One first-person, unverified exploit write-up.
- One documented bot swarm on Candy Machine that took down the network. It was availability, not randomness.

**Sources read:**
- **Candy Machine v2 source** (metaplex-program-library @ `58d10c46`): https://github.com/metaplex-foundation/metaplex-program-library/blob/58d10c46e66ca9d9c6288999ca9289c986587c7f/candy-machine/program/src/processor/mint.rs, lines 540-548:
  ```rust
      let data = recent_slothashes.data.borrow();
      let most_recent = array_ref![data, 12, 8];

      let index = u64::from_le_bytes(*most_recent);
      let modded: usize = index
          .checked_rem(candy_machine.data.items_available)
  ```
  This is the same `array_ref![data, 12, 8]` SlotHashes read as MPL-Hybrid. CMv2 at least restricted CPI callers (L151-164: "Restrict Who can call Candy Machine via CPI", with `punish_bots` for other callers) and inspected neighbouring instructions. MPL-Hybrid's capture has neither check (see §5).
- **Metaplex Candy Machine settings docs**: https://www.metaplex.com/docs/smart-contracts/candy-machine/settings
  - "It can be advisable to utilize Hidden Settings for the reveal mechanic, as the 'random' minting process of the assets is not entirely unpredictable and can be influenced by sufficient resources and malicious intent."
  - The Hidden Settings `hash` is a commitment to the index-to-metadata map, which buyers can check after reveal.
- **Metaplex Core Candy Machine "Anti-Bot Protection Best Practices"**: https://www.metaplex.com/docs/smart-contracts/core-candy-machine/anti-bot-protection-best-practices
  - Without protection, bots can "Use predictable patterns to snipe rare items".
  - Under "Predictable URI Vulnerability", the page lists `…/metadata/0.json, 1.json, 2.json` as a mistake that lets bots "Pre-fetch all me[tadata]… target specific indexes". **MPL-Hybrid's escrow design requires exactly this sequential `<base>/<index>.json` layout** (create-escrow docs, §5).
  - Recommends placeholder metadata, a third-party signer guard with backend-controlled minting, and a pre-committed hashed reveal mapping.
- **CoinDesk, 2022-05-01**: https://www.coindesk.com/tech/2022/05/01/solana-goes-dark-for-7-hours-as-bots-swarm-candy-machine-nft-minting-tool
  - Bots swarmed Candy Machine with "four million transaction requests and 100 gigabits of data every second".
  - The network halted at 4:32 p.m. ET (14:32 MDT) and restarted around 11 p.m. ET (21:00 MDT).
  - Metaplex responded with a 0.01 SOL "botting penalty".
  - This is bot abuse and liveness, **not** a randomness exploit.
- **Unverified first-person claim**: GitHub gist by `staccDOTsol` (the same handle family as the pump.fun exploiter in §2), "The Candy Machine NFT Rarity Exploit": https://gist.github.com/staccDOTsol/09ef242e4715bce5fe4539c497cd5790
  - The author says he used an on-chain program that aborted mints below a rarity threshold ("Rarity below specified threshold"). Later he appended "a custom instruction at the end of the standard minting transaction" that aborts unless the rarity is high enough, claiming a "100%" success rate.
  - He also quotes an unnamed Metaplex maintainer: "V3 probably has the same issue".
  - **Not independently verified.** No transaction signatures were checked. The author is not a neutral source. Treat it as an illustration of the technique, not evidence of losses.
- Searching for a Solana casino or coinflip drained through predictable randomness found none. The closest hit was MetaWin (2024), which was a hot-wallet withdrawal exploit, not randomness. I did not open that page and don't rely on it.

**Relevance:**
- **B-01 / B-02**: the vendor itself documents that slot-hash "randomness" in Metaplex's own mint tool is manipulable. That supports **Critical** for any value-bearing pick based on SlotHashes/Clock.
- The claimed "append an aborting instruction after the mint" variant matters for B-01's fix list. A CPI-caller check alone does **not** stop predict-and-abort, because a later top-level instruction in the same transaction can read the outcome and revert. B-01 already calls the CPI check "defense in depth only". That wording should stay, and the commit/reveal (settle in a later transaction, unconditionally) remains the real fix.
- **B-12**: MPL-Hybrid's mandatory sequential index URIs are the "Predictable URI" anti-pattern Metaplex warns about. The rarity of every index can be scraped ahead of time.

## 6. Token-2022 transfer-fee extension: authorities, maximum fee, delay on fee changes

**Sources read:**
- Solana docs, "Transfer Fees": https://solana.com/docs/tokens/extensions/transfer-fees
- Solana Program Library docs, Token-2022 extensions (spl.solana.com now redirects here): https://www.solana-program.com/docs/token-2022/extensions
- Source code: `git clone https://github.com/solana-program/token-2022`, HEAD `8867f751c0f69367ba03af4f85510b5611989491` (2026-09-23):
  - `interface/src/extension/transfer_fee/mod.rs`
  - `program/src/extension/transfer_fee/processor.rs`
  - `program/src/processor.rs`
  - `interface/src/instruction.rs`

**Authorities** (there are two, both stored in the mint's `TransferFeeConfig`, mod.rs L136-141):
- `transfer_fee_config_authority`: "Optional authority to set the fee". SPL docs: "Transfer fee authority: entity that can modify the fees".
- `withdraw_withheld_authority`: "Withdraw from mint instructions must be signed by this key". SPL docs: "entity that can move tokens withheld on the mint or token accounts … can move these tokens wherever they wish using withdraw_withheld_tokens_from_accounts or harvest_withheld_tokens_to_mint".
- Either can be rotated or removed with `SetAuthority`, using `AuthorityType::TransferFeeConfig` / `AuthorityType::WithheldWithdraw` (interface/src/instruction.rs L1249-1252; processor.rs L839-866). If an authority is already `None`, `SetAuthority` fails with `AuthorityTypeNotSupported`, so removing it is permanent.
- `HarvestWithheldTokensToMint` is **permissionless** (Solana docs: "Use permissionless HarvestWithheldTokensToMint"). Fees are withheld **in the recipient's token account**. SPL docs: "it is impossible to close an account that holds any tokens, including withheld ones."

**Maximum fee:**
- `pub const MAX_FEE_BASIS_POINTS: u16 = 10_000;` (mod.rs L26), i.e. up to **100%**.
- Enforced both at initialization and in `process_set_transfer_fee`: `if transfer_fee_basis_points > MAX_FEE_BASIS_POINTS { return Err(TokenError::TransferFeeExceedsMaximum.into()); }` (processor.rs L91-93).
- `maximum_fee` is a per-transfer cap in token base units. It is a plain `u64` with no protocol cap, so it can be set to `u64::MAX`, which means no cap in practice.

**Delay on fee changes.** From processor.rs L101-110 ([file](https://github.com/solana-program/token-2022/blob/8867f751c0f69367ba03af4f85510b5611989491/program/src/extension/transfer_fee/processor.rs)):
```rust
    let epoch = Clock::get()?.epoch;
    if u64::from(extension.newer_transfer_fee.epoch) <= epoch {
        extension.older_transfer_fee = extension.newer_transfer_fee;
    }
    // set two epochs ahead to avoid rug pulls at the end of an epoch
    let newer_fee_start_epoch = epoch.saturating_add(2);
```
- Solana docs: "Use SetTransferFee to update the next transfer fee configuration, which takes effect starting two epochs later."
- In practice: a change made in epoch N takes effect at the **start of epoch N+2**. The minimum real notice is therefore a little over **one** full epoch (if the change is sent at the very end of epoch N), not two full epochs. A pending, not-yet-active fee can be overwritten; each overwrite re-schedules it to epoch+2.
- The new fee applies via `get_epoch_fee(epoch)`, which switches when `epoch >= newer_transfer_fee.epoch` (mod.rs L152-153).

**Other details that matter for the design:**
- SPL docs: "Transferring tokens with a transfer fee requires using transfer_checked or transfer_checked_with_fee instead of transfer. Otherwise, the transfer will fail." MPL-Hybrid calls `token::transfer` (unchecked) against `Program<'info, Token>` (§5). A Token-2022 fork has to switch to the token interface **and** to `transfer_checked`.
- SPL docs warn that with MintCloseAuthority, "if a mint is closed and reinitialized with the transfer fee extension, there may be existing token accounts without the transfer fee extension, allowing them to bypass fee checks."

**Relevance:**
- **B-04 (Critical) confirmed.**
  - Config authority: can raise the fee to 10,000 bps with an uncapped `maximum_fee`.
  - Withdraw authority: can send withheld fees "wherever they wish".
  - The only protocol protection is the epoch+2 schedule. Correct B-04's "(I believe 2 epochs) [needs source]" to "takes effect at the start of epoch current+2 (minimum notice ≈ 1 epoch + remainder)". Any program-level timelock should be measured against that.
- **B-06**: add the `transfer_checked` requirement.
- **B-03**: add "no MintCloseAuthority" to the extension allowlist, because of the documented fee-bypass risk on close and reinit.
- **B-13**: confirmed that withheld fees block account closure until harvested.

## Summary: what should change in threat-model.md (suggestions only; threat-model.md not edited)

Sections above are in the order they were researched: 5, 1, 2, 3, 4, 6.

| Finding | Effect of research |
|---|---|
| B-01 | **Critical confirmed from source.** MPL-Hybrid capture.rs L167-183 @ `aacf1a53` matches sim Part B exactly (SlotHashes bytes 12..20, `saturating_sub(unix_timestamp)`, `wrapping_mul(escrow.count)`, `% (max-min) + min`). Capture is CPI-callable (no instructions-sysvar check). Remove "[needs source]". **New sub-issue (suggest Low/Info):** `max` is exclusive, but the docs tell creators to set max = the last file index, so the top-index metadata can never be picked. Keep the note that a CPI check is not enough: predict-and-abort also works with a trailing top-level instruction. |
| B-02 | Supported by the Metaplex docs admitting Candy Machine's slot-hash "random" is influenceable. No independent Solana lottery-exploit incident found. |
| B-03 | pump.fun 2024-05 (privileged withdraw key, ~12,300 SOL) is a direct precedent; Critical stands. Add MintCloseAuthority to the forbidden-extension list (documented fee-bypass risk on close/reinit). |
| B-04 | **Critical confirmed.** Two authorities; fee up to 10,000 bps; `maximum_fee` uncapped u64; a change takes effect at the start of epoch current+2 (notice ≥ ~1 epoch, not 2 full epochs). Replace "(I believe 2 epochs) [needs source]". |
| B-06 | **The condition is now certain.** Shipped MPL-Hybrid only supports classic SPL Token (`Program<'info, Token>`; FAQ: "only supports … SPL Token fungible assets"), so a Token-2022 fee mint *requires* a fork or custom swap. The fork must use `transfer_checked` (plain `transfer` fails on fee mints). MPL-Hybrid V2 already has `BurnOnCapture` (a burn leg): don't enable it on a fee mint. Severity High is still appropriate. The "[needs source]" on Token-2022 support is resolved: it is not supported. |
| B-10 | Strongly supported by data: 36.5% of supply held by bundled accounts at migration (MELT); persistent sniper cohorts; StonkFun publicly acknowledged sniping problems. Keep High. Reasonable to treat as a launch blocker. |
| B-11 | Supported: sandwiching continued after the Jito mempool shutdown (≥ $7.7M victim losses in Feb-Jun 2025 via Jito bundles; private mempools, validator removals). Medium is reasonable given the low per-victim median (~$5). |
| B-12 | MPL-Hybrid requires sequential `<base><index>.json` URIs, which is exactly the "Predictable URI" anti-pattern Metaplex warns about in its Candy Machine docs. Sim Part A fee inputs are confirmed: protocol fee 0.005 SOL on **both** capture and release (derived from Rent, and Metaplex "may change the fee amounts"). The V1 project SOL fee is also charged on both legs. |
| B-15 | Pin `aacf1a53` (or whichever commit is deployed). Note that the protocol fee is computed from the Rent sysvar, so it would change if rent parameters change. |

**Not found / not verified:**
- No credible StonkFun exploit, hack, or rug report.
- No independently documented Solana lottery/raffle randomness exploit.
- The flash-loan venue in the pump.fun incident is inconsistent across sources.
- Whether failed Jito bundles cost anything is not stated in the Jito docs I read.
- Whether ignoring the CPI result at capture.rs L236-237 is ever reachable. I believe a CPI failure aborts the transaction, but I didn't verify that.
