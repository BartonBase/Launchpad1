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
 * explicit confirm), like the rest of the app. Devnet only (the config is pinned per cluster).
 */
import { Keypair, PublicKey, type Connection, type Transaction } from "@solana/web3.js";
import BN from "bn.js";
import {
  DynamicBondingCurveClient,
  SwapMode,
  deriveDbcPoolAddress,
  getBaseFeeHandler,
  getCurrentPoint,
  getPriceFromSqrtPrice,
  getTotalFeeNumeratorFromIncludedFeeAmount,
  type PoolConfig,
  type VirtualPool,
} from "@meteora-ag/dynamic-bonding-curve-sdk";
import { CLUSTER } from "@/config/cluster";
import { DBC_PLATFORM_CONFIG } from "@/config/integrations";
import { WRAPPED_SOL_MINT } from "@/config/programs";

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
  /** Total trading fee in basis points (base fee, from the config). */
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
  // cliffFeeNumerator is out of 1e9 (FEE_DENOMINATOR): 10_000_000 = 1%.
  const feeBps = Math.round(Number(cfg.poolFees.baseFee.cliffFeeNumerator.toString()) / 100_000);
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

/** Fee schedule of a DBC config, for display (anti-snipe = a fee that starts high and decays). */
export interface FeeSchedule {
  /** Fee on a trade at the very start (the launch slot), percent. */
  readonly startPct: number;
  /** Fee once the decay is over, percent. */
  readonly endPct: number;
  /** True when the start fee is above the end fee (an anti-snipe window exists). */
  readonly antiSnipe: boolean;
  /** Decay length, e.g. "60 s" or "150 slots"; null when there is no decay. */
  readonly window: string | null;
}
const pct = (num: BN | bigint) => Number(num.toString()) / 1e7; // numerator / 1e9 × 100

export function feeScheduleOf(cfg: PoolConfig): FeeSchedule {
  const b = cfg.poolFees.baseFee;
  const h = getBaseFeeHandler(b.cliffFeeNumerator, b.firstFactor, b.secondFactor, b.thirdFactor, b.baseFeeMode);
  const startPct = pct(b.cliffFeeNumerator);
  const endPct = pct(h.getMinBaseFeeNumerator());
  // Fee scheduler modes (0 linear, 1 exponential): firstFactor = periods, secondFactor = period length.
  const periods = b.baseFeeMode <= 1 ? Number(b.firstFactor) : 0;
  const len = b.baseFeeMode <= 1 ? Number(b.secondFactor.toString()) : 0;
  const unit = cfg.activationType === 1 ? "s" : "slots";
  return { startPct, endPct, antiSnipe: startPct > endPct, window: startPct > endPct && periods * len > 0 ? `${periods * len} ${unit}` : null };
}

export interface DevBuyQuote {
  readonly lamports: bigint;
  /** Estimated tokens received, base units. */
  readonly tokensOut: bigint;
  readonly minimumAmountOut: bigint;
  /** Curve fee paid on this buy (trading + protocol), lamports. */
  readonly feeLamports: bigint;
  /** Fee rate applied to this buy, percent (includes any anti-snipe surcharge). */
  readonly feePct: number;
  readonly schedule: FeeSchedule;
  readonly decimals: number;
}

/**
 * Quote for the creator's first buy, made in the launch transaction (elapsed time 0, so the config's
 * full anti-snipe fee applies unless the config grants the first swap the minimum fee). The pool
 * doesn't exist yet, so the quote runs on a fresh virtual pool built from the config.
 */
