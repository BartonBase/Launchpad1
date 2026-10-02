/**
 * Per-cluster integration settings (Switchboard reveal, Meteora DBC, lookup tables).
 */
import { PublicKey } from "@solana/web3.js";
import type { ClusterName } from "./cluster";

/**
 * Meteora DBC platform config used by register_dbc_launch. Devnet: DuQYHUC… is the platform
 * devnet config; it only works with the devnet-e2e hybrid_launch build (0.1 SOL graduation
 * floor + this config compiled into APPROVED_DBC_CONFIGS). Its graduation threshold
 * (migration_quote_threshold) is READ FROM CHAIN, never hard-coded. null = no DBC path.
 */
export const DBC_PLATFORM_CONFIG: Record<ClusterName, PublicKey | null> = {
  localnet: null,
  devnet: new PublicKey("DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9"),
};

/**
 * Address lookup tables the app may use (settle-with-mint for large collections). The preview
 * blocks any transaction that loads accounts from a table NOT listed here, and every listed table
 * must be frozen (no authority) or it is rejected too. Empty on devnet: creating a table is a
 * send, which this pass doesn't do; settle for the 100-NFT E2E launch fits without one (see
 * README "Reveal and settle").
 */
export const PINNED_LOOKUP_TABLES: Record<ClusterName, readonly PublicKey[]> = {
  localnet: [],
  devnet: [],
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
};
