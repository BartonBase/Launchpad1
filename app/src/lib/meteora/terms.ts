/**
 * Launch terms read from a DBC config on chain (fees, anti-snipe schedule, creator share, graduation target),
 * plus the mainnet safety check on the config itself. Pure functions over the decoded PoolConfig (unit-tested).
 */
import type { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { BaseFeeMode, getFeeSchedulerMinBaseFeeNumerator, type PoolConfig } from "@meteora-ag/dynamic-bonding-curve-sdk";
import type { ClusterName } from "@/config/cluster";
import { WRAPPED_SOL_MINT } from "@/config/programs";
import { MAINNET_FEE_CLAIMER, MAINNET_LAUNCH_TERMS, type LaunchTermsView } from "@/config/launchTerms";

/** DBC fee numerators are out of 1e9: 10_000_000 = 1% = 100 bps. */
const toBps = (numerator: BN | bigint | number) => Math.round(Number(numerator.toString()) / 100_000);

type BaseFee = PoolConfig["poolFees"]["baseFee"];

/** Steady-state base fee (bps): the END of a fee schedule (after anti-snipe), or the flat fee. */
export function steadyFeeBps(b: BaseFee): number {
  if (b.baseFeeMode === BaseFeeMode.FeeSchedulerLinear || b.baseFeeMode === BaseFeeMode.FeeSchedulerExponential) {
    return toBps(getFeeSchedulerMinBaseFeeNumerator(b.cliffFeeNumerator, b.firstFactor, b.thirdFactor, b.baseFeeMode));
  }
  return toBps(b.cliffFeeNumerator); // rate limiter: the cliff fee is the base fee
}

/** Anti-snipe schedule: starting fee + duration (seconds), or null when the fee is flat. */
export function antiSnipe(b: BaseFee, activationType: number): { startBps: number; seconds: number } | null {
  const scheduled = b.baseFeeMode === BaseFeeMode.FeeSchedulerLinear || b.baseFeeMode === BaseFeeMode.FeeSchedulerExponential;
  if (!scheduled || b.firstFactor === 0) return null;
  const start = toBps(b.cliffFeeNumerator);
  if (start <= steadyFeeBps(b)) return null;
  const points = b.firstFactor * Number(b.secondFactor.toString());
  return { startBps: start, seconds: activationType === 1 ? points : Math.round(points * 0.4) }; // slots ≈ 400 ms
}

export function termsFromConfig(cfg: PoolConfig): LaunchTermsView {
  const b = cfg.poolFees.baseFee;
  const a = antiSnipe(b, cfg.activationType);
  return {
    tradingFeeBps: steadyFeeBps(b),
    creatorFeePercent: cfg.creatorTradingFeePercentage,
    antiSnipeStartBps: a?.startBps ?? null,
    antiSnipeSeconds: a?.seconds ?? null,
    graduationSol: Number(BigInt(cfg.migrationQuoteThreshold.toString())) / 1e9,
    creatorCostSol: MAINNET_LAUNCH_TERMS.creatorCostSol,
  };
}

/** Fallback terms for copy before the mainnet config exists. */
export const PLANNED_MAINNET_TERMS: LaunchTermsView = {
  tradingFeeBps: MAINNET_LAUNCH_TERMS.tradingFeeBps,
  creatorFeePercent: MAINNET_LAUNCH_TERMS.creatorFeePercent,
  antiSnipeStartBps: MAINNET_LAUNCH_TERMS.antiSnipeStartBps,
  antiSnipeSeconds: MAINNET_LAUNCH_TERMS.antiSnipeSeconds,
  graduationSol: MAINNET_LAUNCH_TERMS.graduationSol,
  creatorCostSol: MAINNET_LAUNCH_TERMS.creatorCostSol,
};

/**
 * Problems that make a platform config unsafe to launch on (empty = OK). On mainnet the config must pay its fees
 * to Barton's fee wallet and quote in SOL, so a wrong NEXT_PUBLIC_DBC_CONFIG_MAINNET can never route creators'
 * launches (and Armory's fee share) to someone else's config.
 */
export function configProblems(cfg: Pick<PoolConfig, "feeClaimer" | "quoteMint">, cluster: ClusterName, expectedFeeClaimer: PublicKey = MAINNET_FEE_CLAIMER): string[] {
  const out: string[] = [];
  if (!cfg.quoteMint.equals(WRAPPED_SOL_MINT)) out.push("The platform config does not quote in SOL.");
  if (cluster === "mainnet-beta" && !cfg.feeClaimer.equals(expectedFeeClaimer)) out.push("The platform config's fee wallet is not Armory's fee wallet.");
  return out;
}