export async function quoteDevBuy(conn: Connection, lamports: bigint, slippageBps = 100): Promise<DevBuyQuote> {
  const config = platformDbcConfig();
  if (!config) throw new Error(`No Meteora DBC platform config is pinned for ${CLUSTER.label}.`);
  const cfg = await dbcClient(conn).state.getPoolConfig(config);
  if (!cfg) throw new Error("DBC platform config not found.");
  const currentPoint = await getCurrentPoint(conn, cfg.activationType);
  const zero = new BN(0);
  const fresh = {
    quoteReserve: zero, baseReserve: cfg.swapBaseAmount, sqrtPrice: cfg.sqrtStartPrice, activationPoint: currentPoint,
    volatilityTracker: { lastUpdateTimestamp: zero, padding: [], sqrtPriceReference: cfg.sqrtStartPrice, volatilityAccumulator: zero, volatilityReference: zero },
  };
  const firstMin = cfg.enableFirstSwapWithMinFee === 1;
  const q = dbcClient(conn).pool.swapQuote2({
    virtualPool: { poolState: fresh } as unknown as VirtualPool, config: cfg, swapBaseForQuote: false, hasReferral: false, eligibleForFirstSwapWithMinFee: firstMin,
    currentPoint, slippageBps, swapMode: SwapMode.ExactIn, amountIn: new BN(lamports.toString()),
  });
  const amount = new BN(lamports.toString());
  const feeNum = firstMin
    ? getBaseFeeHandler(cfg.poolFees.baseFee.cliffFeeNumerator, cfg.poolFees.baseFee.firstFactor, cfg.poolFees.baseFee.secondFactor, cfg.poolFees.baseFee.thirdFactor, cfg.poolFees.baseFee.baseFeeMode).getMinBaseFeeNumerator()
    : getTotalFeeNumeratorFromIncludedFeeAmount(cfg.poolFees, fresh.volatilityTracker as never, currentPoint, currentPoint, amount, 1);
  return {
    lamports,
    tokensOut: big(q.outputAmount),
    minimumAmountOut: big(q.minimumAmountOut),
    feeLamports: big(q.tradingFee) + big(q.protocolFee),
    feePct: pct(feeNum),
    schedule: feeScheduleOf(cfg),
    decimals: cfg.tokenDecimal,
  };
}

/**
 * DBC `initialize_virtual_pool_with_spl_token` on the platform config, plus the creator's optional
 * first buy (dev buy) in the SAME transaction (SDK createPoolWithFirstBuy). DBC creates the mint
 * (the caller's local keypair co-signs), mints the fixed 1B supply into the curve, and the config
 * revokes mint authority. Graduation threshold, fees and the DAMM v2 migration are fixed by the config.
 */
export async function buildDbcCreatePoolTx(
  conn: Connection,
  creator: PublicKey,
  mint: Keypair,
  p: { name: string; symbol: string; uri: string },
  devBuy: { lamports: bigint; minimumAmountOut: bigint } | null = null,
): Promise<{ tx: Transaction; pool: PublicKey }> {
  const config = platformDbcConfig();
  if (!config) throw new Error(`No Meteora DBC platform config is pinned for ${CLUSTER.label}.`);
  const createPoolParam = { name: p.name, symbol: p.symbol, uri: p.uri, payer: creator, poolCreator: creator, config, baseMint: mint.publicKey };
  const c = dbcClient(conn).creator;
  const tx =
    devBuy && devBuy.lamports > 0n
      ? await c.createPoolWithFirstBuy({
          createPoolParam,
          firstBuyParam: { buyer: creator, receiver: creator, buyAmount: new BN(devBuy.lamports.toString()), minimumAmountOut: new BN(devBuy.minimumAmountOut.toString()), referralTokenAccount: null },
        })
      : await c.createPool(createPoolParam);
  return { tx, pool: deriveDbcPoolAddress(WRAPPED_SOL_MINT, mint.publicKey, config) };
}

/**
 * Plain launch ("Launch" type): the DBC pool, optionally with the dev buy in the same transaction.
 * No Armory program is involved.
 */
export async function buildPlainLaunchTx(
  conn: Connection,
  creator: PublicKey,
  p: { name: string; symbol: string; uri: string },
  devBuy: { lamports: bigint; minimumAmountOut: bigint } | null = null,
  mint: Keypair = Keypair.generate(),
): Promise<{ tx: Transaction; signers: Keypair[]; mint: PublicKey; pool: PublicKey }> {
  const { tx, pool } = await buildDbcCreatePoolTx(conn, creator, mint, p, devBuy);
  return { tx, signers: [mint], mint: mint.publicKey, pool };
}
