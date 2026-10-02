/**
 * Public stats for the home page, computed from chain data the app already reads:
 * - launches: listed Launch + Hybrid tokens (same list as Explore);
 * - curveTrades: successful transactions on each bonding-curve pool, minus the lifecycle steps that
 *   also touch the pool (launch, Hybrid register, migration, leftover, surplus / migration-fee
 *   withdrawals, opening the NFT vault), all known from the pool and vault state. The launch
 *   transaction is not counted, so a dev buy isn't either. One signature list per pool; no per-
 *   transaction reads (the public devnet RPC rate-limits those);
 * - captures: total captures recorded by each Hybrid vault (on-chain counter).
 */
import { PublicKey, type Connection } from "@solana/web3.js";
import { dbcClient, platformDbcConfig } from "@/lib/meteora/dbc";

export interface PoolFlags {
  readonly hybrid: boolean;
  readonly vaultOpen: boolean;
  readonly isMigrated: boolean;
  readonly isWithdrawLeftover: boolean;
  readonly surplusWithdrawals: number;
  readonly migrationFeeWithdrawStatus: number;
}

/** Non-trade transactions that touch a DBC pool, from its state. */
export function lifecycleTxCount(f: PoolFlags): number {
  let bits = 0;
  for (let s = f.migrationFeeWithdrawStatus; s; s >>= 1) bits += s & 1;
  return 1 + (f.hybrid ? 1 : 0) + (f.isMigrated ? 1 : 0) + (f.isWithdrawLeftover ? 1 : 0) + (f.vaultOpen ? 1 : 0) + f.surplusWithdrawals + bits;
}

async function withRetry<T>(fn: () => Promise<T>, tries = 3): Promise<T> {
  for (let n = 0; ; n++) {
    try {
      return await fn();
    } catch (e) {
      if (n + 1 >= tries) throw e;
      await new Promise((r) => setTimeout(r, 800 * 2 ** n));
    }
  }
}

export async function countCurveTrades(conn: Connection, pools: readonly { pool: string; hybrid: boolean; vaultOpen: boolean }[]): Promise<number> {
  const cfg = platformDbcConfig();
  if (!cfg || pools.length === 0) return 0;
  const states = new Map((await withRetry(() => dbcClient(conn).state.getPoolsByConfig(cfg))).map((p) => [p.publicKey.toBase58(), p.account.poolState] as const));
  let total = 0;
  for (const p of pools) {
    const st = states.get(p.pool);
    if (!st) continue;
    const sigs = (await withRetry(() => conn.getSignaturesForAddress(new PublicKey(p.pool), { limit: 1000 }))).filter((s) => !s.err).length;
    const n = (b: unknown) => (Number(b) ? 1 : 0);
    total += Math.max(
      0,
      sigs -
        lifecycleTxCount({
          hybrid: p.hybrid,
          vaultOpen: p.vaultOpen,
          isMigrated: !!n(st.isMigrated),
          isWithdrawLeftover: !!n(st.isWithdrawLeftover),
          surplusWithdrawals: n(st.isPartnerWithdrawSurplus) + n(st.isProtocolWithdrawSurplus) + n(st.isCreatorWithdrawSurplus),
          migrationFeeWithdrawStatus: Number(st.migrationFeeWithdrawStatus ?? 0),
        }),
    );
  }
  return total;
}
