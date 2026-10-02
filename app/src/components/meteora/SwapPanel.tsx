"use client";
/**
 * Buy / sell panel for a live launch. Venue:
 *  - "dbc":  Meteora Dynamic Bonding Curve (before graduation), @meteora-ag/dynamic-bonding-curve-sdk
 *  - "damm": Meteora DAMM v2 pool (after graduation), @meteora-ag/cp-amm-sdk
 * Live exact-in quote (output, minimum after slippage, fees, price impact), then the standard
 * useSafeSend flow: allowlist check -> simulation -> preview -> explicit confirm -> wallet signs.
 */
import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { CLUSTER, explorerTxUrl } from "@/config/cluster";
import { BETA_DEPOSIT_CAP, DEPOSIT_CAP_TAG } from "@/config/armory";
import { useChainRead } from "@/hooks/useChain";
import { formatUnits } from "@/lib/armory/format";
import { useSafeSend } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";
import { buildCurveSwapTx, quoteCurveSwap, type Side, type SwapQuoteView } from "@/lib/meteora/dbc";
import { buildDammSwapTx, quoteDammSwap } from "@/lib/meteora/damm";

const SLIPPAGES = [50, 100, 300, 500] as const;

function parseAmount(raw: string, decimals: number): bigint | null {
  const s = raw.trim().replace(/,/g, "");
  if (!/^\d*(\.\d*)?$/.test(s) || s === "" || s === ".") return null;
  const [w = "0", f = ""] = s.split(".");
  if (f.length > decimals) return null;
  const v = BigInt(w || "0") * 10n ** BigInt(decimals) + BigInt((f + "0".repeat(decimals)).slice(0, decimals) || "0");
  return v > 0n ? v : null;
}

