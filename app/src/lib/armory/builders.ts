/**
 * Transaction builders for the Armory flows. Each returns an UNSIGNED transaction (plus any local
 * signers) for useSafeSend, which then validates, simulates, previews and asks for confirmation.
 * Nothing here signs or sends.
 */
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
  type AddressLookupTableAccount,
  type Connection,
  type TransactionInstruction,
} from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import { DBC_PLATFORM_CONFIG, PINNED_LOOKUP_TABLES } from "@/config/integrations";
import { ADDRESS_LOOKUP_TABLE_PROGRAM_ID, DBC_PROGRAM_ID, SWITCHBOARD_PROGRAM_ID } from "@/config/programs";
import { decodeDbcConfig, dbcPoolPda, initializeVirtualPoolWithSplTokenIx } from "@/lib/generated/dbc";
import type { BuildResult } from "@/lib/tx/useSafeSend";
import { leafSourceFor, type LeafSource } from "./leaves";
import { resolveOracle, rightKeyFromLogs, type OracleState } from "./oracle";
import { fetchReveal, type RevealFetcher } from "./reveal";
import { isExactTierFee } from "@/config/armory";
import { decodeLaunchConfig, launchIx, registerDbcLaunchIx, type LaunchParams } from "@/lib/generated/hybridLaunch";
import {
  assetPda,
  decodeRequest,
  decodeVault,
  expireRequestIx,
  initRandomnessIx,
  requestCaptureIx,
  requestRerollIx,
  revealRandomnessIx,
  settleIx,
  unwrapIx,
  type MintArgs,
  type Vault,
} from "@/lib/generated/hybridVault";
import { createAtaIdempotentIx } from "@/lib/generated/spl";
import { decodeQueue, oracleCandidates, oracleStatsPda, sbLutAccounts } from "@/lib/generated/switchboard";
import { findIdleRandomness } from "./reads";

/** Request.kind values (hybrid_vault state.rs). */
const REQUEST_KIND_REROLL = 1;

const CU_LIMIT = ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 });
const CU_LIMIT_INIT = ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 });
const CU_LIMIT_SETTLE = ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 });
/** Switchboard program state PDA ["STATE"] (= 4UFmCeb… on devnet, checked in test:devnet). */
const sbProgramStatePda = () => PublicKey.findProgramAddressSync([new TextEncoder().encode("STATE")], SWITCHBOARD_PROGRAM_ID)[0];

async function loadVault(conn: Connection, vault: PublicKey): Promise<Vault> {
  const info = await conn.getAccountInfo(vault);
  if (!info) throw new Error("Vault account not found on this cluster.");
  const v = decodeVault(new Uint8Array(info.data));
  if (!v.open) throw new Error("Converting is not open for this collection yet (it opens at graduation).");
  return v;
}

/** Client-side exact-tier guard (devnet vault has no 6061 check). */
async function assertTierFee(conn: Connection, v: Vault): Promise<void> {
  const info = await conn.getAccountInfo(v.launchConfig);
  if (!info) throw new Error("LaunchConfig not found.");
  const lc = decodeLaunchConfig(new Uint8Array(info.data));
  if (!isExactTierFee(lc.ratioWholeTokens, lc.feeLamports)) {
    throw new Error(`Stored fee ${lc.feeLamports} lamports is not the exact tier for ratio ${lc.ratioWholeTokens}. Refusing to build.`);
  }
}

function legacy(payer: PublicKey, ixs: TransactionInstruction[]): Transaction {
  const tx = new Transaction().add(...ixs);
  tx.feePayer = payer;
  return tx;
}

/** Simulate a v0 message exactly like the pipeline does (no signatures, fresh blockhash). */
async function simV0(conn: Connection, tx: VersionedTransaction) {
  const r = await conn.simulateTransaction(tx, { sigVerify: false, replaceRecentBlockhash: true, commitment: "confirmed" });
  return r.value;
}

function v0(payer: PublicKey, recentBlockhash: string, ixs: TransactionInstruction[], alts: AddressLookupTableAccount[] = []): VersionedTransaction {
  return new VersionedTransaction(new TransactionMessage({ payerKey: payer, recentBlockhash, instructions: ixs }).compileToV0Message(alts));
}

