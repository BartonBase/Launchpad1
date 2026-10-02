/**
 * Armory product constants, feature flags and launch types.
 * Sources: docs/BRIEF.md (2026-10-01 wins), docs/DECISIONS.md (ADR-013/016/018/019/020),
 * docs/lazy-mint-interface.md. On-chain values are always read from chain where they exist;
 * these constants are for UI math BEFORE an account exists and are cross-checked in tests.
 */
import type { ClusterName } from "./cluster";

export const BRAND = { name: "Armory", ticker: "ARMS" } as const;

export const LAMPORTS_PER_SOL = 1_000_000_000n;
export const TOTAL_SUPPLY_WHOLE = 1_000_000_000n;
export const MIN_COLLECTION_SIZE = 100;
export const MAX_COLLECTION_SIZE = 10_000;
export const DEFAULT_GRADUATION_SOL = 85;
/** Product rule from the wizard spec (NOTE.md: valid iff T >= 85). The UI floor is
 * max(this, on-chain cluster minimum); devnet's on-chain floor is lower (0.1 SOL test build). */
export const DESIGN_MIN_GRADUATION_LAMPORTS = 85_000_000_000n;
/**
 * Minimum graduation threshold accepted by the DEPLOYED hybrid_launch build, per cluster.
 * Devnet runs the devnet-e2e test build: 0.1 SOL (README in src/lib/generated/idl; the constant
 * is absent from that IDL). The default/mainnet build uses 10 SOL. On-chain LaunchConfigs always
 * carry their own threshold, which the UI reads from chain.
 */
export const MIN_GRADUATION_LAMPORTS: Record<ClusterName, bigint> = {
  localnet: 10_000_000_000n,
  devnet: 100_000_000n,
};

/** Wrap ratios (whole tokens per NFT). 10k dropped (BRIEF 2026-09-25 4:55 PM). */
export const RATIOS = [50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000] as const;
export type Ratio = (typeof RATIOS)[number];

/** Flat SOL tier fee on capture and re-roll; release is free (ADR-013, ADR-019 exact tier). */
export function tierFeeLamports(ratio: number): bigint {
  if (ratio === 50_000) return 2_000_000n;
  if (ratio === 100_000 || ratio === 200_000) return 5_000_000n;
  if (ratio >= 500_000 && ratio <= 5_000_000 && (RATIOS as readonly number[]).includes(ratio)) return 10_000_000n;
  throw new RangeError(`Unsupported ratio ${ratio}`);
}
export const MAX_FEE_LAMPORTS = 10_000_000n;

/**
 * Client-side exact-tier check (ADR-019). The devnet vault predates QA-FEE-03 and never raises
 * 6061 FeeNotTier, so the UI refuses to build capture/re-roll for a LaunchConfig whose stored fee
 * is not exactly the tier for its ratio (or is above the 0.01 SOL cap).
 */
export function isExactTierFee(ratio: bigint | number, storedFeeLamports: bigint): boolean {
  try {
    const tier = tierFeeLamports(Number(ratio));
    return storedFeeLamports === tier && storedFeeLamports > 0n && storedFeeLamports <= MAX_FEE_LAMPORTS;
  } catch {
    return false;
  }
}

export function maxCollectionSize(ratio: number): number {
  return Math.min(MAX_COLLECTION_SIZE, Number(TOTAL_SUPPLY_WHOLE / BigInt(ratio)));
}

/**
 * Lazy-mint refundable deposit per capture / re-roll request (hybrid_vault MINT_ESCROW_LAMPORTS,
 * docs/lazy-mint-interface.md): (3,570,480 rent + 1,500,000 Core fee) x 125%.
 */
export const MINT_ESCROW_LAMPORTS = 6_338_100n;
/** Real first-mint spend range (ADR-018 / CD Q1, mainnet-rate figures): ~0.0031–0.0044 SOL. */
export const FIRST_MINT_SPEND_RANGE_LAMPORTS = [3_066_000n, 4_353_600n] as const;
/** Base signature fee; the preview replaces it with getFeeForMessage when available. */
export const BASE_TX_FEE_LAMPORTS = 5_000n;
/** Account sizes (bytes incl. 8-byte discriminator) for rent that is refunded at settle/expire. */
export const REQUEST_ACCOUNT_BYTES = 304;
export const RAND_LOCK_ACCOUNT_BYTES = 49;