export function SwapPanel({ venue, pool, mint, symbol, decimals, feeBps }: { venue: "dbc" | "damm"; pool: string; mint: string; symbol: string | null; decimals: number; feeBps?: number }) {
  const router = useRouter();
  const { publicKey, connected } = useWallet();
  const safeSend = useSafeSend();
  const [side, setSide] = useState<Side>("buy");
  const [amount, setAmount] = useState("");
  const [slip, setSlip] = useState<number>(100);
  // Re-read quote + balances after each confirmed trade (the signature changes the read keys).
  const done = safeSend.status === "success" ? (safeSend.signature ?? "") : "";
  const [debounced, setDebounced] = useState("");
  const tk = symbol || "tokens";
  const inDecimals = side === "buy" ? 9 : decimals;
  const outDecimals = side === "buy" ? decimals : 9;

  useEffect(() => {
    const t = setTimeout(() => setDebounced(amount), 350);
    return () => clearTimeout(t);
  }, [amount]);
  const amountIn = useMemo(() => parseAmount(debounced, inDecimals), [debounced, inDecimals]);
  const poolKey = useMemo(() => new PublicKey(pool), [pool]);
  const mintKey = useMemo(() => new PublicKey(mint), [mint]);

  const quote = useChainRead<SwapQuoteView>(
    amountIn !== null ? `quote:${venue}:${pool}:${side}:${amountIn}:${slip}:${done}` : null,
    (c) => (venue === "dbc" ? quoteCurveSwap(c, poolKey, side, amountIn!, slip) : quoteDammSwap(c, poolKey, mintKey, decimals, side, amountIn!, slip)),
  );
  const balances = useChainRead(
    publicKey ? `bal:${publicKey.toBase58()}:${mint}:${done}` : null,
    async (c) => {
      const [sol, toks] = await Promise.all([c.getBalance(publicKey!, "confirmed"), c.getParsedTokenAccountsByOwner(publicKey!, { mint: mintKey }, "confirmed")]);
      const tok = toks.value.reduce((s, a) => s + BigInt((a.account.data.parsed as { info: { tokenAmount: { amount: string } } }).info.tokenAmount.amount), 0n);
      return { sol: BigInt(sol), tok };
    },
  );

  useEffect(() => {
    if (done) router.refresh(); // server-rendered curve progress + price
  }, [done, router]);

  const q = quote.data;
  const insufficient = balances.data && amountIn !== null ? (side === "buy" ? amountIn > balances.data.sol : amountIn > balances.data.tok) : false;
  const canSubmit = connected && amountIn !== null && !!q && !quote.error && !insufficient && !safeSend.busy && q.amountOut > 0n;

  const submit = () => {
    if (!q || amountIn === null) return;
    const minOut = q.minimumAmountOut;
    void safeSend.start(async ({ connection, payer }) =>
      venue === "dbc" ? buildCurveSwapTx(connection, payer, poolKey, side, amountIn, minOut) : buildDammSwapTx(connection, payer, poolKey, mintKey, side, amountIn, minOut),
    );
  };
  const setMax = () => {
    if (!balances.data) return;
    if (side === "sell") setAmount(formatUnits(balances.data.tok, decimals).replace(/,/g, ""));
    else {
      const keep = 20_000_000n; // keep 0.02 SOL for fees / rent
      const v = balances.data.sol > keep ? balances.data.sol - keep : 0n;
      setAmount(formatUnits(v, 9).replace(/,/g, ""));
    }
  };

  const fmtOut = (v: bigint) => `${formatUnits(v, outDecimals, side === "buy" ? 2 : 6)} ${side === "buy" ? tk : "SOL"}`;
  const feeSol = (v: bigint) => `${formatUnits(v, 9, 6)} SOL`;

  return (
    <section className="card space-y-3 p-5" aria-labelledby="trade-h" data-testid="trade-panel" data-venue={venue}>
      <div className="flex items-center justify-between gap-2">
        <h2 id="trade-h" className="font-semibold">Trade</h2>
        <span className="tag tag-accent" data-testid="trade-venue">{venue === "dbc" ? "Meteora DBC · bonding curve" : "Meteora DAMM v2 pool"}</span>
      </div>
      <div className="seg" role="tablist" aria-label="Buy or sell">
        <button type="button" role="tab" aria-selected={side === "buy"} onClick={() => { setSide("buy"); setAmount(""); }} data-testid="trade-tab-buy">Buy</button>
        <button type="button" role="tab" aria-selected={side === "sell"} onClick={() => { setSide("sell"); setAmount(""); }} data-testid="trade-tab-sell">Sell</button>
      </div>
      <div className="bg-surface-2 rounded-panel border-border border p-4">
        <div className="flex items-center justify-between">
          <label htmlFor="trade-amount" className="text-muted text-xs">You pay ({side === "buy" ? "SOL" : tk})</label>
          {balances.data && (
            <button type="button" className="text-accent-text text-xs" onClick={setMax} data-testid="trade-max">
              Balance {side === "buy" ? `${formatUnits(balances.data.sol, 9, 4)} SOL` : `${formatUnits(balances.data.tok, decimals, 2)} ${tk}`} · Max
            </button>
          )}
        </div>
        <input id="trade-amount" className="input num mt-1" inputMode="decimal" placeholder={side === "buy" ? "0.01" : "100000"} value={amount} onChange={(e) => setAmount(e.target.value)} data-testid="trade-amount" />
      </div>
      <div className="flex items-center justify-between gap-2 text-xs">
        <span className="text-muted">Slippage</span>
        <div className="flex gap-1" role="group" aria-label="Slippage tolerance">
          {SLIPPAGES.map((s) => (
            <button key={s} type="button" aria-pressed={slip === s} onClick={() => setSlip(s)} className={`rounded-chip border px-2 py-1 ${slip === s ? "border-accent-border bg-accent-tint text-fg" : "border-border text-muted"}`} data-testid={`trade-slippage-${s}`}>
              {s / 100}%
            </button>
          ))}
        </div>
      </div>
      <dl data-testid="trade-quote">
        <div className="fact"><dt>You receive (est.)</dt><dd data-testid="trade-out">{q ? fmtOut(q.amountOut) : amountIn !== null && quote.loading ? "Quoting…" : "—"}</dd></div>
        <div className="fact"><dt>Minimum received ({slip / 100}% slippage)</dt><dd data-testid="trade-min-out">{q ? fmtOut(q.minimumAmountOut) : "—"}</dd></div>
        <div className="fact"><dt>Trading fee{feeBps !== undefined ? ` (${feeBps / 100}%)` : ""}</dt><dd data-testid="trade-fee">{q ? feeSol(q.tradingFee + (venue === "dbc" ? q.protocolFee : 0n)) : "—"}</dd></div>
        {venue === "dbc" && <div className="fact"><dt>… of which Meteora protocol fee</dt><dd>{q ? feeSol(q.protocolFee) : "—"}</dd></div>}
        {q?.priceImpactPct != null && <div className="fact"><dt>Price impact</dt><dd className={q.priceImpactPct > 5 ? "text-warning" : ""}>{q.priceImpactPct.toFixed(2)}%</dd></div>}
        <div className="fact"><dt>Route</dt><dd>{venue === "dbc" ? "Bonding curve (DBC)" : "DAMM v2 pool"}</dd></div>
        <div className="fact"><dt>Tax</dt><dd>No transfer tax</dd></div>
      </dl>
      {quote.error && amountIn !== null && <p role="alert" className="text-warning text-xs" data-testid="trade-quote-error">{quote.error}</p>}
      {q?.partial && <p className="text-warning text-xs">This amount would fill the rest of the curve; only part of it can be used. Try a smaller amount.</p>}
      {insufficient && <p className="text-warning text-xs">Not enough {side === "buy" ? "SOL" : tk} in your wallet.</p>}
      {amount !== "" && parseAmount(amount, inDecimals) === null && <p className="text-warning text-xs">Enter a positive amount (up to {inDecimals} decimals).</p>}
      <div className="capslot" data-testid="trade-capslot">
        <span className="tag tag-ua">Unaudited beta</span> Deposit cap <b>{BETA_DEPOSIT_CAP.sol} SOL per wallet</b> <span className="tag tag-demo">{DEPOSIT_CAP_TAG}</span>
      </div>
      <button type="button" className="btn btn-primary btn-lg w-full" disabled={!canSubmit} onClick={submit} data-testid="trade-submit">
        {!connected ? "Connect a wallet to trade" : side === "buy" ? `Buy ${tk}` : `Sell ${tk}`}
      </button>
      {safeSend.status === "error" && !safeSend.preview && <p role="alert" className="text-negative text-xs" data-testid="trade-error">{safeSend.error}</p>}
      {safeSend.status === "success" && safeSend.signature && (
        <p role="status" className="text-positive-text text-xs" data-testid="trade-success">
          Done. <a className="underline" href={explorerTxUrl(safeSend.signature)} target="_blank" rel="noopener noreferrer">View transaction ↗</a>
        </p>
      )}
      <p className="text-dim text-xs">
        {CLUSTER.label} test SOL only. Quotes come from the Meteora {venue === "dbc" ? "DBC" : "cp-amm"} SDK; your transaction is simulated and shown to you before your wallet signs. The minimum received protects you from price moves.
      </p>
      <TxPreviewModal safeSend={safeSend} />
    </section>
  );
}
