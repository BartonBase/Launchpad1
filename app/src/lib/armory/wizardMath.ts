/**
 * Launch wizard live math, per launch type. Mirrors design/directions/a-obsidian/launch.html
 * (render()) and NOTE.md "Wizard math". Pure; unit-tested in tests/wizardMath.test.ts.
 */
import {
  DESIGN_MIN_GRADUATION_LAMPORTS,
  LAMPORTS_PER_SOL,
  MIN_COLLECTION_SIZE,
  TOTAL_SUPPLY_WHOLE,
  maxCollectionSize,
  tierFeeLamports,
} from "@/config/armory";

export type WizardType = "plain" | "hybrid" | "burn";

/** Example curve prices used by the design (SOL per token) and $/SOL. Examples only. */
export const EXAMPLE_LAUNCH_PRICE_SOL = 2.8e-8;
export const EXAMPLE_GRADUATION_PRICE_SOL = 5.7e-7;
export const EXAMPLE_SOL_USD = 150;
/** Example Plain split (set by the approved DBC config): 800M curve / 200M DEX pool. */
export const EXAMPLE_CURVE_SPLIT = { curve: 800_000_000n, dex: 200_000_000n } as const;
/** Burn pays the mint directly every burn (NOTE.md "≈0.005 SOL"). Display estimate only. */
export const BURN_MINT_COST_ESTIMATE_LAMPORTS = 5_000_000n;

export interface WizardInput {
  readonly type: WizardType;
  readonly ratio: number;
  readonly size: number;
  readonly targetLamports: bigint | null;
  /** On-chain floor for the cluster (devnet 0.1 SOL). */
  readonly chainMinLamports: bigint;
}

export interface WizardMath {
  readonly minTargetLamports: bigint;
  readonly targetOk: boolean;
  readonly maxSize: number;
  readonly sizeOk: boolean;
  readonly sizeError: "low" | "high" | null;
  readonly valid: boolean;
  /** Hybrid: max tokens in NFT form at once. Burn: most that can ever burn. */
  readonly nftTokens: bigint;
  /** Percentage of supply (number, for display). */
  readonly nftPct: number;
  /** Hybrid: always tokens. Burn: never burned = final supply floor. */
  readonly restTokens: bigint;
  readonly feeLamports: bigint | null;
  readonly nftPriceLaunchSol: number;
  readonly nftPriceGraduationSol: number;
}

export function wizardMath(i: WizardInput): WizardMath {
  const minTargetLamports = i.chainMinLamports > DESIGN_MIN_GRADUATION_LAMPORTS ? i.chainMinLamports : DESIGN_MIN_GRADUATION_LAMPORTS;
  const targetOk = i.targetLamports !== null && i.targetLamports >= minTargetLamports;
  const maxSize = maxCollectionSize(i.ratio);
  const sizeError = !(i.size >= MIN_COLLECTION_SIZE) ? "low" : i.size > maxSize ? "high" : null;
  const sizeOk = Number.isInteger(i.size) && sizeError === null;
  const plain = i.type === "plain";
  const raw = sizeOk ? BigInt(i.size) * BigInt(i.ratio) : 0n;
  const nftTokens = plain ? 0n : raw > TOTAL_SUPPLY_WHOLE ? TOTAL_SUPPLY_WHOLE : raw;
  return {
    minTargetLamports,
    targetOk,
    maxSize,
    sizeOk,
    sizeError,
    valid: targetOk && (plain || sizeOk),
    nftTokens,
    nftPct: Number((nftTokens * 10_000n) / TOTAL_SUPPLY_WHOLE) / 100,
    restTokens: TOTAL_SUPPLY_WHOLE - nftTokens,
    feeLamports: plain ? null : tierFeeLamports(i.ratio),
    nftPriceLaunchSol: i.ratio * EXAMPLE_LAUNCH_PRICE_SOL,
    nftPriceGraduationSol: i.ratio * EXAMPLE_GRADUATION_PRICE_SOL,
  };
}

/** Design's SOL formatter: >=1 -> 2 dp, >=0.01 -> 3 dp, else 2 significant digits. */
export function designSol(n: number): string {
  return n >= 1 ? n.toFixed(2) : n >= 0.01 ? n.toFixed(3) : n.toPrecision(2);
}

export const lamportsToSolNumber = (l: bigint) => Number(l) / Number(LAMPORTS_PER_SOL);
