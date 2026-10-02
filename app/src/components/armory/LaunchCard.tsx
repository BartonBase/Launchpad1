import Link from "next/link";
import type { LaunchDTO } from "@/lib/armory/reads";
import { compact, formatSol, shortAddr } from "@/lib/armory/format";
import { TypeIcon } from "./TypeIcon";

export function StateTag({ state }: { state: LaunchDTO["state"] }) {
  if (state === "graduated") return <span className="tag tag-ok"><i />Graduated</span>;
  if (state === "curve") return <span className="tag tag-accent"><i />On curve</span>;
  return <span className="tag" title="Launched without a bonding curve: converting can never open.">No curve</span>;
}

export function LaunchCard({ l, image }: { l: LaunchDTO; image?: string | null }) {
  const title = l.tokenName ?? l.collectionName ?? `Token ${shortAddr(l.mint)}`;
  const pct = l.collectionSize ? Math.round((l.mintedCount / l.collectionSize) * 100) : 0;
  return (
    <Link href={`/t/${l.mint}`} className="card flex flex-col gap-3 p-4 hover:border-[var(--arm-color-border-strong)]" data-testid="launch-card" data-type="hybrid" data-phase={l.state}>
      <div className="bg-surface-2 rounded-panel border-border relative flex aspect-[4/3] items-center justify-center overflow-hidden border" aria-hidden="true">
        {image ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={image} alt="" loading="lazy" className="size-full object-cover" data-testid="token-image" />
        ) : (
          <span className="hd text-dim text-4xl">{title.slice(0, 1).toUpperCase()}</span>
        )}
        <span className="absolute top-2 left-2"><StateTag state={l.state} /></span>
      </div>
      <div className="flex items-start justify-between gap-2">
        <h2 className="truncate font-semibold">{title} {l.tokenSymbol && <span className="text-muted font-mono text-xs font-normal">{l.tokenSymbol}</span>}</h2>
        <span className="text-dim font-mono text-xs">{shortAddr(l.mint)}</span>
      </div>
      <div>
        <div className="progress"><span style={{ width: `${l.state === "graduated" ? pct : 0}%` }} /></div>
        <p className="text-muted mt-1 flex justify-between text-xs">
          <span>{l.state === "graduated" ? "Converting open" : l.state === "curve" ? "Bonding curve" : "No curve"}</span>
          <span>{l.mintedCount} of {l.collectionSize} minted</span>
        </p>
      </div>
      <div className="border-border flex items-center justify-between border-t pt-3 text-xs">
        <span className="tchip"><TypeIcon type="hybrid" size={14} />Hybrid</span>
        <span className="text-muted">{compact(l.ratioWholeTokens)} = 1 NFT · fee {formatSol(l.feeLamports, 3)}</span>
      </div>
    </Link>
  );
}