export interface VrfRequestBuild {
  readonly tx: VersionedTransaction;
  /** Fresh randomness keypair when init_randomness is bundled (co-signs after Confirm). */
  readonly signers: Keypair[];
  readonly oracle: PublicKey;
  readonly staleProofs: readonly PublicKey[];
  readonly initRandomness: boolean;
  /** Simulations run to settle on the oracle (the pipeline simulates once more for the preview). */
  readonly attempts: number;
  readonly simulatedOk: boolean;
  readonly simulationNote: string | null;
  readonly unitsConsumed: number | null;
}

/**
 * request_capture / request_reroll with simulate-first oracle selection (src/lib/armory/oracle.ts).
 * If the vault has no idle randomness account, init_randomness (fresh keypair) is bundled in front,
 * as in scripts/devnet-e2e. Returns a v0 transaction (no lookup table needed).
 */
export async function buildVrfRequestTx(
  conn: Connection,
  user: PublicKey,
  vault: PublicKey,
  kind: { type: "capture" } | { type: "reroll"; index: number },
  opts: { recentBlockhash?: string; /** tests only: start from a different candidate */ reorderCandidates?: (c: PublicKey[]) => PublicKey[] } = {},
): Promise<VrfRequestBuild> {
  const v = await loadVault(conn, vault);
  await assertTierFee(conn, v);
  const [idle, qi] = await Promise.all([findIdleRandomness(conn, vault), conn.getAccountInfo(v.sbQueue)]);
  if (!qi) throw new Error("Switchboard queue not found.");
  const ordered = await oracleCandidates(decodeQueue(new Uint8Array(qi.data)), vault, v.nextSeq);
  const candidates = opts.reorderCandidates ? opts.reorderCandidates(ordered) : ordered;
  const signers: Keypair[] = [];
  const pre: TransactionInstruction[] = [];
  let randomness = idle;
  if (!randomness) {
    const r = Keypair.generate();
    randomness = r.publicKey;
    signers.push(r);
    const recentSlot = BigInt(await conn.getSlot("finalized")) - 1n;
    const { lutSigner, lut } = sbLutAccounts(r.publicKey, recentSlot, SWITCHBOARD_PROGRAM_ID, ADDRESS_LOOKUP_TABLE_PROGRAM_ID);
    pre.push(initRandomnessIx({ payer: user, vault, randomness: r.publicKey, sbQueue: v.sbQueue, recentSlot, sbProgramState: sbProgramStatePda(), sbLutSigner: lutSigner, sbLut: lut }));
  }
  const blockhash = opts.recentBlockhash ?? (await conn.getLatestBlockhash("confirmed")).blockhash;
  const rnd = randomness;
  const make = (s: OracleState) => {
    const a = { user, vault, vaultData: v, randomness: rnd, sbOracle: s.oracle, staleProofs: s.stale };
    const ix = kind.type === "capture" ? requestCaptureIx(a) : requestRerollIx({ ...a, index: kind.index });
    return v0(user, blockhash, [pre.length ? CU_LIMIT_INIT : CU_LIMIT, ...pre, ix]);
  };
  const res = await resolveOracle(candidates, make, (tx) => simV0(conn, tx));
  return {
    tx: res.built, signers, oracle: res.state.oracle, staleProofs: res.state.stale, initRandomness: pre.length > 0,
    attempts: res.attempts, simulatedOk: res.ok, simulationNote: res.reason, unitsConsumed: res.unitsConsumed,
  };
}

export async function buildCaptureTx(conn: Connection, user: PublicKey, vault: PublicKey, recentBlockhash?: string): Promise<BuildResult> {
  const b = await buildVrfRequestTx(conn, user, vault, { type: "capture" }, { recentBlockhash });
  return { tx: b.tx, signers: b.signers };
}

export async function buildRerollTx(conn: Connection, user: PublicKey, vault: PublicKey, index: number, recentBlockhash?: string): Promise<BuildResult> {
  const b = await buildVrfRequestTx(conn, user, vault, { type: "reroll", index }, { recentBlockhash });
  return { tx: b.tx, signers: b.signers };
}

/** Permissionless reveal of a pending request's randomness (payer = connected wallet). */
export async function buildRevealTx(conn: Connection, payer: PublicKey, request: PublicKey, fetcher?: RevealFetcher): Promise<VersionedTransaction> {
  const ri = await conn.getAccountInfo(request);
  if (!ri) throw new Error("Request not found (already settled or expired?).");
  const req = decodeRequest(new Uint8Array(ri.data));
  if (req.revealed) throw new Error("Already revealed; settle next.");
  const vi = await conn.getAccountInfo(req.vault);
  if (!vi) throw new Error("Vault not found.");
  const v = decodeVault(new Uint8Array(vi.data));
  const r = await fetchReveal(conn, req.randomness, fetcher);
  const ix = revealRandomnessIx({
    payer, vault: req.vault, request, randomness: req.randomness, sbOracle: r.randomness.oracle, sbQueue: v.sbQueue,
    sbStats: oracleStatsPda(r.randomness.oracle, SWITCHBOARD_PROGRAM_ID), sbProgramState: sbProgramStatePda(), args: r.args,
  });
  return v0(payer, (await conn.getLatestBlockhash("confirmed")).blockhash, [CU_LIMIT, ix]);
}

