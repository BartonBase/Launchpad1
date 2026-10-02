import { CLUSTER } from "@/config/cluster";

/** Always-visible, loud cluster indicator. Server component (no JS). */
export function ClusterBadge() {
  const bg = CLUSTER.name === "devnet" ? "bg-cluster-devnet" : "bg-cluster-localnet";
  return (
    <span
      data-testid="cluster-badge"
      title={`Connected to ${CLUSTER.label} via ${CLUSTER.rpcUrl} — test funds only`}
      className={`${bg} text-cluster-fg rounded-control px-3 py-1 font-mono text-sm font-bold tracking-widest uppercase`}
    >
      {CLUSTER.label}
    </span>
  );
}
