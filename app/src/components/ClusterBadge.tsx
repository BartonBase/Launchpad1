import { CLUSTER } from "@/config/cluster";

/**
 * Small, quiet network chip shown next to the wallet button. Server component (no JS).
 * Amber dot on test networks (devnet/localnet: tokens have no value), green on mainnet.
 */
export function ClusterBadge() {
  const name = CLUSTER.displayName;
  return (
    <span
      data-testid="cluster-badge"
      data-cluster={CLUSTER.name}
      title={CLUSTER.isMainnet ? "Network: Solana mainnet" : `Network: Solana ${name} (test network, tokens have no value)`}
      className="border-border text-muted rounded-chip inline-flex items-center gap-1.5 border px-2 py-0.5 text-xs"
    >
      <i aria-hidden="true" className={`${CLUSTER.isMainnet ? "bg-positive" : "bg-warning"} inline-block h-1.5 w-1.5 rounded-full`} />
      {name}
    </span>
  );
}