export const MAX_TX_BYTES = 1232;
export function txSize(tx: VersionedTransaction): number {
  return tx.serialize().length;
}

export interface SettleBuild {
  readonly tx: VersionedTransaction;
  readonly assetIndex: number;
  readonly mints: boolean;
  readonly bytes: number;
  readonly lookupTables: readonly string[];
}

/** Lookup tables usable for settle: pinned for the cluster, frozen, active (validated, not trusted). */
async function pinnedTables(conn: Connection): Promise<AddressLookupTableAccount[]> {
  const out: AddressLookupTableAccount[] = [];
  for (const k of PINNED_LOOKUP_TABLES[CLUSTER.name]) {
    const t = (await conn.getAddressLookupTable(k)).value;
    if (t && t.state.authority === undefined && t.isActive()) out.push(t);
  }
  return out;
}

/**
 * Permissionless settle (capture or re-roll). The picked index is learned by simulation: a first
 * pass with a placeholder asset fails WrongAsset and Anchor logs the expected asset after "Right:".
 * If that asset isn't minted yet, the leaf preimage + proof come from the launch's leaf source.
 * Settle-with-mint can exceed the 1,232-byte limit for big collections: then the pinned lookup
 * tables are used (v0), or the build fails with a clear error.
 */
export async function buildSettleTx(conn: Connection, settler: PublicKey, request: PublicKey, opts: { lookupTables?: AddressLookupTableAccount[]; leaves?: LeafSource | null } = {}): Promise<SettleBuild> {
  const ri = await conn.getAccountInfo(request);
  if (!ri) throw new Error("Request not found (already settled or expired?).");
  const req = decodeRequest(new Uint8Array(ri.data));
  if (!req.revealed) throw new Error("Randomness isn't revealed yet; reveal first.");
  const vi = await conn.getAccountInfo(req.vault);
  if (!vi) throw new Error("Vault not found.");
  const v = decodeVault(new Uint8Array(vi.data));
  const kind = req.kind === REQUEST_KIND_REROLL ? "reroll" : "capture";
  const blockhash = (await conn.getLatestBlockhash("confirmed")).blockhash;
  const make = (assetIndex: number, mint: MintArgs | null, alts: AddressLookupTableAccount[] = []) =>
    v0(settler, blockhash, [CU_LIMIT_SETTLE, settleIx({ kind, settler, vault: req.vault, vaultData: v, request, seq: req.seq, randomness: req.randomness, user: req.user, assetIndex, mint })], alts);
  // Pass 1: learn the picked asset.
  const probe = await simV0(conn, make(0, null));
  let assetIndex = 0;
  if (probe.err) {
    const right = rightKeyFromLogs(probe.logs ?? []);
    if (!right) throw new Error(`Settle simulation failed: ${JSON.stringify(probe.err)}`);
    const size = (await conn.getAccountInfo(v.pool))?.data.length;
    const cap = size ? Math.min(10_000, Math.floor(((size - 64) * 8) / 129) + 1) : 10_000;
    assetIndex = Array.from({ length: cap }, (_, i) => i).find((i) => assetPda(req.vault, i).equals(right)) ?? -1;
    if (assetIndex < 0) throw new Error("Couldn't map the program's picked asset to an index.");
  }
  const minted = (await conn.getAccountInfo(assetPda(req.vault, assetIndex))) !== null;
  let mint: MintArgs | null = null;
  if (!minted) {
    const src = opts.leaves === undefined ? leafSourceFor(v.launchConfig) : opts.leaves;
    if (!src) throw new Error(`NFT #${assetIndex} isn't minted yet and this launch has no published traits manifest, so it can't be minted from the app yet.`);
    mint = await src.mintArgs(v.launchConfig, assetIndex);
  }
  let tx = make(assetIndex, mint);
  let used: AddressLookupTableAccount[] = [];
  if (txSize(tx) > MAX_TX_BYTES) {
    used = opts.lookupTables ?? (await pinnedTables(conn));
    if (used.length === 0) throw new Error(`Settle-with-mint is ${txSize(tx)} bytes (> ${MAX_TX_BYTES}) and no lookup table is pinned for ${CLUSTER.label}.`);
    tx = make(assetIndex, mint, used);
    if (txSize(tx) > MAX_TX_BYTES) throw new Error(`Settle-with-mint is still ${txSize(tx)} bytes with the pinned lookup tables.`);
  }
  return { tx, assetIndex, mints: !minted, bytes: txSize(tx), lookupTables: used.map((t) => t.key.toBase58()) };
}

