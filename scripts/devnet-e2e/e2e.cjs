// DEVNET ONLY. Full run: DBC pool on the platform config -> register_dbc_launch -> init_vault ->
// buys to the 0.1 SOL threshold -> DBC migration to DAMM v2 -> withdraw_leftover -> open_vault ->
// init_randomness -> request_capture -> gateway-signed reveal (third party) -> settle_capture with mint.
// Idempotent: every step checks chain state first, so a rerun resumes. Throwaway keys under $KEYS_DIR.
// Env: DEPLOYER_KEYPAIR (required), KEYS_DIR (default .keys/devnet-e2e), DRY_RUN=1 (simulate only; stops at the first write).
const anchor = require("@coral-xyz/anchor");
const D = require("@meteora-ag/dynamic-bonding-curve-sdk");
const sb = require("@switchboard-xyz/on-demand");
const { Connection, Keypair, PublicKey, SystemProgram, VersionedTransaction, TransactionMessage, ComputeBudgetProgram, SYSVAR_SLOT_HASHES_PUBKEY } = require("@solana/web3.js");
const fs = require("fs"), path = require("path"), crypto = require("crypto");
const ROOT = path.resolve(__dirname, "../..");
const URL = "https://api.devnet.solana.com", DEVNET_GENESIS = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";
const conn = new Connection(URL, "confirmed");
const P = (s) => new PublicKey(s);
const LAUNCH = P("9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8"), VAULT = P("BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG");
const DBC_CONFIG = P("DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9");
const SB = P("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2"), SB_STATE = P("4UFmCebEmzESoDTtHrmaftXj7YAsAH4HMios3yMWyVUT"), QUEUE = P("EYiAmGSdsQTuCw413V5BzaruWuCCSDgTPtBGvLkXHbe7");
const TOKEN = P("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), ATA = P("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"), WSOL = P("So11111111111111111111111111111111111111112");
const ALT = P("AddressLookupTab1e1111111111111111111111111"), CORE = P("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"), FEE = P("7J3AajxfajAMgGmmwzfZeYGNieRtHRCuEgNTfd4gTjDN");
const N = 100, RATIO = 1_000_000;
const DRY = process.env.DRY_RUN === "1";
// REHEARSAL=1: DBC-only dress rehearsal (pool, buys, migration, leftover) on a separate throwaway mint;
// skips every hybrid_* step. Used to measure the DBC costs before the program upgrade.
const REHEARSAL = process.env.REHEARSAL === "1";
const KEYS = path.resolve(ROOT, process.env.KEYS_DIR || ".keys/devnet-e2e");
const OUT = path.join(KEYS, "run.json");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const pda = (seeds, prog) => PublicKey.findProgramAddressSync(seeds, prog)[0];
const ata = (owner, mint) => pda([owner.toBuffer(), TOKEN.toBuffer(), mint.toBuffer()], ATA);
const u32le = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const u64le = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const sha = (...parts) => crypto.createHash("sha256").update(Buffer.concat(parts.map((p) => Buffer.from(p)))).digest();
const load = (p) => Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(p))));
const key = (name) => { const p = path.join(KEYS, name + ".json"); if (!fs.existsSync(p)) fs.writeFileSync(p, JSON.stringify(Array.from(Keypair.generate().secretKey)), { mode: 0o600 }); return load(p); };
const out = fs.existsSync(OUT) ? JSON.parse(fs.readFileSync(OUT)) : { txs: {} };
const save = () => fs.writeFileSync(OUT, JSON.stringify(out, null, 1));
const link = (sig) => `https://explorer.solana.com/tx/${sig}?cluster=devnet`;

