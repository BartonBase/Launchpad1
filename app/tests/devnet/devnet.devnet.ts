/**
 * DEVNET checks (free public RPC). Not part of `npm test`; run `npm run test:devnet`.
 * SIMULATE ONLY: getAccountInfo / getProgramAccounts / simulateTransaction (sigVerify false).
 * Nothing is signed or sent. The funded throwaway test wallet is loaded BY PATH at runtime
 * (DEVNET_TEST_WALLET, default /workspace/scratch/frontend-devnet/wallet.json) and only its public
 * key is used, as fee payer / user of the simulations. The secret key is never printed, copied or
 * imported by app code.
 */
import { appendFileSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import { HYBRID_LAUNCH_PROGRAM_ID, HYBRID_VAULT_PROGRAM_ID, MEMO_PROGRAM_ID, PLATFORM_FEE_RECIPIENT, SWITCHBOARD_PROGRAM_ID } from "@/config/programs";
import { CLUSTER } from "@/config/cluster";
import { MIN_GRADUATION_LAMPORTS } from "@/config/armory";
import { DBC_PLATFORM_CONFIG, PINNED_LOOKUP_TABLE_CONTENTS, PINNED_LOOKUP_TABLES, SWITCHBOARD_GATEWAY_RE } from "@/config/integrations";
import { fetchLaunch, fetchLaunches, fetchProgramStatus, findIdleRandomness, fetchHoldings } from "@/lib/armory/reads";
import { buildUnwrapTx, fetchDbcGraduationLamports, buildVrfRequestTx, MAX_TX_BYTES, txSize } from "@/lib/armory/builders";
import { leafSourceFor } from "@/lib/armory/leaves";
import { checkGatewayUrl, gatewayRevealFetcher, parseRevealResponse, revealRequestBody } from "@/lib/armory/reveal";
import { hybridVaultErrors, hybridLaunchErrors } from "@/lib/generated/idlMeta";
import { assetPda, decodeVault, settleIx } from "@/lib/generated/hybridVault";
import { decodeCoreAssetOwner } from "@/lib/generated/core";
import { decodeOracleGatewayUri, decodeQueue, decodeRandomness } from "@/lib/generated/switchboard";
import { simulate } from "@/lib/tx/simulate";
import { validateInstructions } from "@/lib/tx/validate";
import { checkLookups, contentsMatch, resolveLookups } from "@/lib/tx/lookup";
import { viewMessage, messageBytes, messageFingerprint } from "@/lib/tx/message";
import { buildTxPreview } from "@/lib/tx/preview";
import { buildPlainLaunchTx, quoteDevBuy } from "@/lib/meteora/dbc";

const conn = new Connection("https://api.devnet.solana.com", "confirmed");
const log = (...a: unknown[]) => { const line = `[devnet] ${a.map((x) => (typeof x === "string" ? x : JSON.stringify(x, (_k, v) => (typeof v === "bigint" ? v.toString() : v)))).join(" ")}\n`; console.log(line); if (process.env.DEVNET_LOG) appendFileSync(process.env.DEVNET_LOG, line); };

// E2E launch (2026-10-01). Fee is read from chain, never hard-coded.
const E2E_MINT = new PublicKey("3GC9zFWzE2fVTFM7Q9Zo3BqCUArPYQEK57UVv4zvpJAu");
const E2E_VAULT = new PublicKey("HSgG9nxvRLjqwnb7MeHzo2okA6eKRyLdBuPvP9MxYMwa");
const E2E_LC = new PublicKey("FKAzFsdDGShgtiMntihnM6BUd8D6fDNMYsUut6en2kYD");
const E2E_DBC_POOL = "6VbgZdmKtsCJyf3h8w4AWFBXtKFrxBX9Lm7ScBZYTk7d";
// The app's devnet test launch "Armory Test" (ARMT; 2026-10-01): DBC config
// DuQY… + register_dbc_launch, graduated to DAMM v2, vault open. Capture minted #56 (back in the
// vault); re-roll minted #68 (held by their throwaway wallet).
const ARMT = {
  mint: "Dhv3VkqeTYJiahzcVJLmJ1snApdYtQxsAKeUnv5GTNos", launchConfig: "AGVZd96xUp1WSadGY6TWwsZi6C5CzY7aNsk6fHm66F6m",
  vault: "MwFTkPQ2TReZo5axKkXTUKC4sBoBmrSKs4JwAhFjPHK", pool: "B8r2rRuhoSWSdyMxDFZL2YRrRWqg13iiZfWyxSBqUA3D",
  dbcPool: "8n3dZKxPfYYV7kZkDxW8CrtKTm4RKhtHyQ1WqdFhapj3", collection: "8Gnr6DbAgz9XHmniFwSRAKXSvotH1QG1iDQkP5LpzBYs",
  alt: "5xhFeeakpaggTw9Ntbjt8yuZSHEoXPTUd63h4tVmZVsU", nft68: "A2NZs5gZF78T2FCWe9bEdLaaK18ZtGwSDndidCpmqT2n",
} as const;
const WALLET_PUBKEY = "7TyRAirKobno8RmcxvLjtM2kuCj7WBK5UyRCJJZHipZX";

/** Public key of the throwaway wallet, read from its keypair file at runtime (secret stays in memory). */
function testWallet(): PublicKey {
  const path = process.env.DEVNET_TEST_WALLET ?? "/workspace/scratch/frontend-devnet/wallet.json";
  const kp = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(readFileSync(path, "utf8")) as number[]));
  if (kp.publicKey.toBase58() !== WALLET_PUBKEY) throw new Error("Unexpected test wallet");
  return kp.publicKey;
}

