import Link from "next/link";
import type { PlainLaunchDTO } from "@/lib/meteora/plain";
import { formatSol, shortAddr } from "@/lib/armory/format";
import { TypeIcon } from "@/components/armory/TypeIcon";
import { tokenImage } from "@/lib/meteora/tokenImage";

/** Explore / home card for a Plain launch (Meteora DBC pool on the platform config). */
export function PlainCard({ p }: { p: PlainLaunchDTO }) {
  const c = p.curve;
  const img = tokenImage(p.uri);
  return (
    <Link href={`/t/${p.mint}`} className="card flex flex-col gap-3 p-4 hover:border-[var(--arm-color-border-strong)]" data-testid="launch-card" data-type="plain" data-phase={c.migrated ? "graduated" : "curve"}>
      <div className="bg-surface-2 rounded-panel border-border relative flex aspect-[4/3] items-center justify-center overflow-hidden border" aria-hidden="true">
        {img ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={img} alt="" loading="lazy" className="size-full object-cover" data-testid="token-image" />
        ) : (
          <span className="hd text-dim text-4xl">{(p.symbol || p.name).slice(0, 1).toUpperCase()}</span>
        )}
        <span className="absolute top-2 left-2">{c.migrated ? <span className="tag tag-ok"><i />Graduated</span> : <span className="tag tag-accent"><i />On curve</span>}</span>
      </div>
      <div className="flex items-start justify-between gap-2">
        <h2 className="truncate font-semibold">{p.name} {p.symbol && <span className="text-muted font-mono text-xs font-normal">{p.symbol}</span>}</h2>
        <span className="text-dim font-mono text-xs">{shortAddr(p.mint)}</span>
      </div>
      <div>
        <div className="progress"><span style={{ width: `${Math.max(c.progressPct, 1)}%` }} /></div>
        <p className="text-muted mt-1 flex justify-between text-xs">
          <span>{c.migrated ? "Graduated · trading in the pool" : `${formatSol(c.quoteReserveLamports, 3)} of ${formatSol(c.thresholdLamports, 1)}`}</span>
          <span>{c.progressPct.toFixed(0)}%</span>
        </p>
      </div>
      <div className="border-border flex items-center justify-between border-t pt-3 text-xs">
        <span className="tchip"><TypeIcon type="plain" size={14} />Launch</span>
        <span className="text-muted">Just the coin</span>
      </div>
    </Link>
  );
}
