import Link from "next/link";
import { Suspense } from "react";
import { StatsStrip, StatsStripFallback } from "@/components/armory/StatsStrip";
import { Knight } from "@/components/armory/Knight";
import { LaunchCard } from "@/components/armory/LaunchCard";
import { LaunchTypes } from "@/components/armory/LaunchTypes";
import { ReadError } from "@/components/armory/ReadError";
import { cachedRead } from "@/lib/armory/server";
import { fetchLaunches } from "@/lib/armory/reads";
import { mintDepositText } from "@/config/armory";
import { fetchPlainLaunches } from "@/lib/meteora/plain";
import { featuredRank, isHybridListed, isListed } from "@/lib/armory/listing";
import { PlainCard } from "@/components/meteora/PlainCard";
import { resolveTokenImages } from "@/lib/meteora/tokenImage";

const STEPS = [
  ["01", "Buy the token", "Every launch has a supply fixed at 1,000,000,000 tokens. When the curve fills, the token graduates and converting opens."],
  ["02", "Capture an NFT", "Lock exactly the ratio (say 1,000,000 tokens) and receive a random NFT from the collection, picked with Switchboard randomness that anyone can verify."],
  ["03", "Collect, show, list", "It is a Metaplex Core NFT in your wallet. Keep it, re-roll it for another random one, or trade it on NFT marketplaces."],
  ["04", "Switch back", "Release any NFT for exactly the ratio in tokens, whatever its rarity. Releasing has no platform fee."],
] as const;

