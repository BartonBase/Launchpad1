/**
 * Step 3 of the pipeline: simulate against the configured cluster and diff
 * balances of every writable (static) account.
 *
 * Caveats (surfaced as preview warnings, never hidden):
 *  - Pre-state is fetched with getMultipleAccountsInfo right before the
 *    simulation; the two reads are not atomic, so a concurrent change to the
 *    same accounts could skew the diff.
 *  - Accounts loaded through address lookup tables are diffed only when the tables were
 *    resolved (src/lib/tx/lookup.ts); otherwise the preview blocks the transaction.
 */
import {
  PublicKey,
  VersionedTransaction,
  type AccountInfo,
  type Connection,
  type SimulatedTransactionAccountInfo,
  type TransactionError,
} from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@/config/programs";
import { toVersionedMessage, viewMessage, type AnyTransaction } from "./message";
import { NO_LOOKUPS, type ResolvedLookups } from "./lookup";

export interface SolChange {
  readonly address: string;
  readonly isSigner: boolean;
  readonly isFeePayer: boolean;
  readonly preLamports: bigint;
  readonly postLamports: bigint;
  readonly deltaLamports: bigint;
}

export interface TokenChange {
  /** Token account address. */
  readonly account: string;
  readonly mint: string;
  /** Token account owner (wallet). */
  readonly owner: string;
  readonly tokenProgram: string;
  readonly ownedBySigner: boolean;
  /** null when the mint could not be fetched. */
  readonly decimals: number | null;
  readonly preAmount: bigint;
  readonly postAmount: bigint;
  readonly deltaAmount: bigint;
}

export interface InvokedProgram {
  readonly programId: string;
  /** 1 = top level, >1 = CPI. */
  readonly depth: number;
}

export interface SimulationResult {
  readonly ok: boolean;
  /** Human readable error, null on success. */
  readonly error: string | null;
  readonly rawError: TransactionError | string | null;
  readonly logs: readonly string[];
  readonly unitsConsumed: number | null;
  readonly solChanges: readonly SolChange[];
  readonly tokenChanges: readonly TokenChange[];
  readonly invokedPrograms: readonly InvokedProgram[];
  readonly diffedAccounts: number;
  readonly usesLookupTables: boolean;
  readonly slot: number | null;
}

export interface SimulateOptions {
  /** The wallet that will sign; balances it owns are flagged in the preview. */
  readonly signer: PublicKey;
  readonly commitment?: "processed" | "confirmed" | "finalized";
  /** Resolved lookup tables, so ALT-loaded writable accounts are diffed too. */
  readonly lookups?: ResolvedLookups;
}

// ---- tiny binary helpers (no Buffer dependency in the browser) -------------

