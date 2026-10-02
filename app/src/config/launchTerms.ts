/**
 * Mainnet Launch-type terms (ADR-022; final parameters relayed by Barton, 2026-10-02). These are the values the
 * mainnet DBC config is created with. They are ONLY a fallback for copy shown before the config exists: once
 * NEXT_PUBLIC_DBC_CONFIG_MAINNET is set, the launch form reads the live values from chain (src/lib/meteora/terms.ts)
 * and refuses to launch if the config's fee claimer is not Barton's fee wallet below.
 *
 * A DBC config is immutable: changing any of these later means creating a NEW config and redeploying the app.
 */
import { PublicKey } from "@solana/web3.js";
import type { ClusterName } from "./cluster";

/** Barton's mainnet fee wallet: fee claimer of the mainnet DBC config (a public address, not a key). */
export const MAINNET_FEE_CLAIMER = new PublicKey("BVKxZMjuXryATqCeifPgh6Ee9H93BH5UGv8eML66yT3j");

/** Solana's incinerator: the mainnet config's leftover receiver (unsold curve dust can never move). */
export const INCINERATOR = new PublicKey("1nc1nerator11111111111111111111111111111111");

/** Meteora DBC protocol share of every trading fee (SDK PROTOCOL_FEE_PERCENT). */
export const DBC_PROTOCOL_FEE_PERCENT = 20;

export const MAINNET_LAUNCH_TERMS = {
  /** Curve trading fee after the anti-snipe window, taken in SOL on buys and sells. */
  tradingFeeBps: 200,
  /** Creator's share of the post-protocol (non-Meteora) part of the trading fee. */
  creatorFeePercent: 50,
  /** Anti-snipe: the fee starts at 50% and decays to the trading fee over 60 s. */
  antiSnipeStartBps: 5_000,
  antiSnipeSeconds: 60,
  /** Graduation: the curve migrates to a Meteora DAMM v2 pool at 40 SOL raised. */
  graduationSol: 40,
  /** DAMM v2 pool fee after graduation: 1%, collected in SOL only. */
  dammFeeBps: 100,
  /**
   * What the creator pays to launch: network rent for the new mint, curve pool, two vaults and metadata, plus the
   * transaction fee (no Armory fee; poolCreationFee = 0). Measured on the devnet e2e launch (2026-10-02); the
   * transaction preview always shows the exact simulated amount before signing.
   */
  creatorCostSol: 0.0206,
} as const;

/** Steady curve trade fee of each cluster's platform config (devnet config DuQYHUC…: flat 1%). For static copy only. */
export const CURVE_FEE_BPS: Record<ClusterName, number> = { localnet: 100, devnet: 100, "mainnet-beta": MAINNET_LAUNCH_TERMS.tradingFeeBps };

export const curveFeeText = (c: ClusterName) => pct(CURVE_FEE_BPS[c]);

const pct = (bps: number) => `${(bps / 100).toLocaleString("en-US", { maximumFractionDigits: 2 })}%`;

export interface LaunchTermsView {
  readonly tradingFeeBps: number;
  readonly creatorFeePercent: number;
  readonly antiSnipeStartBps: number | null;
  readonly antiSnipeSeconds: number | null;
  readonly graduationSol: number;
  readonly creatorCostSol: number;
}

/** Plain-English disclosure lines for the launch form (unit-tested). */
export function launchDisclosure(t: LaunchTermsView): string[] {
  const creatorOfFee = (t.tradingFeeBps / 100) * (1 - DBC_PROTOCOL_FEE_PERCENT / 100) * (t.creatorFeePercent / 100);
  const lines = [
    `Launching costs you about ${t.creatorCostSol.toFixed(4)} SOL in network rent and fees. Armory charges no launch fee.`,
    `Every trade on the curve pays a ${pct(t.tradingFeeBps)} fee in SOL. Meteora keeps ${DBC_PROTOCOL_FEE_PERCENT}% of it; the rest is split ${t.creatorFeePercent}/${100 - t.creatorFeePercent} between you (the creator) and Armory, so you earn about ${creatorOfFee.toLocaleString("en-US", { maximumFractionDigits: 2 })}% of curve volume. Claim it any time from Portfolio.`,
  ];
  if (t.antiSnipeStartBps !== null && t.antiSnipeSeconds !== null && t.antiSnipeStartBps > t.tradingFeeBps) {
    lines.push(`Anti-snipe: for the first ${t.antiSnipeSeconds} seconds after launch the fee starts at ${pct(t.antiSnipeStartBps)} and falls to ${pct(t.tradingFeeBps)}, so bots can't cheaply buy the first block. This applies to your own first buy too.`);
  }
  lines.push(`At ${t.graduationSol.toLocaleString("en-US")} SOL raised the coin graduates to a Meteora DAMM v2 pool with permanently locked liquidity.`);
  return lines;
}
