import type { Metadata } from "next";
import Link from "next/link";
import { CLUSTER } from "@/config/cluster";
import { LAUNCH_TYPES, launchTypeStatus, type LaunchTypeId } from "@/config/armory";
import { LaunchCard } from "@/components/armory/LaunchCard";
import { ReadError } from "@/components/armory/ReadError";
import { TypeIcon } from "@/components/armory/TypeIcon";
import { cachedRead } from "@/lib/armory/server";
import { fetchLaunches } from "@/lib/armory/reads";
import { fetchPlainLaunches } from "@/lib/meteora/plain";
import { isListed } from "@/lib/armory/listing";
import { PlainCard } from "@/components/meteora/PlainCard";
import { EmptyState } from "@/components/armory/EmptyState";

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
  const NEAR_PCT = 50;
  const phase = PHASES.find((p) => p.id === sp.phase)?.id ?? "all";
  const q = (sp.q ?? "").trim().slice(0, 64).toLowerCase();
  const cur: Sp = { type, phase, q };
  const [launches, plains] = await Promise.all([cachedRead("launches", 30_000, fetchLaunches), cachedRead("plain-launches", 20_000, fetchPlainLaunches)]);
  const all = launches.ok ? launches.value.filter((l) => isListed(l.collectionName)) : [];
  const allPlain = plains.ok ? plains.value.filter((p) => isListed(p.name)) : [];
  const list = all.filter(
    (l) =>
      (type === "all" || type === "hybrid") &&
      (phase === "all" || (phase === "near" ? false : l.state === phase)) &&
      (!q || l.mint.toLowerCase().includes(q) || (l.collectionName ?? "").toLowerCase().includes(q)),
  );
  const plainList = allPlain.filter(
    (p) =>
      (type === "all" || type === "plain") &&
      (phase === "all" || (phase === "graduated" ? p.curve.migrated : phase === "curve" ? !p.curve.migrated : !p.curve.migrated && p.curve.progressPct >= NEAR_PCT)) &&
      (!q || p.mint.toLowerCase().includes(q) || p.name.toLowerCase().includes(q) || p.symbol.toLowerCase().includes(q)),
  );
  const pending = (t: LaunchTypeId) => launchTypeStatus(t, CLUSTER.name) !== "live";
  const total = all.length + allPlain.length;
  return (
    <div className="mx-auto max-w-(--container-site) space-y-6 px-4 py-10">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="max-w-2xl space-y-2">
          <h1 className="hd text-4xl md:text-5xl">Explore launches</h1>
          <p className="text-muted">Every token launched on Armory, with its type and phase. A Launch is just the coin on a bonding curve; a Hybrid opens its NFT collection at graduation.</p>
        </div>
        <form action="/explore" role="search" className="w-full sm:w-72">
          {type !== "all" && <input type="hidden" name="type" value={type} />}
          {phase !== "all" && <input type="hidden" name="phase" value={phase} />}
          <input name="q" defaultValue={sp.q ?? ""} maxLength={64} className="input" placeholder="Search by name or address" aria-label="Search launches" data-testid="explore-search" />
        </form>
      </div>
      <dl className="card grid grid-cols-2 sm:grid-cols-4" data-testid="explore-stats">
        {[
          ["Launches", launches.ok ? String(total) : "—"],
          ["On the curve", launches.ok ? String(all.filter((l) => l.state === "curve").length + allPlain.filter((p) => !p.curve.migrated).length) : "—"],
          ["Graduated", launches.ok ? String(all.filter((l) => l.state === "graduated").length + allPlain.filter((p) => p.curve.migrated).length) : "—"],
          ["SOL on curves", plains.ok ? `${(allPlain.reduce((s, p) => s + Number(p.curve.migrated ? 0 : p.curve.quoteReserveLamports), 0) / 1e9).toFixed(3)}` : "—"],
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
          {(["plain", "hybrid"] as const).map((t) => (
            <Link key={t} href={href(cur, { type: t })} aria-current={type === t ? "page" : undefined} data-testid={`type-filter-${t}`} className={`rounded-chip inline-flex items-center gap-1.5 border px-3 py-1.5 text-sm ${type === t ? "border-accent-border bg-accent-tint text-fg" : "border-border text-muted hover:text-fg"}`}>
              <TypeIcon type={t} size={14} />{LAUNCH_TYPES.find((x) => x.id === t)!.name}
              {pending(t) && <span className="tag tag-demo">Pending</span>}
            </Link>
          ))}
          {(["burn", "tax", "raffle"] as const).map((t) => (
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
      ) : type === "burn" ? (
        <div className="card-soon rounded-panel space-y-2 border p-6" data-testid="explore-pending-type">
          <p className="font-medium">Burn launches <span className="tag tag-soon ml-1">Coming soon</span></p>
          <p className="text-sm">Burn launches open soon. <Link href="/launch" className="text-accent-text">Start a launch</Link> in the meantime.</p>
        </div>
      ) : list.length + plainList.length === 0 ? (
        <EmptyState k="a" title={q ? "No launches match your search" : "Nothing here yet"} testId="explore-empty">
          <p>{phase === "near" ? `Nothing is near graduation (${NEAR_PCT}%+ of its curve target) right now.` : q ? "Try another name, symbol or token address." : "Nothing matches."}</p>
          {type === "plain" && <p><Link href="/launch?type=plain" className="text-accent-text">Start a launch</Link>: one transaction, and your coin is trading.</p>}
        </EmptyState>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3" data-testid="explore-grid">
          {plainList.map((p) => <PlainCard key={p.mint} p={p} />)}
          {list.map((l) => <LaunchCard key={l.mint} l={l} />)}
        </div>
      )}
      {!plains.ok && <p className="text-warning text-sm">Some launches couldn&apos;t be loaded right now. Refresh to try again.</p>}
      <section className="space-y-3" aria-labelledby="soon-h">
        <h2 id="soon-h" className="font-semibold">Coming soon</h2>
        <div className="grid gap-3 sm:grid-cols-3">
          {LAUNCH_TYPES.filter((t) => t.id === "burn" || t.id === "tax" || t.id === "raffle").map((t) => (
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
