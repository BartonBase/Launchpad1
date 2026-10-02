/**
 * Per-cluster integration settings (Switchboard reveal, Meteora DBC, lookup tables).
 * Mainnet (ADR-022): only the Meteora DBC config; no Switchboard, no lookup tables, no Armory programs.
 */
import { PublicKey } from "@solana/web3.js";
import { ClusterConfigError, type ClusterName } from "./cluster";

/**
 * Parses an optional base58 address from the env. Unset/empty = null; set but invalid = throw at module load, so a
 * typo fails `next build` instead of shipping a site that silently can't launch.
 */
export function parseEnvAddress(raw: string | undefined, what: string): PublicKey | null {
  const v = raw?.trim();
  if (!v) return null;
  try {
    const k = new PublicKey(v);
    if (k.toBase58() !== v) throw new Error("not canonical base58");
    return k;
  } catch {
    throw new ClusterConfigError(`${what}="${v}" is not a valid Solana address.`);
  }
}

/**
 * Armory's MAINNET DBC config address (ADR-022, Launch type only). Created and signed by Barton's fee wallet;
 * set NEXT_PUBLIC_DBC_CONFIG_MAINNET once it exists. Until then mainnet shows launches as "opening soon".
 */
export const DBC_CONFIG_MAINNET: PublicKey | null = parseEnvAddress(process.env.NEXT_PUBLIC_DBC_CONFIG_MAINNET, "NEXT_PUBLIC_DBC_CONFIG_MAINNET");

/**
 * Meteora DBC platform config used by register_dbc_launch. Devnet: DuQYHUC… is the platform
 * devnet config; it only works with the devnet-e2e hybrid_launch build (0.1 SOL graduation
 * floor + this config compiled into APPROVED_DBC_CONFIGS). Its graduation threshold
 * (migration_quote_threshold) is READ FROM CHAIN, never hard-coded. null = no DBC path.
 */
export const DBC_PLATFORM_CONFIG: Record<ClusterName, PublicKey | null> = {
  localnet: null,
  devnet: new PublicKey("DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9"),
  // Pools are discovered with getPoolsByConfig(config) (src/lib/meteora/plain.ts); no Armory program involved.
  "mainnet-beta": DBC_CONFIG_MAINNET,
};

/**
 * Address lookup tables the app may use (settle-with-mint). The preview blocks any transaction
 * that loads accounts from a table NOT listed here. A listed table must either be frozen (no
 * authority) or match its pinned contents below exactly (checked against chain before every
 * preview: an authority can only append or close, never rewrite an index, so a verified prefix
 * can't be redirected). Signers and per-request accounts (request, escrow, rand lock, randomness,
 * asset) are never in a table.
 */
export const PINNED_LOOKUP_TABLES: Record<ClusterName, readonly PublicKey[]> = {
  localnet: [],
  // Devnet ARMT settle table (Solana Program Engineer, 2026-10-01; authority An3Zmi…, 21 entries).
  devnet: [new PublicKey("5xhFeeakpaggTw9Ntbjt8yuZSHEoXPTUd63h4tVmZVsU")],
  "mainnet-beta": [], // no lookup tables on mainnet (Launch type only)
};

/** Exact expected contents (base58, in index order) of each pinned table, keyed by table address. */
export const PINNED_LOOKUP_TABLE_CONTENTS: Readonly<Record<string, readonly string[]>> = {
  "5xhFeeakpaggTw9Ntbjt8yuZSHEoXPTUd63h4tVmZVsU": [
    "BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG", // 0 hybrid_vault
    "9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8", // 1 hybrid_launch
    "MwFTkPQ2TReZo5axKkXTUKC4sBoBmrSKs4JwAhFjPHK", // 2 vault
    "AGVZd96xUp1WSadGY6TWwsZi6C5CzY7aNsk6fHm66F6m", // 3 launch_config
    "B8r2rRuhoSWSdyMxDFZL2YRrRWqg13iiZfWyxSBqUA3D", // 4 pool_index
    "8Gnr6DbAgz9XHmniFwSRAKXSvotH1QG1iDQkP5LpzBYs", // 5 collection
    "FrVu4UMvxthMYuS8tsoZ5F8S3kS4j6oP7vHwt1Sddd3a", // 6 vault_authority
    "FqRypseqJx28E8kCusxcxStnyoFTcoishSC2msohZktZ", // 7 vault_tokens
    "BwsDh6Ka6b5BJe8bxWG3Hum5DxxzGiQQUgnbZzwvTFxD", // 8 randomness_authority
    "Dhv3VkqeTYJiahzcVJLmJ1snApdYtQxsAKeUnv5GTNos", // 9 mint
    "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d", // 10 mpl_core
    "Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2", // 11 switchboard
    "EYiAmGSdsQTuCw413V5BzaruWuCCSDgTPtBGvLkXHbe7", // 12 sb_queue
    "4UFmCebEmzESoDTtHrmaftXj7YAsAH4HMios3yMWyVUT", // 13 sb_program_state
    "7J3AajxfajAMgGmmwzfZeYGNieRtHRCuEgNTfd4gTjDN", // 14 fee_wallet
    "11111111111111111111111111111111", // 15 system_program
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // 16 token_program
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", // 17 associated_token_program
    "SysvarS1otHashes111111111111111111111111111", // 18 slot_hashes
    "ComputeBudget111111111111111111111111111111", // 19 compute_budget
    "So11111111111111111111111111111111111111112", // 20 wsol_mint
  ],
};

/**
 * Switchboard oracle gateways the browser may call for a randomness reveal. Devnet oracles
 * publish gateway_uri = https://<ipv4>.xip.switchboard-oracles.xyz/devnet (all 9 queue oracles
 * on 2026-10-01). The CSP allows exactly that host pattern; this regex rejects anything else
 * before a request is made.
 */
export const SWITCHBOARD_GATEWAY_RE: Record<ClusterName, RegExp | null> = {
  localnet: null,
  devnet: /^https:\/\/(?:\d{1,3}\.){3}\d{1,3}\.xip\.switchboard-oracles\.xyz\/devnet$/,
  "mainnet-beta": null, // no Switchboard on mainnet (Launch type only)
};
