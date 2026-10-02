import { CLUSTER } from "@/config/cluster";

/** Small, quiet network chip shown next to the wallet button. Server component (no JS). */
export function ClusterBadge() {
  const name = CLUSTER.name.charAt(0).toUpperCase() + CLUSTER.name.slice(1);
  return (
    <span
      data-testid="cluster-badge"
      title={`Network: ${name}`}
      className="border-border text-muted rounded-chip inline-flex items-center gap-1.5 border px-2 py-0.5 text-xs"
    >
      <i aria-hidden="true" className="bg-warning inline-block h-1.5 w-1.5 rounded-full" />
      {name}
    </span>
  );
}
