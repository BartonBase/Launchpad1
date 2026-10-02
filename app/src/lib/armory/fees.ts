/**
 * Capture / re-roll cost math (lazy minting, ADR-016/018). Pure and unit-tested.
 *
 * What leaves the wallet at request time:
 *   tier fee (never refunded) + refundable mint deposit (6,338,100) + rent for the temporary
 *   request + rand-lock accounts (refunded at settle/expire) + network fee.
 * At settle: deposit fully refunded if the pick was already minted; if the pick is minted for the
 * first time, ~0.0031–0.0044 SOL of it pays rent + the Metaplex Core fee and the rest is refunded.
 * Expire refunds the deposit in full. The tier fee is the only thing never refunded.
 */
import { BASE_TX_FEE_LAMPORTS, FIRST_MINT_SPEND_RANGE_LAMPORTS, MINT_ESCROW_LAMPORTS } from "@/config/armory";

export interface RequestCostInput {
  readonly tierFeeLamports: bigint;
  /** Rent for the Request + RandLock accounts (refunded at settle/expire). */
  readonly tempRentLamports: bigint;
  readonly networkFeeLamports?: bigint;
  readonly depositLamports?: bigint;
}

export interface RequestCost {
  readonly tierFee: bigint;
  readonly deposit: bigint;
  readonly tempRent: bigint;
  readonly networkFee: bigint;
  /** Everything that must be in the wallet to submit the request. */
  readonly requiredBalance: bigint;
  /** Worst case actually spent after settle (fee + max first-mint spend + network fee). */
  readonly maxNetCost: bigint;
  /** Best case spent after settle (pick already minted): fee + network fee. */
  readonly minNetCost: bigint;
}

export function requestCost(i: RequestCostInput): RequestCost {
  const deposit = i.depositLamports ?? MINT_ESCROW_LAMPORTS;
  const networkFee = i.networkFeeLamports ?? BASE_TX_FEE_LAMPORTS;
  return {
    tierFee: i.tierFeeLamports,
    deposit,
    tempRent: i.tempRentLamports,
    networkFee,
    requiredBalance: i.tierFeeLamports + deposit + i.tempRentLamports + networkFee,
    maxNetCost: i.tierFeeLamports + FIRST_MINT_SPEND_RANGE_LAMPORTS[1] + networkFee,
    minNetCost: i.tierFeeLamports + networkFee,
  };
}

export type BalanceCheck = { ok: true } | { ok: false; shortfall: bigint };
export function checkBalance(balanceLamports: bigint, cost: RequestCost): BalanceCheck {
  return balanceLamports >= cost.requiredBalance ? { ok: true } : { ok: false, shortfall: cost.requiredBalance - balanceLamports };
}

/**
 * Rent-exempt minimum with current Solana parameters ((128 + bytes) x 3,480 x 2). Used only as a
 * display fallback until getMinimumBalanceForRentExemption answers.
 */
export function estimateRentExempt(bytes: number): bigint {
  return BigInt((128 + bytes) * 3480 * 2);
}
