import Link from "next/link";
import { BRAND } from "@/config/armory";
import { CLUSTER } from "@/config/cluster";

export function SiteFooter() {
  return (
    <footer className="border-border text-muted mt-16 border-t">
      <div className="mx-auto grid max-w-(--container-site) gap-6 px-4 py-10 text-sm md:grid-cols-[2fr_1fr_1fr]">
        <div className="space-y-2">
          <p className="wordmark text-fg">{BRAND.name}</p>
          <p className="max-w-md">
            Launch a Solana meme coin on its own, or with an NFT collection built in, on Meteora&apos;s Dynamic Bonding Curve.
            Unaudited demo on {CLUSTER.label}: test funds only, not financial advice.
          </p>
          <p className="text-dim font-mono text-xs">RPC {CLUSTER.rpcUrl}</p>
        </div>
        <nav aria-label="Product" className="flex flex-col gap-1.5">
          <Link href="/explore" className="hover:text-fg">Explore</Link>
          <Link href="/launch" className="hover:text-fg">Launch a token</Link>
          <Link href="/portfolio" className="hover:text-fg">Portfolio</Link>
          <Link href="/meteora" className="hover:text-fg">Built on Meteora</Link>
        </nav>
        <nav aria-label="Trust" className="flex flex-col gap-1.5">
          <Link href="/trust" className="hover:text-fg">Trust &amp; security</Link>
          <Link href="/faq" className="hover:text-fg">FAQ</Link>
          <Link href="/bug-bounty" className="hover:text-fg">Bug bounty</Link>
        </nav>
      </div>
    </footer>
  );
}
