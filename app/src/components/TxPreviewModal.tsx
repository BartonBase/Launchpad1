"use client";

/**
 * TxPreviewModal — renders a TxPreview and is the ONLY place a user can
 * confirm a transaction.
 *
 * Anti-spoofing measures ("unspoofable-ish"):
 *  - Rendered in a portal directly under <body>, above everything
 *    (max z-index, inline positioning styles that page CSS classes can't
 *    accidentally override).
 *  - While open, every other <body> child is made `inert`, so page content
 *    (including anything derived from untrusted collection metadata) cannot
 *    receive clicks/focus or overlay the Confirm button.
 *  - Content comes only from the TxPreview model (message + simulation), never
 *    from page-provided strings. Full program IDs + message fingerprint shown.
 *  - Confirm is disabled for a short arming delay (anti double-click /
 *    click-jacking) and whenever the preview has blocking errors.
 */
import { useEffect, useRef, useState, useSyncExternalStore, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { explorerTxUrl } from "@/config/cluster";
import type { SafeSend } from "@/lib/tx/useSafeSend";
import type { TxPreview } from "@/lib/tx/preview";

const ARM_DELAY_MS = 1200;
const noopSubscribe = () => () => {};

function useIsClient(): boolean {
  return useSyncExternalStore(noopSubscribe, () => true, () => false);
}

function deltaClass(n: bigint): string {
  if (n > 0n) return "text-positive";
  if (n < 0n) return "text-negative";
  return "text-muted";
}

function Mono({ children }: { children: string }) {
  return <span className="font-mono text-xs break-all">{children}</span>;
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="space-y-2">
      <h3 className="text-muted text-xs font-semibold tracking-wide uppercase">{title}</h3>
      {children}
    </section>
  );
}

const STATUS_TEXT: Partial<Record<SafeSend["status"], string>> = {
  signing: "Waiting for your wallet to sign…",
  sending: "Sending through the app's RPC…",
  confirming: "Waiting for confirmation…",
  success: "Confirmed.",
};

export function TxPreviewModal({ safeSend }: { safeSend: SafeSend }) {
  const isClient = useIsClient();
  const { status, preview, error } = safeSend;
  const open =
    preview !== null &&
    (status === "awaiting-confirmation" ||
      status === "signing" ||
      status === "sending" ||
      status === "confirming" ||
      status === "success" ||
      (status === "error" && preview !== null));
  if (!isClient || !open || !preview) return null;
  return createPortal(
    <ModalBody key={preview.fingerprint} safeSend={safeSend} preview={preview} error={error} />,
    document.body,
  );
}

