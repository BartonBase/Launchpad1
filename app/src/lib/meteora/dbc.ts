/**
 * Meteora Dynamic Bonding Curve (DBC) integration, built on the official
 * @meteora-ag/dynamic-bonding-curve-sdk. Used by:
 *  - Plain launches: `buildPlainLaunchTx` creates a 1B SPL token + curve on Armory's platform DBC
 *    config (DBC_PLATFORM_CONFIG). No Armory program is involved.
 *  - Curve trading: `quoteCurveSwap` / `buildCurveSwapTx` (buy and sell on the curve, exact-in,
 *    with slippage protection via minimumAmountOut).
 *  - Curve progress: `fetchCurveState` (SOL raised vs the config's migration threshold, migrated
 *    flag, spot price).
 * Every transaction built here goes through useSafeSend (allowlist -> simulate -> preview ->
 * explicit confirm), like the rest of the app. The config is pinned per cluster (devnet; mainnet from
 * NEXT_PUBLIC_DBC_CONFIG_MAINNET).
 */
import { Keypair, PublicKey, type Connection, type Transaction } from "@solana/web3.js";
import BN from "bn.js";
import {
  DynamicBondingCurveClient,
  SwapMode,
  deriveDbcPoolAddress,
  getCurrentPoint,
  getPriceFromSqrtPrice,
  type PoolConfig,
  type VirtualPool,
} from "@meteora-ag/dynamic-bonding-curve-sdk";
import { CLUSTER } from "@/config/cluster";
import { DBC_PLATFORM_CONFIG } from "@/config/integrations";
import { WRAPPED_SOL_MINT } from "@/config/programs";
import type { LaunchTermsView } from "@/config/launchTerms";
import { PLANNED_MAINNET_TERMS, configProblems, steadyFeeBps, termsFromConfig } from "./terms";

const clients = new WeakMap<Connection, DynamicBondingCurveClient>();
export function dbcClient(conn: Connection): DynamicBondingCurveClient {
  let c = clients.get(conn);
  if (!c) {
    c = DynamicBondingCurveClient.create(conn, "confirmed");
    clients.set(conn, c);
  }
  return c;
}

export function platformDbcConfig(): PublicKey | null {
  return DBC_PLATFORM_CONFIG[CLUSTER.name];
}

/** The DBC pool a mint would have on the platform config (SOL quote). */
export function platformPoolForMint(mint: PublicKey): PublicKey | null {
  const cfg = platformDbcConfig();
  return cfg ? deriveDbcPoolAddress(WRAPPED_SOL_MINT, mint, cfg) : null;
}

/** Serializable curve snapshot (strings for bigints so it can cross the server/client boundary). */
export interface CurveStateDTO {
  readonly pool: string;
  readonly config: string;
  readonly baseMint: string;
  readonly creator: string;
  readonly quoteReserveLamports: string;
  readonly thresholdLamports: string;
  /** 0..100, one decimal. */
  readonly progressPct: number;
  readonly migrated: boolean;
  /** Curve finished (threshold reached) but migration not done yet. */
  readonly curveComplete: boolean;
  /** Spot price, SOL per whole token. */
  readonly priceSol: number;
  readonly baseDecimals: number;
  /** Steady trading fee in basis points (after any anti-snipe schedule), from the config. */
  readonly feeBps: number;
  /** 1 = DAMM v2 (MigrationOption). */
  readonly migrationOption: number;
}

export function curveStateFrom(pool: PublicKey, vp: VirtualPool, cfg: PoolConfig): CurveStateDTO {
  const s = vp.poolState;
  const reserve = BigInt(s.quoteReserve.toString());
  const threshold = BigInt(cfg.migrationQuoteThreshold.toString());
  const pct = threshold > 0n ? Math.min(100, Number((reserve * 1000n) / threshold) / 10) : 0;
  const price = Number(getPriceFromSqrtPrice(s.sqrtPrice, cfg.tokenDecimal, 9).toString());
  // Steady fee after any anti-snipe schedule (the cliff fee is the schedule's START, e.g. 50%).
  const feeBps = steadyFeeBps(cfg.poolFees.baseFee);
  return {
    pool: pool.toBase58(),
    config: s.config.toBase58(),
    baseMint: s.baseMint.toBase58(),
    creator: s.creator.toBase58(),
    quoteReserveLamports: reserve.toString(),
    thresholdLamports: threshold.toString(),
    progressPct: s.isMigrated ? 100 : pct,
    migrated: s.isMigrated === 1,
    curveComplete: reserve >= threshold,
    priceSol: Number.isFinite(price) ? price : 0,
    baseDecimals: cfg.tokenDecimal,
    feeBps,
    migrationOption: cfg.migrationOption,
  };
}