/**
 * Meteora DBC launch (simulate-only path for now): DBC creates the mint + curve on the cluster's
 * pinned platform config, then register_dbc_launch records the immutable LaunchConfig, in one
 * transaction. The graduation threshold is the config's migration_quote_threshold (read from chain).
 */
export async function buildDbcLaunchTx(
  conn: Connection,
  creator: PublicKey,
  p: { name: string; symbol: string; uri: string; ratioWholeTokens: bigint; collectionSize: bigint },
): Promise<{ tx: VersionedTransaction; signers: Keypair[]; mint: PublicKey; dbcConfig: PublicKey; graduationLamports: bigint; pool: PublicKey }> {
  const dbcConfig = DBC_PLATFORM_CONFIG[CLUSTER.name];
  if (!dbcConfig) throw new Error(`No Meteora DBC platform config is pinned for ${CLUSTER.label}.`);
  const ci = await conn.getAccountInfo(dbcConfig);
  if (!ci || !ci.owner.equals(DBC_PROGRAM_ID)) throw new Error("DBC platform config not found (or not owned by Meteora DBC).");
  const cfg = decodeDbcConfig(new Uint8Array(ci.data));
  const mint = Keypair.generate();
  const pool = dbcPoolPda(dbcConfig, mint.publicKey, cfg.quoteMint);
  const ixs = [
    initializeVirtualPoolWithSplTokenIx({ config: dbcConfig, quoteMint: cfg.quoteMint, creator, payer: creator, baseMint: mint.publicKey, name: p.name, symbol: p.symbol, uri: p.uri }),
    registerDbcLaunchIx({ creator, mint: mint.publicKey, dbcConfig, dbcPool: pool, ratioWholeTokens: p.ratioWholeTokens, collectionSize: p.collectionSize }),
  ];
  const tx = v0(creator, (await conn.getLatestBlockhash("confirmed")).blockhash, [CU_LIMIT, ...ixs]);
  return { tx, signers: [mint], mint: mint.publicKey, dbcConfig, graduationLamports: cfg.migrationQuoteThreshold, pool };
}

/** Graduation threshold of the cluster's DBC platform config, or null when there's no DBC path. */
export async function fetchDbcGraduationLamports(conn: Connection): Promise<bigint | null> {
  const k = DBC_PLATFORM_CONFIG[CLUSTER.name];
  if (!k) return null;
  const ci = await conn.getAccountInfo(k);
  return ci ? decodeDbcConfig(new Uint8Array(ci.data)).migrationQuoteThreshold : null;
}

export async function buildUnwrapTx(conn: Connection, user: PublicKey, vault: PublicKey, index: number): Promise<Transaction> {
  const v = await loadVault(conn, vault);
  return legacy(user, [createAtaIdempotentIx(user, user, v.mint), unwrapIx({ user, vault, vaultData: v, index })]);
}

export async function buildExpireTx(conn: Connection, caller: PublicKey, vault: PublicKey, request: PublicKey): Promise<Transaction> {
  const [vi, ri] = await conn.getMultipleAccountsInfo([vault, request]);
  if (!vi || !ri) throw new Error("Vault or request not found.");
  return legacy(caller, [
    expireRequestIx({ caller, vault, vaultData: decodeVault(new Uint8Array(vi.data)), request, requestData: decodeRequest(new Uint8Array(ri.data)) }),
  ]);
}

/** Native hybrid launch (deployed). A throwaway mint keypair is generated locally and co-signs. */
export function buildNativeLaunchTx(creator: PublicKey, params: LaunchParams): { tx: Transaction; signers: Keypair[]; mint: PublicKey } {
  const mint = Keypair.generate();
  return { tx: legacy(creator, [launchIx(creator, mint.publicKey, params)]), signers: [mint], mint: mint.publicKey };
}
