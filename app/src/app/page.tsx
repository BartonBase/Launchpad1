import Link from "next/link";
import { CLUSTER } from "@/config/cluster";
import { MascotPlaceholder } from "@/components/armory/MascotPlaceholder";
import { LaunchCard } from "@/components/armory/LaunchCard";
import { LaunchTypes } from "@/components/armory/LaunchTypes";
import { ReadError } from "@/components/armory/ReadError";
import { cachedRead } from "@/lib/armory/server";
import { fetchLaunches } from "@/lib/armory/reads";
import { mintDepositText } from "@/config/armory";
import { fetchPlainLaunches } from "@/lib/meteora/plain";
import { PlainCard } from "@/components/meteora/PlainCard";

const STEPS = [
  ["01", "Buy the token", "Every launch starts as a fixed 1,000,000,000 token supply. When the curve fills, the token graduates and converting opens."],
  ["02", "Capture an NFT", "Lock exactly the ratio (say 1,000,000 tokens) and receive a random NFT from the collection, picked with Switchboard randomness that anyone can verify."],
  ["03", "Collect, show, list", "It is a Metaplex Core NFT in your wallet. Keep it, re-roll it for another random one, or trade it on NFT marketplaces."],
  ["04", "Switch back", "Release any NFT for exactly the ratio in tokens, whatever its rarity. Releasing has no platform fee."],
] as const;

export default async function HomePage() {
  const [launches, plains] = await Promise.all([cachedRead("launches", 30_000, fetchLaunches), cachedRead("plain-launches", 20_000, fetchPlainLaunches)]);
  const plainList = plains.ok ? plains.value.slice(0, 3) : [];
  return (
    <div className="mx-auto max-w-(--container-site) space-y-20 px-4 py-10 md:py-16">
      <section className="grid items-center gap-10 lg:grid-cols-[1.1fr_1fr]">
        <div className="space-y-6">
          <span className="tag tag-accent">Unaudited demo · {CLUSTER.label} · Built on Meteora</span>
          <div className="flex items-start justify-between gap-3 md:gap-6">
            <h1 className="hd text-[34px] sm:text-6xl lg:text-7xl">
              Trade the meme.
              <br />
              <span className="text-muted">Collect the art.</span>
            </h1>
            <MascotPlaceholder className="w-[76px]! md:w-[168px]!" size={168} />
          </div>
          <p className="text-muted max-w-xl text-base">
            Launch a Solana meme coin on Meteora&apos;s Dynamic Bonding Curve, on its own or with an NFT collection built in. With a{" "}
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
        <div className="card bg-glass shadow-float space-y-4 p-5" aria-label="How converting works">
          <div className="flex items-center justify-between">
            <p className="eyebrow">Switch any time</p>
            <span className="tag tag-demo">Example</span>
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

      <section className="card grid gap-6 p-6 md:grid-cols-[2fr_1fr] md:p-10" aria-labelledby="meteora" data-testid="home-meteora">
        <div className="space-y-3">
          <p className="eyebrow">Built on Meteora</p>
          <h2 id="meteora" className="hd text-2xl md:text-3xl">Curve first, real pool after</h2>
          <p className="text-muted">
            Every launch with a market starts on a Meteora <strong className="text-fg">Dynamic Bonding Curve</strong> with Armory&apos;s own config (fixed 1B supply,
            mint authority revoked, 1% curve fee). When the curve fills, liquidity migrates to a <strong className="text-fg">Meteora DAMM v2</strong> pool with the LP
            locked. Buy and sell right on the token page, with live quotes and slippage protection. Hybrid launches add the NFT converter on top, and it only opens after
            the move to DAMM v2.
          </p>
        </div>
        <div className="flex flex-col items-start gap-2 md:items-end md:justify-center">
          <Link href="/meteora" className="btn btn-primary">How Armory uses Meteora</Link>
          <Link href="/launch?type=plain" className="text-accent-text text-sm">Launch a Plain token →</Link>
        </div>
      </section>

      <section className="space-y-6" aria-labelledby="live">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div className="space-y-2">
            <p className="eyebrow">Live on {CLUSTER.label}</p>
            <h2 id="live" className="hd text-3xl md:text-4xl">Latest launches</h2>
          </div>
          <Link href="/explore" className="btn">View all</Link>
        </div>
        {launches.ok ? (
          launches.value.length + plainList.length > 0 ? (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {plainList.map((p) => (
                <PlainCard key={p.mint} p={p} />
              ))}
              {launches.value.slice(0, 6 - plainList.length).map((l) => (
                <LaunchCard key={l.mint} l={l} />
              ))}
            </div>
          ) : (
            <p className="text-muted">No launches on {CLUSTER.label} yet.</p>
          )
        ) : (
          <ReadError what="launches" error={launches.error} />
        )}
      </section>

      <section className="card grid gap-6 p-6 md:grid-cols-[2fr_1fr] md:p-10" aria-labelledby="trust">
        <div className="space-y-3">
          <h2 id="trust" className="hd text-2xl md:text-3xl">Built to be checked, not trusted</h2>
          <p className="text-muted">
            Supply is fixed at 1,000,000,000 and mint authority is revoked at launch. The conversion rate and fee are
            written once and can&apos;t be changed by any setting. Every transaction is simulated and previewed before
            you sign. The programs are <strong className="text-fg">not audited yet</strong>: this is a devnet demo with test funds only.
          </p>
        </div>
        <div className="flex flex-col items-start gap-2 md:items-end md:justify-center">
          <Link href="/trust" className="btn btn-primary">Trust &amp; security</Link>
          <Link href="/bug-bounty" className="text-accent-text text-sm">Bug bounty →</Link>
        </div>
      </section>
    </div>
  );
}
