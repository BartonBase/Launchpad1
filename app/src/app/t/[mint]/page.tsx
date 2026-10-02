/**
 * Token page, /t/{mint} (design/sitemap.md). Panels per launch type come from tokenPanels().
 * Live: Hybrid launches (hybrid_launch LaunchConfig) and Plain launches (Meteora DBC pools on the
 * platform config), both read from devnet. Curve launches get the Meteora curve-progress card and a
 * real buy/sell panel (DBC before graduation, DAMM v2 after). Burn is "Coming soon", so it only
 * exists as a design preview at /t/example-burn (only with NEXT_PUBLIC_DESIGN_PREVIEWS=1).
 * Panel state: ?panel=capture|release|reroll (old ?action= is accepted).
 */
import { GraduationPanel } from "@/components/armory/GraduationPanel";
import type { Metadata } from "next";
import Link from "next/link";
import { PublicKey } from "@solana/web3.js";
import { TOTAL_SUPPLY_WHOLE, tierFeeLamports } from "@/config/armory";
import { parsePublicKey } from "@/lib/validate";
import { cachedRead } from "@/lib/armory/server";
import { fetchLaunch, type LaunchDTO } from "@/lib/armory/reads";
import { shortAddr } from "@/lib/armory/format";
import { tokenPanels, type TokenType } from "@/lib/armory/tokenPanels";
import { HybridPanels, type HybridPanel } from "@/components/armory/HybridPanels";
import { CurveProgress } from "@/components/meteora/CurveProgress";
import { SwapPanel } from "@/components/meteora/SwapPanel";
import { fetchCurveState, type CurveStateDTO } from "@/lib/meteora/dbc";
import { fetchDammPool, type DammPoolDTO } from "@/lib/meteora/damm";
import { fetchPlainLaunch, type PlainLaunchDTO } from "@/lib/meteora/plain";
import { resolveTokenImage } from "@/lib/meteora/tokenImage";
import { ReadError } from "@/components/armory/ReadError";
import {
  BurnHoldings,
  BurnPanels,
  FactsCard,
  FeesControl,
  HoldingsTokensOnly,
  MarketCard,
  NobodyCanChange,
  PhaseCard,
  PlainFacts,
  TokenHeader,
  TradeNfts,
  TradePanel,
  type TokenView,
} from "@/components/armory/TokenSections";

type Params = Promise<{ mint: string }>;
type Search = Promise<{ panel?: string; action?: string; fresh?: string }>;

/** Design previews (fake data). Off unless NEXT_PUBLIC_DESIGN_PREVIEWS=1, so the live site never shows them. */
const EXAMPLES: Record<string, TokenView> = process.env.NEXT_PUBLIC_DESIGN_PREVIEWS !== "1" ? {} : {
  "example-plain": {
    type: "plain", phase: "curve", example: true, name: "Salt Flats", symbol: "SALT", mint: null, collection: null, decimals: 6,
    ratioWhole: null, collectionSize: null, minted: 0, feeLamports: null, feeIsExactTier: true, feeRecipient: null,
    graduationLamports: 85_000_000_000n, mintAuthority: null, freezeAuthority: null, supplyWhole: TOTAL_SUPPLY_WHOLE,
  },
  "example-burn": {
    type: "burn", phase: "graduated", example: true, name: "Ferro", symbol: "FERRO", mint: null, collection: null, decimals: 6,
    ratioWhole: 1_000_000n, collectionSize: 500, minted: 312, feeLamports: tierFeeLamports(1_000_000), feeIsExactTier: true, feeRecipient: null,
    graduationLamports: 85_000_000_000n, mintAuthority: null, freezeAuthority: null, supplyWhole: TOTAL_SUPPLY_WHOLE - 312_000_000n,
  },
};

