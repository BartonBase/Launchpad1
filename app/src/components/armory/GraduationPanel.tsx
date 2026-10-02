"use client";
/**
 * Graduation steps after the curve fills. All three are permissionless, so any visitor can push a
 * launch forward: migrate the DBC pool to DAMM v2, send the curve's unsold tokens to the locked
 * buffer (the config's leftover receiver), and, for Hybrid, open the NFT vault (graduation proof =
 * the launch's recorded DBC pool). Every action goes through the safe-send preview.
 */
import { useState } from "react";
import { useRouter } from "next/navigation";
import { PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { useChainRead } from "@/hooks/useChain";
import { buildMigrateTx, buildOpenVaultTx, buildWithdrawLeftoverTx } from "@/lib/armory/builders";
import { dbcClient } from "@/lib/meteora/dbc";
import { useSafeSend, type BuildFn } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";

export function GraduationPanel({ dbcPool, curveComplete, migrated, vault, vaultOpen }: { dbcPool: string; curveComplete: boolean; migrated: boolean; vault: string | null; vaultOpen: boolean }) {
  const { connected } = useWallet();
  const safeSend = useSafeSend();
  const router = useRouter();
  const [nonce, setNonce] = useState(0);
  const pool = new PublicKey(dbcPool);
  const leftover = useChainRead(migrated ? `dbc:leftover:${dbcPool}` : null, async (c) => (await dbcClient(c).state.getPool(pool))?.poolState.isWithdrawLeftover === 1, nonce);
  const needMigrate = curveComplete && !migrated;
  const needLeftover = migrated && leftover.data === false;
  const needOpen = migrated && vault !== null && !vaultOpen;
  if (!needMigrate && !needLeftover && !needOpen) return null;
  const run = (b: BuildFn) =>
    void safeSend.start(b).then(() => {
      setNonce((n) => n + 1);
      router.refresh();
    });
  return (
    <section className="card space-y-3 p-5" data-testid="graduation-panel">
      <p className="eyebrow">Graduation</p>
      <p className="text-muted text-sm">
        {needMigrate ? "The curve is full. Anyone can finish the graduation: move the liquidity to a locked trading pool." : "The token has graduated. Anyone can finish the remaining steps."}
      </p>
      <div className="flex flex-wrap gap-2">
        {needMigrate && (
          <button type="button" className="btn btn-primary" disabled={!connected || safeSend.busy} onClick={() => run(({ connection, payer }) => buildMigrateTx(connection, payer, pool))} data-testid="graduation-migrate">
            Graduate to the trading pool
          </button>
        )}
        {needLeftover && (
          <button type="button" className="btn" disabled={!connected || safeSend.busy} onClick={() => run(({ connection, payer }) => buildWithdrawLeftoverTx(connection, payer, pool))} data-testid="graduation-leftover">
            Send unsold tokens to the locked buffer
          </button>
        )}
        {needOpen && vault && (
          <button type="button" className="btn btn-primary" disabled={!connected || safeSend.busy} onClick={() => run(({ connection, payer }) => buildOpenVaultTx(connection, payer, new PublicKey(vault)))} data-testid="graduation-open-vault">
            Open the NFT vault
          </button>
        )}
      </div>
      {!connected && <p className="text-dim text-xs">Connect a wallet to continue.</p>}
      {safeSend.status === "error" && !safeSend.preview && <p role="alert" className="text-negative text-sm">{safeSend.error}</p>}
      <TxPreviewModal safeSend={safeSend} />
    </section>
  );
}
