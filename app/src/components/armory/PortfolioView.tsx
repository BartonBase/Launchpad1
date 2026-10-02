"use client";
import Link from "next/link";
import { PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { useChainRead } from "@/hooks/useChain";
import { fetchHoldings, fetchLaunches, fetchUserRequests } from "@/lib/armory/reads";
import { formatSol, formatTokens, shortAddr } from "@/lib/armory/format";
import { StateTag } from "./LaunchCard";
import { TypeIcon } from "./TypeIcon";
import { EmptyState } from "./EmptyState";
import { Knight } from "./Knight";

/** Portfolio per design/system portfolio.html (lean): summary, positions, NFTs, history. */
export function PortfolioView() {
  const { publicKey } = useWallet();
  const user = publicKey?.toBase58() ?? null;
  const data = useChainRead(user && `portfolio:${user}`, async (c) => {
    const owner = new PublicKey(user!);
    const [launches, sol, requests] = await Promise.all([fetchLaunches(c), c.getBalance(owner), fetchUserRequests(c, owner)]);
    const hs = await Promise.all(launches.map((l) => fetchHoldings(c, owner, l)));
    const rows = launches.map((l, i) => ({ l, h: hs[i]! })).filter(({ h }) => h.tokenBase > 0n || h.nftIndexes.length > 0);
    return { rows, sol: BigInt(sol), requests };
  });

  if (!user)
    return (
      <EmptyState k="a" title="Connect a wallet" testId="portfolio-connect">
        <p>Connect a wallet to see your tokens, NFTs and pending requests.</p>
      </EmptyState>
    );
  if (data.loading) return <p className="text-muted" role="status">Loading from chain…</p>;
  if (data.error) return <p role="alert" className="text-warning">Couldn&apos;t load your portfolio: {data.error}</p>;
  const d = data.data!;
  const nftCount = d.rows.reduce((n, r) => n + r.h.nftIndexes.length, 0);
  const title = (l: (typeof d.rows)[number]["l"]) => l.collectionName ?? `Token ${shortAddr(l.mint)}`;
  return (
    <div className="space-y-6" data-testid="portfolio">
      <dl className="card grid grid-cols-2 md:grid-cols-4" data-testid="portfolio-summary">
        <div className="p-4"><dt className="text-muted text-xs">SOL · {shortAddr(user, 4)}</dt><dd className="num text-xl font-semibold" data-testid="portfolio-sol">{formatSol(d.sol, 4)}</dd></div>
        <div className="border-border p-4 md:border-l"><dt className="text-muted text-xs">Tokens held</dt><dd className="num text-xl font-semibold">{d.rows.filter((r) => r.h.tokenBase > 0n).length}</dd></div>
        <div className="border-border p-4 md:border-l"><dt className="text-muted text-xs">Hybrid NFTs</dt><dd className="num text-xl font-semibold">{nftCount}</dd></div>
        <div className="border-border p-4 md:border-l"><dt className="text-muted text-xs">Burn NFTs</dt><dd className="text-xl font-semibold"><span className="tag tag-soon">Coming soon</span></dd></div>
      </dl>
      <section className="card overflow-x-auto" aria-labelledby="pos-h">
        <h2 id="pos-h" className="border-border border-b p-4 font-semibold">Positions</h2>
        {d.rows.length === 0 ? (
          <div className="flex items-end gap-4 px-4 pt-2" data-testid="portfolio-empty">
            <Knight k="a" sizes="96px" className="w-[84px] shrink-0 sm:w-[96px]" />
            <p className="text-muted pb-5">No Armory tokens or NFTs in this wallet yet. <Link href="/explore" className="text-accent-text">Explore launches</Link>.</p>
          </div>
        ) : (
          <table className="w-full text-sm" data-testid="positions">
            <thead className="text-muted text-left text-xs">
              <tr><th className="p-3 font-medium">Token</th><th className="p-3 font-medium">Type</th><th className="p-3 font-medium">Phase</th><th className="p-3 text-right font-medium">Tokens</th><th className="p-3 text-right font-medium">NFTs</th><th className="p-3 text-right font-medium">Value</th><th className="p-3" /></tr>
            </thead>
            <tbody className="divide-border divide-y">
              {d.rows.map(({ l, h }) => {
                const convertible = l.vaultOpen ? h.tokenBase / BigInt(l.ratioBase) : 0n;
                return (
                  <tr key={l.mint} data-testid="portfolio-row">
                    <td className="p-3"><Link href={`/t/${l.mint}`} className="font-medium hover:underline">{title(l)}</Link></td>
                    <td className="p-3"><span className="tchip"><TypeIcon type="hybrid" size={14} />Hybrid</span></td>
                    <td className="p-3"><StateTag state={l.state} /></td>
                    <td className="num p-3 text-right">{formatTokens(h.tokenBase, l.decimals)}</td>
                    <td className="num p-3 text-right">{h.nftIndexes.length}</td>
                    <td className="text-muted p-3 text-right" title="Price not available yet">—</td>
                    <td className="p-3 text-right whitespace-nowrap">
                      {convertible > 0n && <Link href={`/t/${l.mint}?panel=capture`} className="btn btn-sm btn-primary">Convert {convertible.toString()}</Link>}{" "}
                      {h.nftIndexes.length > 0 && <Link href={`/t/${l.mint}?panel=release`} className="btn btn-sm">Release</Link>}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </section>
      {nftCount > 0 && (
        <section className="card p-4" aria-labelledby="nfts-h" data-testid="portfolio-nfts">
          <h2 id="nfts-h" className="mb-2 font-semibold">NFTs by collection</h2>
          <ul className="space-y-1 text-sm">
            {d.rows.filter((r) => r.h.nftIndexes.length > 0).map(({ l, h }) => (
              <li key={l.mint} className="fact"><span>{title(l)}</span><span className="num">{h.nftIndexes.map((i) => `#${i}`).join(", ")}</span></li>
            ))}
          </ul>
        </section>
      )}
      {d.requests.length > 0 && (
        <div className="card p-4" data-testid="portfolio-requests">
          <h2 className="mb-2 font-semibold">Pending requests</h2>
          <ul className="divide-border divide-y text-sm">
            {d.requests.map((r) => (
              <li key={r.address} className="flex justify-between py-2">
                <span>#{r.seq} · {r.kind}</span>
                <span className="text-muted">{r.revealed ? "Switchboard revealed, waiting for settle" : "Waiting for Switchboard randomness"} · deposit {formatSol(r.mintEscrowLamports, 4)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      <section className="card card-soon p-4" data-testid="portfolio-history">
        <h2 className="font-semibold">History</h2>
        <p className="text-sm">Captures, releases and trades will be listed here soon. For now, see your wallet&apos;s history on the explorer.</p>
      </section>
    </div>
  );
}