async function simLegacy(tx: Transaction) {
  tx.recentBlockhash = (await conn.getLatestBlockhash()).blockhash;
  const r = await conn.simulateTransaction(tx);
  return describe_(r.value);
}
async function simV0(tx: VersionedTransaction) {
  return describe_((await conn.simulateTransaction(tx, { sigVerify: false, replaceRecentBlockhash: true })).value);
}
function describe_(v: { err: unknown; unitsConsumed?: number; logs: string[] | null }) {
  const code = /"Custom":(\d+)/.exec(JSON.stringify(v.err))?.[1];
  const name = code ? (hybridVaultErrors[Number(code)]?.name ?? hybridLaunchErrors[Number(code)]?.name ?? "?") : null;
  return { err: v.err, code: code ? Number(code) : null, name, units: v.unitsConsumed, logs: v.logs ?? [] };
}

/** The same steps useSafeSend runs before showing the preview (validate → lookups → simulate → preview). */
async function pipelinePreview(tx: VersionedTransaction, payer: PublicKey, pinned: PublicKey[] = []) {
  const validation = validateInstructions(tx, CLUSTER.name);
  const lookups = await resolveLookups(conn, tx.message, pinned);
  const view = viewMessage(tx, lookups);
  const simulation = validation.ok ? await simulate(conn, tx, { signer: payer, lookups }) : null;
  return buildTxPreview({ cluster: CLUSTER, view, fingerprint: await messageFingerprint(messageBytes(tx)), validation, simulation, feeLamports: 5000n, signer: payer.toBase58() });
}