function ModalBody({ safeSend, preview, error }: { safeSend: SafeSend; preview: TxPreview; error: string | null }) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const [armed, setArmed] = useState(false);
  const { status } = safeSend;

  // Arm the Confirm button after a delay.
  useEffect(() => {
    const t = setTimeout(() => setArmed(true), ARM_DELAY_MS);
    return () => clearTimeout(t);
  }, []);

  // Make the rest of the page inert while the modal is open.
  useEffect(() => {
    const overlay = overlayRef.current;
    const touched: Element[] = [];
    for (const el of Array.from(document.body.children)) {
      if (el === overlay || el.hasAttribute("inert")) continue;
      el.setAttribute("inert", "");
      touched.push(el);
    }
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      touched.forEach((el) => el.removeAttribute("inert"));
      document.body.style.overflow = prevOverflow;
    };
  }, []);

  // Escape cancels (only when not mid-signing).
  const { cancel } = safeSend;
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") cancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cancel]);

  const awaiting = status === "awaiting-confirmation";
  const canConfirm = awaiting && preview.canSend && armed && !safeSend.previewOnly;
  const done = status === "success" || status === "error";

  return (
    <div
      ref={overlayRef}
      id="lp-tx-preview"
      data-testid="tx-preview"
      role="dialog"
      aria-modal="true"
      aria-labelledby="lp-tx-preview-title"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 2147483647,
        isolation: "isolate",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "1rem",
        background: "rgba(0,0,0,0.6)",
        pointerEvents: "auto",
      }}
    >
      <div
        className="bg-surface text-fg border-border rounded-card w-full max-w-2xl overflow-y-auto border shadow-2xl"
        style={{ maxHeight: "90vh" }}
      >
        <div className="border-border flex items-center justify-between gap-2 border-b px-5 py-3">
          <h2 id="lp-tx-preview-title" className="text-lg font-semibold">
            Review transaction
          </h2>
          <span className="bg-accent text-accent-fg rounded-control px-2 py-0.5 font-mono text-xs font-bold">
            {preview.clusterLabel}
          </span>
        </div>

        <div className="space-y-5 px-5 py-4 text-sm">
          {preview.errors.length > 0 && (
            <div role="alert" className="bg-danger-bg text-negative rounded-control space-y-1 p-3">
              <p className="font-semibold">Sending is blocked:</p>
              <ul className="list-disc pl-5">
                {preview.errors.map((e) => (
                  <li key={e} className="break-all">{e}</li>
                ))}
              </ul>
            </div>
          )}
          {preview.warnings.length > 0 && (
            <div className="bg-warning-bg text-warning rounded-control space-y-1 p-3">
              <ul className="list-disc pl-5">
                {preview.warnings.map((w) => (
                  <li key={w} className="break-all">{w}</li>
                ))}
              </ul>
            </div>
          )}

          <Section title="Summary">
            <ul className="space-y-1">
              {preview.summary.map((s) => (
                <li key={s} className="break-all">{s}</li>
              ))}
            </ul>
          </Section>

          <Section title="Programs invoked">
            <ul className="space-y-2">
              {preview.programs.map((p) => (
                <li key={p.programId} className="border-border rounded-control border p-2">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className={p.allowlisted ? "font-medium" : "text-negative font-bold"}>
                      {p.name ?? "UNKNOWN PROGRAM"}
                    </span>
                    {p.cpiOnly && <span className="text-muted text-xs">(called via CPI)</span>}
                    {!p.allowlisted && <span className="text-negative text-xs font-bold">NOT ALLOWLISTED</span>}
                  </div>
                  <Mono>{p.programId}</Mono>
                </li>
              ))}
            </ul>
          </Section>

          <Section title="SOL balance changes">
            {preview.solChanges.length === 0 ? (
              <p className="text-muted">None detected.</p>
            ) : (
              <ul className="space-y-1">
                {preview.solChanges.map((c) => (
                  <li key={c.address} className="flex flex-wrap justify-between gap-2">
                    <span>
                      <Mono>{c.address}</Mono>
                      {c.isFeePayer && <span className="text-muted ml-1 text-xs">(you, fee payer)</span>}
                    </span>
                    <span className={`font-mono ${deltaClass(c.deltaLamports)}`}>{c.display}</span>
                  </li>
                ))}
              </ul>
            )}
          </Section>

          <Section title="Token balance changes">
            {preview.tokenChanges.length === 0 ? (
              <p className="text-muted">None detected.</p>
            ) : (
              <ul className="space-y-2">
                {preview.tokenChanges.map((c) => (
                  <li key={c.account} className="border-border rounded-control border p-2">
                    <div className="flex flex-wrap justify-between gap-2">
                      <span className="text-xs">
                        Mint <Mono>{c.mint}</Mono>
                      </span>
                      <span className={`font-mono ${deltaClass(c.deltaAmount)}`}>{c.display}</span>
                    </div>
                    <div className="text-muted text-xs">
                      Owner <Mono>{c.owner}</Mono> {c.ownedBySigner && "(you)"}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </Section>

          <Section title="Accounts written">
            <ul className="space-y-1">
              {preview.writableAccounts.map((a) => (
                <li key={a.address}>
                  <Mono>{a.address}</Mono>
                  {a.isSigner && <span className="text-muted ml-1 text-xs">(signer)</span>}
                </li>
              ))}
            </ul>
          </Section>

          {preview.lookupTables.length > 0 && (
            <Section title="Accounts loaded from lookup tables">
              <ul className="space-y-2" data-testid="tx-lookup-tables">
                {preview.lookupTables.map((t) => (
                  <li key={t.address}>
                    <p>
                      Table <Mono>{t.address}</Mono>{" "}
                      <span className={t.pinned && (t.frozen || t.verified) ? "text-muted text-xs" : "text-negative text-xs"}>
                        ({t.pinned ? "pinned" : "NOT pinned"}, {t.frozen ? "frozen" : t.verified ? "contents match the pinned list" : "has an authority"})
                      </span>
                    </p>
                    <ul className="mt-1 space-y-0.5 pl-3">
                      {t.writable.map((k) => <li key={`w${k}`}><Mono>{k}</Mono> <span className="text-muted text-xs">(writable)</span></li>)}
                      {t.readonly.map((k) => <li key={`r${k}`}><Mono>{k}</Mono> <span className="text-muted text-xs">(read-only)</span></li>)}
                    </ul>
                  </li>
                ))}
              </ul>
            </Section>
          )}

          <Section title="Fees & compute">
            <p>
              Network fee: <span className="font-mono">{preview.feeDisplay}</span> · Compute units:{" "}
              <span className="font-mono">{preview.unitsConsumed ?? "unknown"}</span>
            </p>
            <p className="text-muted text-xs">
              Message fingerprint (SHA-256): <Mono>{preview.fingerprint}</Mono>
            </p>
          </Section>

          {preview.logs.length > 0 && (
            <details>
              <summary className="text-muted cursor-pointer text-xs">Simulation logs ({preview.logs.length})</summary>
              <pre className="bg-surface-2 rounded-control mt-2 max-h-48 overflow-auto p-2 font-mono text-xs whitespace-pre-wrap">
                {preview.logs.join("\n")}
              </pre>
            </details>
          )}

          {STATUS_TEXT[status] && <p className="font-medium">{STATUS_TEXT[status]}</p>}
          {status === "success" && safeSend.signature && (
            <p className="text-xs">
              Signature: <Mono>{safeSend.signature}</Mono>{" "}
              <a className="underline" href={explorerTxUrl(safeSend.signature)} target="_blank" rel="noopener noreferrer">
                Explorer
              </a>
            </p>
          )}
          {status === "error" && error && (
            <p role="alert" className="text-negative break-all">
              {error}
            </p>
          )}
        </div>

        {safeSend.previewOnly && (
          <p className="border-border text-warning border-t px-5 py-3 text-sm" role="note" data-testid="tx-preview-only">
            Preview only: {safeSend.previewOnly}
          </p>
        )}
        <div className="border-border flex justify-end gap-3 border-t px-5 py-3">
          <button
            type="button"
            onClick={cancel}
            disabled={safeSend.busy}
            data-testid="tx-cancel"
            className="btn"
          >
            {done ? "Close" : "Cancel"}
          </button>
          {!done && (
            <button
              type="button"
              onClick={() => void safeSend.confirm()}
              disabled={!canConfirm}
              data-testid="tx-confirm"
              className="btn btn-primary"
            >
              {safeSend.previewOnly ? "Preview only" : !preview.canSend ? "Blocked" : awaiting && !armed ? "Review…" : "Confirm & sign"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
