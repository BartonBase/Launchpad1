import { cachedRead } from "@/lib/armory/server";
import { fetchLaunches } from "@/lib/armory/reads";
import { fetchPlainLaunches } from "@/lib/meteora/plain";
import { isHybridListed, isListed } from "@/lib/armory/listing";
import { countCurveTrades } from "@/lib/armory/stats";

/** Home stats strip (server component, streamed in a Suspense boundary so the page never waits on it). */
export async function StatsStrip() {
  const [launches, plains] = await Promise.all([cachedRead("launches", 30_000, fetchLaunches), cachedRead("plain-launches", 20_000, fetchPlainLaunches)]);
  const hy = launches.ok ? launches.value.filter(isHybridListed) : null;
  const pl = plains.ok ? plains.value.filter((p) => isListed(p.name, p.mint)) : null;
  const pools = [...(hy ?? []).map((l) => l.dbcPool), ...(pl ?? []).map((p) => p.curve.pool)].filter((p): p is string => !!p);
  const trades = hy && pl ? await cachedRead(`curve-trades:${pools.length}`, 300_000, (c) => countCurveTrades(c, pools)) : null;
  const items: [string, string, string][] = [
    ["Tokens launched", hy && pl ? String(hy.length + pl.length) : "—", "Launch and Hybrid tokens listed on Armory"],
    ["Curve trades", trades?.ok ? String(trades.value) : "—", "Buys and sells on the bonding curves, read from chain"],
    ["NFTs captured", hy ? String(hy.reduce((n, l) => n + Number(l.totalCaptures), 0)) : "—", "Captures recorded by the Hybrid vaults"],
  ];
  return (
    <dl className="card grid grid-cols-3" data-testid="home-stats">
      {items.map(([k, v, t], i) => (
        <div key={k} className={`p-4 md:p-5 ${i ? "border-border border-l" : ""}`} title={t}>
          <dt className="text-muted text-xs">{k}</dt>
          <dd className="num text-xl font-semibold md:text-2xl" data-testid={`home-stat-${i}`}>{v}</dd>
        </div>
      ))}
    </dl>
  );
}

export function StatsStripFallback() {
  return (
    <dl className="card grid grid-cols-3" aria-busy="true">
      {["Tokens launched", "Curve trades", "NFTs captured"].map((k, i) => (
        <div key={k} className={`p-4 md:p-5 ${i ? "border-border border-l" : ""}`}>
          <dt className="text-muted text-xs">{k}</dt>
          <dd className="num text-dim text-xl font-semibold md:text-2xl">…</dd>
        </div>
      ))}
    </dl>
  );
}
