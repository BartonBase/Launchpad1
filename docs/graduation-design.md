# Graduation design: Core mint cost, pre-mint vs lazy, curve choice, economics, art binding

> **UPDATE 2026-09-25 5:13 PM MT: Barton chose LAZY MINTING and dropped the affordability rule** (DECISIONS ADR-016,
> Q-H5). The N ≤ 10,000 cap and the N ≥ 100 minimum stay. The 10k ratio is dropped (§4.2). Batch pre-mint (§2.1–§2.4, §2.6)
> is **superseded** and kept only for the record. The implemented design is §2.5 and §4.4; the exact interface is
> [lazy-mint-interface.md](lazy-mint-interface.md). Fees are now a flat tiered SOL fee charged at request and never
> refunded (ADR-015); release is free. The "2% of N in tokens" assumption below is historical.

- **Status:** research, measurement and design only. No program code was written. All sources were checked **2026-09-25** unless stated otherwise.
- **Author:** executor agent, 2026-09-25 (MT).
- **Scratch and evidence:** `/workspace/scratch/graduation/` (not in the repo):
  - `measure/measure.js` → `output.txt`, `results.json`: Core mint measurements.
  - `measure/prefund_grief.js`, `measure/prefund_drain.js` (+ `.out`): the PDA pre-funding griefing test and its mitigation.
  - `econ/econ.py` → `output.txt`: affordability tables.
  - `artbind/leaf.js`: reference leaf, root and proof implementation.
  - `research/dbc/`: clone of the Meteora DBC source at `f552f20a` (2026-09-09, program version 0.2.1).
  - `research/meteora_*.md`, `research/ray_*.md`: cached docs.
- **Inputs:** BRIEF.md Decisions (incl. 2026-09-25 fee and graduation), ARCHITECTURE.md, DECISIONS.md, hybrid-rarity-and-assignment.md, stonkfun-lessons.md, security/auditor-a/round1.md, security/auditor-b/round1.md, security/merged/round1-merged.md.
- **Fee assumption used throughout:** capture and re-roll pay 2% of N in collection tokens to ONE fixed fee address. The fee is charged at request time and never refunded. Re-rolls also pay a small SOL minimum. Release returns exactly N.

> **Auditor ID mapping.** The task's "B-05 / B-06 / B-07" are **Auditor A's** IDs in `security/auditor-a/round1.md`:
> - B-05: curve anti-sniping.
> - B-06: graduation trigger and LP custody.
> - B-07: launch supply sent to a person.
>
> Auditor B numbers its findings R1-B-xx:
> - R1-B-03: sniping.
> - R1-B-06: escrow cornering and queue griefing.
> - R1-B-09: migration.
> - R1-B-11: the commitment excludes image bytes.
> - R1-B-13: manifest withholding.
>
> The merged list maps these to M-03, M-10, M-13, M-20, M-21 and M-22. §6 maps every curve, graduation, LP and launch-supply finding to the design below.

---

## 0. Summary

