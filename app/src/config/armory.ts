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
/**
 * Design copy (NOTE.md 10/1): "Default 85 SOL · minimum 10 SOL". 10 SOL is the production build's
 * MIN_GRADUATION_THRESHOLD_LAMPORTS; validation always uses the active cluster's chain minimum
 * (MIN_GRADUATION_LAMPORTS below), which is 0.1 SOL on devnet.
 */
export const PRODUCTION_MIN_GRADUATION_LAMPORTS = 10_000_000_000n;
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
/** Shared copy (NOTE.md 10/1 chain alignment). Live figures come from chain where available. */
const sol4 = (l: bigint) => (Number(l) / 1e9).toFixed(4); // 6,338,100 -> "0.0063"
/** FIRST_MINT_SPEND_RANGE_LAMPORTS (0.0031–0.0044) rounded down to the design's "≈ 0.003–0.004". */
export const FIRST_MINT_RANGE_TEXT = "≈ 0.003–0.004 SOL";
export const mintDepositText = (depositLamports: bigint = MINT_ESCROW_LAMPORTS, firstMint?: readonly [bigint, bigint]) =>
  `Mint deposit ${sol4(depositLamports)} SOL, refunded except ${firstMint ? `≈ ${sol4(firstMint[0])}–${sol4(firstMint[1])} SOL` : FIRST_MINT_RANGE_TEXT} if your NFT is minted for the first time`;
export const BURN_MINT_TEXT = `${FIRST_MINT_RANGE_TEXT}, paid directly by the burner (no deposit)`;

/** Base signature fee; the preview replaces it with getFeeForMessage when available. */
export const BASE_TX_FEE_LAMPORTS = 5_000n;
/** Account sizes (bytes incl. 8-byte discriminator) for rent that is refunded at settle/expire. */
export const REQUEST_ACCOUNT_BYTES = 304;
export const RAND_LOCK_ACCOUNT_BYTES = 49;
/**
 * One-time randomness setup, paid only when the vault has no free Switchboard randomness account
 * (init_randomness is bundled): the 480-byte randomness account plus two Switchboard-owned accounts
 * (165 and 152 bytes). Measured by simulation on devnet, 2026-10-01: 5,999,480 lamports at devnet
 * rent. Not refunded; the account stays with the vault and is reused by later requests.
 */
export const RANDOMNESS_SETUP_ACCOUNT_BYTES = [480, 165, 152] as const;
/**
 * First-mint spend = rent for the new Core asset + the Metaplex Core create fee (kept in the asset).
 * The asset is ~97–282 bytes depending on name/URI; devnet settle measured 2,658,240 lamports
 * (100 bytes at devnet rent + 1,500,000). FIRST_MINT_SPEND_RANGE_LAMPORTS is the same at mainnet rent.
 */
export const FIRST_MINT_ASSET_BYTES_RANGE = [97, 282] as const;
export const CORE_CREATE_FEE_LAMPORTS = 1_500_000n;

// (The beta deposit-cap placeholder was removed from the UI on 2026-10-02: no cap exists on-chain.)

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
  { id: "plain", name: "Launch", short: "Just the coin", description: "A classic 1B meme coin on a bonding curve that graduates to a locked liquidity pool. No NFTs, no converter, no platform fee." },
  { id: "hybrid", name: "Hybrid", short: "Coin and NFT, both ways", description: "Lock a fixed number of tokens to get a random NFT, and return the NFT for exactly those tokens, any time after graduation." },
  { id: "burn", name: "Burn", short: "Burn coins to mint an NFT", description: "Burn a fixed number of tokens to mint the next NFT in the collection. One-way: the tokens are gone and the NFT can't be turned back." },
  { id: "tax", name: "Tax split", short: "Transfer tax to NFT holders", description: "A transfer fee, fixed at launch, shared with NFT holders." },
  { id: "raffle", name: "Raffle", short: "Tax-funded holder raffle", description: "A transfer fee builds a round pot that one holder wins." },
];

/**
 * Feature flag: tax + raffle. OFF by default. Even when ON there are no builders for these
 * modes, so they can never send a transaction from this app (see launchTypeStatus).
 */
export const FF_TAX_RAFFLE = process.env.NEXT_PUBLIC_FF_TAX_RAFFLE === "1";

/**
 * Which launch modes can be LAUNCHED per cluster.
 * - Hybrid: the deployed hybrid_launch / hybrid_vault programs.
 * - Plain (devnet, 2026-10-02): straight on Meteora DBC with the official SDK against the platform
 *   config DuQYHUC… (initialize_virtual_pool_with_spl_token; config: fixed 1B supply, mint authority
 *   revoked, immutable metadata, leftover to the locked ["dbc_buffer"] PDA, migrates to DAMM v2).
 *   No Armory program is needed, so the undeployed launch_plain is not used.
 *   Verified on devnet: plain launch 4awRSUS5…, buy F31NFAf6…, sell tXVXFGjw….
 * - Burn: launch_burn is not deployed (InstructionFallbackNotFound on devnet), so it is "Coming
 *   soon" everywhere, like Tax split and Raffle.
 */
export const DEPLOYED_MODES: Record<ClusterName, readonly LaunchTypeId[]> = {
  localnet: ["hybrid"],
  devnet: ["hybrid", "plain"],
};

/** Modes shown as "Coming soon" on every cluster (no builders / programs not deployed). */
export const COMING_SOON_MODES: readonly LaunchTypeId[] = ["burn", "tax", "raffle"];

export function launchTypeStatus(id: LaunchTypeId, cluster: ClusterName): LaunchTypeStatus {
  if (COMING_SOON_MODES.includes(id)) return "coming-soon"; // regardless of FF: no builders exist
  return DEPLOYED_MODES[cluster].includes(id) ? "live" : "pending-deploy";
}

export const STATUS_LABEL: Record<LaunchTypeStatus, string> = {
  live: "Live",
  "pending-deploy": "Pending deploy",
  "coming-soon": "Coming soon",
};

/** Program-upgrade copy for the Trust page, FAQ and Authorities panel (approved by Barton, 2026-10-01). */
export const PROGRAM_UPGRADES_COPY = {
  status: "Not locked yet",
  today: "Today: one development key per program.",
  planned: "Planned for mainnet: a 3-of-5 multisig plus a 7-day public delay.",
  after: "Frozen after the audit.",
} as const;

/** Approved key-custody rule (Barton, 2026-10-01). Shown verbatim on the Trust page. */
export const KEY_CUSTODY_RULE = "No AI agent holds mainnet keys. Mainnet keys are held by humans only.";