function parseMint(raw: string): PublicKey | null {
  try {
    const r = parsePublicKey(decodeURIComponent(raw));
    return r.ok ? r.value : null;
  } catch {
    return null;
  }
}
/** ttl 0 = read through (after the visitor's own transaction; see FRESH_WINDOW_MS). */
const load = (mint: PublicKey, ttl = 15_000) => cachedRead(`launch:${mint.toBase58()}`, ttl, (c) => fetchLaunch(c, mint));
const loadPlain = (mint: PublicKey, ttl = 10_000) => cachedRead(`plain:${mint.toBase58()}`, ttl, (c) => fetchPlainLaunch(c, mint));
const loadCurve = (pool: string, ttl = 10_000) => cachedRead(`curve:${pool}`, ttl, (c) => fetchCurveState(c, new PublicKey(pool)));
const loadDamm = (mint: string, decimals: number, ttl = 15_000) => cachedRead(`damm:${mint}`, ttl, (c) => fetchDammPool(c, new PublicKey(mint), decimals));
/** `?fresh=<ms timestamp>` (set by the transaction dialog after a confirmed tx) skips the cache briefly. */
const FRESH_WINDOW_MS = 60_000;
function freshTtl(v: string | undefined): 0 | undefined {
  const at = Number(v);
  return Number.isFinite(at) && Math.abs(Date.now() - at) < FRESH_WINDOW_MS ? 0 : undefined;
}

function plainView(p: PlainLaunchDTO): TokenView {
  return {
    type: "plain", phase: p.curve.migrated ? "graduated" : "curve", example: false, name: p.name, symbol: p.symbol || null, mint: p.mint,
    collection: null, decimals: p.curve.baseDecimals, ratioWhole: null, collectionSize: null, minted: 0, feeLamports: null, feeIsExactTier: true,
    feeRecipient: null, graduationLamports: BigInt(p.curve.thresholdLamports), mintAuthority: p.mintAuthority, freezeAuthority: p.freezeAuthority,
    supplyWhole: BigInt(p.supplyBase) / 10n ** BigInt(p.curve.baseDecimals),
  };
}

function hybridView(l: LaunchDTO): TokenView {
  return {
    type: "hybrid", phase: l.state, example: false, name: l.tokenName ?? l.collectionName ?? `Token ${shortAddr(l.mint)}`, symbol: l.tokenSymbol ?? null, mint: l.mint,
    collection: l.collection, decimals: l.decimals, ratioWhole: BigInt(l.ratioWholeTokens), collectionSize: l.collectionSize,
    minted: l.mintedCount, feeLamports: BigInt(l.feeLamports), feeIsExactTier: l.feeIsExactTier, feeRecipient: l.feeRecipient,
    graduationLamports: BigInt(l.graduationThresholdLamports), mintAuthority: l.mintAuthority, freezeAuthority: l.freezeAuthority,
    supplyWhole: BigInt(l.supplyBase) / 10n ** BigInt(l.decimals),
  };
}

export async function generateMetadata({ params }: { params: Params }): Promise<Metadata> {
  const raw = (await params).mint;
  if (EXAMPLES[raw]) return { title: `${EXAMPLES[raw].name} (example)` };
  const mint = parseMint(raw);
  if (!mint) return { title: "Invalid address" };
  const r = await load(mint);
  if (r.ok && r.value) return { title: r.value.tokenName ?? r.value.collectionName ?? `Token ${shortAddr(r.value.mint)}` };
  const pl = await loadPlain(mint);
  return { title: pl.ok && pl.value ? `${pl.value.name}${pl.value.symbol ? ` (${pl.value.symbol})` : ""}` : `Token ${shortAddr(mint.toBase58())}` };
}

function Shell({ children }: { children: React.ReactNode }) {
  return <section className="mx-auto max-w-3xl space-y-4 px-4 py-16">{children}</section>;
}