async function send(label, ixs, payer, signers, watch = []) {
  const { blockhash } = await conn.getLatestBlockhash();
  const tx = new VersionedTransaction(new TransactionMessage({ payerKey: payer.publicKey, recentBlockhash: blockhash, instructions: ixs }).compileToV0Message());
  tx.sign(signers);
  const sim = await conn.simulateTransaction(tx, { sigVerify: false });
  if (sim.value.err) { console.log(label, "SIM FAIL", JSON.stringify(sim.value.err), "\n  " + (sim.value.logs || []).slice(-8).join("\n  ")); throw new Error(label + " failed in simulation"); }
  if (DRY) { console.log(label, "DRY_RUN ok, cu", sim.value.unitsConsumed); throw new Error("DRY_RUN stop before first write: " + label); }
  const pre = await Promise.all(watch.map((k) => conn.getBalance(k)));
  const sig = await conn.sendTransaction(tx); await conn.confirmTransaction(sig, "confirmed");
  const post = await Promise.all(watch.map((k) => conn.getBalance(k)));
  const t = await conn.getTransaction(sig, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  out.txs[label] = { sig, link: link(sig), fee: t?.meta?.fee, cu: t?.meta?.computeUnitsConsumed, spent: Object.fromEntries(watch.map((k, i) => [k.toBase58(), pre[i] - post[i]])) };
  save(); console.log(label, link(sig), JSON.stringify(out.txs[label].spent));
  return sig;
}
async function simOk(ixs, payer, signers) {
  const { blockhash } = await conn.getLatestBlockhash();
  const tx = new VersionedTransaction(new TransactionMessage({ payerKey: payer.publicKey, recentBlockhash: blockhash, instructions: ixs }).compileToV0Message());
  tx.sign(signers);
  const s = await conn.simulateTransaction(tx, { sigVerify: false });
  return { ok: !s.value.err, err: s.value.err, logs: s.value.logs };
}
const exists = async (k) => !!(await conn.getAccountInfo(k));

// Committed leaves (same scheme as programs/hybrid_vault/src/merkle.rs).
const SCHEMA = sha("mintmark-devnet-e2e-schema");
function leafPre(i) {
  const tv = [...Array(8)].map((_, k) => (i * 7 + k * 3) % 11);
  return { traitValues: tv, salt: [...Buffer.alloc(32, i & 255)], imageSha256: [...Buffer.alloc(32, (i + 1) & 255)], jsonSha256: [...Buffer.alloc(32, (i + 2) & 255)], uri: `ipfs://bafydevnete2e${i}` };
}
function leafHash(lc, i, l) {
  const tvb = Buffer.alloc(16); l.traitValues.forEach((v, k) => tvb.writeUInt16LE(v, 2 * k));
  const th = sha(SCHEMA, tvb), ah = sha(l.salt, l.imageSha256, l.jsonSha256, Buffer.from(l.uri, "utf8"));
  return sha([0], "mintmark-leaf-v2", lc.toBuffer(), u32le(i), th, ah);
}
function merkle(leaves) {
  const levels = [leaves];
  while (levels.at(-1).length > 1) { const c = levels.at(-1), nx = []; for (let j = 0; j < c.length; j += 2) nx.push(sha([1], c[j], c[j + 1] || c[j])); levels.push(nx); }
  const proofs = leaves.map((_, i0) => { let i = i0; const p = []; for (const lv of levels.slice(0, -1)) { p.push(lv[i ^ 1] || lv[i]); i >>= 1; } return p; });
  return { root: levels.at(-1)[0], proofs };
}

(async () => {
  if ((await conn.getGenesisHash()) !== DEVNET_GENESIS) throw new Error("REFUSING: not devnet");
  fs.mkdirSync(KEYS, { recursive: true, mode: 0o700 });
  const deployer = load(path.resolve(ROOT, process.env.DEPLOYER_KEYPAIR || ".keys/devnet-only-deployer.json"));
  const C = key("creator"), U = key("user"), S = key("settler"), M = key("mint"), POOLKP = key("vault_pool"), R = key("randomness");
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(deployer), {});
  const L = new anchor.Program(require(path.join(ROOT, "target/idl/hybrid_launch.json")), provider);
  const V = new anchor.Program(require(path.join(ROOT, "target/idl/hybrid_vault.json")), provider);
  const dbc = new D.DynamicBondingCurveClient(conn, "confirmed");
  const sbp = await sb.AnchorUtils.loadProgramFromConnection(conn);
  const mint = M.publicKey;
  const dbcPool = D.deriveDbcPoolAddress(WSOL, mint, DBC_CONFIG);
  const lc = pda([Buffer.from("launch_config"), mint.toBuffer()], LAUNCH);
  const buffer = pda([Buffer.from("dbc_buffer")], LAUNCH);
  const vault = pda([Buffer.from("vault"), lc.toBuffer()], VAULT);
  const va = pda([Buffer.from("vault_authority"), vault.toBuffer()], VAULT), ra = pda([Buffer.from("randomness_authority"), vault.toBuffer()], VAULT);
  const vt = pda([Buffer.from("vault_tokens"), vault.toBuffer()], VAULT), coll = pda([Buffer.from("collection"), vault.toBuffer()], VAULT);
  Object.assign(out, { mint: mint.toBase58(), dbcPool: dbcPool.toBase58(), launchConfig: lc.toBase58(), vault: vault.toBase58(), creator: C.publicKey.toBase58(), user: U.publicKey.toBase58(), settler: S.publicKey.toBase58() });
  out.balances_start = out.balances_start || { deployer: await conn.getBalance(deployer.publicKey) };
  save();

  // 0. Fund the throwaway actors from the deployer.
  const FUND = REHEARSAL ? [["creator", C, 40_000_000], ["user", U, 115_000_000], ["settler", S, 3_000_000]] : [["creator", C, 60_000_000], ["user", U, 160_000_000], ["settler", S, 25_000_000]];
  for (const [name, kp, amt] of FUND) {
    const b = await conn.getBalance(kp.publicKey);
    if (!out.txs["fund_" + name] && b < amt * 0.8) await send("fund_" + name, [SystemProgram.transfer({ fromPubkey: deployer.publicKey, toPubkey: kp.publicKey, lamports: amt - b })], deployer, [deployer]);
  }
  // 1. DBC pool on the platform config (creator C; DBC creates the mint and revokes its authorities).
  if (!(await exists(dbcPool))) {
    const tx = await dbc.creator.createPool({ name: "Mintmark Devnet E2E", symbol: "MME2E", uri: "https://example.invalid/mintmark-e2e.json", payer: C.publicKey, poolCreator: C.publicKey, config: DBC_CONFIG, baseMint: mint });
    await send("dbc_create_pool", tx.instructions, C, [C, M], [C.publicKey]);
  }
  // 2. register_dbc_launch (needs the devnet-e2e hybrid_launch build: 0.1 SOL floor + platform config allowlisted).
  if (!REHEARSAL && !(await exists(lc))) {
    const ix = await L.methods.registerDbcLaunch({ ratioWholeTokens: new anchor.BN(RATIO), collectionSize: new anchor.BN(N) }).accountsStrict({
      creator: C.publicKey, mint, dbcConfig: DBC_CONFIG, dbcPool, launchConfig: lc, bufferAuthority: buffer, bufferTokens: ata(buffer, mint), feeRecipient: FEE,
      tokenProgram: TOKEN, associatedTokenProgram: ATA, systemProgram: SystemProgram.programId }).instruction();
    await send("register_dbc_launch", [ix], C, [C], [C.publicKey]);
  }
  // 3. init_vault with a real committed trait root.
  const pre = [...Array(N)].map((_, i) => leafPre(i));
  const { root, proofs } = merkle(pre.map((l, i) => leafHash(lc, i, l)));
  if (!REHEARSAL && !(await exists(vault))) {
    const size = 64 + 16 * N + Math.ceil(N / 8);
    const create = SystemProgram.createAccount({ fromPubkey: C.publicKey, newAccountPubkey: POOLKP.publicKey, lamports: await conn.getMinimumBalanceForRentExemption(size), space: size, programId: VAULT });
    const ix = await V.methods.initVault({ traitRoot: [...root], traitSchemaHash: [...SCHEMA], collectionName: "Mintmark Devnet E2E", collectionUri: "ipfs://bafydevnete2ecollection", sbQueue: QUEUE })
      .accountsStrict({ creator: C.publicKey, launchConfig: lc, mint, vault, vaultAuthority: va, randomnessAuthority: ra, vaultTokens: vt, pool: POOLKP.publicKey, collection: coll, mplCoreProgram: CORE, tokenProgram: TOKEN, systemProgram: SystemProgram.programId }).instruction();
    await send("init_vault", [ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }), create, ix], C, [C, POOLKP], [C.publicKey]);
  }
  // 4. Buy up to the migration threshold (PartialFill stops exactly at the threshold).
  const getVp = async () => (await dbc.state.getPool(dbcPool)).poolState;
  let vp = await getVp();
  const cfg = await dbc.state.getPoolConfig(DBC_CONFIG);
  out.threshold = cfg.migrationQuoteThreshold.toString();
  let buys = 0;
  while (vp.quoteReserve.lt(cfg.migrationQuoteThreshold) && buys < 4) {
    const need = cfg.migrationQuoteThreshold.sub(vp.quoteReserve).muln(103).divn(100).addn(10_000); // + fee headroom; PartialFill refunds the rest
    const tx = await dbc.pool.swap2({ owner: U.publicKey, pool: dbcPool, swapBaseForQuote: false, referralTokenAccount: null, payer: U.publicKey, swapMode: D.SwapMode.PartialFill, amountIn: need, minimumAmountOut: new anchor.BN(0) });
    await send("dbc_buy_" + buys, tx.instructions, U, [U], [U.publicKey]);
    buys++; await sleep(1500); vp = await getVp();
  }
  // 5. Graduate: permissionless DBC migration to DAMM v2 (payer = deployer).
  vp = await getVp();
  if (!vp.isMigrated) {
    const r = await dbc.migration.migrateToDammV2({ payer: deployer.publicKey, virtualPool: dbcPool, pool: dbcPool, dammConfig: D.DAMM_V2_MIGRATION_FEE_ADDRESS[0] });
    await send("dbc_migrate_damm_v2", r.transaction.instructions, deployer, [deployer, r.firstPositionNftKeypair, r.secondPositionNftKeypair], [deployer.publicKey]);
  }
  // 6. Unsold buffer -> ATA(["dbc_buffer"], mint) (permissionless; DBC pays only the config's leftover receiver).
  try { const tx = await dbc.migration.withdrawLeftover({ payer: S.publicKey, pool: dbcPool }); await send("dbc_withdraw_leftover", tx.instructions, S, [S], [S.publicKey]); }
  catch (e) { console.log("withdraw_leftover skipped:", String(e.message).slice(0, 120)); }
  if (REHEARSAL) { out.balances_end = { deployer: await conn.getBalance(deployer.publicKey), creator: await conn.getBalance(C.publicKey), user: await conn.getBalance(U.publicKey), settler: await conn.getBalance(S.publicKey) }; out.pool_after = { isMigrated: (await getVp()).isMigrated, migrationProgress: (await getVp()).migrationProgress }; save(); console.log(JSON.stringify(out, null, 1)); return; }
  // 7. open_vault with the migrated DBC pool as the graduation proof.
  let vd = await V.account.vault.fetch(vault);
  if (!vd.open) {
    const ix = await V.methods.openVault().accountsStrict({ caller: S.publicKey, vault, launchConfig: lc, pool: POOLKP.publicKey, vaultTokens: vt, collection: coll, graduationProof: dbcPool }).instruction();
    await send("open_vault", [ix], S, [S], [S.publicKey]);
  }
  // 8. init_randomness (permissionless; the third party pays).
  if (!(await exists(R.publicKey))) {
    const recent = (await conn.getSlot("finalized")) - 1;
    const lutSigner = pda([Buffer.from("LutSigner"), R.publicKey.toBuffer()], SB);
    const ix = await V.methods.initRandomness(new anchor.BN(recent)).accountsStrict({ payer: S.publicKey, vault, randomness: R.publicKey, randomnessAuthority: ra, sbRewardEscrow: ata(R.publicKey, WSOL),
      sbQueue: QUEUE, systemProgram: SystemProgram.programId, tokenProgram: TOKEN, associatedTokenProgram: ATA, wrappedSolMint: WSOL, sbProgramState: SB_STATE, sbLutSigner: lutSigner,
      sbLut: pda([lutSigner.toBuffer(), u64le(recent)], ALT), addressLookupTableProgram: ALT, switchboardProgram: SB }).instruction();
    await send("init_randomness", [ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }), ix], S, [S, R], [S.publicKey]);
  }
  // 9. request_capture by the user. The program picks the oracle; find it (and any stale proofs) by simulation.
  vd = await V.account.vault.fetch(vault);
  let seq = out.seq !== undefined ? out.seq : vd.nextSeq.toNumber();
  const req = pda([Buffer.from("request"), vault.toBuffer(), u64le(seq)], VAULT);
  const esc = pda([Buffer.from("mint_escrow"), vault.toBuffer(), u64le(seq)], VAULT);
  const lock = pda([Buffer.from("rand_lock"), R.publicKey.toBuffer()], VAULT);
  if (!out.txs.request_capture && !(await exists(req))) {
    const q = await new sb.Queue(sbp, QUEUE).loadData();
    const oracles = q.oracleKeys.slice(0, q.oracleKeysLen);
    const base = { user: U.publicKey, vault, launchConfig: lc, pool: POOLKP.publicKey, mint, userToken: ata(U.publicKey, mint), vaultTokens: vt, feeRecipient: FEE, request: req, mintEscrow: esc,
      randLock: lock, randomness: R.publicKey, randomnessAuthority: ra, sbQueue: QUEUE, slotHashes: SYSVAR_SLOT_HASHES_PUBKEY, switchboardProgram: SB, tokenProgram: TOKEN, systemProgram: SystemProgram.programId };
    let chosen = null, lastErr = null;
    for (const o of oracles) for (const stale of [[], ...oracles.filter((x) => !x.equals(o)).map((x) => [x])]) {
      if (chosen) break;
      const ix = await V.methods.requestCapture().accountsStrict({ ...base, sbOracle: o }).remainingAccounts(stale.map((k) => ({ pubkey: k, isSigner: false, isWritable: false }))).instruction();
      const s = await simOk([ix], U, [U]); if (s.ok) chosen = ix; else lastErr = (s.logs || []).slice(-3).join(" | ");
    }
    if (!chosen) throw new Error("request_capture: no oracle/stale-proof combination simulates: " + lastErr);
    out.seq = seq; out.userBeforeCapture = await conn.getBalance(U.publicKey); save();
    await send("request_capture", [chosen], U, [U], [U.publicKey]);
  }
  // 10. Gateway-signed reveal through the vault's permissionless reveal_randomness (third party pays).
  if (!out.txs.reveal_randomness) {
    await sleep(4000);
    const rd = await new sb.Randomness(sbp, R.publicKey).loadData();
    const od = await new sb.Oracle(sbp, rd.oracle).loadData();
    const gw = new sb.Gateway(String.fromCharCode(...od.gatewayUri).replace(/\0+$/, ""));
    let rev = null;
    for (let i = 0; i < 20 && !rev; i++) { try { rev = await gw.fetchRandomnessReveal({ randomnessAccount: R.publicKey, slothash: anchor.utils.bytes.bs58.encode(Buffer.from(rd.seedSlothash)), slot: rd.seedSlot.toNumber(), rpc: URL }); } catch (e) { await sleep(3000); } }
    if (!rev) throw new Error("gateway reveal not available");
    const ix = await V.methods.revealRandomness({ signature: [...Buffer.from(rev.signature, "base64")], recoveryId: rev.recovery_id, value: [...Buffer.from(rev.value)] }).accountsStrict({
      payer: S.publicKey, vault, request: req, randLock: lock, randomness: R.publicKey, randomnessAuthority: ra, sbOracle: rd.oracle, sbQueue: QUEUE,
      sbStats: pda([Buffer.from("OracleRandomnessStats"), rd.oracle.toBuffer()], SB), slotHashes: SYSVAR_SLOT_HASHES_PUBKEY, systemProgram: SystemProgram.programId,
      sbRewardEscrow: ata(R.publicKey, WSOL), tokenProgram: TOKEN, wrappedSolMint: WSOL, sbProgramState: SB_STATE, switchboardProgram: SB }).instruction();
    await send("reveal_randomness", [ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }), ix], S, [S], [S.publicKey]);
  }
  // 11. settle_capture by the third party; the selected index is found by simulating each asset (free).
  if (!out.txs.settle_capture) {
    let done = false;
    for (let i = 0; i < N && !done; i++) {
      const asset = pda([Buffer.from("asset"), vault.toBuffer(), u32le(i)], VAULT);
      const mintArgs = (await exists(asset)) ? null : { leaf: pre[i], proof: proofs[i].map((p) => [...p]) };
      const ix = await V.methods.settleCapture(mintArgs).accountsStrict({ settler: S.publicKey, vault, launchConfig: lc, pool: POOLKP.publicKey, request: req, randLock: lock, randomness: R.publicKey,
        vaultAuthority: va, mintEscrow: esc, vaultTokens: vt, asset, collection: coll, user: U.publicKey, mplCoreProgram: CORE, systemProgram: SystemProgram.programId }).instruction();
      const ixs = [ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 }), ix];
      if ((await simOk(ixs, S, [S])).ok) { out.settledIndex = i; out.asset = asset.toBase58(); out.minted = !!mintArgs; await send("settle_capture", ixs, S, [S], [S.publicKey, U.publicKey, esc]); done = true; }
    }
    if (!done) throw new Error("settle_capture: no index simulates");
  }
  // 12-13 (EXTRAS=1): re-roll the captured NFT (same randomness account), then release (unwrap) the result.
  if (process.env.EXTRAS === "1") {
    const q = await new sb.Queue(sbp, QUEUE).loadData();
    const oracles = q.oracleKeys.slice(0, q.oracleKeysLen);
    const assetOf = (i) => pda([Buffer.from("asset"), vault.toBuffer(), u32le(i)], VAULT);
    const revealAndSettle = async (sq, rq, es, label, kind) => {
      if (!out.txs["reveal_" + label]) {
        await sleep(4000);
        const rd = await new sb.Randomness(sbp, R.publicKey).loadData();
        const od = await new sb.Oracle(sbp, rd.oracle).loadData();
        const gw = new sb.Gateway(String.fromCharCode(...od.gatewayUri).replace(/\0+$/, ""));
        let rev = null;
        for (let i = 0; i < 20 && !rev; i++) { try { rev = await gw.fetchRandomnessReveal({ randomnessAccount: R.publicKey, slothash: anchor.utils.bytes.bs58.encode(Buffer.from(rd.seedSlothash)), slot: rd.seedSlot.toNumber(), rpc: URL }); } catch (e) { await sleep(3000); } }
        if (!rev) throw new Error("gateway reveal not available");
        const ix = await V.methods.revealRandomness({ signature: [...Buffer.from(rev.signature, "base64")], recoveryId: rev.recovery_id, value: [...Buffer.from(rev.value)] }).accountsStrict({
          payer: S.publicKey, vault, request: rq, randLock: lock, randomness: R.publicKey, randomnessAuthority: ra, sbOracle: rd.oracle, sbQueue: QUEUE,
          sbStats: pda([Buffer.from("OracleRandomnessStats"), rd.oracle.toBuffer()], SB), slotHashes: SYSVAR_SLOT_HASHES_PUBKEY, systemProgram: SystemProgram.programId,
          sbRewardEscrow: ata(R.publicKey, WSOL), tokenProgram: TOKEN, wrappedSolMint: WSOL, sbProgramState: SB_STATE, switchboardProgram: SB }).instruction();
        await send("reveal_" + label, [ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }), ix], S, [S], [S.publicKey]);
      }
      if (!out.txs["settle_" + label]) {
        for (let i = 0; i < N; i++) {
          const asset = assetOf(i);
          const mintArgs = (await exists(asset)) ? null : { leaf: pre[i], proof: proofs[i].map((p) => [...p]) };
          const accts = { settler: S.publicKey, vault, launchConfig: lc, pool: POOLKP.publicKey, request: rq, randLock: lock, randomness: R.publicKey,
            vaultAuthority: va, mintEscrow: es, vaultTokens: vt, asset, collection: coll, user: U.publicKey, mplCoreProgram: CORE, systemProgram: SystemProgram.programId };
          const ix = await (kind === "reroll" ? V.methods.settleReroll(mintArgs) : V.methods.settleCapture(mintArgs)).accountsStrict(accts).instruction();
          const ixs = [ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 }), ix];
          if ((await simOk(ixs, S, [S])).ok) { out[label + "Index"] = i; out[label + "Asset"] = asset.toBase58(); out[label + "Minted"] = !!mintArgs; await send("settle_" + label, ixs, S, [S], [S.publicKey, U.publicKey, es, FEE]); return i; }
        }
        throw new Error("settle_" + label + ": no index simulates");
      }
      return out[label + "Index"];
    };
    // Re-roll the NFT captured above.
    if (out.rerollSeq === undefined) out.rerollSeq = (await V.account.vault.fetch(vault)).nextSeq.toNumber();
    const rs = out.rerollSeq;
    const rreq = pda([Buffer.from("request"), vault.toBuffer(), u64le(rs)], VAULT), resc = pda([Buffer.from("mint_escrow"), vault.toBuffer(), u64le(rs)], VAULT);
    if (!(await exists(rreq)) && !out.txs.request_reroll) {
      let chosen = null, lastErr = null;
      for (const o of oracles) for (const stale of [[], ...oracles.filter((x) => !x.equals(o)).map((x) => [x])]) {
        if (chosen) break;
        const ix = await V.methods.requestReroll(out.settledIndex).accountsStrict({ user: U.publicKey, vault, launchConfig: lc, pool: POOLKP.publicKey, vaultTokens: vt, feeRecipient: FEE, vaultAuthority: va,
          asset: assetOf(out.settledIndex), collection: coll, mplCoreProgram: CORE, request: rreq, mintEscrow: resc, randLock: lock, randomness: R.publicKey, randomnessAuthority: ra, sbQueue: QUEUE,
          sbOracle: o, slotHashes: SYSVAR_SLOT_HASHES_PUBKEY, switchboardProgram: SB, systemProgram: SystemProgram.programId }).remainingAccounts(stale.map((k) => ({ pubkey: k, isSigner: false, isWritable: false }))).instruction();
        const sm = await simOk([ix], U, [U]); if (sm.ok) chosen = ix; else lastErr = (sm.logs || []).slice(-3).join(" | ");
      }
      if (!chosen) throw new Error("request_reroll: nothing simulates: " + lastErr);
      out.userBeforeReroll = await conn.getBalance(U.publicKey); save();
      await send("request_reroll", [chosen], U, [U], [U.publicKey, FEE]);
    }
    const ri = await revealAndSettle(rs, rreq, resc, "reroll", "reroll");
    out.userAfterReroll = await conn.getBalance(U.publicKey); save();
    // Release (unwrap) the re-rolled NFT: returns exactly RATIO tokens, no SOL fee.
    if (!out.txs.unwrap) {
      const bal = async () => BigInt((await conn.getTokenAccountBalance(ata(U.publicKey, mint))).value.amount);
      const t0 = await bal();
      const ix = await V.methods.unwrap(ri).accountsStrict({ user: U.publicKey, vault, launchConfig: lc, pool: POOLKP.publicKey, mint, vaultAuthority: va, vaultTokens: vt, userToken: ata(U.publicKey, mint),
        asset: assetOf(ri), collection: coll, mplCoreProgram: CORE, tokenProgram: TOKEN, systemProgram: SystemProgram.programId }).instruction();
      await send("unwrap", [ix], U, [U], [U.publicKey, FEE]);
      out.unwrapTokensReturned = (await bal() - t0).toString(); save();
    }
  }
  out.userAfterSettle = await conn.getBalance(U.publicKey);
  out.userNetSolForCapture = out.userBeforeCapture - out.userAfterSettle;
  out.balances_end = { deployer: await conn.getBalance(deployer.publicKey), creator: await conn.getBalance(C.publicKey), user: out.userAfterSettle, settler: await conn.getBalance(S.publicKey) };
  save(); console.log(JSON.stringify(out, null, 1));
})().catch((e) => { save(); console.error("STOPPED:", e.message); process.exit(1); });