describe.sequential("devnet", { timeout: 180_000 }, () => {
  const wallet = testWallet();

  it("both programs exist and are executable", async () => {
    const st = await fetchProgramStatus(conn);
    expect(st.map((p) => p.programId)).toEqual([HYBRID_LAUNCH_PROGRAM_ID.toBase58(), HYBRID_VAULT_PROGRAM_ID.toBase58()]);
    for (const p of st) expect(p.executable).toBe(true);
  });

  let open: Awaited<ReturnType<typeof fetchLaunches>>[number] | undefined;
  it("reads the E2E launch (DBC launch, graduated, open vault, authorities revoked)", async () => {
    const all = await fetchLaunches(conn);
    expect(all.length).toBeGreaterThan(0);
    open = (await fetchLaunch(conn, E2E_MINT)) ?? undefined;
    expect(open?.vault).toBe(E2E_VAULT.toBase58());
    expect(open?.launchConfig).toBe(E2E_LC.toBase58());
    expect(open?.vaultOpen).toBe(true);
    expect(open?.mintAuthority).toBeNull();
    expect(open?.freezeAuthority).toBeNull();
    // A DBC launch (DuQY... config + register_dbc_launch, graduated 2026-09-26), not native.
    expect(open?.dbcPool).toBe(E2E_DBC_POOL);
    expect(open?.state).toBe("graduated");
    // The 10/1 app capture run minted #48 (capture) and #91 (re-roll) through the app's builders.
    expect(open!.mintedCount).toBeGreaterThanOrEqual(4);
    expect(open!.totalCaptures).not.toBe("0");
    expect(await conn.getAccountInfo(assetPda(E2E_VAULT, 48))).not.toBeNull();
    expect(await conn.getAccountInfo(assetPda(E2E_VAULT, 91))).not.toBeNull();
    log("E2E launch", { ratio: open!.ratioWholeTokens, n: open!.collectionSize, feeLamportsFromChain: open!.feeLamports, minted: open!.mintedCount, idleRandomness: (await findIdleRandomness(conn, E2E_VAULT))?.toBase58() ?? null });
  });

  it("reads the ARMT test launch (DBC, graduated, vault open, #56 and #68 minted, leaf root matches)", async () => {
    const l = await fetchLaunch(conn, new PublicKey(ARMT.mint));
    expect(l).not.toBeNull();
    expect(l!.launchConfig).toBe(ARMT.launchConfig);
    expect(l!.vault).toBe(ARMT.vault);
    expect(l!.dbcPool).toBe(ARMT.dbcPool);
    expect(l!.collection).toBe(ARMT.collection);
    expect(l!.state).toBe("graduated");
    expect(l!.vaultOpen).toBe(true);
    expect(l!.ratioWholeTokens).toBe("1000000");
    expect(l!.collectionSize).toBe(100);
    expect(l!.feeLamports).toBe("10000000");
    expect(l!.decimals).toBe(6);
    expect(l!.mintAuthority).toBeNull();
    expect(l!.mintedCount).toBeGreaterThanOrEqual(2);
    const vault = new PublicKey(ARMT.vault);
    const v = decodeVault(new Uint8Array((await conn.getAccountInfo(vault))!.data));
    expect(v.pool.toBase58()).toBe(ARMT.pool);
    expect(assetPda(vault, 68).toBase58()).toBe(ARMT.nft68);
    const [a56, a68] = await conn.getMultipleAccountsInfo([assetPda(vault, 56), assetPda(vault, 68)]);
    expect(a56).not.toBeNull();
    expect(a68).not.toBeNull();
    const lc = new PublicKey(ARMT.launchConfig);
    expect(Buffer.from(await leafSourceFor(lc)!.root(lc)).toString("hex")).toBe(Buffer.from(v.traitRoot).toString("hex"));
    log("ARMT", { minted: l!.mintedCount, captures: l!.totalCaptures, rerolls: l!.totalRerolls, nft68Owner: decodeCoreAssetOwner(new Uint8Array(a68!.data))?.toBase58() });
  });

  it("ARMT settle-with-mint through the pinned lookup table: contents verified, accounts listed; a wrong pinned list blocks", async () => {
    const altKey = new PublicKey(ARMT.alt);
    expect(PINNED_LOOKUP_TABLES.devnet.map(String)).toContain(ARMT.alt);
    const table = (await conn.getAddressLookupTable(altKey)).value!;
    expect(table.isActive()).toBe(true);
    expect(contentsMatch(table, PINNED_LOOKUP_TABLE_CONTENTS[ARMT.alt])).toBe(true);
    const vault = new PublicKey(ARMT.vault);
    const v = decodeVault(new Uint8Array((await conn.getAccountInfo(vault))!.data));
    const lc = new PublicKey(ARMT.launchConfig);
    const mint = await leafSourceFor(lc)!.mintArgs(lc, 99);
    const ix = settleIx({ kind: "capture", settler: wallet, vault, vaultData: v, request: PublicKey.unique(), seq: v.nextSeq, randomness: PublicKey.unique(), user: wallet, assetIndex: 99, mint });
    const msg = (alts: typeof table[]) => new VersionedTransaction(new TransactionMessage({ payerKey: wallet, recentBlockhash: PublicKey.default.toBase58(), instructions: [ix] }).compileToV0Message(alts));
    const plain = msg([]);
    const withAlt = msg([table]);
    // Signers and per-request accounts (request, escrow, rand lock, randomness, asset) stay static.
    const loaded = new Set(table.state.addresses.map(String));
    for (const k of [wallet, assetPda(vault, 99)]) expect(loaded.has(k.toBase58())).toBe(false);
    const p = await pipelinePreview(withAlt, wallet, [...PINNED_LOOKUP_TABLES.devnet]);
    log("ARMT settle ALT", { bytesPlain: txSize(plain), bytesAlt: txSize(withAlt), tables: p.lookupTables.map((t) => ({ a: t.address, pinned: t.pinned, frozen: t.frozen, verified: t.verified, n: t.writable.length + t.readonly.length })), lookupErrors: p.errors.filter((e) => e.includes("Lookup table")) });
    expect(txSize(withAlt)).toBeLessThan(txSize(plain));
    expect(p.lookupTables[0]?.verified).toBe(true);
    expect(p.errors.filter((e) => e.includes("Lookup table"))).toEqual([]);
    // Same table, but a pinned list that differs by one entry: blocked before preview.
    const fetched = new Map([[ARMT.alt, table]]);
    const wrong = { [ARMT.alt]: [...PINNED_LOOKUP_TABLE_CONTENTS[ARMT.alt]!.slice(0, -1), PublicKey.unique().toBase58()] };
    expect(checkLookups(withAlt.message, fetched, [altKey], wrong).errors.some((e) => e.includes("doesn't match its pinned contents"))).toBe(true);
  });

  it("capture from the funded wallet: oracle retry + full pipeline preview (simulate only)", async () => {
    const h = await fetchHoldings(conn, wallet, open!);
    log("wallet", wallet.toBase58(), "SOL", (await conn.getBalance(wallet)) / 1e9, "tokens(base)", h.tokenBase.toString(), "nfts", h.nftIndexes);
    const b = await buildVrfRequestTx(conn, wallet, E2E_VAULT, { type: "capture" });
    log("capture build", { ok: b.simulatedOk, note: b.simulationNote, attempts: b.attempts, oracle: b.oracle.toBase58(), stale: b.staleProofs.map(String), initRandomness: b.initRandomness, cu: b.unitsConsumed, bytes: txSize(b.tx) });
    expect(b.simulatedOk).toBe(true);
    const p = await pipelinePreview(b.tx, wallet);
    const fee = BigInt(open!.feeLamports);
    const tok = p.tokenChanges.find((t) => t.owner === wallet.toBase58());
    const feeRx = p.solChanges.find((c) => c.address === PLATFORM_FEE_RECIPIENT.toBase58());
    log("capture preview", { canSend: p.canSend, errors: p.errors, cu: p.unitsConsumed, walletTokenDelta: tok?.deltaAmount.toString(), feeRecipientDelta: feeRx?.deltaLamports.toString(), programs: p.programs.map((x) => `${x.name ?? x.programId}${x.cpiOnly ? " (cpi)" : ""}`) });
    expect(p.canSend).toBe(true);
    expect(tok?.deltaAmount).toBe(-BigInt(open!.ratioBase));
    expect(feeRx?.deltaLamports).toBe(fee);
  });

  it("capture oracle retry recovers from a deliberately wrong first oracle (6050 -> Right:)", async () => {
    const b = await buildVrfRequestTx(conn, wallet, E2E_VAULT, { type: "capture" }, { reorderCandidates: (c) => [...c.slice(1), c[0]!] });
    log("capture (rotated candidates)", { ok: b.simulatedOk, note: b.simulationNote, attempts: b.attempts, oracle: b.oracle.toBase58(), stale: b.staleProofs.map(String) });
    expect(b.simulatedOk).toBe(true);
    expect(b.attempts).toBeGreaterThan(1);
  });

  it("re-roll: simulate from the funded wallet (and from any wallet that holds an NFT)", async () => {
    const h = await fetchHoldings(conn, wallet, open!);
    const assets = await conn.getMultipleAccountsInfo(Array.from({ length: open!.collectionSize }, (_, i) => assetPda(E2E_VAULT, i)));
    const held = assets.flatMap((a, i) => {
      const owner = a ? decodeCoreAssetOwner(new Uint8Array(a.data)) : null;
      return owner ? [{ i, owner: owner.toBase58(), onCurve: PublicKey.isOnCurve(owner.toBytes()) }] : [];
    });
    log("minted NFTs and owners", held, "wallet NFTs", h.nftIndexes);
    const holder = held.find((x) => x.onCurve);
    if (h.nftIndexes.length || holder) {
      const user = h.nftIndexes.length ? wallet : new PublicKey(holder!.owner);
      const index = h.nftIndexes[0] ?? holder!.i;
      const b = await buildVrfRequestTx(conn, user, E2E_VAULT, { type: "reroll", index });
      log("re-roll build", { user: user.toBase58(), index, ok: b.simulatedOk, note: b.simulationNote, attempts: b.attempts, oracle: b.oracle.toBase58(), stale: b.staleProofs.map(String), cu: b.unitsConsumed });
      expect(b.simulatedOk).toBe(true);
      const p = await pipelinePreview(b.tx, user);
      expect(p.canSend).toBe(true);
    }
    // The funded wallet holds no NFT, so its re-roll of a minted index must be rejected by the
    // program itself (ownership/hand-in), after account validation and with the oracle resolved.
    const idx = held[0]?.i ?? 0;
    const w = await buildVrfRequestTx(conn, wallet, E2E_VAULT, { type: "reroll", index: idx });
    const r = await simV0(w.tx);
    log("re-roll from funded wallet", { index: idx, ok: w.simulatedOk, attempts: w.attempts, oracle: w.oracle.toBase58(), err: r.err, code: r.code, name: r.name, tail: r.logs.slice(-4) });
    expect(r.logs.some((l) => l.includes(`Program ${HYBRID_VAULT_PROGRAM_ID.toBase58()} invoke [1]`))).toBe(true);
    expect([6050, 6053]).not.toContain(r.code ?? 0);
  });

  it("settle-with-mint: E2E leaf source matches the on-chain trait_root; size vs the 1,232-byte limit", async () => {
    const vi = await conn.getAccountInfo(E2E_VAULT);
    const v = decodeVault(new Uint8Array(vi!.data));
    const src = leafSourceFor(E2E_LC)!;
    const root = await src.root(E2E_LC);
    expect(Buffer.from(root).toString("hex")).toBe(Buffer.from(v.traitRoot).toString("hex"));
    const mint = await src.mintArgs(E2E_LC, 99);
    const ix = settleIx({ kind: "capture", settler: wallet, vault: E2E_VAULT, vaultData: v, request: PublicKey.unique(), seq: v.nextSeq, randomness: PublicKey.unique(), user: wallet, assetIndex: 99, mint });
    const tx = new VersionedTransaction(new TransactionMessage({ payerKey: wallet, recentBlockhash: PublicKey.default.toBase58(), instructions: [ix] }).compileToV0Message());
    const bytes = txSize(tx);
    // Proof depth for N=10,000 is 14 (vs 7 here): estimate the big-collection size too.
    log("settle-with-mint", { index: 99, proofLen: mint.proof.length, bytes, limit: MAX_TX_BYTES, estimatedBytesN10000: bytes + 7 * 32 + 40 });
    expect(bytes).toBeLessThanOrEqual(MAX_TX_BYTES);
  });

  it("reveal: every queue oracle's gateway passes the allowlist; the SDK gateway answers (read-only)", async () => {
    const vi = await conn.getAccountInfo(E2E_VAULT);
    const v = decodeVault(new Uint8Array(vi!.data));
    const q = decodeQueue(new Uint8Array((await conn.getAccountInfo(v.sbQueue))!.data));
    const infos = await conn.getMultipleAccountsInfo([...q.oracleKeys]);
    const urls = infos.map((i) => decodeOracleGatewayUri(new Uint8Array(i!.data)));
    log("gateways", urls);
    for (const u of urls) expect(() => checkGatewayUrl(u, "devnet")).not.toThrow();
    expect(SWITCHBOARD_GATEWAY_RE.devnet!.test("https://evil.example/devnet")).toBe(false);
    // Ask the assigned oracle's gateway for the (already revealed) E2E randomness: proves the SDK
    // path + host from node. No transaction is built from it.
    const r = await findIdleRandomness(conn, E2E_VAULT);
    if (!r) return log("no randomness account to probe");
    const rd = decodeRandomness(new Uint8Array((await conn.getAccountInfo(r))!.data));
    log("idle randomness", r.toBase58(), { oracle: rd.oracle.toBase58(), seedSlot: rd.seedSlot.toString(), revealSlot: rd.revealSlot.toString() });
    if (rd.oracle.equals(PublicKey.default)) return log("idle randomness was never committed; nothing to probe");
    const oi = await conn.getAccountInfo(rd.oracle);
    const url = checkGatewayUrl(decodeOracleGatewayUri(new Uint8Array(oi!.data)));
    try {
      const rev = parseRevealResponse(await gatewayRevealFetcher(url, revealRequestBody({ randomness: r, seedSlothash: rd.seedSlothash, seedSlot: rd.seedSlot, rpc: CLUSTER.rpcUrl })));
      log("gateway reveal", { url, sigBytes: rev.signature.length, recoveryId: rev.recoveryId, valueLen: rev.value.length });
    } catch (e) {
      log("gateway reveal probe failed (non-fatal)", url, String(e).slice(0, 200));
    }
  });

  it("lookup tables: a real devnet table is resolved and listed; unpinned/unfrozen tables block the preview", async () => {
    // Switchboard's lookup table for the E2E randomness account (created by init_randomness).
    const r = await findIdleRandomness(conn, E2E_VAULT);
    expect(r).not.toBeNull();
    const sigs = await conn.getSignaturesForAddress(r!, { limit: 50 });
    const oldest = sigs[sigs.length - 1]!;
    const t = await conn.getTransaction(oldest.signature, { maxSupportedTransactionVersion: 0, commitment: "confirmed" });
    const keys = t!.transaction.message.getAccountKeys().staticAccountKeys;
    const infos = await conn.getMultipleAccountsInfo(keys);
    const alt = keys.find((k, i) => infos[i]?.owner.toBase58() === "AddressLookupTab1e1111111111111111111111111");
    expect(alt).toBeDefined();
    const table = (await conn.getAddressLookupTable(alt!)).value!;
    log("devnet ALT", alt!.toBase58(), "entries", table.state.addresses.length, "authority", table.state.authority?.toBase58() ?? null);
    const target = table.state.addresses[0]!;
    const memo = new TransactionInstruction({ programId: MEMO_PROGRAM_ID, keys: [{ pubkey: target, isSigner: false, isWritable: false }], data: Buffer.from("alt-check") });
    const tx = new VersionedTransaction(new TransactionMessage({ payerKey: wallet, recentBlockhash: PublicKey.default.toBase58(), instructions: [memo] }).compileToV0Message([table]));
    expect(tx.message.addressTableLookups.length).toBe(1);
    const unpinned = await pipelinePreview(tx, wallet, []);
    const pinned = await pipelinePreview(tx, wallet, [alt!]);
    log("ALT preview (unpinned)", { canSend: unpinned.canSend, errors: unpinned.errors, tables: unpinned.lookupTables });
    log("ALT preview (pinned)", { canSend: pinned.canSend, errors: pinned.errors });
    expect(unpinned.lookupTables[0]?.readonly).toContain(target.toBase58());
    expect(unpinned.errors.some((e) => e.includes("not pinned"))).toBe(true);
    expect(pinned.errors.some((e) => e.includes("not pinned"))).toBe(false);
    if (table.state.authority) expect(pinned.canSend).toBe(false); // has an authority -> still blocked
  });

  it("launch with dev buy: DBC pool + first buy in one tx from the funded wallet (simulate only)", async () => {
    expect(DBC_PLATFORM_CONFIG.devnet?.toBase58()).toBe("DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9");
    expect(await fetchDbcGraduationLamports(conn)).toBe(MIN_GRADUATION_LAMPORTS.devnet);
    const q = await quoteDevBuy(conn, 10_000_000n, 100);
    expect(q.tokensOut > 0n).toBe(true);
    const b = await buildPlainLaunchTx(conn, wallet, { name: "Armory Sim", symbol: "ARMSIM", uri: "https://example.invalid/armory-sim.json" }, { lamports: 10_000_000n, minimumAmountOut: q.minimumAmountOut });
    b.tx.feePayer = wallet;
    b.tx.recentBlockhash = (await conn.getLatestBlockhash()).blockhash;
    b.tx.partialSign(...b.signers); // throwaway mint key, never persisted
    const r = await conn.simulateTransaction(b.tx);
    log("pool + dev buy", { mint: b.mint.toBase58(), pool: b.pool.toBase58(), tokensOut: q.tokensOut.toString(), feePct: q.feePct, err: r.value.err, tail: r.value.logs?.slice(-3) });
    expect(r.value.err).toBeNull();
  });

  it("unwrap still simulates as before", async () => {
    const u = await simLegacy(await buildUnwrapTx(conn, wallet, E2E_VAULT, 0));
    log("unwrap (wallet, not owner)", u.err ? `ERR ${u.code} ${u.name}` : "OK");
    expect(u.logs.some((l) => l.includes(`Program ${HYBRID_VAULT_PROGRAM_ID.toBase58()} invoke [1]`))).toBe(true);
    expect(SWITCHBOARD_PROGRAM_ID.toBase58()).toBe("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
  });
});