export default async function TokenPage({ params, searchParams }: { params: Params; searchParams: Search }) {
  const raw = (await params).mint;
  const sp = await searchParams;
  const fresh = freshTtl(sp.fresh);
  const p = sp.panel ?? sp.action;
  const panel: HybridPanel = p === "capture" || p === "release" || p === "reroll" ? p : null;

  let view: TokenView | null = EXAMPLES[raw] ?? null;
  let launch: LaunchDTO | null = null;
  let curve: CurveStateDTO | null = null;
  let plainUri: string | null = null;
  if (!view) {
    const mint = parseMint(raw);
    if (!mint) {
      return (
        <Shell>
          <h1 className="hd text-3xl">Invalid token address</h1>
          <p className="text-muted">That isn&apos;t a valid Solana address.</p>
          <Link href="/explore" className="btn">Back to explore</Link>
        </Shell>
      );
    }
    const r = await load(mint, fresh);
    if (!r.ok) return <Shell><ReadError what="this token" error={r.error} /></Shell>;
    let plain: PlainLaunchDTO | null = null;
    if (!r.value) {
      const pl = await loadPlain(mint, fresh);
      if (!pl.ok) return <Shell><ReadError what="this token" error={pl.error} /></Shell>;
      plain = pl.value;
    }
    if (!r.value && !plain) {
      return (
        <Shell>
          <div data-testid="token-not-found" className="space-y-4">
            <h1 className="hd text-3xl">No Armory launch at this address</h1>
            <p className="text-muted">
              <span className="font-mono">{shortAddr(mint.toBase58(), 6)}</span> wasn&apos;t launched on Armory. Check the address, or browse every launch on the explore page.
            </p>
            <Link href="/explore" className="btn">Back to explore</Link>
          </div>
        </Shell>
      );
    }
    if (r.value) {
      launch = r.value;
      view = hybridView(launch);
      if (launch.dbcPool) {
        const cr = await loadCurve(launch.dbcPool, fresh);
        curve = cr.ok ? cr.value : null;
      }
    } else if (plain) {
      view = plainView(plain);
      plainUri = plain.uri;
      curve = plain.curve;
    }
  }
  if (!view) return null;
  if (!view.example && view.image === undefined) {
    const uri = launch ? launch.tokenUri : view.type === "plain" ? plainUri : null;
    view = { ...view, image: await resolveTokenImage(uri) };
  }
  let damm: DammPoolDTO | null = null;
  if (curve?.migrated && view.mint) {
    const d = await loadDamm(view.mint, view.decimals, fresh);
    damm = d.ok ? d.value : null;
  }
  const priceSol = damm?.priceSol ?? (curve && !curve.migrated ? curve.priceSol : null);

  const panels = tokenPanels(view.type as TokenType, view.phase);
  return (
    <div className="mx-auto max-w-(--container-site) space-y-6 px-4 py-8" data-testid="token-page" data-type={view.type} data-phase={view.phase}>
      <nav aria-label="Breadcrumb" className="text-muted text-sm">
        <Link href="/explore" className="hover:text-fg">Explore</Link> <span aria-hidden="true">/</span> <span className="text-fg">{view.name}</span>
      </nav>
      <TokenHeader v={view} priceSol={priceSol} />
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_380px]">
        <div className="min-w-0 space-y-6">
          {panels.has("market") && <MarketCard v={view} priceSol={priceSol} />}
          {curve ? <CurveProgress curve={curve} dammPool={damm?.pool ?? null} /> : <PhaseCard v={view} />}
          {panels.has("plainFacts") && <PlainFacts />}
          {curve && curve.pool && (curve.curveComplete || curve.migrated) && (
            <GraduationPanel dbcPool={curve.pool} curveComplete={curve.curveComplete} migrated={curve.migrated} vault={launch?.vault ?? null} vaultOpen={launch?.vaultOpen ?? false} />
          )}
          {view.type === "hybrid" && launch && <HybridPanels launch={launch} initialPanel={panel} />}
          {view.type === "burn" && (
            <>
              <BurnPanels v={view} />
              <BurnHoldings />
            </>
          )}
          {view.type === "plain" && <HoldingsTokensOnly />}
          {panels.has("tradeNfts") && <TradeNfts />}
          <div className="grid gap-6 md:grid-cols-2">
            <FeesControl v={view} />
            <NobodyCanChange v={view} />
          </div>
        </div>
        {/* Phone: the trade panel comes right after the header (aside is display:contents below lg); facts stay last. */}
        <aside className="contents lg:block lg:space-y-6">
          <div className="order-first lg:order-none">
            {curve && !curve.migrated && view.mint ? (
              <SwapPanel venue="dbc" pool={curve.pool} mint={view.mint} symbol={view.symbol} decimals={view.decimals} feeBps={curve.feeBps} />
            ) : damm && view.mint ? (
              <SwapPanel venue="damm" pool={damm.pool} mint={view.mint} symbol={view.symbol} decimals={view.decimals} />
            ) : (
              <TradePanel v={view} />
            )}
          </div>
          <FactsCard v={view} />
        </aside>
      </div>
    </div>
  );
}
