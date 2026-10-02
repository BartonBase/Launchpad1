/**
 * Cluster configuration, switchable by env (release/mainnet-app, Barton 2026-10-02).
 *
 *   NEXT_PUBLIC_SOLANA_CLUSTER   localnet (default) | devnet | mainnet-beta ("mainnet" is accepted as an alias)
 *   NEXT_PUBLIC_SOLANA_RPC_URL   optional custom RPC for that cluster (e.g. Helius:
 *                                https://mainnet.helius-rpc.com/?api-key=<key>). Public: it ships in the browser
 *                                bundle, so use a key with a domain allowlist. Without it the cluster's public
 *                                endpoint is used (rate-limited: requests retry on 429, see src/lib/rpc.ts).
 *   SOLANA_RPC_URL               optional SERVER-ONLY RPC for server-rendered reads (never sent to the browser).
 *
 * SAFETY:
 * - The cluster is never guessed: an unknown name throws, and nothing silently falls back to another cluster.
 * - An RPC URL must match the selected cluster: a devnet build refuses an RPC URL that names mainnet (and a mainnet
 *   build refuses one that names devnet/testnet or points at localhost), so a mis-set env can't make a "devnet"
 *   site sign real-money transactions or vice versa. (The old blanket /mainnet/i ban also rejected legitimate
 *   mainnet RPC URLs; it is replaced by this match check.)
 * - No credentials in URLs (user:pass@); http:// only for localnet.
 *
 * Dependency-free (no @solana/web3.js) so next.config.ts and tests can import it.
 */

export const SUPPORTED_CLUSTERS = ["localnet", "devnet", "mainnet-beta"] as const;
export type ClusterName = (typeof SUPPORTED_CLUSTERS)[number];

export interface ClusterConfig {
  readonly name: ClusterName;
  /** Upper-case label used in transaction previews. */
  readonly label: "LOCALNET" | "DEVNET" | "MAINNET";
  /** Short human name for the network chip ("Mainnet", "Devnet", "Localnet"). */
  readonly displayName: "Localnet" | "Devnet" | "Mainnet";
  readonly isMainnet: boolean;
  /** HTTP(S) JSON-RPC endpoint (browser + server unless SOLANA_RPC_URL overrides the server side). */
  readonly rpcUrl: string;
  /** WebSocket endpoint for subscriptions. */
  readonly wsUrl: string;
  /** True when rpcUrl is the cluster's free public endpoint (heavily rate-limited on mainnet). */
  readonly publicRpc: boolean;
  /** Origins the CSP connect-src must allow for this cluster. */
  readonly connectSrc: readonly string[];
  /** Solana Explorer `cluster` query value; null = mainnet (Explorer's default, no param). */
  readonly explorerClusterParam: string | null;
  /** Genesis hash of the cluster (null for localnet, which varies). Used to refuse a wrong-cluster RPC at runtime. */
  readonly genesisHash: string | null;
}

/** Kept for compatibility with older imports; mainnet is allowed now (explicit opt-in via the env). */
export class MainnetForbiddenError extends Error {
  constructor(detail: string) {
    super(`Mainnet is not allowed here: ${detail}`);
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
    displayName: "Localnet",
    isMainnet: false,
    rpcUrl: "http://127.0.0.1:8899",
    wsUrl: "ws://127.0.0.1:8900",
    publicRpc: true,
    connectSrc: ["http://127.0.0.1:8899", "ws://127.0.0.1:8900", "http://localhost:8899", "ws://localhost:8900"],
    explorerClusterParam: "custom&customUrl=" + encodeURIComponent("http://127.0.0.1:8899"),
    genesisHash: null,
  },
  devnet: {
    name: "devnet",
    label: "DEVNET",
    displayName: "Devnet",
    isMainnet: false,
    rpcUrl: "https://api.devnet.solana.com",
    wsUrl: "wss://api.devnet.solana.com",
    publicRpc: true,
    connectSrc: ["https://api.devnet.solana.com", "wss://api.devnet.solana.com"],
    explorerClusterParam: "devnet",
    genesisHash: "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG",
  },
  "mainnet-beta": {
    name: "mainnet-beta",
    label: "MAINNET",
    displayName: "Mainnet",
    isMainnet: true,
    rpcUrl: "https://api.mainnet-beta.solana.com",
    wsUrl: "wss://api.mainnet-beta.solana.com",
    publicRpc: true,
    connectSrc: ["https://api.mainnet-beta.solana.com", "wss://api.mainnet-beta.solana.com"],
    explorerClusterParam: null,
    genesisHash: "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
  },
};

const ALIASES: Readonly<Record<string, ClusterName>> = { mainnet: "mainnet-beta", "mainnet-beta": "mainnet-beta", devnet: "devnet", localnet: "localnet" };

