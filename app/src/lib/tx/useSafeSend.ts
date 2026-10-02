"use client";

/**
 * useSafeSend — the only sanctioned way for UI code to send a transaction.
 *
 *   build -> validate (allowlist) -> simulate -> preview -> [explicit user
 *   confirm] -> sign -> tamper check -> send via OUR pinned RPC -> confirm
 *
 * Sending is impossible unless the preview has no blocking errors and the
 * user clicked Confirm. The exact transaction object that was simulated is
 * the one that gets signed; if the wallet returns a different message, we
 * abort and ask the user to review again.
 */
import { useCallback, useRef, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import {
  Transaction,
  VersionedTransaction,
  type Connection,
  type Keypair,
  type PublicKey,
} from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import { bytesEqual, isVersioned, messageBytes, messageFingerprint, toVersionedMessage, viewMessage, type AnyTransaction } from "./message";
import { buildTxPreview, type TxPreview } from "./preview";
import { estimateFee, simulate } from "./simulate";
import { validateInstructions } from "./validate";
import { resolveLookups } from "./lookup";
import { PINNED_LOOKUP_TABLES } from "@/config/integrations";

export type SafeSendStatus =
  | "idle"
  | "building"
  | "validating"
  | "simulating"
  | "awaiting-confirmation"
  | "signing"
  | "sending"
  | "confirming"
  | "success"
  | "error";

export interface BuildContext {
  readonly connection: Connection;
  readonly payer: PublicKey;
  readonly recentBlockhash: string;
  readonly lastValidBlockHeight: number;
}

/**
 * A build may return extra LOCAL signers (e.g. a freshly generated mint keypair). They sign
 * after the user confirms and before the wallet; signatures are not part of the message, so the
 * previewed message bytes are unchanged.
 */
export type BuildResult = AnyTransaction | { readonly tx: AnyTransaction; readonly signers: readonly Keypair[] };
export type BuildFn = (ctx: BuildContext) => Promise<BuildResult> | BuildResult;

interface Pending {
  readonly tx: AnyTransaction;
  readonly signers: readonly Keypair[];
  readonly bytes: Uint8Array;
  readonly blockhash: string;
  readonly lastValidBlockHeight: number;
}

export interface SafeSend {
  readonly status: SafeSendStatus;
  readonly preview: TxPreview | null;
  readonly error: string | null;
  readonly signature: string | null;
  readonly busy: boolean;
  /** Set when the open preview came from a preview-only start: Confirm is never enabled. */
  readonly previewOnly: string | null;
  /**
   * Build + validate + simulate; opens the preview. Never signs. With `opts.previewOnly` (a note
   * shown in the modal) nothing is kept for confirm(), so the transaction can't be signed or sent.
   */
  start(build: BuildFn, opts?: { previewOnly?: string }): Promise<void>;
  /** Called ONLY from the preview modal's Confirm button. */
  confirm(): Promise<void>;
  cancel(): void;
}

const BUSY: readonly SafeSendStatus[] = ["building", "validating", "simulating", "signing", "sending", "confirming"];

export function useSafeSend(): SafeSend {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [status, setStatus] = useState<SafeSendStatus>("idle");
  const [preview, setPreview] = useState<TxPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [signature, setSignature] = useState<string | null>(null);
  const [previewOnly, setPreviewOnly] = useState<string | null>(null);
  const pending = useRef<Pending | null>(null);
  const statusRef = useRef<SafeSendStatus>("idle");

  const set = useCallback((s: SafeSendStatus) => {
    statusRef.current = s;
    setStatus(s);
  }, []);

  const fail = useCallback(
    (msg: string) => {
      pending.current = null;
      setError(msg);
      set("error");
    },
    [set],
  );

  const start = useCallback(
    async (build: BuildFn, opts?: { previewOnly?: string }) => {
      if (BUSY.includes(statusRef.current) || statusRef.current === "awaiting-confirmation") return;
      setPreviewOnly(opts?.previewOnly ?? null);
      setError(null);
      setSignature(null);
      setPreview(null);
      pending.current = null;

      const payer = wallet.publicKey;
      if (!payer || !wallet.connected) return fail("Connect a wallet first.");

      try {
        set("building");
        const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
        const built = await build({ connection, payer, recentBlockhash: blockhash, lastValidBlockHeight });
        const tx = "signers" in built ? built.tx : built;
        const signers = "signers" in built ? built.signers : [];
        if (tx instanceof Transaction) {
          tx.feePayer ??= payer;
          tx.recentBlockhash ??= blockhash;
        }

        set("validating");
        const validation = validateInstructions(tx, CLUSTER.name);
        // v0: fetch + validate every referenced lookup table (pinned; frozen or contents match the pinned list) and expand it, so the
        // preview lists and diffs ALT-loaded accounts instead of hiding them.
        const lookups = await resolveLookups(connection, toVersionedMessage(tx), PINNED_LOOKUP_TABLES[CLUSTER.name]);
        const view = viewMessage(tx, lookups);
        const bytes = messageBytes(tx);
        const fingerprint = await messageFingerprint(bytes);
        const extraWarnings: string[] = [];
        if (!wallet.signTransaction) {
          extraWarnings.push(
            "This wallet cannot sign without broadcasting, so the app cannot verify the signed transaction before it is sent.",
          );
        }

        let simulation = null;
        let fee: bigint | null = null;
        if (validation.ok) {
          set("simulating");
          [simulation, fee] = await Promise.all([
            simulate(connection, tx, { signer: payer, lookups }),
            estimateFee(connection, tx),
          ]);
        }

        const p = buildTxPreview({
          cluster: CLUSTER,
          view,
          fingerprint,
          validation,
          simulation,
          feeLamports: fee,
          signer: payer.toBase58(),
          extraWarnings,
        });
        pending.current = p.canSend && !opts?.previewOnly
          ? { tx, signers, bytes, blockhash: isVersioned(tx) ? tx.message.recentBlockhash : tx.recentBlockhash!, lastValidBlockHeight }
          : null;
        setPreview(p);
        // Always show the preview (also when blocked) so the user sees why.
        set("awaiting-confirmation");
      } catch (e) {
        fail(e instanceof Error ? e.message : String(e));
      }
    },
    [connection, wallet, fail, set],
  );

  const confirm = useCallback(async () => {
    const p = pending.current;
    if (statusRef.current !== "awaiting-confirmation" || !p || !preview?.canSend) return;
    pending.current = null; // one-shot: a second click can never re-send
    try {
      let sig: string;
      if (p.signers.length > 0) {
        if (p.tx instanceof VersionedTransaction) p.tx.sign([...p.signers]);
        else p.tx.partialSign(...p.signers);
      }
      if (wallet.signTransaction) {
        set("signing");
        const signed: AnyTransaction = await wallet.signTransaction(p.tx);
        // Tamper check: the wallet must sign exactly what was previewed.
        if (!bytesEqual(messageBytes(signed), p.bytes)) {
          const recheck = validateInstructions(signed, CLUSTER.name);
          return fail(
            "The wallet modified the transaction after preview" +
              (recheck.ok ? "" : ` (${recheck.errors.join("; ")})`) +
              ". Nothing was sent. Please review again.",
          );
        }
        set("sending");
        // Broadcast through the app's pinned localnet/devnet RPC, never the wallet's default RPC.
        sig = await connection.sendRawTransaction(signed.serialize(), {
          skipPreflight: false,
          preflightCommitment: "confirmed",
          maxRetries: 3,
        });
      } else {
        set("sending");
        sig = await wallet.sendTransaction(p.tx, connection, { skipPreflight: false, preflightCommitment: "confirmed" });
      }
      setSignature(sig);
      set("confirming");
      const res = await connection.confirmTransaction(
        { signature: sig, blockhash: p.blockhash, lastValidBlockHeight: p.lastValidBlockHeight },
        "confirmed",
      );
      if (res.value.err) return fail(`Transaction failed on-chain: ${JSON.stringify(res.value.err)}`);
      set("success");
    } catch (e) {
      fail(e instanceof Error ? e.message : String(e));
    }
  }, [connection, wallet, preview, fail, set]);

  const cancel = useCallback(() => {
    if (BUSY.includes(statusRef.current)) return; // can't cancel mid-signing; wallet UI owns that
    pending.current = null;
    setPreview(null);
    setPreviewOnly(null);
    setError(null);
    set("idle");
  }, [set]);

  return {
    status,
    preview,
    error,
    signature,
    busy: BUSY.includes(status),
    previewOnly,
    start,
    confirm,
    cancel,
  };
}
