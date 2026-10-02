/**
 * Public stats for the home page, computed from chain data the app already reads:
 * - launches: listed Launch + Hybrid tokens (same list as Explore);
 * - curveTrades: swap transactions on each launch's bonding-curve pool (logs contain "Instruction: Swap");
 * - captures: total captures recorded by each Hybrid vault (on-chain counter).
 * Each transaction is classified once and remembered, so later refreshes only read new signatures.
 */
import { PublicKey, type Connection } from "@solana/web3.js";

const seen = new Map<string, boolean>(); // signature -> is a swap

export const isSwapLog = (logs: readonly string[] | null | undefined): boolean =>
  !!logs?.some((l) => /^Program log: Instruction: Swap2?$/.test(l));

async function withRetry<T>(fn: () => Promise<T>, tries = 4): Promise<T> {
  for (let n = 0; ; n++) {
    try {
      return await fn();
    } catch (e) {
      if (n + 1 >= tries) throw e;
      await new Promise((r) => setTimeout(r, 700 * 2 ** n));
    }
  }
}

export async function countCurveTrades(conn: Connection, pools: readonly string[]): Promise<number> {
  let total = 0;
  for (const pool of pools) {
    const sigs = (await withRetry(() => conn.getSignaturesForAddress(new PublicKey(pool), { limit: 1000 }))).filter((s) => !s.err).map((s) => s.signature);
    const fresh = sigs.filter((s) => !seen.has(s));
    for (let i = 0; i < fresh.length; i += 10) {
      const chunk = fresh.slice(i, i + 10);
      const txs = await withRetry(() => conn.getTransactions(chunk, { maxSupportedTransactionVersion: 0, commitment: "confirmed" }));
      txs.forEach((t, k) => {
        if (t) seen.set(chunk[k]!, isSwapLog(t.meta?.logMessages));
      });
    }
    total += sigs.filter((s) => seen.get(s)).length;
  }
  return total;
}

export interface HomeStats {
  readonly launches: number;
  readonly curveTrades: number | null;
  readonly captures: number;
}
