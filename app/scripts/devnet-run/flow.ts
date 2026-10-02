/**
 * Devnet capture run (node-side only; never imported by the app). Runs capture → reveal → settle →
 * re-roll → reveal → settle → release on the graduated E2E launch through the SAME steps as
 * useSafeSend: build (app builders) → validate (allowlist) → resolve lookups → simulate → preview →
 * refuse unless canSend → sign → tamper check (message bytes unchanged) → send → confirm.
 *
 *   NEXT_PUBLIC_SOLANA_CLUSTER=devnet npx tsx scripts/devnet-run/flow.ts
 *
 * The throwaway keypair is read from DEVNET_TEST_WALLET (default under /workspace/scratch) and is
 * never printed. Resumable: pending requests of the wallet are picked up instead of re-captured.
 * Writes a JSON log to DEVNET_RUN_LOG (default /tmp/devnet-run.json).
 */
import { readFileSync, writeFileSync } from "node:fs";
import { Connection, Keypair, PublicKey, Transaction, VersionedTransaction } from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import { PINNED_LOOKUP_TABLES } from "@/config/integrations";
import { MINT_ESCROW_LAMPORTS, FIRST_MINT_SPEND_RANGE_LAMPORTS } from "@/config/armory";
import { buildRevealTx, buildSettleTx, buildUnwrapTx, buildVrfRequestTx } from "@/lib/armory/builders";
import { fetchHoldings, fetchLaunch, fetchUserRequests } from "@/lib/armory/reads";
import { decodeRequest, decodeVault, requestPda } from "@/lib/generated/hybridVault";
import { bytesEqual, messageBytes, messageFingerprint, toVersionedMessage, viewMessage, type AnyTransaction } from "@/lib/tx/message";
import { buildTxPreview } from "@/lib/tx/preview";
import { estimateFee, simulate } from "@/lib/tx/simulate";
import { validateInstructions } from "@/lib/tx/validate";
import { resolveLookups } from "@/lib/tx/lookup";

const MINT = new PublicKey(process.env.RUN_MINT ?? "3GC9zFWzE2fVTFM7Q9Zo3BqCUArPYQEK57UVv4zvpJAu");
const RESERVE = 50_000_000; // keep 0.05 SOL
const STEP_BUDGET = 20_000_000; // worst case per request step (fee 0.01 + deposit 0.0063 + rent + tx fees)
const LOG = process.env.DEVNET_RUN_LOG ?? "/tmp/devnet-run.json";
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const big = (_k: string, v: unknown) => (typeof v === "bigint" ? v.toString() : v);

if (CLUSTER.name !== "devnet") throw new Error("REFUSING: cluster is not devnet");
const conn = new Connection(CLUSTER.rpcUrl, "confirmed");
const wallet = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(readFileSync(process.env.DEVNET_TEST_WALLET ?? "/workspace/scratch/frontend-devnet/wallet.json", "utf8")) as number[]));
const me = wallet.publicKey;
const out: { wallet: string; steps: Record<string, unknown>[]; start?: number; end?: number } = { wallet: me.toBase58(), steps: [] };
const save = () => writeFileSync(LOG, JSON.stringify(out, big, 2));
const log = (m: string, o: unknown = "") => console.log(`[run] ${m}`, typeof o === "string" ? o : JSON.stringify(o, big));

