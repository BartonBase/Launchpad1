/**
 * Token page, /t/{mint} (design/sitemap.md). Panels per launch type come from tokenPanels().
 * Live: Hybrid launches read from devnet. Plain and Burn are pending deploy, so they only exist as
 * design previews at /t/example-plain and /t/example-burn (Example data, controls disabled).
 * Panel state: ?panel=capture|release|reroll (old ?action= is accepted).
 */
import type { Metadata } from "next";
import Link from "next/link";
import { PublicKey } from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import { TOTAL_SUPPLY_WHOLE, tierFeeLamports } from "@/config/armory";
import { parsePublicKey } from "@/lib/validate";
import { cachedRead } from "@/lib/armory/server";
import { fetchLaunch, type LaunchDTO } from "@/lib/armory/reads";
import { shortAddr } from "@/lib/armory/format";
import { tokenPanels, type TokenType } from "@/lib/armory/tokenPanels";
import { HybridPanels, type HybridPanel } from "@/components/armory/HybridPanels";
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
type Search = Promise<{ panel?: string; action?: string }>;

const EXAMPLES: Record<string, TokenView> = {
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
const load = (mint: PublicKey) => cachedRead(`launch:${mint.toBase58()}`, 15_000, (c) => fetchLaunch(c, mint));

function hybridView(l: LaunchDTO): TokenView {
  return {
    type: "hybrid", phase: l.state, example: false, name: l.collectionName ?? `Token ${shortAddr(l.mint)}`, symbol: null, mint: l.mint,
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
  return { title: r.ok && r.value ? (r.value.collectionName ?? `Token ${shortAddr(r.value.mint)}`) : `Token ${shortAddr(mint.toBase58())}` };
}

function Shell({ children }: { children: React.ReactNode }) {
  return <section className="mx-auto max-w-3xl space-y-4 px-4 py-16">{children}</section>;
}

export default async function TokenPage({ params, searchParams }: { params: Params; searchParams: Search }) {
  const raw = (await params).mint;
  const sp = await searchParams;
  const p = sp.panel ?? sp.action;
  const panel: HybridPanel = p === "capture" || p === "release" || p === "reroll" ? p : null;

  let view: TokenView | null = EXAMPLES[raw] ?? null;
  let launch: LaunchDTO | null = null;
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
    const r = await load(mint);
    if (!r.ok) return <Shell><ReadError what="this token" error={r.error} /></Shell>;
    if (!r.value) {
      return (
        <Shell>
          <div data-testid="token-not-found" className="space-y-4">
            <h1 className="hd text-3xl">No Armory launch at this address</h1>
            <p className="text-muted">
              <span className="font-mono">{shortAddr(mint.toBase58(), 6)}</span> wasn&apos;t launched by Armory on {CLUSTER.label}. Plain and Burn launches are pending deploy.
            </p>
            <Link href="/explore" className="btn">Back to explore</Link>
          </div>
        </Shell>
      );
    }
    launch = r.value;
    view = hybridView(launch);
  }

  const panels = tokenPanels(view.type as TokenType, view.phase);
  return (
    <div className="mx-auto max-w-(--container-site) space-y-6 px-4 py-8" data-testid="token-page" data-type={view.type} data-phase={view.phase}>
      <nav aria-label="Breadcrumb" className="text-muted text-sm">
        <Link href="/explore" className="hover:text-fg">Explore</Link> <span aria-hidden="true">/</span> <span className="text-fg">{view.name}</span>
      </nav>
      <TokenHeader v={view} />
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_380px]">
        <div className="min-w-0 space-y-6">
          {panels.has("market") && <MarketCard v={view} />}
          <PhaseCard v={view} />
          {panels.has("plainFacts") && <PlainFacts />}
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
        <aside className="space-y-6">
          <TradePanel v={view} />
          <FactsCard v={view} />
        </aside>
      </div>
    </div>
  );
}