export default async function HomePage() {
  const [launches, plains] = await Promise.all([cachedRead("launches", 30_000, fetchLaunches), cachedRead("plain-launches", 20_000, fetchPlainLaunches)]);
  const plainList = plains.ok ? plains.value.filter((p) => isListed(p.name, p.mint)) : [];
  const hybrids = launches.ok ? launches.value.filter(isHybridListed) : [];
  // Featured demo collections first, then the newest launches; six cards.
  const cards = [
    ...hybrids.map((l) => ({ kind: "hybrid" as const, mint: l.mint, uri: l.tokenUri ?? null, l })),
    ...plainList.map((p) => ({ kind: "plain" as const, mint: p.mint, uri: p.uri, p })),
  ]
    .map((c, i) => ({ c, i }))
    .sort((a, b) => featuredRank(a.c.mint) - featuredRank(b.c.mint) || (a.c.kind === b.c.kind ? a.i - b.i : a.c.kind === "plain" ? -1 : 1))
    .map(({ c }) => c)
    .slice(0, 6);
  const images = await resolveTokenImages(cards.map((c) => c.uri));
  return (
    <div className="mx-auto max-w-(--container-site) space-y-20 px-4 py-10 md:py-16">
      <section className="grid items-center gap-8 lg:grid-cols-[1.05fr_1fr] lg:gap-6">
        <div className="space-y-6">
          <span className="tag tag-accent">Coins and collections on Solana</span>
          <h1 className="hd text-[40px] leading-[1.02] sm:text-6xl lg:text-7xl">
            Trade the meme.
            <br />
            <span className="text-muted">Collect the art.</span>
          </h1>
          <p className="text-muted max-w-xl text-base">
            Launch a Solana meme coin on a bonding curve, on its own or with an NFT collection built in. With a{" "}
            <strong className="text-fg">Hybrid</strong> launch the coin is also an NFT: once it graduates, a fixed number
            of tokens converts into one NFT, and that NFT always converts back for exactly the same number of tokens.
          </p>
          <div className="flex flex-wrap gap-3">
            <Link href="/explore" className="btn btn-primary btn-lg" data-testid="cta-explore">
              Explore launches
            </Link>
            <Link href="/launch" className="btn btn-lg" data-testid="cta-launch">
              Launch a token
            </Link>
          </div>
        </div>
        <div className="relative lg:min-h-[700px]">
          <Knight
            k="b"
            glow
            priority
            decorative={false}
            sizes="(min-width: 1024px) 470px, (min-width: 640px) 380px, 280px"
            className="mx-auto w-[280px] sm:w-[380px] lg:mr-0 lg:ml-auto lg:w-[470px]"
          />
          <div className="card bg-glass shadow-float relative z-10 -mt-24 space-y-4 p-5 sm:-mt-32 lg:absolute lg:bottom-0 lg:-left-4 lg:mt-0 lg:w-[22rem]" aria-label="How converting works">
            <div className="flex items-center justify-between">
              <p className="eyebrow">Switch any time</p>
            </div>
            <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-3">
              <div className="bg-surface-2 rounded-panel border-border border p-4">
                <p className="text-muted text-xs">Tokens</p>
                <p className="num text-xl font-semibold">1,000,000</p>
              </div>
              <span aria-hidden="true" className="text-accent-text text-xl">⇄</span>
              <div className="bg-surface-2 rounded-panel border-border border p-4">
                <p className="text-muted text-xs">NFTs</p>
                <p className="num text-xl font-semibold">1 NFT</p>
              </div>
            </div>
            <p className="text-muted text-sm">
              Exact both ways, no tokens taken. Capturing costs a flat platform fee (0.002–0.01 SOL by ratio). {mintDepositText()}.
              Releasing back has no platform fee. Opens when a token graduates on its bonding curve.
            </p>
          </div>
        </div>
      </section>

      <section aria-label="Armory on devnet" className="-mt-8 md:-mt-10">
        <Suspense fallback={<StatsStripFallback />}>
          <StatsStrip />
        </Suspense>
      </section>

      <section className="space-y-6" aria-labelledby="how">
        <div className="space-y-2">
          <p className="eyebrow">How it works</p>
          <h2 id="how" className="hd text-3xl md:text-4xl">One asset, two forms</h2>
        </div>
        <ol className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {STEPS.map(([n, t, d]) => (
            <li key={n} className="card space-y-2 p-5">
              <p className="text-accent-text font-mono text-xs">{n}</p>
              <h3 className="font-semibold">{t}</h3>
              <p className="text-muted text-sm">{d}</p>
            </li>
          ))}
        </ol>
        <h3 className="pt-4 text-lg font-semibold">Launch styles</h3>
        <LaunchTypes />
      </section>

      <section className="card grid gap-6 p-6 md:grid-cols-[2fr_1fr] md:p-10" aria-labelledby="curve" data-testid="home-curve">
        <div className="space-y-3">
          <p className="eyebrow">Fair launch</p>
          <h2 id="curve" className="hd text-2xl md:text-3xl">Curve first, real pool after</h2>
          <p className="text-muted">
            Every launch starts on a <strong className="text-fg">bonding curve</strong>: a fixed 1B supply, mint authority revoked and a 1% trading fee. When the
            curve fills, the token <strong className="text-fg">graduates</strong> and its liquidity moves to a trading pool with the LP locked for good. Buy and
            sell right on the token page, with live quotes and slippage protection. Hybrid launches add the NFT converter on top, and it opens at graduation.
          </p>
        </div>
        <div className="flex flex-col items-start gap-2 md:items-end md:justify-center">
          <Link href="/launch?type=plain" className="btn btn-primary">Start a launch</Link>
          <Link href="/explore" className="text-accent-text text-sm">See what&apos;s trading →</Link>
        </div>
      </section>

      <section className="space-y-6" aria-labelledby="live">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div className="space-y-2">
            <p className="eyebrow">Live now</p>
            <h2 id="live" className="hd text-3xl md:text-4xl">Latest launches</h2>
          </div>
          <Link href="/explore" className="btn">View all</Link>
        </div>
        {launches.ok ? (
          cards.length > 0 ? (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {cards.map((c, i) => (c.kind === "plain" ? <PlainCard key={c.mint} p={c.p} image={images[i]} /> : <LaunchCard key={c.mint} l={c.l} image={images[i]} />))}
            </div>
          ) : (
            <p className="text-muted">No launches yet. Be the first.</p>
          )
        ) : (
          <ReadError what="launches" error={launches.error} />
        )}
      </section>
    </div>
  );
}
