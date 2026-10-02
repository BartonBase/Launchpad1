/**
 * Bonding-curve progress and graduation-to-DAMM-v2 indicator (server component; data read from
 * chain by the page via the Meteora DBC SDK).
 */
import { explorerAddressUrl } from "@/config/cluster";
import { formatSol, shortAddr } from "@/lib/armory/format";
import type { CurveStateDTO } from "@/lib/meteora/dbc";
import { fmtPrice } from "@/components/armory/TokenSections";

export function CurveProgress({ curve, dammPool }: { curve: CurveStateDTO; dammPool: string | null }) {
  const steps = [
    { k: "Bonding curve", d: "Meteora DBC", on: true },
    { k: "Graduation", d: `${formatSol(curve.thresholdLamports, 2)} raised`, on: curve.curveComplete || curve.migrated },
    { k: "DAMM v2 pool", d: "Liquidity migrated, LP locked", on: curve.migrated },
  ];
  return (
    <section className="card space-y-4 p-5" aria-labelledby="curve-h" data-testid="curve-progress" data-migrated={curve.migrated}>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 id="curve-h" className="font-semibold">Curve progress</h2>
        {curve.migrated ? <span className="tag tag-ok"><i />Graduated to DAMM v2</span> : curve.curveComplete ? <span className="tag tag-pd"><i />Curve full · migrating</span> : <span className="tag tag-accent"><i />On the curve</span>}
      </div>
      <div>
        <div className="progress" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={curve.progressPct} aria-label="Curve progress">
          <span style={{ width: `${Math.max(curve.progressPct, 1)}%` }} />
        </div>
        <p className="text-muted mt-1 flex justify-between text-xs">
          <span data-testid="curve-raised">{curve.migrated ? "Curve complete" : `${formatSol(curve.quoteReserveLamports, 4)} of ${formatSol(curve.thresholdLamports, 2)} raised`}</span>
          <span>{curve.progressPct.toFixed(1)}%</span>
        </p>
      </div>
      <ol className="grid gap-2 sm:grid-cols-3">
        {steps.map((s, i) => (
          <li key={s.k} className={`rounded-panel border p-3 text-sm ${s.on ? "border-accent-border bg-accent-tint" : "border-border bg-surface-2"}`}>
            <p className="text-accent-text font-mono text-xs">{String(i + 1).padStart(2, "0")} {s.k}</p>
            <p className="text-muted text-xs">{s.d}</p>
          </li>
        ))}
      </ol>
      <dl className="text-xs">
        <div className="fact"><dt>DBC pool</dt><dd><a className="text-accent-text" href={explorerAddressUrl(curve.pool)} target="_blank" rel="noopener noreferrer">{shortAddr(curve.pool, 4)} ↗</a></dd></div>
        <div className="fact"><dt>Curve config</dt><dd><a className="text-accent-text" href={explorerAddressUrl(curve.config)} target="_blank" rel="noopener noreferrer">{shortAddr(curve.config, 4)} ↗</a></dd></div>
        {dammPool && <div className="fact"><dt>DAMM v2 pool</dt><dd><a className="text-accent-text" href={explorerAddressUrl(dammPool)} target="_blank" rel="noopener noreferrer" data-testid="damm-pool-link">{shortAddr(dammPool, 4)} ↗</a></dd></div>}
        <div className="fact"><dt>{curve.migrated ? "Curve final price" : "Spot price"}</dt><dd>{curve.priceSol > 0 ? fmtPrice(curve.priceSol) : "—"}</dd></div>
        <div className="fact"><dt>Curve trade fee</dt><dd>{curve.feeBps / 100}%</dd></div>
      </dl>
      <p className="text-dim text-xs">
        Read live from chain with the Meteora DBC SDK. When {formatSol(curve.thresholdLamports, 2)} is raised the curve closes and its liquidity migrates to a Meteora DAMM v2 pool, where trading continues.
      </p>
    </section>
  );
}
