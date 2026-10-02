/**
 * Cluster configuration.
 *
 * SAFETY: this app is localnet/devnet ONLY. Any attempt to configure
 * mainnet-beta (by cluster name or by RPC URL) throws at module load, which
 * fails `next dev`, `next build` (next.config.ts imports this module) and the
 * browser bundle alike.
 *
 * This module is deliberately dependency-free (no @solana/web3.js) so it can
 * be imported from next.config.ts and from tests.
 */

export const SUPPORTED_CLUSTERS = ["localnet", "devnet"] as const;
export type ClusterName = (typeof SUPPORTED_CLUSTERS)[number];

export interface ClusterConfig {
  readonly name: ClusterName;
  /** Human label for the always-visible badge. */
  readonly label: "LOCALNET" | "DEVNET";
  /** HTTP(S) JSON-RPC endpoint. */
  readonly rpcUrl: string;
  /** WebSocket endpoint for subscriptions. */
  readonly wsUrl: string;
  /** Origins the CSP connect-src must allow for this cluster. */
  readonly connectSrc: readonly string[];
  /** Explorer link builder hint (Solana Explorer query param). */
  readonly explorerClusterParam: string;
}

export class MainnetForbiddenError extends Error {
  constructor(detail: string) {
    super(
      `Mainnet is forbidden in this app (localnet/devnet only): ${detail}`,
    );
    this.name = "MainnetForbiddenError";
  }
}

export class ClusterConfigError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ClusterConfigError";
  }
}

const PRESETS: Record<ClusterName, ClusterConfig> = {
  localnet: {
    name: "localnet",
    label: "LOCALNET",
    rpcUrl: "http://127.0.0.1:8899",
    wsUrl: "ws://127.0.0.1:8900",
    connectSrc: [
      "http://127.0.0.1:8899",
      "ws://127.0.0.1:8900",
      "http://localhost:8899",
      "ws://localhost:8900",
    ],
    explorerClusterParam:
      "custom&customUrl=" + encodeURIComponent("http://127.0.0.1:8899"),
  },
  devnet: {
    name: "devnet",
    label: "DEVNET",
    rpcUrl: "https://api.devnet.solana.com",
    wsUrl: "wss://api.devnet.solana.com",
    connectSrc: ["https://api.devnet.solana.com", "wss://api.devnet.solana.com"],
    explorerClusterParam: "devnet",
  },
};

/** Anything that smells like mainnet. Checked case-insensitively. */
const MAINNET_PATTERNS: readonly RegExp[] = [
  /mainnet/i, // mainnet, mainnet-beta, api.mainnet-beta.solana.com, *.mainnet.* RPC providers
];

function assertNotMainnet(value: string | undefined, what: string): void {
  if (value === undefined) return;
  for (const re of MAINNET_PATTERNS) {
    if (re.test(value)) {
      throw new MainnetForbiddenError(`${what}="${value}"`);
    }
  }
}

export interface ClusterEnv {
  /** Value of NEXT_PUBLIC_SOLANA_CLUSTER. */
  readonly cluster?: string | undefined;
  /**
   * Optional NEXT_PUBLIC_SOLANA_RPC_URL override. Only allowed if it matches
   * one of the selected cluster's preset endpoints (e.g. localhost vs
   * 127.0.0.1), because the CSP connect-src is derived from the presets.
   */
  readonly rpcUrl?: string | undefined;
}

/**
 * Pure resolver (unit-tested). Throws MainnetForbiddenError for mainnet and
 * ClusterConfigError for anything else invalid. Never silently falls back to
 * a different cluster than the one requested.
 */
export function resolveCluster(env: ClusterEnv): ClusterConfig {
  const rawCluster = env.cluster?.trim();
  const rawRpc = env.rpcUrl?.trim();

  // Mainnet check happens first, on the raw values, before anything else.
  assertNotMainnet(rawCluster, "NEXT_PUBLIC_SOLANA_CLUSTER");
  assertNotMainnet(rawRpc, "NEXT_PUBLIC_SOLANA_RPC_URL");

  const name = (rawCluster === undefined || rawCluster === ""
    ? "localnet"
    : rawCluster.toLowerCase()) as string;

  if (!(SUPPORTED_CLUSTERS as readonly string[]).includes(name)) {
    throw new ClusterConfigError(
      `Unsupported NEXT_PUBLIC_SOLANA_CLUSTER="${rawCluster}". ` +
        `Allowed: ${SUPPORTED_CLUSTERS.join(", ")}.`,
    );
  }
  const preset = PRESETS[name as ClusterName];

  if (rawRpc === undefined || rawRpc === "") return preset;

  let url: URL;
  try {
    url = new URL(rawRpc);
  } catch {
    throw new ClusterConfigError(`NEXT_PUBLIC_SOLANA_RPC_URL is not a valid URL.`);
  }
  if (url.username || url.password) {
    throw new ClusterConfigError(
      "NEXT_PUBLIC_SOLANA_RPC_URL must not contain credentials (it is public).",
    );
  }
  const normalized = url.origin; // strips path/query; presets are origins
  const allowedHttp = preset.connectSrc.filter((o) => o.startsWith("http"));
  if (!allowedHttp.includes(normalized) || url.pathname !== "/" || url.search) {
    throw new ClusterConfigError(
      `NEXT_PUBLIC_SOLANA_RPC_URL="${rawRpc}" is not allowed for ${name}. ` +
        `Allowed: ${allowedHttp.join(", ")}.`,
    );
  }
  const wsUrl = normalized
    .replace(/^http:/, "ws:")
    .replace(/^https:/, "wss:")
    .replace(/:8899$/, ":8900");
  return { ...preset, rpcUrl: normalized, wsUrl };
}

/**
 * The active cluster. `process.env.NEXT_PUBLIC_*` must be referenced
 * literally so Next.js can inline the values into the client bundle.
 * Evaluated at module load => mainnet config fails fast everywhere.
 */
export const CLUSTER: ClusterConfig = resolveCluster({
  cluster: process.env.NEXT_PUBLIC_SOLANA_CLUSTER,
  rpcUrl: process.env.NEXT_PUBLIC_SOLANA_RPC_URL,
});

export function explorerTxUrl(signature: string, cluster: ClusterConfig = CLUSTER): string {
  return `https://explorer.solana.com/tx/${encodeURIComponent(signature)}?cluster=${cluster.explorerClusterParam}`;
}

export function explorerAddressUrl(address: string, cluster: ClusterConfig = CLUSTER): string {
  return `https://explorer.solana.com/address/${encodeURIComponent(address)}?cluster=${cluster.explorerClusterParam}`;
}