| Question | Answer |
|---|---|
| Per-NFT Core mint cost (our setup: collection member, 8-trait Attributes plugin, Arweave URI, royalties on the collection) | **0.005070 SOL** raw = 0.003570 rent (385-byte account) + 0.0015 Metaplex Core protocol fee. **About 0.00509 SOL all-in** once tx fees, priority fees and the crank reward are added. Lean variant (no Attributes plugin, `ar://` URI): 0.003351 raw / 0.00337 all-in. |
| Pre-mint vs lazy | **DECIDED: lazy mint (ADR-016).** Original recommendation, superseded: **Batch pre-mint.** This is Barton's model, and marketplaces then show the full collection at graduation. It must have an **on-chain affordability rule at launch**, a **hard cap of N ≤ 10,000**, and the drain-before-create fix for PDA griefing. Lazy mint is cheaper and lower-risk and is the documented fallback if Barton wants large collections at low thresholds. |
| Curve | **Meteora Dynamic Bonding Curve (DBC).** It is open source with 3 audit firms, migration is permissionless, LP can be 100% permanently locked in positions owned by our PDA, the migration-fee slice can route to our PDA, it has a fee-scheduler anti-snipe, and it verifies well on-chain. It creates the mint itself, so `hybrid_launch` must CPI into it (§3.2). |
| Threshold | **85 SOL** default (the current market reference: LaunchLab "JustSendit" 85 SOL). It rises automatically when N × cost doesn't fit in a ≤10% slice. |
| Affordability | **Dropped with lazy mint (ADR-016).** Original analysis: at 85 SOL with a 10% slice, pre-mint funds about **1,670 NFTs** (1,336 with a 25% margin). 44 of the 72 (ratio × max-N × raise × slice) combos are unaffordable. **Drop the 10k ratio** (mint cost exceeds the NFT's token value) and cap N. |
| Art binding | A salted Merkle leaf over (index, traits, sha256(image), sha256(JSON), content-addressed URI). The root is committed before the curve opens and the mint is checked on-chain. An off-chain verifier checks the image bytes. |

---

## 1. Per-NFT Core mint cost (measured on localnet)

### 1.1 Method
- **Validator:** `solana-test-validator` (Agave 4.1.2) with the required ports, `--reset`, and ledger `/workspace/scratch/graduation/ledger`.
- **Program:** `--bpf-program CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d mpl_core.so`. The `.so` was dumped from devnet (sha256 `5680d237…`).
- **SDK:** `@metaplex-foundation/mpl-core` 1.10.0 (npm latest as of 2026-09-25).
- **Recorded:** account data length, `getMinimumBalanceForRentExemption`, payer lamport delta, the tx `fee` from `getTransaction`, `computeUnitsConsumed`, and serialized tx size.

### 1.2 Results

| Case | Account size | Rent (lamports) | CU | Tx bytes (1 create) |
|---|---|---|---|---|
| Collection, minimal | 125 B | 1,760,880 | 5,626 | — |
| Collection + Royalties plugin (500 bps, ruleset None) | 194 B | 2,241,120 | 13,306 | — |
| Asset, no collection, empty name and URI | 75 B | 1,412,880 | 6,840 | — |
| **(a) Minimal asset:** in collection, name `Mintmark #99999` (15 B), `https://arweave.net/<43>` URI (63 B) | 153 B | 1,955,760 | 12,146 | 437 |
| (a′) Same with an `ar://<43>` URI (48 B) | 138 B | 1,851,360 | 12,012 | — |
| **(b) Our setup:** (a) + Attributes plugin with 8 traits (authority None); royalties on the collection | **385 B** | **3,570,480** | **26,296** | **643** |
| (b′) (b) with an asset-level Royalties plugin as well | 436 B | 3,925,440 | 29,199 | 685 |

Rent is `(128 + data_len) × 3,480 × 2` = **6,960 lamports per byte** plus a fixed 890,880 for the 128-byte overhead. Measured points: 0 B → 890,880; 165 B → 2,039,280; 500 B → 4,370,880.

**Core protocol fee.** Metaplex charges **0.0015 SOL per Create** and 0.00004872 SOL per "Execute". This is documented at https://github.com/metaplex-foundation/disclosures/blob/main/protocol-fees.md and https://www.metaplex.com/docs/smart-contracts/core/faq (checked 2026-09-25). **We confirmed it on localnet:**
- Every new asset account holds `rent + 1,500,000` lamports.
- Payer delta = rent + 1,500,000 + tx fee.
- The fee sits as excess lamports in the asset and is collected by Metaplex later.
- **Risk:** Metaplex can change the fee through a program upgrade. Budget a margin for this (§2.6).

**Tx fee.** Measured at 10,000 lamports because each create had 2 signers (payer + asset keypair). A PDA asset created by CPI from `hybrid_vault` needs 1 signer, so 5,000 lamports base plus any priority fee.

**Royalties.** Neither marketplace needs a Royalties plugin to list. With none, royalties are 0 (see marketplaces-and-ratios.md). If the creator wants royalties, put them **on the collection**: a one-time 480 bytes of rent. An asset-level plugin costs +51 B = +354,960 lamports **per asset**.

### 1.3 How many creates fit per transaction (1,232-byte limit)
- **Keypair-signed creates** (each asset keypair signs, 64-byte signature):
  - Our setup: 2 per tx fits (1,054 B, 52,592 CU); 3 = 1,465 B, too big.
  - Minimal: 4 per tx fits (1,052 B); 5 = 1,257 B, too big.
  - The Core create instruction data is 303 B (attr8) or 98 B (minimal), with 8 accounts.
- **PDA creates through `hybrid_vault`** (no asset signatures, v0 tx + address lookup table), which is how we'd actually mint:
  - About 310 B of tx overhead leaves about 920 B for instruction data.
  - Each asset also needs its **Merkle leaf preimage and proof** for on-chain verification (§5):
    - A compact leaf is about 136 B.
    - A multiproof for k aligned leaves costs about (depth − log2 k) × 32 B. Depth is 14 at 10k and 17 at 100k.
  - Result: **about 2 verified creates per tx**. 4 per tx is borderline at 10k.
  - CU per verified create is about 26k (Core) + about 15–25k (hashing and proof, estimated), so 2–3 per tx is far under the 1.4M CU limit.
  - A leaf-buffer variant (write leaves in cheap txs, then verify and mint ~16 per tx) gets to about 0.26 tx per asset. It only halves the tx count and adds a state machine. **Not recommended.**

### 1.4 Per-NFT and total cost

Per-NFT all-in assumes: rent + Core fee, a 5,000-lamport base fee per tx, **20,000 lamports priority per tx (assumed)**, 2–3 assets per tx, and a 10,000-lamport crank reward per asset.

| Collection size | Our setup, rent + Core fee | Our setup, all-in (0.00509) | Lean, all-in (0.00337) |
|---|---|---|---|
| 100 | 0.51 SOL | 0.51 SOL | 0.34 SOL |
| 1,000 | 5.07 | 5.09 | 3.37 |
| 5,000 | 25.35 | 25.44 | 16.85 |
| 10,000 | 50.70 | 50.89 | 33.70 |
| 20,000 | 101.41 | 101.78 | 67.39 |
| 100,000 | 507.05 | 508.88 | 336.97 |

Collection account (one-time): 0.0022 SOL rent + 0.0015 SOL Core fee. Rent is 70% of the cost and the Core fee is 30%. Tx and crank costs are under 1%.

**Rent is recoverable in theory, but not for us.** A Core burn returns the rent, but assets are never burned in our design. Treat it as a sunk cost.

---

## 2. Batched pre-mint at graduation

### 2.1 Design
State lives in `hybrid_vault` and is created at launch (all fields immutable except the progress fields):
- `GraduationMint { launch_config, collection, n, minted_bitmap: [u8; ceil(n/8)], minted_count: u32, funded_lamports, verified: bool }`.
- The bitmap is 1,250 B at 10k. Use a separate zero-copy account for large N.

**Asset address.** `asset_pda(i) = PDA(["asset", launch_config, i_le])`. It is index-derived and deterministic, so minting is idempotent: index i can only ever be one account.

**`crank_mint(indices[], leaves[], multiproof)`.** Permissionless. For each index i:
1. Require `bitmap[i] == 0`.
2. Recompute the leaf from the supplied preimage and verify the proof against the committed `trait_root`.
3. **Drain pre-funded lamports.** If `asset_pda(i)` already holds lamports, move them to the graduation fund with `invoke_signed` (system transfer signed by the PDA). See §2.4 F1: this is a measured griefing vector.
4. CPI Core `CreateV2` with:
   - owner = vault PDA; collection = our collection; update authority = collection.
   - name = `"<Name> #" + i` derived on-chain.
   - URI rendered from the leaf's content id.
   - Attributes plugin built from the leaf's trait indices, authority None (immutable).
   - No other plugins.
5. Set `bitmap[i] = 1`, `minted_count += 1`.
6. Pay the cranker `REWARD_PER_ASSET` (proposed 10,000 lamports) from the graduation fund, **only after a verified create**.

**Why a bitmap rather than a "next index" counter.** Many cranks can work in parallel on disjoint index ranges and nobody can race the counter. Progress resumes from any state: a rerun skips set bits.

**Funding.** The graduation fund is a `hybrid_vault` PDA. It is filled by the DBC migration-fee slice routed to our PDA (§3.3 d). It pays:
- Core create lamports (rent + Core fee), with the PDA as payer via `invoke_signed`.
- Crank rewards.

Crankers pay their own tx fees, which the reward covers. No person ever holds the fund. Any residue after `open_converting` goes to a fixed, disclosed destination. Proposed: the DAMM v2 LP (buy-and-add) or the fee address. **Barton's call.**

**`open_converting()`.** Permissionless. The **HARD RULE**, enforced on-chain, is that all of the following must hold:
- `graduation.verified == true` (§3.3 e);
- `minted_count == n`;
- every bitmap bit is set;
- the Core collection's `current_size == n`, read from the collection account (owner = Core, key = our collection PDA);
- the collection update authority == our PDA.

Only then is `converting_open = true` set. Nothing else sets it, and there's no admin override. `request_capture` / `request_reroll` / `release` require `converting_open`.

**Immutability.**
- The collection is created at launch as a PDA with update authority = `hybrid_vault` PDA, and `hybrid_vault` has **no** update or plugin instruction.
- The Attributes authority is None.
- Royalties (if any) are fixed at launch.

### 2.2 Tx count and wall-clock estimate

| N | Crank txs (~2/tx) | Crank txs if 3/tx | At 10 landed tx/s | At 50 landed tx/s |
|---|---|---|---|---|
| 1,000 | 500 | 334 | ~50 s | ~10 s |
| 10,000 | 5,000 | 3,334 | ~8 min | ~2 min |
| 20,000 | 10,000 | 6,667 | ~17 min | ~3.5 min |
| 100,000 | 50,000 | 33,334 | ~83 min | ~17 min |

**Throughput ceiling.** Every create writes the collection account (Core increments `current_size`). The per-block write-lock CU limit on one account is **12M CU**, per https://solana.com/upgrades/100m-cu-blocks (checked 2026-09-25; the block limit is 100M on mainnet since 2026-07-29). At ~26–30k CU per create plus our verification (~50k per asset), that's about 200+ assets per block ≈ 500/s in theory. In practice, tx landing rate is what binds.
- A search summary claimed SIMD-0306 raised the per-account limit to 40M. The official page says 12M. **Unverified; we used 12M.**
- These are **estimates, not mainnet measurements.**

### 2.3 Safe max collection size
- **Technical:** 20,000 is safe (≤10k txs, well under 30 min even at 10 tx/s). 100,000 is feasible but takes ~1 h+ and needs ~509 SOL of funding.
- **Economic (this is what binds):** see §4. At the 85 SOL default with a 10% slice and a 25% margin, **N ≤ 1,336** (our setup) or **2,017** (lean).
- **Recommendation:** a global **hard cap of N ≤ 10,000**, plus the per-launch affordability rule in §4.3, which keeps each launch within its own budget.

### 2.4 Failure modes and mitigations

| # | Failure | Mitigation |
|---|---|---|
| F1 | **PDA pre-funding griefing (measured).** Core `CreateV2` uses system `create_account`, which fails "already in use" if the target already holds lamports. Anyone can send ≥890,880 lamports (the rent minimum; 1 lamport is rejected) to `asset_pda(i)` and block index i forever. That breaks `minted_count == n` and freezes converting. Cost to the attacker is about 0.00089 SOL per index, and a few indices are enough. | **Drain-then-create in the same ix:** `hybrid_vault` signs a system transfer of the PDA's whole balance to the graduation fund, then CPIs Core create. Tested with a keypair asset standing in for a PDA: `DRAIN_THEN_CREATE_OK` (`measure/prefund_drain.out`). Apply the same to the collection PDA and to any other PDA we `create_account`. The attacker's lamports end up funding us. Test: `attack_prefund_asset_pda_does_not_block_mint`. |
| F2 | Crank stalls: nobody runs it | Permissionless with a per-asset reward. Our own cranker, and the frontend offers "help mint" to users. Reward size is fixed at launch. |
| F3 | Funding shortfall (Core fee raised, rent change, mis-sized slice) | Launch-time affordability check with a 25% margin (§4.3). Permissionless `top_up_graduation_fund` (anyone can donate). Cost is read from the live rent sysvar and the live fee at crank time. A shortfall stalls; it never skips or partially opens. |
| F4 | Bad leaf or proof submitted by a cranker | The tx fails and no state changes. Only correct leaves can mint, so a malicious cranker can only waste their own fee. |
| F5 | Congestion or fee spikes | Priority fees come out of the reward margin. Resumable, no deadline. |
| F6 | Manifest withheld (R1-B-13 / M-22): no one can crank without leaf preimages and art ids | Salts and the manifest must be posted at graduation. **Preferred:** the encrypted manifest is posted to Arweave at launch and the decryption key is revealed at graduation (§5.3). **Fallback:** none is needed for funds. Before converting opens, no user has deposited tokens into the vault, and tokens stay tradeable on the DEX. A withheld manifest costs users the NFT feature, not value. Say this plainly in the UI. |
| F7 | Core program upgrade changes CreateV2 | Pin the expected Core program id. A compatibility regression test before mainnet. Crank failures are resumable. |
| F8 | Parallel cranks collide on the same index | The bitmap check makes the second one fail cheaply. The client shards the index ranges. |

### 2.5 Lazy minting: DECIDED and implemented (ADR-016)
Asset i is minted inside settle (`settle_capture` / `settle_reroll`) the first time VRF picks it, straight to the
user. Interface: [lazy-mint-interface.md](lazy-mint-interface.md). Code: `programs/hybrid_vault/src/{pool.rs,
asset_source.rs, instructions/request.rs, instructions/settle.rs, instructions/expire.rs}`.

| Aspect | Implementation |
|---|---|
| Graduation gate | `open_vault` needs verified graduation AND the vault's Core collection (owner Core, update authority = vault_authority PDA, `num_minted == current_size == vault.minted_count`) AND pool capacity == N. There's no `minted == N` requirement, no mint crank (`mint_assets` removed) and no graduation fund. |
| Who pays the mint | The requester. `request_capture` / `request_reroll` move `MINT_ESCROW_LAMPORTS` = (rent(385 B) 3,570,480 + Core fee 1,500,000) × 125% = **6,338,100 lamports** into a per-request system-owned PDA `["mint_escrow", vault, seq]`, next to the principal. The request fails closed (`MintCostConstantStale`) if live rent + fee exceeds the escrow, so an underfunded request can't exist. |
| Settle | Permissionless. If the pick is already minted and held by the vault: Core transfer. Otherwise the settler supplies the Merkle leaf + proof, verified against `vault.trait_root` (committed before the curve), and Core create runs with payer = the escrow PDA, owner = user, collection set + verified, name `#i`, URI = leaf URI (immutable). The rest of the escrow always goes back to the user in the same ix. The settler pays only its tx fee; the tip is 0 (T-HV-16). |
| Expire | Refunds the principal (tokens or the handed-in NFT) + the full escrow. The tier fee is never refunded. |
| Blind VRF selection | Uniform over **all** indices the vault holds: never-minted + minted-and-returned. The pool starts as a lazy Fisher-Yates array of 0..N−1 (slot value 0 = own index, O(1) init) with swap-remove, so minting state can't bias assignment. |
| Mint-once | A minted bitmap in the pool account (bit i = asset i exists). `AlreadyMinted` on a double mint; `minted_count ≤ N` is asserted every ix. |
| Re-roll | The handed-in (already minted) asset goes back to the vault as-is: no re-mint, no burn. |
| Account size | Pool v2 `hvpool02` = 64 + 16N + ceil(N/8) bytes (161,314 B at 10k, ~1.12 SOL rent), created by the creator right before `init_vault`. The bitmap is 1,250 B at 10k. |
| Pre-fund griefing (F1) | Lamports pre-sent to asset(i) are drained into the escrow before create, then refunded to the user. Tested. |
| Marketplace indexing | **Accepted downside:** Tensor/ME see only minted assets, so the collection page fills in as captures happen, and rarity ranks can shift. The full committed manifest (Merkle root + Arweave manifest) is the canonical census. |
| Liveness | Leaf availability matters on every first mint (R1-B-13 / M-22). Mitigated by the Arweave manifest and the principal + escrow refund on `expire`. |
| Tx size | Settle-with-mint at N = 10k (proof depth 14) is ~1.2 KB, so clients should send v0 + ALT. |

### 2.6 Recommendation (SUPERSEDED by ADR-016)
Originally: **batch pre-mint**, because it:
- matched Barton's earlier model;
- gave a complete, browsable, rarity-ranked collection on marketplaces at the moment converting opened;
- confined the risks to things we can enforce: the affordability rule, the drain fix, the bitmap and the hard rule.

Barton chose lazy (5:13 PM MT, 2026-09-25), accepting the incomplete collection page at graduation in exchange for
no graduation fund, no affordability rule and no crank risk.

---

## 3. Curve choice: Meteora DBC vs Raydium LaunchLab

### 3.1 Facts

| | **Meteora DBC** | **Raydium LaunchLab** |
|---|---|---|
| Program id | `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN` (same on mainnet and devnet) | Mainnet `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj`; devnet `DRay6fNdQ5J82H7xV6uq2aV3mNrUZ1J4PgSKsWgptcm6` |
| Upgrade authority (read on-chain 2026-09-25) | Mainnet `JADaUV8kvDpDbJr55wxXJHVaBS3VCj8thZZHjfeuCVLd` (system-owned account; whether it's a multisig is **unverified**), last deploy slot 445,503,633 | `FytDrVzDybM1TwFQPGb8qaxZR7dBCzNeqT3vtQsceZQK` (shared Raydium authority; docs say Squads), slot 446,221,802 |
| Source | Open: https://github.com/MeteoraAg/dynamic-bonding-curve (0.2.1, commit f552f20a) | **Closed.** IDL only: https://github.com/raydium-io/raydium-idl |
| Audits | Offside Labs (0.1.1–0.2.1), OtterSec (0.1.3), Zenith (0.1.3–0.2.0): https://docs.meteora.ag/resources/audits/dbc | Halborn (Q2 2025): https://docs.raydium.io/security/audits |
| SDKs | `@meteora-ag/dynamic-bonding-curve-sdk` 1.5.13 (npm, 2026-09-24), Rust CPI examples in the repo, dbc-go | `@raydium-io/raydium-sdk-v2` 0.2.73-alpha (LaunchLab module) |

### 3.2 (a) Pre-existing mint?
**Neither venue accepts a pre-existing mint.**
- **DBC** `initialize_virtual_pool_with_spl_token` requires a fresh `base_mint` that is `init` and a **signer**. It sets mint authority to DBC's `pool_authority` PDA (`FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM`), mints the whole `pre_migration_token_supply` into DBC's `base_vault` PDA, and **revokes mint authority in the same ix**. It never sets a freeze authority. It also creates Metaplex token metadata, with `token_update_authority` selectable; use **Immutable**. `creator` is a Signer; `payer` can be anyone.
- **LaunchLab** `initialize_v2` also requires a fresh `base_mint` keypair signer. It puts all supply in its `base_vault`, revokes mint authority, and sets no freeze authority. `creator` is not a signer.

**Conflict with `hybrid_launch` (ADR-010).** Our program creates the mint and mints 1B into our `launch_vault` PDA. Under either curve, that supply could not get into the curve's vault without a transfer that the curve doesn't support.

**Reconciliation (recommended): `hybrid_launch` v2 CPIs into DBC.** The steps, in one flow:
1. The mint address is a `hybrid_launch` PDA `["mint", launch_config]`. No keypair exists for anyone to use in pre-creating a pool. DBC's `init` needs the mint's signature, which the PDA provides via `invoke_signed`.
   - Anchor `init` tolerates pre-funded lamports, but **we still need to test a pre-funded mint PDA against DBC's init**.
2. CPI `create_config` (permissionless; the config is a new signer account; DBC has no config-update instruction). Settings:
   - quote = WSOL; SPL Token; decimals as `hybrid_launch` requires;
   - `pre_migration_token_supply` = 1B × 10^d;
   - fee scheduler (§3.3 f); `enable_first_swap_with_min_fee = false`;
   - migration → DAMM v2;
   - 100% permanent-locked liquidity to our PDA;
   - `migration_fee_percentage` from §4.3 with `creator_migration_fee_percentage = 100`;
   - `fee_claimer` = our PDA, `leftover_receiver` = our PDA;
   - `token_update_authority` = Immutable.
3. CPI pool init with `creator` = `hybrid_launch` PDA (signs via `invoke_signed`).
4. **Post-checks** in the same ix (re-read everything):
   - mint supply == 1B × 10^d, decimals, mint authority None, freeze authority None;
   - base_vault owner == DBC pool_authority and balance == supply;
   - pool.config == our config, pool.creator == our PDA, pool.base_mint == our mint;
   - metadata update authority None.
   - Store `dbc_config` and `dbc_pool` in `LaunchConfig`.

**Effects on the existing design:**
- **As implemented (`register_dbc_launch`):** the `LaunchConfig` fields are kept, with new meanings for DBC launches. `launch_vault` holds DBC's `pool_authority` PDA (`FhVo3mqL…`, owner of the base vault), `launch_vault_bump` = 0, and `launch_destination` holds the DBC pool's `base_vault` (where DBC minted the 1B supply). Native `launch` still stores our own `launch_vault` PDA and its ATA. Consumers must not assume `launch_vault` is a hybrid_launch PDA; check `dbc_pool != default` first.
- `launch_vault` is retired. DBC's `base_vault` (a program PDA with no human signer) takes its place. This satisfies Auditor A's B-07 / T-CURVE-09 and merged M-13, because the supply never touches a person.
- **ADR-010 needs amending.** That's for the engineer; not done here.

**Supply caveat: flag for Barton.** DBC requires `pre_migration_token_supply ≥ swap_base × 1.25 + migration_base` (constant `SWAP_BUFFER_PERCENTAGE = 25`, `config.rs:877-901`). At migration, leftover base tokens are handled by `get_burnable_amount_post_migration`:
- `min(pre − post, leftover)` is **burned**;
- the rest goes to `leftover_receiver`.

So either:
- (i) post-graduation supply falls below 1B by up to about the unsold buffer, which breaks "exactly 1B forever"; or
- (ii) set `post_migration_token_supply = pre` so nothing burns, and the leftover (up to ~20% of supply, depending on the curve) goes to a `hybrid_vault` "locked leftover" PDA with no outflow. That's disclosed as locked supply, but it's still counted in the 1B.

**Recommendation: (ii).** Alternatively, add it to the LP. That needs a small custom instruction and is untested.

LaunchLab needs no buffer, but its other problems (below) outweigh that.

### 3.3 Comparison on (b)–(i)

**(b) Migration threshold.**
- DBC: `migration_quote_threshold` is fully configurable per config.
  - Meteora's mainnet keepers auto-migrate pools whose threshold is ≥10 SOL (or ≥750 USDC), per https://docs.meteora.ag/developer-guides/dbc (cached `research/meteora_*`). Keepers include `Asi5DTGE…` and `DeQ8dPv6…`.
  - Manual migrator UI: https://migrator.meteora.ag.
- LaunchLab:
  - "JustSendit" is a fixed 85 SOL.
  - Custom LaunchLab is ≥24 SOL per the user-flow doc, but GlobalConfig `min_quote_fund_raising` defaults to 30 SOL. **The docs are inconsistent.**
  - Sources: https://docs.raydium.io/products/launchlab/overview, https://docs.raydium.io/products/launchlab/global-config.

**(c) Migration mechanics.**
- DBC:
  - **Permissionless.** Any signer pays. Meteora keepers usually do it within seconds. We run a backup cranker.
  - New configs migrate only to DAMM v2 (DAMM v1 is deprecated in 0.2.1).
  - Migration creates the DAMM v2 pool at the curve's migration price, with position NFTs owned by the creator and partner.
  - Configurable %LP permanent-locked vs vesting vs liquid. At least 10% must be locked at day 1 (`MIN_LOCKED_LIQUIDITY_BPS = 1000`).
  - 0.2% of migrated liquidity goes to the protocol (`PROTOCOL_LIQUIDITY_MIGRATION_FEE_BPS = 20`).
  - **Front-run safety:** the DAMM v2 migration config requires `pool_creator_authority == DBC pool_authority`, so nobody else can pre-create the migration pool at a skewed price.
- LaunchLab:
  - **Privileged.** `MigrateToCpswap` must be signed by `global_config.migrate_to_cpswap_wallet`, a Raydium crank. This is exactly the R1-B-09 / M-10 "privileged migrate key" concern.
  - LP is split by platform/creator/burn scales. Since the 2026-08-17 upgrade, the locked LP is platform-owned with a Fee Key sent to `platform_nft_wallet`. `burn_scale` is burned (a full burn is possible).
  - Sources: https://docs.raydium.io/products/launchlab/instructions, https://docs.raydium.io/products/launchlab/platform-config.

**LP custody (B-06 / M-10).**
- **DBC:** set 100% permanent-locked liquidity, with creator and partner = our PDAs. The LP can then never be withdrawn by anyone. Only fee claims remain, and our PDA routes them to a fixed destination, or use DAMM v2's compounding fee mode so fees stay in the pool.
- LaunchLab: burn_scale = 100% is also custody-free, but migration stays privileged.

**(d) A proceeds slice to our PDA, custodied by no one.**
- **DBC: yes.**
  - `migration_fee_percentage` (integer 0–99% of the migration quote threshold, `MAX_MIGRATION_FEE_PERCENTAGE = 99`) is split between creator and partner by `creator_migration_fee_percentage`.
  - `withdraw_migration_fee` requires `sender == pool.creator` (creator share) or `== config.fee_claimer` (partner share). The sender chooses the destination account.
  - With creator = our PDA, **only our program** can withdraw, and our `graduate` ix sends it to the graduation-fund PDA (WSOL, unwrapped by closing to the PDA).
  - Creator and partner trading fees and the curve surplus can route to our PDAs the same way.
  - The protocol takes 20% of trading fees (not ours).
- **LaunchLab: no.** `migrate_fee` goes to Raydium's `migrate_fee_owner`. The creator gets only trading fees (`creator_fee_vault`). There's no proceeds slice.
  - Workaround: raise extra via trading fees. That's weaker, and Raydium admin can change GlobalConfig fees (the platform fee cap was raised to 500 bps on 2026-08-26).

**(e) Verifying graduation on-chain, permissionlessly and without spoofing.** A `graduate(pool)` ix in `hybrid_vault` checks:
1. `pool.owner == dbcij3LW…` (DBC program id pinned as a constant).
2. The first 8 bytes == Anchor discriminator `sha256("account:VirtualPool")[..8]`. This rejects any other account type, including other DBC accounts with the same seeds.
3. The key == PDA re-derived under the DBC program from `["pool", config, max(base_mint, quote_mint), min(base_mint, quote_mint)]` (byte order as in the DBC source).
4. `pool.config == LaunchConfig.dbc_config`, `pool.base_mint == LaunchConfig.mint`, `pool.creator == our PDA`.
5. `pool.is_migrated == 1` and `migration_progress == 3` (CreatedPool). "Curve complete" alone (`quote_reserve ≥ threshold`) isn't enough.

Then it sets `graduated = true` (one-shot, before any transfer), CPIs `withdraw_migration_fee` into the fund, and records `graduation_slot`.

Offsets come from the IDL of the pinned DBC version. **If Meteora upgrades the layout, `graduate` fails closed** and we ship a fix through our multisig + 7-day timelock. A fake pool account fails check 1 (owner) or check 3 (PDA). A real but different pool fails check 4.

**LaunchLab:** a similar owner + discriminator check on its PoolState would work, but closed source makes the layout harder to trust.

**(f) Anti-sniping (B-05 / M-03).**
- **DBC:**
  - A **fee scheduler**: linear or exponential decay from a cliff fee (max 99%, `MAX_FEE_BPS = 9900`) down to a base fee (min 0.25%). SDK examples use 90% → 1.2% over 60 s. We propose **99% → 1% exponential over 10 minutes**.
  - An optional volatility-driven **dynamic fee**. Enable it.
  - The size-based rate limiter is **deprecated and rejected** for new configs (0.2.1 changelog).
  - **No per-wallet caps, no allowlist, and no delayed activation:** `activation_point = get_current_point()` at pool init (`ix_initialize_virtual_pool_with_spl_token.rs:257`).
  - Alpha Vault (Meteora's fair-launch vault) supports DLMM and DAMM, **not DBC** (https://docs.meteora.ag/helper-products/alpha-vault/what-is-alpha-vault).
  - **Known bypasses:**
    - bots wait out the decay (intended);
    - the creator first-swap-at-min-fee bundle path (`enable_first_swap_with_min_fee`), which we disable;
    - same-block bundles right after pool creation (they pay the cliff fee);
    - splitting across wallets (irrelevant to a fee, fatal to caps).
- **LaunchLab:** no anti-snipe mechanism is documented. The docs even suggest a creator initial buy. An `open_time` precondition appears in the docs but conflicts with "no open_time argument" elsewhere (**inconsistent, unverified**).
- **Our layer (either venue):**
  - The pool is created by a **permissionless `open_curve` at the stored `open_slot`** announced at launch, so there's a public notice window. No one can create the pool earlier: the mint is our PDA, and only `hybrid_launch` can sign for it.
  - Converting is closed until graduation (removes NFT sniping, R1-B-04).
  - No creator pre-buy at a privileged price. Creators buy like everyone else.
  - Live top-holder and bundle disclosure on the token page.

**(g) Audits:** see §3.1. DBC's current 0.2.1 is covered by Offside Labs. LaunchLab has one Halborn report and no public source.

**(h) Devnet:**
- DBC: the same id is deployed on devnet (authority `DHLXnJ…`, slot 503,167,099). The migrator supports devnet.
- LaunchLab: a separate devnet id `DRay6fNd…` (authority `DRayw6sn…`, slot 496,182,200). The mainnet id isn't on devnet.

**(i) SDKs:** see §3.1.

### 3.4 Recommendation and threshold
**Meteora DBC.**
- It is the only one of the two with **permissionless migration**, a **proceeds slice routable to our PDA**, **program-custodied permanently locked LP**, open source with multiple audits, and an on-chain anti-snipe fee.
- Costs:
  - it creates the mint (solved by the CPI reconciliation);
  - the 25% buffer supply rule (§3.2);
  - it's upgradeable by Meteora's key (external trust; see §6);
  - it has no true delayed activation or per-wallet caps (§6 flags this against M-03).

**Threshold:**
- **Default 85 SOL** `migration_quote_threshold`. This matches the dominant market reference (LaunchLab JustSendit 85 SOL). DBC's keeper floor is 10 SOL, which we're well above.
- **N-dependent minimum:** `threshold ≥ N × C × 1.25 / 0.10`, where C = 0.00509 SOL all-in. Examples:
  - N = 1,000 → ≥ 63.6 SOL (85 is fine);
  - N = 2,000 → ≥ 127 SOL;
  - N = 5,000 → ≥ 318 SOL;
  - N = 10,000 → ≥ 636 SOL.
- The launch wizard picks max(85, that value). The creator can choose a smaller N instead.

---

## 4. Economics

### 4.1 Per-NFT funding vs measured cost (full table)
Budget = raise × slice. Per-NFT funding is at the ratio's max collection size, compared against C = 0.00509 SOL. Full output is in `econ/output.txt`.

| Ratio | Max N | 85 SOL: 2% / 5% / 10% | 250 SOL: 2% / 5% / 10% | 500 SOL: 2% / 5% / 10% |
|---|---|---|---|---|
| 10k | 100,000 | ✗ ✗ ✗ | ✗ ✗ ✗ | ✗ ✗ ✗ |
| 50k | 20,000 | ✗ ✗ ✗ | ✗ ✗ ✗ | ✗ ✗ ✗ |
| 100k | 10,000 | ✗ ✗ ✗ | ✗ ✗ ✗ | ✗ ✗ ✗ (0.0050 vs 0.00509, just short) |
| 200k | 5,000 | ✗ ✗ ✗ | ✗ ✗ ✗ | ✗ ✗ ✓ |
| 500k | 2,000 | ✗ ✗ ✗ | ✗ ✓ ✓ | ✗ ✓ ✓ |
| 1M | 1,000 | ✗ ✗ ✓ | ✗ ✓ ✓ | ✓ ✓ ✓ |
| 2.5M | 400 | ✗ ✓ ✓ | ✓ ✓ ✓ | ✓ ✓ ✓ |
| 5M | 200 | ✓ ✓ ✓ | ✓ ✓ ✓ | ✓ ✓ ✓ |

**44 of 72 combos are unaffordable** (✗) at max N, before any safety margin.

Max affordable N (full cost, no margin):

| Raise | 2% slice | 5% slice | 10% slice |
|---|---|---|---|
| 85 SOL | 334 | 835 | 1,670 |
| 250 SOL | 982 | 2,456 | 4,912 |
| 500 SOL | 1,965 | 4,912 | 9,825 |

Slice needed to fund N (full cost; lean needs about 2/3 of this):

| N | 85 SOL | 250 SOL | 500 SOL |
|---|---|---|---|
| 100 | 0.6% | 0.2% | 0.1% |
| 200 | 1.2% | 0.4% | 0.2% |
| 400 | 2.4% | 0.8% | 0.4% |
| 1,000 | 6.0% | 2.0% | 1.0% |
| 2,000 | 12.0% | 4.1% | 2.0% |
| 5,000 | 29.9% | 10.2% | 5.1% |
| 10,000 | 59.9% | 20.4% | 10.2% |
| 20,000 | 119.7% | 40.7% | 20.4% |
| 100,000 | 598.7% | 203.6% | 101.8% |

### 4.2 NFT value vs mint cost (why 10k should go)
**Assumption (unverified, illustrative):** market cap at migration ≈ 400 SOL for an 85 SOL raise (pump.fun-like curves graduate around that). The token value of one NFT is then N/1e9 × 400 SOL:
- 10k → 0.004 SOL
- 50k → 0.02
- 100k → 0.04
- 200k → 0.08
- 500k → 0.2
- 1M → 0.4
- 2.5M → 1
- 5M → 2

**At the 10k ratio, the mint cost (0.0051) exceeds the NFT's token value (~0.004).** At 50k it is ~25% of the value. The 2% fee at 10k is 200 tokens, about 0.00008 SOL. That's worthless as a re-roll brake, and the SOL minimum (proposed 0.005 SOL, merged M-19) dominates.

### 4.3 Recommendations
1. **Drop the 10k ratio.** Strongly consider dropping 50k too: per-NFT value is near the mint cost and max-N collections are unfundable.
2. **Cap N ≤ 10,000** globally. This retires the 20k/100k sizes.
3. **Make the slice scale with N, enforced on-chain at launch:**
   - `migration_fee_percentage = ceil(100 × N × C_budget / threshold)`, where `C_budget` = 0.0064 SOL (0.00509 × 1.25 margin, rounded up).
   - **Reject** the launch if that exceeds 10%. The creator must then lower N or raise the threshold.
   - DBC stores the percentage as an integer and it's immutable once the config is created, which fits.
4. Leftover funds after `open_converting` go to a fixed destination (LP or fee address, **Barton's call**), never to a person's discretion.
5. **Optional cost cut:** drop the on-chain Attributes plugin and use an `ar://` URI (lean: −34%, 0.00337/NFT).
   - Traits then live only in the hash-committed JSON.
   - Downside: on-chain programs and some indexers can't read traits from the account. Whether Tensor or ME prefer the plugin or the JSON is **unverified** (see marketplaces doc).

### 4.4 Lazy-mint economics (DECIDED, ADR-016)
- **Protocol / graduation slice:** 0 SOL. `graduation_slice_pct` is always 0; the affordability rule is removed.
- **First capture of each index:** the requester escrows 0.0063381 SOL; the actual spend is ~0.00507 SOL (rent
  0.00357048 + Core fee 0.0015) and the rest (~0.00127) comes back at settle. This is on top of the principal and the flat tier
  fee (ADR-015; charged at request, never refunded).
- **Picks of an already-minted, returned index:** the full escrow comes back; no mint cost.
- **Re-rolls:** the same escrow rule; the hand-in is never re-minted or burned.
- **Totals:** at most 10,000 × 0.00507 = 50.7 SOL spread over first captures, instead of a slice of the raise. Any
  N from 100 to 10,000 works at any threshold.
- The 10k ratio is dropped (mint cost > NFT token value, §4.2).

---

## 5. Binding revealed art to the pre-graduation commitment (M-21 / R1-B-11)

### 5.1 Leaf and tree (reference: `scratch/graduation/artbind/leaf.js`)
```
traits_hash = sha256(trait_schema_hash ‖ u16le[8] trait_value_indices)
H_art       = sha256(salt32 ‖ sha256(image_bytes) ‖ sha256(metadata_json_bytes) ‖ uri_utf8)
leaf_i      = sha256(0x00 ‖ "mintmark-leaf-v2" ‖ launch_config ‖ u32le(i) ‖ traits_hash ‖ H_art)
node        = sha256(0x01 ‖ left ‖ right)          // positional, domain-separated from leaves
```
- The metadata JSON contains `image` (content-addressed URI), `image_sha256`, `attributes` identical to the trait indices, and `name`.
- The on-chain URI is the JSON's content-addressed URI.
- The reference script builds 1,000 leaves: depth 10, 320 B proof. Verification succeeds, and swapping one image makes it fail.
- This supersedes M-21's `mintmark-trait-v1` leaf. It adds the salt, the image hash, the URI and the domain separation.

### 5.2 Commit, preview, reveal
1. **At launch, before the curve opens** (same ix as launch, stored in immutable `LaunchConfig`): `trait_root`, `leaf_count = N`, `trait_schema_hash`, the census hash, and the hash of the encrypted-manifest key (§5.3). DBC pool creation (`open_curve`) requires them to be set.
2. **During the curve:** publish per index `(i, trait_value_indices, H_art, proof)`. Anyone can verify the traits against the root, so traits are previewable. Art stays hidden because `H_art` is salted: images can't be brute-forced from their hashes, and the URI isn't revealed.
3. **At graduation:** publish salts, JSON, images and the full manifest (on Arweave/IPFS). The crank supplies each leaf preimage at mint.

### 5.3 The URI-before-publication problem
A content-addressed URI must be known when the root is committed, but the content must stay unpublished. Options:
- **(a) IPFS CIDv1, recommended.** Compute CIDs offline (`ipfs add --only-hash --cid-version 1`). Commit them. Pin at graduation and mirror to Arweave. The CID is itself a hash of the content, so a swap is impossible.
- (b) Arweave ANS-104 data items pre-signed at launch (id = sha256(signature)) and posted at graduation. Whether bundlers accept items signed long before posting is **unverified**. L1 Arweave txs need a recent anchor, so they can't be pre-signed weeks ahead.
- **Hardening against withholding (M-22 / F6):** upload the art and manifest **encrypted** to Arweave at launch. Commit `sha256(key)`. Reveal the key at graduation, and anyone can then decrypt and pin. Who holds the key until graduation (creator? Barton multisig? a timelock-encryption service?) is an **open question for Barton**.

### 5.4 On-chain checks at mint
`crank_mint` (or lazy `settle`):
1. Recomputes `leaf_i` from the supplied preimage and verifies the proof against `trait_root`.
2. Renders the URI from the leaf's CID/id bytes.
3. Builds the Attributes plugin from `trait_value_indices` via the committed schema (names and values stored on-chain at launch; about 1–4 KB).
4. Derives the name from i.

The URI and attributes therefore **cannot differ** from the commitment. The Attributes authority is None and there's no update authority path, so nothing can change after minting either. **On-chain code cannot check image bytes.** That is off-chain (§5.5).

### 5.5 Off-chain verifier (to build; spec)
`verify-collection --launch <config>`:
1. Read the root and schema from chain.
2. For every asset, read the on-chain URI and attributes. Fetch the JSON and check `sha256(json) == json_sha256`. Fetch the image and check `sha256(image) == image_sha256` (and the CID).
3. Recompute `H_art` and the leaf, and check the leaf is in the root.
4. Check the JSON `attributes` equal the on-chain attributes.

Exit non-zero on any mismatch, and publish the report. Tests: `verify_script_detects_swapped_image`, `launch_rejects_mutable_http_uri`.

### 5.6 Permanence assumption
- Arweave: permanence depends on the endowment model and gateway availability.
- IPFS: availability depends on pinning.
- If both disappear, the on-chain hashes still prove what the art *was*, but can't serve it.
- Mitigation: pin with ≥2 providers, keep an Arweave copy, and publish the full archive torrent.
- State this plainly in the creator UI.

---

## 6. Mapping merged audit findings to the recommended curve (DBC) and our program

| Finding | Requirement | How DBC + our program meet it | Can the curve satisfy it? |
|---|---|---|---|
| **M-03** (Critical A / High B): sniping and bundles (A-D05, T-CURVE-03/05, B-03, Auditor A's "B-05") | (1) activation point ≠ pool creation; (2) decaying anti-snipe fee **or** per-tx/per-slot caps; (3) creator buy ≤2% via the same rules, disclosed; (4) no one can pre-create the pool; (5) published rules; (6) anti-snipe fee proceeds to LP or burn, never to a wallet | (1) **Emulated:** our permissionless `open_curve` creates the pool at a stored `open_slot`, so pool creation *is* the scheduled activation. (2) Fee scheduler 99% → 1% exponential over 10 min + dynamic fee. (3) The creator is our PDA and never buys. Human creators buy through the curve like anyone else, with a disclosed optional cap of 2% enforced only by the UI. (4) The mint is our PDA and only `hybrid_launch` can sign for it, so nobody can create a pool for it. (5) Opening rules are published at launch. (6) Partner and creator trading-fee shares go to our PDAs → graduation fund / LP. | **Partly NO.** (1) There's no native future activation: `attack_bundle_create_and_buy_before_activation_fails` can't pass as written, because a bundle can buy in the same block as our `open_curve` (at the 99% cliff fee). (2) **No per-tx/per-slot caps** (the rate limiter is deprecated), so `attack_opening_window_cap_exceeded_fails` can't pass. (3) We can't cap a human creator's buy on-chain. (6) **20% of trading fees go to Meteora's protocol wallet**, not LP or burn. Ask the auditors to accept a fee-based-only gate (as the M-03 remediation's "either … or" allows) and amend the tests to the fee-at-slot form. |
| **M-10** (High): graduation and migration (A-D06, T-CURVE-07/08, B-09, Auditor A's "B-06") | Permissionless, deterministic, one-shot; seeded at the final curve price; fails closed on a pre-existing pool; LP burned or PDA-locked with no withdraw; carve-out program-computed, paid to a PDA that only the crank spends | DBC migration is permissionless (keepers + our cranker) and one-shot (`migration_progress`), at the curve's migration price. DAMM v2 pool creation is restricted to DBC's `pool_authority`, so it can't be pre-created. 100% permanent-locked LP with owners = our PDAs, no withdraw path. Carve-out = `migration_fee_percentage` from the §4.3 formula, withdrawable only by our PDA into the graduation fund, spent only by `crank_mint`. `graduate` is one-shot. | **Yes.** Caveats: the 0.2% protocol liquidity fee, and DBC/DAMM v2 remain Meteora-upgradeable (see M-09 row). |
| **M-13** (High): mint and freeze authority, supply, launch destination (T-CURVE-01/09, A-D07, Auditor A's "B-07") | Authorities None; exact supply; supply never at a person; distribution only to venue program accounts | DBC mints supply directly into its `base_vault` PDA and revokes mint authority in the same ix; no freeze. `hybrid_launch` v2 re-asserts all of this post-CPI. `launch_vault` is retired. | **Yes, with a supply caveat.** The 25% buffer rule means either burning below 1B or a locked-leftover PDA (§3.2). ADR-010 must be amended; `launch_vault_tokens_only_move_to_curve_program_accounts` becomes `supply_minted_only_into_dbc_base_vault`. |
| **M-22** (Medium): graduation mint crank stall / manifest withheld (B-13) | Permissionless, resumable, idempotent crank; open only when minted == N and bound; fail closed on shortfall; publish max safe N | §2: bitmap + counter, index-derived PDAs, **drain-before-create** (new finding F1), the hard rule including the collection `current_size`, affordability rejected at launch, encrypted manifest (§5.3), N ≤ 10,000. | Not a curve property. Our program satisfies it. |
| **M-21** (Medium): image bytes | Content-addressed URIs; image hash in the commitment | §5 | Not a curve property. |
| **M-25 / M-31**: integration math, slippage, MEV (T-CURVE-02/04/06, B-08) | min_out / max_in on composite flows | DBC `swap` takes `minimum_amount_out`. Our "buy NFT with SOL" composite takes `max_in`, `expected_ratio`, `expected_fee`. DBC uses u128 math and is audited, so there's no custom curve. | Yes (the curve side is audited; our composites need tests). |
| **M-09 / M-08**: upgrade keys, operator powers | No single key over value; timelocked upgrades | Our programs: Squads ≥3-of-5 + 7-day timelock. **DBC and DAMM v2 are upgradeable by Meteora** (`JADaUV8k…`; multisig status unverified), outside our control. | **NO for the external venue.** Mitigation: disclose it; pin the expected program data hash in monitoring; `graduate` fails closed on layout change. |
| **M-27**: allocations outside the pool | No team allocations | DBC `locked_vesting` = 0, no creator allocation; crank mints only to the vault. | Yes |
| **T-CURVE-10** (no platform wallet routes value) | — | Every DBC fee and slice claimant is our PDA, except Meteora's protocol share. | Yes, except the protocol share |
| **R1-B-04**: launch-window rare farming | — | Moot, since converting is closed until graduation. | — |

---

## 7. Recommendations
1. Use **Meteora DBC** through a `hybrid_launch` v2 CPI flow: mint PDA, `create_config`, pool init with creator = our PDA, then post-checks.
2. **Batch pre-mint** with a bitmap crank, the drain-before-create fix, the hard rule (graduated ∧ minted == N ∧ collection `current_size` == N), permissionless top-up, and a per-asset reward paid after verification.
3. **N ≤ 10,000.** Drop the 10k ratio and consider dropping 50k. Scale the slice with N via `migration_fee_percentage`, capped at 10%, and reject the launch otherwise. Default threshold 85 SOL, rising with N.
4. 100% permanently locked LP, owned by our PDAs, compounding fees or fees to a fixed destination.
5. Anti-snipe: 99% → 1% exponential fee scheduler over 10 min, dynamic fee on, first-swap-with-min-fee disabled, scheduled permissionless `open_curve`, top-holder disclosure.
6. Salted leaf v2 art binding, IPFS CIDs (plus an Arweave mirror), an encrypted manifest at launch, and the verifier script.
7. Collection-level Royalties only (ruleset None) if royalties are wanted. No delegate or permanent plugins.

## 8. Open questions for Barton
1. Batch pre-mint (N ≤ 10k, slice scales, or a higher threshold for large N) or lazy mint (any N, capturer pays ~0.005 SOL, incomplete marketplace page at graduation)?
2. DBC buffer supply: accept a burn below 1B, or keep the leftover in a locked PDA (recommended), or add it to LP?
3. Where do curve trading fees (creator and partner shares), the curve surplus, and the leftover graduation fund go: LP, the fee address, or the creator?
4. Max slice percentage (proposed 10%) and default threshold (proposed 85 SOL)?
5. Drop the 10k ratio? 50k? (2.5M and 5M: see marketplaces-and-ratios.md Part B.)
6. Who holds the art decryption key until graduation?
7. Tensor verification: the Creator Portal suggests signing with the update-authority wallet. Our update authority is a PDA, so the claim must go through the creator's X account. OK?
8. Accept a fee-only anti-snipe gate (no per-wallet caps, no native delayed activation) for M-03?

## 9. New threat rows (proposed for THREAT_MODEL.md; not written there)

| ID | Threat | Severity | Mitigation | Test |
|---|---|---|---|---|
| T-GRAD-01 | **Crank griefing via PDA pre-funding.** 0.00089 SOL per index blocks Core create forever, so converting never opens (measured) | High | Drain-then-create in the same ix (tested); applies to asset, collection and mint PDAs | `attack_prefund_asset_pda_does_not_block_mint` |
| T-GRAD-02 | Crank griefing via junk txs, index collisions or stalling | Low | Bitmap, invalid proofs revert, permissionless with a reward, sharded ranges | `mint_crank_resumes_after_interruption_by_any_caller` |
| T-GRAD-03 | **Funding shortfall** (mis-sized slice, Core fee raise, rent change) | High | Launch-time affordability with 25% margin; live cost read; top-up; never partial-open | `launch_rejected_when_slice_cannot_fund_n`; `crank_stalls_not_skips_on_shortfall` |
| T-GRAD-04 | **Graduation spoofing** with a fake or other pool account | Critical | Owner == DBC, VirtualPool discriminator, PDA re-derivation, config/mint/creator match, `is_migrated` | `attack_fake_pool_account_graduate_fails` (×4 variants) |
| T-GRAD-05 | **Art swap** after reveal | High | Salted leaf with image/JSON hashes + CID URI; on-chain URI/attributes from the leaf; immutable authorities; verifier | `verify_script_detects_swapped_image` |
| T-GRAD-06 | **Premature opening** of converting | Critical | Hard rule: graduated ∧ minted == N ∧ bitmap full ∧ collection `current_size` == N; no override | `converting_closed_until_fully_minted` |
| T-GRAD-07 | DBC layout or program upgrade breaks or subverts `graduate` | Medium | Pinned offsets, fail closed, monitoring, timelocked fix | `graduate_fails_closed_on_unknown_layout` |
| T-GRAD-08 | Manifest or key withheld at graduation | High | Encrypted manifest on Arweave at launch + key-hash commitment; tokens stay tradeable | `crank_can_run_from_public_manifest_only` |
| T-GRAD-09 | Mint PDA pre-funded before `open_curve` (DBC `init`) | Medium | Test that DBC/Anchor init tolerates lamports; otherwise drain first | `attack_prefund_mint_pda_does_not_block_open_curve` |
| T-GRAD-10 | (If LaunchLab were chosen) privileged Raydium migrator stalls or reorders migration | High | Don't choose LaunchLab | — |

## 10. Unverified or estimated
- Mainnet wall-clock minting throughput (estimated from the 12M CU per-account write limit and assumed landing rates); the SIMD-0306 40M claim.
- Whether DBC's `JADaUV8k…` and Raydium's `FytDrVzD…` upgrade authorities are multisigs.
- DBC `init` behaviour with a pre-funded mint PDA; the exact CU for on-chain leaf verification (estimated at 15–25k).
- Whether bundlers accept delayed ANS-104 items.
- LaunchLab `open_time` behaviour and its 24 vs 30 SOL minimum (inconsistent docs).
- The illustrative 400 SOL migration market cap.
- Priority fee of 20k lamports per tx (assumed).
- Keeper auto-migration latency (docs only, not observed).