/**
 * BETA DEPOSIT CAP — PLACEHOLDER. No cap exists on-chain yet (itinerary step 7/11; launch gate).
 * Shown site-wide and in the wizard/trade flows so the UI slot exists; NOT ENFORCED.
 */
export const BETA_DEPOSIT_CAP = {
  sol: 10, // design DEP_CAP "10 SOL per wallet", always tagged Example (NOTE.md 2026-10-01)
  enforcedOnChain: false,
  note: "Example value: no cap exists in BRIEF, docs or the programs yet. Not enforced.",
} as const;

// ---------------------------------------------------------------------------
// Launch types + feature flags
// ---------------------------------------------------------------------------

export type LaunchTypeId = "plain" | "hybrid" | "burn" | "tax" | "raffle";
export type LaunchTypeStatus = "live" | "pending-deploy" | "coming-soon";

export interface LaunchType {
  readonly id: LaunchTypeId;
  readonly name: string;
  readonly short: string;
  readonly description: string;
}

export const LAUNCH_TYPES: readonly LaunchType[] = [
  { id: "plain", name: "Plain", short: "Just the coin", description: "A classic 1B memecoin on a bonding curve. No NFTs, no converter, no platform fee." },
  { id: "hybrid", name: "Hybrid", short: "Coin and NFT, both ways", description: "Lock a fixed number of tokens to get a random NFT, and return the NFT for exactly those tokens, any time after graduation." },
  { id: "burn", name: "Burn", short: "Burn coins to mint an NFT", description: "Burn a fixed number of tokens to mint the next NFT in the collection. One-way: the tokens are gone and the NFT can't be turned back." },
  { id: "tax", name: "Tax split", short: "Transfer tax to NFT holders", description: "A transfer fee, fixed at launch, shared with NFT holders. Can't be launched yet." },
  { id: "raffle", name: "Raffle", short: "Tax-funded holder raffle", description: "A transfer fee builds a round pot that one holder wins. Can't be launched yet." },
];

/**
 * Feature flag: tax + raffle. OFF by default. Even when ON there are no builders for these
 * modes, so they can never send a transaction from this app (see launchTypeStatus).
 */
export const FF_TAX_RAFFLE = process.env.NEXT_PUBLIC_FF_TAX_RAFFLE === "1";

/**
 * Which launch modes are DEPLOYED per cluster. Devnet (confirmed by the Solana Program Engineer
 * 2026-10-01 and by simulating each discriminator: launch_plain / launch_burn hit Anchor's
 * InstructionFallbackNotFound): hybrid only. Plain and burn live on branch fix/modes-1-5 @ c43be58,
 * no deploy date.
 */
export const DEPLOYED_MODES: Record<ClusterName, readonly LaunchTypeId[]> = {
  localnet: ["hybrid"],
  devnet: ["hybrid"],
};

export function launchTypeStatus(id: LaunchTypeId, cluster: ClusterName): LaunchTypeStatus {
  if (id === "tax" || id === "raffle") return "coming-soon"; // regardless of FF: no builders exist
  return DEPLOYED_MODES[cluster].includes(id) ? "live" : "pending-deploy";
}

export const STATUS_LABEL: Record<LaunchTypeStatus, string> = {
  live: "Live on devnet",
  "pending-deploy": "Pending deploy",
  "coming-soon": "Coming soon",
};

/**
 * Program-upgrade copy for the Trust page and the Authorities panel (CD, 2026-10-01).
 * PENDING APPROVAL: Barton has not approved this wording yet. Keep it in this one constant so
 * it can be changed in one place.
 */
export const PROGRAM_UPGRADES_COPY = {
  today: "Today on devnet: one development key per program.",
  planned: "Planned for mainnet: 3-of-5 multisig, 7-day public delay.",
} as const;