export function base64ToBytes(b64: string): Uint8Array {
  const bin = globalThis.atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

function readU64LE(data: Uint8Array, offset: number): bigint {
  let v = 0n;
  for (let i = 7; i >= 0; i--) v = (v << 8n) | BigInt(data[offset + i] ?? 0);
  return v;
}

const TOKEN_ACCOUNT_LEN = 165;
const ACCOUNT_TYPE_ACCOUNT = 2; // Token-2022 AccountType::Account at byte 165
const MINT_DECIMALS_OFFSET = 44;

export interface ParsedTokenAccount {
  readonly mint: PublicKey;
  readonly owner: PublicKey;
  readonly amount: bigint;
  readonly tokenProgram: PublicKey;
}

/** Parse an SPL Token / Token-2022 token account; null if not one. */
export function parseTokenAccount(owner: PublicKey | string, data: Uint8Array): ParsedTokenAccount | null {
  const ownerKey = typeof owner === "string" ? new PublicKey(owner) : owner;
  const isToken = ownerKey.equals(TOKEN_PROGRAM_ID);
  const isToken2022 = ownerKey.equals(TOKEN_2022_PROGRAM_ID);
  if (!isToken && !isToken2022) return null;
  if (isToken && data.length !== TOKEN_ACCOUNT_LEN) return null;
  if (isToken2022) {
    const plain = data.length === TOKEN_ACCOUNT_LEN;
    const extended = data.length > TOKEN_ACCOUNT_LEN && data[TOKEN_ACCOUNT_LEN] === ACCOUNT_TYPE_ACCOUNT;
    if (!plain && !extended) return null;
  }
  return {
    mint: new PublicKey(data.subarray(0, 32)),
    owner: new PublicKey(data.subarray(32, 64)),
    amount: readU64LE(data, 64),
    tokenProgram: ownerKey,
  };
}

const INVOKE_RE = /^Program ([1-9A-HJ-NP-Za-km-z]{32,44}) invoke \[(\d+)\]$/;

/** Extract every program invocation (including CPIs) from simulation logs. */
export function parseInvokedPrograms(logs: readonly string[]): InvokedProgram[] {
  const seen = new Map<string, number>();
  for (const line of logs) {
    const m = INVOKE_RE.exec(line);
    if (!m || !m[1] || !m[2]) continue;
    const depth = Number(m[2]);
    const prev = seen.get(m[1]);
    if (prev === undefined || depth < prev) seen.set(m[1], depth);
  }
  return [...seen].map(([programId, depth]) => ({ programId, depth }));
}

function describeError(e: TransactionError | string | null): string | null {
  if (e === null) return null;
  if (typeof e === "string") return e;
  try {
    return JSON.stringify(e, (_k, v: unknown) => (typeof v === "bigint" ? v.toString() : v));
  } catch {
    return String(e);
  }
}

interface AccountSnapshot {
  readonly lamports: bigint;
  readonly owner: string | null;
  readonly data: Uint8Array | null;
}

function snapFromInfo(info: AccountInfo<Buffer> | null): AccountSnapshot {
  if (!info) return { lamports: 0n, owner: null, data: null };
  return { lamports: BigInt(info.lamports), owner: info.owner.toBase58(), data: new Uint8Array(info.data) };
}

function snapFromSim(info: SimulatedTransactionAccountInfo | null): AccountSnapshot {
  if (!info) return { lamports: 0n, owner: null, data: null };
  const [payload, encoding] = info.data;
  return {
    lamports: BigInt(info.lamports),
    owner: info.owner,
    data: encoding === "base64" && payload !== undefined ? base64ToBytes(payload) : null,
  };
}

function tokenOf(s: AccountSnapshot): ParsedTokenAccount | null {
  if (!s.owner || !s.data) return null;
  try {
    return parseTokenAccount(s.owner, s.data);
  } catch {
    return null;
  }
}

export async function simulate(
  connection: Connection,
  tx: AnyTransaction,
  opts: SimulateOptions,
): Promise<SimulationResult> {
  const commitment = opts.commitment ?? "confirmed";
  const view = viewMessage(tx, opts.lookups ?? NO_LOOKUPS);
  const vtx = new VersionedTransaction(toVersionedMessage(tx));
  const addresses = view.writable.map((k) => k.toBase58());
  const signerSet = new Set(view.signers.map((k) => k.toBase58()));
  const feePayer = view.feePayer.toBase58();
  const signerStr = opts.signer.toBase58();

  const base = {
    diffedAccounts: addresses.length,
    usesLookupTables: view.usesLookupTables,
  };

  let preInfos: (AccountInfo<Buffer> | null)[];
  let sim;
  try {
    preInfos = await connection.getMultipleAccountsInfo(view.writable as PublicKey[], { commitment });
    sim = await connection.simulateTransaction(vtx, {
      sigVerify: false,
      replaceRecentBlockhash: true,
      commitment,
      accounts: { encoding: "base64", addresses },
    });
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return {
      ...base,
      ok: false,
      error: `Simulation RPC failed: ${msg}`,
      rawError: msg,
      logs: [],
      unitsConsumed: null,
      solChanges: [],
      tokenChanges: [],
      invokedPrograms: [],
      slot: null,
    };
  }

  const v = sim.value;
  const logs = v.logs ?? [];
  const postInfos = v.accounts ?? [];

  const solChanges: SolChange[] = [];
  const tokenPairs: { account: string; pre: ParsedTokenAccount | null; post: ParsedTokenAccount | null }[] = [];
  addresses.forEach((address, i) => {
    const pre = snapFromInfo(preInfos[i] ?? null);
    const post = snapFromSim(postInfos[i] ?? null);
    const delta = post.lamports - pre.lamports;
    if (delta !== 0n) {
      solChanges.push({
        address,
        isSigner: signerSet.has(address),
        isFeePayer: address === feePayer,
        preLamports: pre.lamports,
        postLamports: post.lamports,
        deltaLamports: delta,
      });
    }
    const preTok = tokenOf(pre);
    const postTok = tokenOf(post);
    if (preTok || postTok) tokenPairs.push({ account: address, pre: preTok, post: postTok });
  });

  // Fetch decimals for every mint involved (one RPC call).
  const mintKeys = [
    ...new Set(tokenPairs.map((p) => (p.post ?? p.pre)!.mint.toBase58())),
  ].map((s) => new PublicKey(s));
  const decimals = new Map<string, number>();
  if (mintKeys.length > 0) {
    try {
      const mints = await connection.getMultipleAccountsInfo(mintKeys, { commitment });
      mints.forEach((m, i) => {
        const d = m?.data[MINT_DECIMALS_OFFSET];
        const key = mintKeys[i];
        if (key && typeof d === "number") decimals.set(key.toBase58(), d);
      });
    } catch {
      // decimals stay unknown; preview shows raw base units
    }
  }

  const tokenChanges: TokenChange[] = [];
  for (const { account, pre, post } of tokenPairs) {
    const ref = (post ?? pre)!;
    const preAmount = pre?.amount ?? 0n;
    const postAmount = post?.amount ?? 0n;
    if (preAmount === postAmount) continue;
    const mint = ref.mint.toBase58();
    tokenChanges.push({
      account,
      mint,
      owner: ref.owner.toBase58(),
      tokenProgram: ref.tokenProgram.toBase58(),
      ownedBySigner: ref.owner.toBase58() === signerStr,
      decimals: decimals.get(mint) ?? null,
      preAmount,
      postAmount,
      deltaAmount: postAmount - preAmount,
    });
  }

  const rawError = v.err ?? null;
  return {
    ...base,
    ok: rawError === null,
    error: describeError(rawError),
    rawError,
    logs,
    unitsConsumed: v.unitsConsumed ?? null,
    solChanges,
    tokenChanges,
    invokedPrograms: parseInvokedPrograms(logs),
    slot: sim.context.slot,
  };
}

/** Network fee for the message (lamports), or null if the RPC can't tell. */
export async function estimateFee(connection: Connection, tx: AnyTransaction): Promise<bigint | null> {
  try {
    const { value } = await connection.getFeeForMessage(toVersionedMessage(tx), "confirmed");
    return value === null ? null : BigInt(value);
  } catch {
    return null;
  }
}