/** Words in an RPC host that name a DIFFERENT cluster than the selected one. */
const FOREIGN_HOST: Record<ClusterName, RegExp | null> = {
  localnet: null, // localnet only accepts its own preset origins (checked below)
  devnet: /mainnet|testnet/i,
  "mainnet-beta": /devnet|testnet|localhost|127\.0\.0\.1|0\.0\.0\.0/i,
};

export interface ClusterEnv {
  /** Value of NEXT_PUBLIC_SOLANA_CLUSTER. */
  readonly cluster?: string | undefined;
  /** Optional NEXT_PUBLIC_SOLANA_RPC_URL. */
  readonly rpcUrl?: string | undefined;
}

/** Validates an RPC URL for a cluster and returns its normalised http + ws endpoints. Throws ClusterConfigError. */
export function validateRpcUrl(name: ClusterName, raw: string, what = "NEXT_PUBLIC_SOLANA_RPC_URL"): { rpcUrl: string; wsUrl: string; origin: string; wsOrigin: string } {
  const preset = PRESETS[name];
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    throw new ClusterConfigError(`${what} is not a valid URL.`);
  }
  if (url.username || url.password) throw new ClusterConfigError(`${what} must not contain credentials (user:pass@).`);
  if (url.hash) throw new ClusterConfigError(`${what} must not contain a #fragment.`);
  if (name === "localnet") {
    const allowed = preset.connectSrc.filter((o) => o.startsWith("http"));
    if (!allowed.includes(url.origin) || url.pathname !== "/" || url.search) {
      throw new ClusterConfigError(`${what}="${raw}" is not allowed for localnet. Allowed: ${allowed.join(", ")}.`);
    }
  } else {
    if (url.protocol !== "https:") throw new ClusterConfigError(`${what} must use https:// on ${name}.`);
    const foreign = FOREIGN_HOST[name];
    if (foreign && foreign.test(url.hostname)) {
      throw new ClusterConfigError(`${what}="${url.origin}" looks like a different cluster than NEXT_PUBLIC_SOLANA_CLUSTER=${name}.`);
    }
  }
  const bare = (u: URL) => (u.pathname === "/" && !u.search ? u.origin : u.toString());
  const ws = new URL(url.toString());
  ws.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  if (name === "localnet" && ws.port === "8899") ws.port = "8900";
  const http = bare(url);
  const wsStr = bare(ws);
  return { rpcUrl: http, wsUrl: wsStr, origin: url.origin, wsOrigin: ws.origin };
}

/**
 * Pure resolver (unit-tested). Throws ClusterConfigError for anything invalid. Never silently falls back to a
 * different cluster than the one requested.
 */
export function resolveCluster(env: ClusterEnv): ClusterConfig {
  const rawCluster = env.cluster?.trim();
  const rawRpc = env.rpcUrl?.trim();
  const key = rawCluster === undefined || rawCluster === "" ? "localnet" : rawCluster.toLowerCase();
  const name = ALIASES[key];
  if (!name) {
    throw new ClusterConfigError(`Unsupported NEXT_PUBLIC_SOLANA_CLUSTER="${rawCluster}". Allowed: ${SUPPORTED_CLUSTERS.join(", ")}.`);
  }
  const preset = PRESETS[name];
  if (rawRpc === undefined || rawRpc === "") return preset;
  const v = validateRpcUrl(name, rawRpc);
  const connectSrc = name === "localnet" ? preset.connectSrc : [v.origin, v.wsOrigin];
  return { ...preset, rpcUrl: v.rpcUrl, wsUrl: v.wsUrl, publicRpc: v.origin === new URL(preset.rpcUrl).origin, connectSrc };
}

/**
 * The active cluster. `process.env.NEXT_PUBLIC_*` must be referenced literally so Next.js can inline the values
 * into the client bundle.
 */
export const CLUSTER: ClusterConfig = resolveCluster({
  cluster: process.env.NEXT_PUBLIC_SOLANA_CLUSTER,
  rpcUrl: process.env.NEXT_PUBLIC_SOLANA_RPC_URL,
});

/** Server-side RPC: SOLANA_RPC_URL (server-only, may hold a private key) if set, else the public cluster RPC. */
export function serverRpcUrl(cluster: ClusterConfig = CLUSTER, raw: string | undefined = process.env.SOLANA_RPC_URL): string {
  const r = raw?.trim();
  return r ? validateRpcUrl(cluster.name, r, "SOLANA_RPC_URL").rpcUrl : cluster.rpcUrl;
}

function explorerQuery(cluster: ClusterConfig): string {
  return cluster.explorerClusterParam === null ? "" : `?cluster=${cluster.explorerClusterParam}`;
}

export function explorerTxUrl(signature: string, cluster: ClusterConfig = CLUSTER): string {
  return `https://explorer.solana.com/tx/${encodeURIComponent(signature)}${explorerQuery(cluster)}`;
}

export function explorerAddressUrl(address: string, cluster: ClusterConfig = CLUSTER): string {
  return `https://explorer.solana.com/address/${encodeURIComponent(address)}${explorerQuery(cluster)}`;
}