/** useSafeSend.start + confirm, minus the human click: refuses to send if the preview blocks. */
async function safeSend(label: string, built: { tx: AnyTransaction; signers?: readonly Keypair[] }, extra: Record<string, unknown> = {}) {
  const { tx } = built;
  const signers = built.signers ?? [];
  if (tx instanceof Transaction) {
    tx.feePayer ??= me;
    tx.recentBlockhash ??= (await conn.getLatestBlockhash("confirmed")).blockhash;
  }
  const validation = validateInstructions(tx, CLUSTER.name);
  const lookups = await resolveLookups(conn, toVersionedMessage(tx), PINNED_LOOKUP_TABLES[CLUSTER.name]);
  const view = viewMessage(tx, lookups);
  const bytes = messageBytes(tx);
  const fingerprint = await messageFingerprint(bytes);
  const [simulation, fee] = validation.ok ? await Promise.all([simulate(conn, tx, { signer: me, lookups }), estimateFee(conn, tx)]) : [null, null];
  const p = buildTxPreview({ cluster: CLUSTER, view, fingerprint, validation, simulation, feeLamports: fee, signer: me.toBase58() });
  const predicted = p.solChanges.find((c) => c.address === me.toBase58())?.deltaLamports ?? 0n;
  const tokenPred = p.tokenChanges.find((c) => c.owner === me.toBase58())?.deltaAmount ?? 0n;
  if (!p.canSend) {
    out.steps.push({ label, blocked: p.errors, warnings: p.warnings, ...extra });
    save();
    throw new Error(`${label}: preview blocks sending: ${p.errors.join(" | ")}`);
  }
  const bal = await conn.getBalance(me, "confirmed");
  if (bal < RESERVE + STEP_BUDGET) throw new Error(`${label}: balance ${bal / 1e9} SOL would dip into the 0.05 SOL reserve; stopping.`);
  // Sign exactly what was previewed, then verify the bytes didn't change.
  if (tx instanceof VersionedTransaction) tx.sign([...signers, wallet]);
  else tx.partialSign(...signers, wallet);
  if (!bytesEqual(messageBytes(tx), bytes)) throw new Error(`${label}: message changed after preview; nothing sent`);
  const sig = await conn.sendRawTransaction(tx.serialize(), { skipPreflight: false, preflightCommitment: "confirmed", maxRetries: 3 });
  const res = await conn.confirmTransaction(sig, "confirmed");
  if (res.value.err) throw new Error(`${label}: failed on-chain ${JSON.stringify(res.value.err)} (${sig})`);
  let t = await conn.getTransaction(sig, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  for (let i = 0; !t && i < 10; i++) { await sleep(1500); t = await conn.getTransaction(sig, { commitment: "confirmed", maxSupportedTransactionVersion: 0 }); }
  const keys = t ? t.transaction.message.getAccountKeys({ accountKeysFromLookups: t.meta?.loadedAddresses }) : null;
  const idx = keys ? keys.staticAccountKeys.findIndex((k) => k.equals(me)) : -1;
  const actual = t?.meta && idx >= 0 ? BigInt(t.meta.postBalances[idx]! - t.meta.preBalances[idx]!) : null;
  const txFee = t?.meta ? BigInt(t.meta.fee) : null;
  const tokPost = t?.meta?.postTokenBalances?.find((b) => b.owner === me.toBase58() && b.mint === MINT.toBase58())?.uiTokenAmount.amount;
  const tokPre = t?.meta?.preTokenBalances?.find((b) => b.owner === me.toBase58() && b.mint === MINT.toBase58())?.uiTokenAmount.amount;
  const step = {
    label, sig, explorer: `https://explorer.solana.com/tx/${sig}?cluster=devnet`,
    previewWalletDelta: predicted, previewFee: fee, actualWalletDelta: actual, actualFee: txFee,
    actualDeltaExFee: actual !== null && txFee !== null ? actual + txFee : null,
    previewTokenDelta: tokenPred, actualTokenDelta: tokPost !== undefined ? BigInt(tokPost) - BigInt(tokPre ?? "0") : null,
    cu: t?.meta?.computeUnitsConsumed, previewCu: p.unitsConsumed, ...extra,
  };
  out.steps.push(step);
  save();
  log(label, step);
  return step;
}

async function reveal(label: string, request: PublicKey) {
  for (let i = 0; i < 12; i++) {
    try {
      return await safeSend(label, { tx: await buildRevealTx(conn, me, request) });
    } catch (e) {
      const m = e instanceof Error ? e.message : String(e);
      if (/preview blocks|failed on-chain|reserve/.test(m) && i >= 5) throw e;
      log(`${label}: not ready (${m.slice(0, 160)}); retrying`);
      await sleep(5000);
    }
  }
  throw new Error(`${label}: gave up`);
}

async function settle(label: string, request: PublicKey) {
  const rq = decodeRequest(new Uint8Array((await conn.getAccountInfo(request))!.data));
  const b = await buildSettleTx(conn, me, request);
  const preBal = await conn.getBalance(me, "confirmed");
  const escrowLamports = rq.mintEscrowLamports;
  const reqRent = (await conn.getAccountInfo(request))!.lamports;
  const s = await safeSend(label, { tx: b.tx }, { assetIndex: b.assetIndex, mints: b.mints, bytes: b.bytes, lookupTables: b.lookupTables, escrowLamports, requestAccountLamports: reqRent });
  return { ...s, preBal, assetIndex: b.assetIndex, mints: b.mints };
}

async function main() {
  if ((await conn.getGenesisHash()) !== "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG") throw new Error("REFUSING: not devnet genesis");
  const launch = await fetchLaunch(conn, MINT);
  if (!launch?.vault || launch.state !== "graduated") throw new Error(`Launch not graduated/open: ${launch?.state}`);
  const vault = new PublicKey(launch.vault);
  out.start = await conn.getBalance(me);
  const h0 = await fetchHoldings(conn, me, launch);
  log("start", { sol: out.start / 1e9, tokens: h0.tokenBase, nfts: h0.nftIndexes, feeLamports: launch.feeLamports, minted: launch.mintedCount, depositUiPredicts: MINT_ESCROW_LAMPORTS, firstMintRangeUi: FIRST_MINT_SPEND_RANGE_LAMPORTS });

  const pendingOf = async () => (await fetchUserRequests(conn, me, vault.toBase58())).sort((a, b) => Number(a.seq) - Number(b.seq));
  const doRequest = async (kind: "capture" | "reroll", index?: number) => {
    let pend = await pendingOf();
    if (!pend.length) {
      const v = decodeVault(new Uint8Array((await conn.getAccountInfo(vault))!.data));
      const b = await buildVrfRequestTx(conn, me, vault, kind === "capture" ? { type: "capture" } : { type: "reroll", index: index! });
      if (!b.simulatedOk) throw new Error(`${kind} build: ${b.simulationNote}`);
      await safeSend(kind, { tx: b.tx, signers: b.signers }, { seq: v.nextSeq, oracle: b.oracle.toBase58(), initRandomness: b.initRandomness, oracleAttempts: b.attempts, handedIn: index });
      pend = await pendingOf();
      for (let i = 0; !pend.length && i < 5; i++) { await sleep(2000); pend = await pendingOf(); }
    }
    const r = pend[0]!;
    const request = new PublicKey(r.address);
    if (!r.revealed) await reveal(`${r.kind} reveal`, request);
    const vNow = decodeVault(new Uint8Array((await conn.getAccountInfo(vault))!.data));
    if (vNow.nextSettleSeq !== BigInt(r.seq)) log("note: settle order", { nextSettleSeq: vNow.nextSettleSeq, ours: r.seq });
    if (!requestPda(vault, BigInt(r.seq)).equals(request)) throw new Error("request PDA mismatch");
    return settle(`${r.kind} settle`, request);
  };

  const cap = await doRequest("capture");
  const h1 = await fetchHoldings(conn, me, (await fetchLaunch(conn, MINT))!);
  log("after capture", { nfts: h1.nftIndexes, tokens: h1.tokenBase });
  const held = h1.nftIndexes.includes(cap.assetIndex) ? cap.assetIndex : h1.nftIndexes[0]!;
  const rr = await doRequest("reroll", held);
  const h2 = await fetchHoldings(conn, me, (await fetchLaunch(conn, MINT))!);
  log("after re-roll", { nfts: h2.nftIndexes, tokens: h2.tokenBase });
  const relIdx = h2.nftIndexes.includes(rr.assetIndex) ? rr.assetIndex : h2.nftIndexes[0]!;
  if ((await conn.getBalance(me)) > RESERVE + 5_000_000) {
    await safeSend("release", { tx: await buildUnwrapTx(conn, me, vault, relIdx) }, { index: relIdx });
  } else log("release skipped (reserve)");
  const h3 = await fetchHoldings(conn, me, (await fetchLaunch(conn, MINT))!);
  out.end = await conn.getBalance(me);
  log("end", { sol: out.end / 1e9, spent: (out.start - out.end) / 1e9, nfts: h3.nftIndexes, tokens: h3.tokenBase });
  Object.assign(out, { holdings: { h0, h1, h2, h3 } });
  save();
}

main().catch((e) => {
  console.error("[run] STOPPED:", e instanceof Error ? e.message : e);
  save();
  process.exit(1);
});