/** Curve state for a DBC pool, or null if the account doesn't exist. */
export async function fetchCurveState(conn: Connection, pool: PublicKey): Promise<CurveStateDTO | null> {
  const c = dbcClient(conn);
  const vp = await c.state.getPool(pool);
  if (!vp) return null;
  const cfg = await c.state.getPoolConfig(vp.poolState.config);
  if (!cfg) return null;
  return curveStateFrom(pool, vp, cfg);
}

export type Side = "buy" | "sell";

export interface SwapQuoteView {
  readonly amountIn: bigint;
  readonly amountOut: bigint;
  readonly minimumAmountOut: bigint;
  /** Fees in the INPUT token's base units for buys (SOL), output side for sells (SOL). */
  readonly tradingFee: bigint;
  readonly protocolFee: bigint;
  /** Percent, when the SDK reports it. */
  readonly priceImpactPct: number | null;
  /** True when the curve can't absorb the whole input (it would complete the curve). */
  readonly partial: boolean;
}

const big = (b: BN | undefined | null) => (b ? BigInt(b.toString()) : 0n);

/** Exact-in quote on the curve. amountIn: lamports for buys, token base units for sells. */
export async function quoteCurveSwap(conn: Connection, pool: PublicKey, side: Side, amountIn: bigint, slippageBps: number): Promise<SwapQuoteView> {
  const c = dbcClient(conn);
  const vp = await c.state.getPool(pool);
  if (!vp) throw new Error("Bonding-curve pool not found.");
  if (vp.poolState.isMigrated) throw new Error("This curve has graduated; trading moved to the DAMM v2 pool.");
  const cfg = await c.state.getPoolConfig(vp.poolState.config);
  if (!cfg) throw new Error("Curve config not found.");
  const currentPoint = await getCurrentPoint(conn, cfg.activationType);
  const q = c.pool.swapQuote2({
    virtualPool: vp, config: cfg, swapBaseForQuote: side === "sell", hasReferral: false, eligibleForFirstSwapWithMinFee: false,
    currentPoint, slippageBps, swapMode: SwapMode.ExactIn, amountIn: new BN(amountIn.toString()),
  });
  return {
    amountIn,
    amountOut: big(q.outputAmount),
    minimumAmountOut: big(q.minimumAmountOut),
    tradingFee: big(q.tradingFee),
    protocolFee: big(q.protocolFee),
    priceImpactPct: null,
    partial: big(q.amountLeft) > 0n,
  };
}

/** Exact-in swap transaction (wraps/unwraps SOL, creates the token account if needed). */
export async function buildCurveSwapTx(conn: Connection, owner: PublicKey, pool: PublicKey, side: Side, amountIn: bigint, minimumAmountOut: bigint): Promise<Transaction> {
  return dbcClient(conn).pool.swap2({
    owner, pool, swapBaseForQuote: side === "sell", referralTokenAccount: null, swapMode: SwapMode.ExactIn,
    amountIn: new BN(amountIn.toString()), minimumAmountOut: new BN(minimumAmountOut.toString()),
  });
}

/**
 * Plain launch: DBC `initialize_virtual_pool_with_spl_token` on the platform config. DBC creates the
 * mint (fresh local keypair, co-signs), mints the fixed 1B supply into the curve, and the config
 * revokes mint authority. Graduation threshold, fees and the DAMM v2 migration are fixed by the config.
 */
export async function buildPlainLaunchTx(
  conn: Connection,
  creator: PublicKey,
  p: { name: string; symbol: string; uri: string },
): Promise<{ tx: Transaction; signers: Keypair[]; mint: PublicKey; pool: PublicKey }> {
  const config = platformDbcConfig();
  if (!config) throw new Error(`No Meteora DBC platform config is pinned for ${CLUSTER.label}.`);
  const mint = Keypair.generate();
  const tx = await dbcClient(conn).creator.createPool({ name: p.name, symbol: p.symbol, uri: p.uri, payer: creator, poolCreator: creator, config, baseMint: mint.publicKey });
  return { tx, signers: [mint], mint: mint.publicKey, pool: deriveDbcPoolAddress(WRAPPED_SOL_MINT, mint.publicKey, config) };
}

export interface PlatformTermsDTO {
  readonly config: string;
  readonly feeClaimer: string;
  readonly terms: LaunchTermsView;
  /** Non-empty = the config is unsafe to launch on (see configProblems). */
  readonly problems: string[];
}

/** Live launch terms of the platform config (fees, anti-snipe, creator share, graduation) + safety problems. */
export async function fetchPlatformTerms(conn: Connection): Promise<PlatformTermsDTO | null> {
  const key = platformDbcConfig();
  if (!key) return null;
  const cfg = await dbcClient(conn).state.getPoolConfig(key);
  if (!cfg) return { config: key.toBase58(), feeClaimer: "", terms: PLANNED_MAINNET_TERMS, problems: ["The platform config account was not found on chain."] };
  return { config: key.toBase58(), feeClaimer: cfg.feeClaimer.toBase58(), terms: termsFromConfig(cfg), problems: configProblems(cfg, CLUSTER.name) };
}
