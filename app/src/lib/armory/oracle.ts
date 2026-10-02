/**
 * Simulate-first oracle selection for request_capture / request_reroll (* 2026-10-01). The PROGRAM picks the oracle (M-04) and the pick shifts with oracle heartbeats, so
 * the app simulates and follows the program's answer:
 *  - 6050 WrongOracle: Anchor's require_keys_eq! logs "Left:" (what we passed) and "Right:" (what the
 *    program wants), each followed by the key on the next log line. Rebuild with the Right key.
 *  - 6053 OracleStale: the oracle we passed is stale; pass it as a remaining account (stale proof)
 *    and try the next candidate.
 * Any other error (or success) ends the loop; the safe-send pipeline then simulates the final
 * transaction again for the preview, so nothing here is trusted for display.
 */
import type { PublicKey, SimulatedTransactionResponse, TransactionError } from "@solana/web3.js";
import { PublicKey as PK } from "@solana/web3.js";

export const ERR_WRONG_ORACLE = 6050;
export const ERR_ORACLE_STALE = 6053;
export const ERR_WRONG_ASSET_NAME = "WrongAsset";

/** Custom program error code of a simulation error, e.g. {InstructionError:[1,{Custom:6050}]}. */
export function customErrorCode(err: TransactionError | string | null | undefined): number | null {
  if (!err || typeof err !== "object") return null;
  const ie = (err as { InstructionError?: [number, unknown] }).InstructionError;
  const inner = ie?.[1];
  if (inner && typeof inner === "object" && "Custom" in inner) {
    const c = (inner as { Custom: unknown }).Custom;
    return typeof c === "number" ? c : null;
  }
  return null;
}

/** The key Anchor logged after "Program log: Right:" (require_keys_eq! failure), or null. */
export function rightKeyFromLogs(logs: readonly string[]): PublicKey | null {
  const i = logs.findIndex((l) => l.trim() === "Program log: Right:");
  const next = i >= 0 ? logs[i + 1] : undefined;
  if (!next) return null;
  try {
    return new PK(next.replace("Program log: ", "").trim());
  } catch {
    return null;
  }
}

export interface OracleState {
  readonly oracle: PublicKey;
  readonly stale: readonly PublicKey[];
}
export type OracleStep =
  | { readonly kind: "done" }
  | { readonly kind: "retry"; readonly next: OracleState; readonly reason: "wrong-oracle" | "stale" }
  | { readonly kind: "stop"; readonly reason: string };

/** Pure decision for one simulation result. `candidates` = queue oracles in the program's order. */
export function nextOracleStep(sim: Pick<SimulatedTransactionResponse, "err" | "logs">, cur: OracleState, candidates: readonly PublicKey[]): OracleStep {
  if (!sim.err) return { kind: "done" };
  const logs = sim.logs ?? [];
  const code = customErrorCode(sim.err) ?? (logs.some((l) => /Error Number: 6050/.test(l)) ? ERR_WRONG_ORACLE : logs.some((l) => /Error Number: 6053/.test(l)) ? ERR_ORACLE_STALE : null);
  if (code === ERR_WRONG_ORACLE) {
    const right = rightKeyFromLogs(logs);
    if (!right) return { kind: "stop", reason: "WrongOracle without a Right: key in the logs" };
    if (right.equals(cur.oracle)) return { kind: "stop", reason: "WrongOracle names the oracle we already passed" };
    return { kind: "retry", reason: "wrong-oracle", next: { oracle: right, stale: cur.stale } };
  }
  if (code === ERR_ORACLE_STALE) {
    const stale = [...cur.stale, cur.oracle];
    // Prefer the program's own answer ("Right:" key) when it names a non-stale oracle.
    const right = rightKeyFromLogs(logs);
    const nextOracle = right && !stale.some((s) => s.equals(right)) ? right : candidates.find((c) => !stale.some((s) => s.equals(c)));
    if (!nextOracle) return { kind: "stop", reason: "every queue oracle is stale" };
    return { kind: "retry", reason: "stale", next: { oracle: nextOracle, stale } };
  }
  return { kind: "stop", reason: `simulation failed: ${JSON.stringify(sim.err)}` };
}

export interface OracleResolution<T> {
  readonly built: T;
  readonly state: OracleState;
  readonly attempts: number;
  readonly ok: boolean;
  readonly reason: string | null;
  readonly unitsConsumed: number | null;
  readonly logs: readonly string[];
}

/**
 * Build → simulate → follow the program's answer until the simulation is clean. Each stale oracle
 * is added to the remaining accounts; the loop ends when every queue oracle was tried (default
 * cap: two simulations per candidate, enough for a WrongOracle hop after each stale one).
 */
export async function resolveOracle<T>(
  candidates: readonly PublicKey[],
  build: (s: OracleState) => Promise<T> | T,
  simulate: (built: T) => Promise<Pick<SimulatedTransactionResponse, "err" | "logs" | "unitsConsumed">>,
  max = Math.max(6, candidates.length * 2 + 1),
): Promise<OracleResolution<T>> {
  if (candidates.length === 0) throw new Error("Switchboard queue has no oracles.");
  let state: OracleState = { oracle: candidates[0]!, stale: [] };
  for (let attempt = 1; ; attempt++) {
    const built = await build(state);
    const sim = await simulate(built);
    const step = nextOracleStep(sim, state, candidates);
    const base = { built, state, attempts: attempt, unitsConsumed: sim.unitsConsumed ?? null, logs: sim.logs ?? [] };
    if (step.kind === "done") return { ...base, ok: true, reason: null };
    if (step.kind === "stop" || attempt >= max) return { ...base, ok: false, reason: step.kind === "stop" ? step.reason : `gave up after ${max} simulations` };
    state = step.next;
  }
}
