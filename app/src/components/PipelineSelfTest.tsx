"use client";

/**
 * Exercises the full safe-send pipeline with a harmless SPL Memo instruction
 * (the Memo program ships with solana-test-validator and exists on devnet).
 * Useful to verify wallet + RPC wiring before real program clients exist.
 */
import { TransactionInstruction, Transaction } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { MEMO_PROGRAM_ID } from "@/config/programs";
import { useSafeSend } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "./TxPreviewModal";

export function PipelineSelfTest() {
  const { connected } = useWallet();
  const safeSend = useSafeSend();

  const run = () =>
    safeSend.start(({ payer }) =>
      new Transaction().add(
        new TransactionInstruction({
          programId: MEMO_PROGRAM_ID,
          keys: [{ pubkey: payer, isSigner: true, isWritable: false }],
          data: new TextEncoder().encode("launchpad pipeline self-test") as unknown as Buffer,
        }),
      ),
    );

  return (
    <div className="border-border bg-surface rounded-card space-y-2 border p-4">
      <p className="text-sm font-medium">Transaction pipeline self-test</p>
      <p className="text-muted text-xs">
        Builds a memo transaction, checks it against the program allowlist, simulates it, and shows the preview. Nothing
        is signed unless you confirm.
      </p>
      <button
        type="button"
        onClick={() => void run()}
        disabled={!connected || safeSend.busy}
        className="bg-accent text-accent-fg rounded-control px-3 py-1.5 text-sm disabled:opacity-40"
      >
        {connected ? "Preview memo transaction" : "Connect a wallet to try"}
      </button>
      {safeSend.status === "error" && !safeSend.preview && (
        <p role="alert" className="text-negative text-xs break-all">
          {safeSend.error}
        </p>
      )}
      <TxPreviewModal safeSend={safeSend} />
    </div>
  );
}
