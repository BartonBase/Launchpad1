import type { Metadata } from "next";
import Link from "next/link";
import { CLUSTER } from "@/config/cluster";
import { LAUNCH_TYPES, STATUS_LABEL, launchTypeStatus, type LaunchTypeId } from "@/config/armory";
import { LaunchCard } from "@/components/armory/LaunchCard";
import { ReadError } from "@/components/armory/ReadError";
import { TypeIcon } from "@/components/armory/TypeIcon";
import { cachedRead } from "@/lib/armory/server";
import { fetchLaunches } from "@/lib/armory/reads";

export const metadata: Metadata = { title: "Explore", description: "Every Armory launch with its type, phase and market." };

const PHASES = [
  { id: "all", label: "All" },
  { id: "curve", label: "On curve" },
  { id: "near", label: "Near graduation" },
  { id: "graduated", label: "Graduated" },
] as const;
type Sp = { type?: string; phase?: string; q?: string };

function href(cur: Sp, patch: Sp): string {
  const n = { ...cur, ...patch };
  const qs = new URLSearchParams(Object.entries(n).filter(([, v]) => v && v !== "all") as [string, string][]).toString();
  return qs ? `/explore?${qs}` : "/explore";
}

export default async function ExplorePage({ searchParams }: { searchParams: Promise<Sp> }) {
  const sp = await searchParams;
  const type = (["plain", "hybrid", "burn"] as const).find((t) => t === sp.type) ?? "all";
  const phase = PHASES.find((p) => p.id === sp.phase)?.id ?? "all";
  const q = (sp.q ?? "").trim().slice(0, 64).toLowerCase();
  const cur: Sp = { type, phase, q };
  const launches = await cachedRead("launches", 30_000, fetchLaunches);
  const all = launches.ok ? launches.value : [];
  const list = all.filter(
    (l) =>
      (type === "all" || type === "hybrid") &&
      (phase === "all" || (phase === "near" ? false : l.state === phase)) &&
      (!q || l.mint.toLowerCase().includes(q) || (l.collectionName ?? "").toLowerCase().includes(q)),
  );
  const pending = (t: LaunchTypeId) => launchTypeStatus(t, CLUSTER.name) !== "live";
  return (
    <div className="mx-auto max-w-(--container-site) space-y-6 px-4 py-10">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="max-w-2xl space-y-2">
          <h1 className="hd text-4xl md:text-5xl">Explore launches</h1>
          <p className="text-muted">Every token launched on Armory, with its type and phase. Plain tokens have no NFTs; Hybrid and Burn tokens open their collection at graduation.</p>
        </div>
        <form action="/explore" role="search" className="w-full sm:w-72">
          {type !== "all" && <input type="hidden" name="type" value={type} />}
          {phase !== "all" && <input type="hidden" name="phase" value={phase} />}
          <input name="q" defaultValue={sp.q ?? ""} maxLength={64} className="input" placeholder="Search by name or address" aria-label="Search launches" data-testid="explore-search" />
        </form>
      </div>
      <dl className="card grid grid-cols-2 sm:grid-cols-4" data-testid="explore-stats">
        {[
          ["Launches", launches.ok ? String(all.length) : "—"],
          ["On the curve", launches.ok ? String(all.filter((l) => l.state === "curve").length) : "—"],
          ["Graduated", launches.ok ? String(all.filter((l) => l.state === "graduated").length) : "—"],
          ["24h volume", "—"],
        ].map(([k, v], i) => (
          <div key={k} className={`p-4 ${i ? "border-border sm:border-l" : ""}`}>
            <dt className="text-muted text-xs">{k}</dt>
            <dd className="num text-xl font-semibold">{v}</dd>
          </div>
        ))}
      </dl>
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <nav aria-label="Filter by type" className="flex flex-wrap items-center gap-1.5" data-testid="type-filter">
          <span className="eyebrow mr-1">Type</span>
          <Link href={href(cur, { type: "all" })} aria-current={type === "all" ? "page" : undefined} className={`rounded-chip border px-3 py-1.5 text-sm ${type === "all" ? "border-accent-border bg-accent-tint text-fg" : "border-border text-muted hover:text-fg"}`}>All types</Link>
          {(["plain", "hybrid", "burn"] as const).map((t) => (
            <Link key={t} href={href(cur, { type: t })} aria-current={type === t ? "page" : undefined} data-testid={`type-filter-${t}`} className={`rounded-chip inline-flex items-center gap-1.5 border px-3 py-1.5 text-sm ${type === t ? "border-accent-border bg-accent-tint text-fg" : "border-border text-muted hover:text-fg"}`}>
              <TypeIcon type={t} size={14} />{LAUNCH_TYPES.find((x) => x.id === t)!.name}
              {pending(t) && <span className="tag tag-demo">Pending</span>}
            </Link>
          ))}
          {(["tax", "raffle"] as const).map((t) => (
            <span key={t} aria-disabled="true" data-testid={`type-filter-${t}`} className="rounded-chip text-dim inline-flex cursor-not-allowed items-center gap-1.5 border border-dashed border-[var(--arm-color-border-strong)] px-3 py-1.5 text-sm">
              <TypeIcon type={t} size={14} />{LAUNCH_TYPES.find((x) => x.id === t)!.name}<span className="tag tag-soon">Soon</span>
            </span>
          ))}
        </nav>
        <nav aria-label="Filter by phase" className="flex flex-wrap items-center gap-1.5" data-testid="phase-filter">
          <span className="eyebrow mr-1">Phase</span>
          {PHASES.map((p) => (
            <Link key={p.id} href={href(cur, { phase: p.id })} aria-current={phase === p.id ? "page" : undefined} className={`rounded-chip border px-3 py-1.5 text-sm ${phase === p.id ? "border-accent-border bg-accent-tint text-fg" : "border-border text-muted hover:text-fg"}`}>{p.label}</Link>
          ))}
        </nav>
        <span className="text-muted ml-auto text-sm">Sort: Newest</span>
      </div>
      {!launches.ok ? (
        <ReadError what="launches" error={launches.error} />
      ) : (type === "plain" || type === "burn") ? (
        <div className="card-soon rounded-panel space-y-2 border p-6" data-testid="explore-pending-type">
          <p className="font-medium">{type === "plain" ? "Plain" : "Burn"} launches: {STATUS_LABEL["pending-deploy"].toLowerCase()} on {CLUSTER.label}</p>
          <p className="text-sm">None can exist yet. <Link href={`/t/example-${type}`} className="text-accent-text">See an example token page</Link> or <Link href={`/launch?type=${type}`} className="text-accent-text">walk through the wizard</Link>.</p>
        </div>
      ) : list.length === 0 ? (
        <p className="text-muted" data-testid="explore-empty">{phase === "near" ? "Near graduation needs live curve progress (Meteora DBC), which isn't wired yet." : "Nothing matches."}</p>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3" data-testid="explore-grid">
          {list.map((l) => <LaunchCard key={l.mint} l={l} />)}
        </div>
      )}
      <section className="space-y-3" aria-labelledby="soon-h">
        <h2 id="soon-h" className="font-semibold">Coming soon</h2>
        <div className="grid gap-3 sm:grid-cols-2">
          {LAUNCH_TYPES.filter((t) => t.id === "tax" || t.id === "raffle").map((t) => (
            <div key={t.id} className="card card-soon flex gap-3 p-4" data-testid={`soon-card-${t.id}`} aria-disabled="true">
              <TypeIcon type={t.id} />
              <div>
                <p className="font-medium">{t.name} <span className="tag tag-soon ml-1">Coming soon</span></p>
                <p className="text-sm">{t.description}</p>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
