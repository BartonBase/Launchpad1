"use client";
/**
 * Fee claim UI for Launch-type tokens (src/lib/meteora/fees.ts). Every claim goes through useSafeSend: program
 * allowlist -> simulation -> preview -> explicit Confirm in the wallet. Nothing is signed from this file.
 *  - CreatorFeesPanel (Portfolio): curve fees + DAMM v2 LP fees for coins the connected wallet launched.
 *  - AdminFeesView (/admin/fees, unlinked): Armory's partner fees on every pool of the platform config. Readable by
 *    anyone; the claim buttons only enable for the config's fee wallet (on mainnet it must also be Barton's wallet).
 */
import Link from "next/link";
import { PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { useChainRead } from "@/hooks/useChain";
import { useSafeSend, type BuildFn } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";
import { formatSol, shortAddr } from "@/lib/armory/format";
import { CLUSTER, explorerAddressUrl } from "@/config/cluster";
import { MAINNET_FEE_CLAIMER } from "@/config/launchTerms";
import { fetchPlatformTerms } from "@/lib/meteora/dbc";
import {
  buildClaimCreatorFeeTx,
  buildClaimLpFeeTx,
  buildClaimPartnerFeeTx,
  buildPartnerSurplusTx,
  fetchCreatorCurveFees,
  fetchLpFees,
  fetchPartnerCurveFees,
  totalLamports,
  type CurveFeeRow,
  type LpFeeRow,
} from "@/lib/meteora/fees";

/** Read-key suffix that changes after each confirmed claim, so the numbers refetch. */
const claimEpoch = (s: { status: string; signature: string | null }) => (s.status === "success" ? (s.signature ?? "done") : "");

function FeeTable({ rows, lp, canClaim, onClaim, onClaimLp, onSurplus, testId }: {
  rows: readonly CurveFeeRow[];
  lp: readonly LpFeeRow[];
  canClaim: boolean;
  onClaim: (r: CurveFeeRow) => void;
  onClaimLp: (r: LpFeeRow) => void;
  onSurplus?: (r: CurveFeeRow) => void;
  testId: string;
}) {
  return (
    <table className="w-full text-sm" data-testid={testId}>
      <thead className="text-muted text-left text-xs">
        <tr><th className="p-3 font-medium">Token</th><th className="p-3 font-medium">Where</th><th className="p-3 text-right font-medium">Unclaimed</th><th className="p-3" /></tr>
      </thead>
      <tbody className="divide-border divide-y">
        {rows.map((r) => (
          <tr key={r.pool} data-testid={`${testId}-row`}>
            <td className="p-3"><Link href={`/t/${r.mint}`} className="font-mono text-xs hover:underline">{shortAddr(r.mint, 4)}</Link></td>
            <td className="text-muted p-3 text-xs">Bonding curve{r.migrated ? " (graduated)" : ""}</td>
            <td className="num p-3 text-right">{formatSol(r.unclaimedLamports, 4)}</td>
            <td className="p-3 text-right whitespace-nowrap">
              <button type="button" className="btn btn-sm btn-primary" disabled={!canClaim || r.unclaimedLamports === 0n} onClick={() => onClaim(r)} data-testid={`${testId}-claim`}>Claim</button>
              {onSurplus && r.surplusPending && (
                <button type="button" className="btn btn-sm ml-2" disabled={!canClaim} onClick={() => onSurplus(r)} data-testid={`${testId}-surplus`}>Withdraw surplus</button>
              )}
            </td>
          </tr>
        ))}
        {lp.map((r) => (
          <tr key={r.position} data-testid={`${testId}-lp-row`}>
            <td className="p-3"><Link href={`/t/${r.mint}`} className="font-mono text-xs hover:underline">{shortAddr(r.mint, 4)}</Link></td>
            <td className="text-muted p-3 text-xs">Trading pool (LP position {shortAddr(r.position, 4)})</td>
            <td className="num p-3 text-right">{formatSol(r.unclaimedLamports, 4)}</td>
            <td className="p-3 text-right">
              <button type="button" className="btn btn-sm btn-primary" disabled={!canClaim || (r.unclaimedLamports === 0n && r.unclaimedBase === 0n)} onClick={() => onClaimLp(r)} data-testid={`${testId}-lp-claim`}>Claim</button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** Portfolio: fees the connected wallet earned as a creator. */
export function CreatorFeesPanel() {
  const { publicKey } = useWallet();
  const safeSend = useSafeSend();
  const user = publicKey?.toBase58() ?? null;
  const data = useChainRead(
    user && `creator-fees:${user}:${claimEpoch(safeSend)}`,
    async (c) => {
      const owner = new PublicKey(user!);
      const rows = await fetchCreatorCurveFees(c, owner);
      const lp = await fetchLpFees(c, owner, new Set(rows.filter((r) => r.migrated).map((r) => r.mint)));
      return { rows, lp };
    },
  );
  if (!user) return null;
  const send = (b: BuildFn) => void safeSend.start(b);
  const d = data.data;
  const total = d ? totalLamports(d.rows) + totalLamports(d.lp) : 0n;
  return (
    <section className="card overflow-x-auto" aria-labelledby="cf-h" data-testid="creator-fees">
      <div className="border-border flex items-center justify-between border-b p-4">
        <h2 id="cf-h" className="font-semibold">Creator fees</h2>
        {d && d.rows.length + d.lp.length > 0 && <span className="num text-sm" data-testid="creator-fees-total">{formatSol(total, 4)} unclaimed</span>}
      </div>
      {data.loading && <p className="text-muted p-4 text-sm" role="status">Loading from chain…</p>}
      {data.error && <p role="alert" className="text-warning p-4 text-sm">Couldn&apos;t load creator fees: {data.error}</p>}
      {d && d.rows.length + d.lp.length === 0 && (
        <p className="text-muted p-4 text-sm" data-testid="creator-fees-empty">Coins you launch on Armory show their unclaimed trading fees here. <Link href="/launch" className="text-accent-text">Launch a token</Link>.</p>
      )}
      {d && d.rows.length + d.lp.length > 0 && (
        <FeeTable
          rows={d.rows}
          lp={d.lp}
          canClaim={!safeSend.busy}
          testId="creator-fees-table"
          onClaim={(r) => send(({ connection, payer }) => buildClaimCreatorFeeTx(connection, payer, new PublicKey(r.pool)))}
          onClaimLp={(r) => send(({ connection, payer }) => buildClaimLpFeeTx(connection, payer, r))}
        />
      )}
      {safeSend.status === "error" && !safeSend.preview && <p role="alert" className="text-negative p-4 text-sm">{safeSend.error}</p>}
      <TxPreviewModal safeSend={safeSend} />
    </section>
  );
}

/** /admin/fees: Armory's partner fees. */
export function AdminFeesView() {
  const { publicKey } = useWallet();
  const safeSend = useSafeSend();
  const platform = useChainRead("admin:platform", (c) => fetchPlatformTerms(c));
  const feeClaimer = platform.data?.feeClaimer ?? null;
  const data = useChainRead(
    feeClaimer ? `admin:fees:${feeClaimer}:${claimEpoch(safeSend)}` : null,
    async (c) => {
      const rows = await fetchPartnerCurveFees(c);
      const lp = await fetchLpFees(c, new PublicKey(feeClaimer!), new Set(rows.filter((r) => r.migrated).map((r) => r.mint)));
      return { rows, lp };
    },
  );
  const me = publicKey?.toBase58() ?? null;
  const isClaimer = me !== null && me === feeClaimer && (!CLUSTER.isMainnet || me === MAINNET_FEE_CLAIMER.toBase58());
  const send = (b: BuildFn) => void safeSend.start(b);
  const d = data.data;

  if (platform.loading) return <p className="text-muted" role="status">Loading from chain…</p>;
  if (!platform.data)
    return <p className="text-muted" data-testid="admin-fees-no-config">{platform.error ? `Couldn't read the platform config: ${platform.error}` : `No ${CLUSTER.displayName} platform config is set yet (NEXT_PUBLIC_DBC_CONFIG_MAINNET).`}</p>;
  return (
    <div className="space-y-6" data-testid="admin-fees">
      <dl className="card grid gap-0 sm:grid-cols-3">
        <div className="p-4"><dt className="text-muted text-xs">Platform config</dt><dd><a className="font-mono text-xs underline" href={explorerAddressUrl(platform.data.config)} target="_blank" rel="noreferrer">{shortAddr(platform.data.config, 6)}</a></dd></div>
        <div className="border-border p-4 sm:border-l"><dt className="text-muted text-xs">Fee wallet (fee claimer)</dt><dd className="font-mono text-xs" data-testid="admin-fees-claimer">{shortAddr(platform.data.feeClaimer, 6)}</dd></div>
        <div className="border-border p-4 sm:border-l"><dt className="text-muted text-xs">Unclaimed, all pools</dt><dd className="num text-xl font-semibold" data-testid="admin-fees-total">{d ? formatSol(totalLamports(d.rows) + totalLamports(d.lp), 4) : "…"}</dd></div>
      </dl>
      {platform.data.problems.length > 0 && <p role="alert" className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm">{platform.data.problems.join(" ")}</p>}
      <p className={`rounded-panel border p-3 text-sm ${isClaimer ? "tag-ok" : "border-border text-muted"}`} data-testid="admin-fees-access">
        {isClaimer ? "Connected as the fee wallet. Claims are previewed before you sign." : me ? "Read-only: the connected wallet is not the fee wallet." : "Read-only. Connect the fee wallet to claim."}
      </p>
      <section className="card overflow-x-auto">
        {data.loading && <p className="text-muted p-4 text-sm" role="status">Loading pools…</p>}
        {data.error && <p role="alert" className="text-warning p-4 text-sm">{data.error}</p>}
        {d && d.rows.length + d.lp.length === 0 && <p className="text-muted p-4 text-sm">No launches on this config yet.</p>}
        {d && d.rows.length + d.lp.length > 0 && (
          <FeeTable
            rows={d.rows}
            lp={d.lp}
            canClaim={isClaimer && !safeSend.busy}
            testId="admin-fees-table"
            onClaim={(r) => send(({ connection, payer }) => buildClaimPartnerFeeTx(connection, payer, new PublicKey(r.pool)))}
            onSurplus={(r) => send(({ connection, payer }) => buildPartnerSurplusTx(connection, payer, new PublicKey(r.pool)))}
            onClaimLp={(r) => send(({ connection, payer }) => buildClaimLpFeeTx(connection, payer, r))}
          />
        )}
      </section>
      {safeSend.status === "error" && !safeSend.preview && <p role="alert" className="text-negative text-sm">{safeSend.error}</p>}
      <TxPreviewModal safeSend={safeSend} />
    </div>
  );
}
