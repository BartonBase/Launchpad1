/**
 * Token page sections shared by Plain / Hybrid / Burn (design: token-plain, token, token-graduated,
 * token-burn; matrix: lib/armory/tokenPanels). Server components; no transaction code here.
 */
import { BETA_DEPOSIT_CAP, BURN_MINT_TEXT, DEPOSIT_CAP_TAG, FIRST_MINT_RANGE_TEXT, MINT_ESCROW_LAMPORTS, TOTAL_SUPPLY_WHOLE } from "@/config/armory";
import { explorerAddressUrl } from "@/config/cluster";
import { formatSol, formatUnits, shortAddr } from "@/lib/armory/format";
import type { Phase, TokenType } from "@/lib/armory/tokenPanels";
import { NO_CURVE_MESSAGE } from "@/lib/armory/errors";
import { MascotPlaceholder } from "./MascotPlaceholder";
import { TypeIcon } from "./TypeIcon";

export interface TokenView {
  readonly type: TokenType;
  readonly phase: Phase;
  readonly example: boolean; // true = design preview of a type that is pending deploy
  readonly name: string;
  readonly symbol: string | null;
  readonly mint: string | null;
  readonly collection: string | null;
  readonly decimals: number;
  readonly ratioWhole: bigint | null;
  readonly collectionSize: number | null;
  readonly minted: number;
  readonly feeLamports: bigint | null;
  readonly feeIsExactTier: boolean;
  readonly feeRecipient: string | null;
  readonly graduationLamports: bigint;
  readonly mintAuthority: string | null;
  readonly freezeAuthority: string | null;
  readonly supplyWhole: bigint;
}

const TNAME: Record<TokenType, string> = { plain: "Plain", hybrid: "Hybrid", burn: "Burn" };
const Ex = ({ on }: { on: boolean }) => (on ? <span className="tag tag-demo">Example</span> : null);
const Fact = ({ k, v, ok, testId }: { k: string; v: React.ReactNode; ok?: boolean; testId?: string }) => (
  <div className="fact" data-testid={testId}>
    <dt>{k}</dt>
    <dd className={ok === true ? "text-positive-text" : ok === false ? "text-warning" : ""}>{v}</dd>
  </div>
);
const verb = (t: TokenType) => (t === "burn" ? "Burning" : "Converting");

/**
 * Design token.html / token-mobile.png: icon · title, chips, glossary · price + address chips.
 * Desktop: three columns. Mobile (<lg): stacked in that order, price and addresses left-aligned.
 */
/** Spot prices on devnet curves are tiny (≈1e-10 SOL), so quote them per 1M tokens below 1e-6. */
export const fmtPrice = (p: number) => (p >= 1e-6 ? `${p >= 0.001 ? p.toFixed(4) : p.toPrecision(3)} SOL` : `${(p * 1e6).toPrecision(3)} SOL / 1M`);

export function TokenHeader({ v, priceSol = null }: { v: TokenView; priceSol?: number | null }) {
  const addr = "border-border text-muted hover:text-fg rounded-control border px-2 py-1 font-mono text-xs";
  return (
    <header className="border-border grid gap-4 border-b pb-6 lg:grid-cols-[auto_minmax(0,1fr)_auto] lg:items-start lg:gap-6" data-testid="token-header">
      <div className="bg-surface-2 border-border flex size-16 items-center justify-center rounded-2xl border lg:mt-12" aria-hidden="true">
        <span className="hd text-dim text-2xl">{v.name.slice(0, 1)}</span>
      </div>
      <div className="min-w-0 space-y-2.5">
        <h1 className="hd text-3xl md:text-4xl">
          {v.name} {v.symbol && <span className="text-muted font-mono text-sm font-normal">{v.symbol}</span>}
        </h1>
        <div className="flex flex-wrap gap-1.5" data-testid="token-chips">
          <span className="tchip" data-testid="type-chip"><TypeIcon type={v.type} size={14} />{TNAME[v.type]}</span>
          {v.phase === "graduated" ? <span className="tag tag-ok"><i />Graduated</span> : v.phase === "curve" ? <span className="tag tag-accent"><i />On curve</span> : <span className="tag" title={NO_CURVE_MESSAGE}>No curve · native test launch</span>}
          {v.phase === "graduated" && <span className="tag">Trading on Meteora DAMM v2</span>}
          {v.phase === "curve" && !v.example && <span className="tag">Meteora DBC curve</span>}
          {v.type !== "plain" && <span className="tag">{verb(v.type)} {v.phase === "graduated" ? "open" : v.phase === "native" ? "can never open" : "opens at graduation"}</span>}
          {v.ratioWhole !== null && v.collectionSize !== null && <span className="tag">{v.collectionSize.toLocaleString("en-US")} NFTs · {formatUnits(v.ratioWhole, 0)}{v.symbol ? ` ${v.symbol}` : ""} each</span>}
          <span className="tag">No transfer tax</span>
          {v.example && <span className="tag tag-soon" data-testid="pending-chip">Coming soon</span>}
          {v.example && <span className="tag tag-demo">Example</span>}
        </div>
        <p className="text-muted max-w-xl text-sm" data-testid="token-glossary">
          <b className="text-fg">Bonding curve</b>: the formula that sets the price before graduation; it rises as people buy. <b className="text-fg">Graduation</b>: when the curve raises its SOL target, trading moves to a Meteora DAMM v2 pool
          {v.type === "hybrid" && <> and the NFT side opens. <b className="text-fg">SPL-404</b>: a token paired with an NFT collection at a fixed rate, convertible both ways.</>}
          {v.type === "burn" && <> and burning opens. <b className="text-fg">Burn</b>: destroy a fixed amount of tokens to mint the next NFT; one-way.</>}
          {v.type === "plain" && <>.</>}
        </p>
      </div>
      <div className="space-y-2 lg:text-right" data-testid="token-price">
        <p className="num text-4xl font-semibold">{priceSol ? (priceSol >= 1e-6 ? priceSol.toPrecision(3) : (priceSol * 1e6).toPrecision(3)) : "—"} <span className="text-muted text-lg font-normal">{priceSol && priceSol < 1e-6 ? "SOL per 1M" : "SOL"}</span></p>
        <p className="text-dim text-xs">{priceSol ? `Spot price per token, read from the Meteora ${v.phase === "graduated" ? "DAMM v2 pool" : "curve"}` : v.example ? "Example page: no live price" : "No market for this launch"}</p>
        {v.mint && (
          <div className="flex flex-wrap gap-1.5 lg:justify-end" data-testid="token-addresses">
            <a href={explorerAddressUrl(v.mint)} target="_blank" rel="noopener noreferrer" className={addr}>Mint {shortAddr(v.mint, 4)} ↗</a>
            {v.collection ? (
              <a href={explorerAddressUrl(v.collection)} target="_blank" rel="noopener noreferrer" className={addr}>Collection {shortAddr(v.collection, 4)} ↗</a>
            ) : v.type !== "plain" ? (
              <span className={addr}>Collection address: at graduation</span>
            ) : null}
          </div>
        )}
      </div>
    </header>
  );
}

export function MarketCard({ v, priceSol = null }: { v: TokenView; priceSol?: number | null }) {
  const burned = TOTAL_SUPPLY_WHOLE - v.supplyWhole;
  const mcap = priceSol ? priceSol * Number(v.supplyWhole) : null;
  return (
    <section className="card space-y-4 p-5" aria-labelledby="mkt-h" data-testid="market-card">
      <div className="flex items-center justify-between gap-2">
        <h2 id="mkt-h" className="font-semibold">Market</h2>
        {v.example ? <span className="tag tag-demo">Example</span> : <span className="tag">Live from Meteora</span>}
      </div>
      <dl className="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
        <div><dt className="text-muted text-xs">Price</dt><dd className="num">{priceSol ? fmtPrice(priceSol) : "—"}</dd></div>
        <div><dt className="text-muted text-xs">Market cap</dt><dd className="num">{mcap ? `${mcap.toLocaleString("en-US", { maximumFractionDigits: 2 })} SOL` : "—"}</dd></div>
        <div><dt className="text-muted text-xs">Venue</dt><dd>{v.example ? "—" : v.phase === "graduated" ? "DAMM v2" : "DBC curve"}</dd></div>
        <div>
          <dt className="text-muted text-xs">{v.type === "burn" ? "Supply now" : "Supply"}</dt>
          <dd className="num">{formatUnits(v.supplyWhole, 0)}</dd>
          <dd className="text-positive-text text-xs">{v.type === "burn" ? `${formatUnits(burned, 0)} burned` : "No one can mint more"}</dd>
        </div>
      </dl>
    </section>
  );
}

export function PhaseCard({ v }: { v: TokenView }) {
  const steps: [string, string][] = [["Liquidity", "Moved to a Meteora DAMM v2 pool"]];
  if (v.type === "hybrid") steps.push(["Converting", "Opens, art revealed; each NFT minted on first capture"], ["NFT trading", "Live on Tensor and Magic Eden"]);
  if (v.type === "burn") steps.push(["Burning", `Opens: burn the ratio to mint the next NFT in collection order`], ["NFT trading", "Minted NFTs trade on marketplaces"]);
  return (
    <section className="card space-y-4 p-5" aria-labelledby="phase-h" data-testid="phase-card">
      <div className="flex items-start gap-4">
        <div className="flex-1 space-y-1">
          <h2 id="phase-h" className="font-semibold">
            {v.phase === "graduated" ? "Graduated" : v.phase === "curve" ? "Bonding curve" : "No bonding curve"} <Ex on={v.example} />
          </h2>
          <p className="text-muted text-sm">
            {v.phase === "native"
              ? `${NO_CURVE_MESSAGE} The supply sits in a program-owned account. Only launches on a bonding curve (the curve path) can graduate.`
              : v.phase === "graduated"
                ? `Graduated at ${formatSol(v.graduationLamports, 2)}${v.collectionSize ? ` · ${v.minted} of ${v.collectionSize.toLocaleString("en-US")} minted so far` : ""}.`
                : `When the curve reaches ${formatSol(v.graduationLamports, 2)}, liquidity moves to a Meteora DAMM v2 pool${v.type === "plain" ? " and nothing else changes" : `, and ${verb(v.type).toLowerCase()} opens`}.`}
          </p>
        </div>
        {v.phase !== "native" && <MascotPlaceholder size={88} className="hidden sm:block" />}
      </div>
      {v.phase !== "native" && (
        <ol className="grid gap-2 sm:grid-cols-3">
          {steps.map(([a, b], i) => (
            <li key={a} className="bg-surface-2 rounded-panel border-border border p-3 text-sm">
              <p className="text-accent-text font-mono text-xs">{String(i + 1).padStart(2, "0")} {a}</p>
              <p className="text-muted text-xs">{b}</p>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

export function TradePanel({ v }: { v: TokenView }) {
  return (
    <section className="card space-y-3 p-5" aria-labelledby="trade-h" data-testid="trade-panel">
      <h2 id="trade-h" className="sr-only">Trade</h2>
      <div className="seg" role="tablist" aria-label="Buy or sell">
        <button type="button" role="tab" aria-selected="true" disabled>Buy</button>
        <button type="button" role="tab" aria-selected="false" disabled>Sell</button>
      </div>
      <div className="bg-surface-2 rounded-panel border-border border p-4">
        <p className="text-muted text-xs">You pay</p>
        <input className="input num mt-1" placeholder="0.00 SOL" disabled aria-label="Amount in SOL" />
      </div>
      <dl>
        <Fact k="Route" v={v.phase === "native" ? "None (no curve)" : v.phase === "graduated" ? "Meteora DAMM v2 pool" : "Meteora DBC curve"} />
        <Fact k="Tax" v="No transfer tax" />
      </dl>
      <div className="capslot" data-testid="trade-capslot">
        <span className="tag tag-ua">Unaudited beta</span> Deposit cap <b>{BETA_DEPOSIT_CAP.sol} SOL per wallet</b> <span className="tag tag-demo">{DEPOSIT_CAP_TAG}</span>
      </div>
      <button type="button" className="btn btn-primary btn-lg w-full" disabled data-testid="trade-submit">
        Buy {v.symbol ?? "tokens"}
      </button>
      <p className="text-dim text-xs" data-testid="trade-disabled-note">{v.example ? "Example page: trading is disabled. Live launches trade on Meteora (DBC curve, then DAMM v2)." : v.phase === "native" ? "This devnet test launch has no curve or pool, so there's nothing to trade. Launches on a Meteora DBC curve are tradable." : "No Meteora pool was found for this token."}</p>
    </section>
  );
}

export function PlainFacts() {
  return (
    <section className="card space-y-2 p-5" aria-labelledby="pf-h" data-testid="plain-facts">
      <h2 id="pf-h" className="font-semibold">What a Plain launch is</h2>
      <ul className="text-muted list-inside list-disc space-y-1 text-sm">
        <li>Just the coin: a classic 1,000,000,000 SPL token on a Meteora Dynamic Bonding Curve.</li>
        <li>No NFT collection, no converting, no platform fee per action. Trade fees only.</li>
        <li>At graduation the curve&apos;s liquidity migrates to a Meteora DAMM v2 pool. Nothing else changes.</li>
      </ul>
    </section>
  );
}

export function BurnPanels({ v }: { v: TokenView }) {
  const ratio = v.ratioWhole ?? 0n;
  const size = v.collectionSize ?? 0;
  const maxBurn = ratio * BigInt(size);
  return (
    <>
      <div className="grid gap-6 md:grid-cols-2">
        <section className="card space-y-3 p-5" aria-labelledby="btm-h" data-testid="burn-to-mint">
          <div className="flex items-start justify-between">
            <div>
              <h2 id="btm-h" className="font-semibold">Burn to mint</h2>
              <p className="text-muted text-xs">Tokens → NFT · one-way</p>
            </div>
            <span className="tag tag-soon">Coming soon</span>
          </div>
          <div className="bg-surface-2 rounded-panel border-border border p-4">
            <p className="text-muted text-xs">You burn</p>
            <p className="num text-2xl font-semibold">{formatUnits(ratio, 0)}</p>
          </div>
          <div className="bg-surface-2 rounded-panel border-border border p-4">
            <p className="text-muted text-xs">You receive · next in collection order</p>
            <p className="text-2xl font-semibold">#{String(v.minted).padStart(4, "0")}</p>
          </div>
          <dl data-testid="burn-cost">
            <Fact k="Rate" v={`${formatUnits(ratio, 0)} = 1 NFT`} />
            <Fact k="Platform fee" v={v.feeLamports !== null ? formatSol(v.feeLamports) : "—"} />
            <Fact k="Mint cost (every burn mints a new NFT)" v={BURN_MINT_TEXT} />
            <Fact k="NFT order" v="Collection order, public" />
          </dl>
          <button type="button" className="btn btn-primary btn-lg w-full" disabled data-testid="burn-submit">Burn {formatUnits(ratio, 0)} {v.symbol ?? ""}</button>
          <p className="text-dim text-xs"><b className="text-fg">There&apos;s no way back.</b> A burned NFT can&apos;t be converted into tokens again, and nobody can re-roll it. Burn launches are coming soon.</p>
        </section>
        <section className="card space-y-3 p-5" aria-labelledby="order-h" data-testid="collection-order">
          <div className="flex items-start justify-between">
            <div>
              <h2 id="order-h" className="font-semibold">Collection order</h2>
              <p className="text-muted text-xs">Who gets what is public before anyone burns</p>
            </div>
            <span className="tag">No randomness</span>
          </div>
          <div className="progress"><span style={{ width: `${size ? (v.minted / size) * 100 : 0}%` }} /></div>
          <p className="text-muted text-xs">{v.minted} of {size} minted <Ex on={v.example} /></p>
          <ul className="text-sm">
            {[0, 1, 2, 3, 4].map((k) => (
              <li key={k} className="fact"><span>#{String(v.minted + k).padStart(4, "0")}</span><span className="text-muted">{k === 0 ? "Next" : `in ${k}`}</span></li>
            ))}
          </ul>
        </section>
      </div>
      <section className="card space-y-2 p-5" aria-labelledby="sb-h" data-testid="supply-burned">
        <h2 id="sb-h" className="font-semibold">Supply and rarity</h2>
        <p className="num font-mono text-sm">
          Supply now = 1,000,000,000 − burned = <span className="text-accent-text">{formatUnits(v.supplyWhole, 0)}</span> <Ex on={v.example} />
        </p>
        <p className="text-muted text-sm">Most that can ever burn: {formatUnits(maxBurn, 0)} ({Number((maxBurn * 10000n) / TOTAL_SUPPLY_WHOLE) / 100}%), so supply never drops below {formatUnits(TOTAL_SUPPLY_WHOLE - maxBurn, 0)}. Rarity is cosmetic; Burn NFTs have no token backing and no floor.</p>
      </section>
    </>
  );
}

export function TradeNfts() {
  return (
    <section className="card space-y-3 p-5" aria-labelledby="tn-h" data-testid="trade-nfts">
      <h2 id="tn-h" className="font-semibold">Trade NFTs</h2>
      <p className="text-muted text-sm">Armory handles the curve and converting. NFT listings and sales happen on marketplaces, which only show NFTs minted so far.</p>
      <div className="flex flex-wrap gap-2">
        <span className="btn" aria-disabled="true">Trade on Tensor ↗</span>
        <span className="btn" aria-disabled="true">Trade on Magic Eden ↗</span>
      </div>
      <p className="text-dim text-xs">Collection links appear once the collection is listed.</p>
    </section>
  );
}

export function FeesControl({ v }: { v: TokenView }) {
  return (
    <section className="card space-y-2 p-5" aria-labelledby="fc-h" data-testid="fees-control">
      <h2 id="fc-h" className="font-semibold">Fees and control</h2>
      <dl>
        <Fact k="Curve trade fee" v="1% · Meteora DBC platform config (devnet)" />
        {v.type !== "plain" && <Fact k={`Platform fee per ${v.type === "burn" ? "burn" : "capture or re-roll"}`} v={v.feeLamports !== null ? formatSol(v.feeLamports) : "—"} />}
        {v.type === "hybrid" && <Fact k="Mint deposit, per capture or re-roll" v={`${formatSol(MINT_ESCROW_LAMPORTS, 4)}, refunded except ${FIRST_MINT_RANGE_TEXT} if the NFT is minted new`} />}
        {v.type === "hybrid" && <Fact k="Converting back" v="No platform fee" />}
        {v.type === "burn" && <Fact k="Mint cost, every burn" v={BURN_MINT_TEXT} />}
        <Fact k="Program upgrades" v="Not locked yet · 1 dev key per program · 3-of-5 multisig + 7-day delay planned for mainnet" />
      </dl>
    </section>
  );
}

export function NobodyCanChange({ v }: { v: TokenView }) {
  const items = ["Supply: 1,000,000,000 at launch, mint authority revoked", "Freeze authority revoked", "No transfer tax", "Unsold curve tokens are locked by the program, not burned"];
  if (v.type === "plain") items.push("No update instruction exists for a Plain launch", "No NFT side, ever");
  if (v.type === "hybrid") items.push("Conversion rate and platform fee (written once at launch)", "Release always returns exactly the ratio");
  if (v.type === "burn") items.push("Burn rate and platform fee (written once at launch)", "Collection order (committed before launch)");
  return (
    <section className="card space-y-2 p-5" aria-labelledby="nc-h" data-testid="nobody-can-change">
      <h2 id="nc-h" className="font-semibold">What nobody can change</h2>
      <ul className="space-y-1 text-sm">
        {items.map((x) => <li key={x}><span className="text-positive-text" aria-hidden="true">✓ </span>{x}</li>)}
      </ul>
    </section>
  );
}

export function FactsCard({ v }: { v: TokenView }) {
  const supplyFixed = v.type !== "burn";
  return (
    <section className="card p-5" aria-labelledby="facts-h" data-testid="facts-card">
      <h2 id="facts-h" className="mb-2 font-semibold">Facts {v.example ? <Ex on /> : <span className="text-dim text-xs font-normal">read from chain</span>}</h2>
      <dl>
        <Fact k="Launch type" v={v.type === "hybrid" ? "Hybrid · SPL-404" : v.type === "burn" ? "Burn · one-way" : "Plain · just the coin"} />
        <Fact k="Phase" v={v.phase === "graduated" ? "Graduated · DAMM v2" : v.phase === "curve" ? "On curve" : "Native test launch"} />
        <Fact k="Supply" v={`${formatUnits(v.supplyWhole, 0)}${supplyFixed ? " · fixed" : " now"}`} />
        <Fact k="Mint authority" v={v.mintAuthority ? `Present: ${shortAddr(v.mintAuthority)}` : "✓ Revoked"} ok={!v.mintAuthority} testId="fact-mint-authority" />
        <Fact k="Freeze authority" v={v.freezeAuthority ? `Present: ${shortAddr(v.freezeAuthority)}` : "✓ Revoked"} ok={!v.freezeAuthority} testId="fact-freeze-authority" />
        <Fact k="Tax" v="No transfer tax" />
        {v.ratioWhole !== null && <Fact k={v.type === "burn" ? "Burn rate" : "Convert rate"} v={`${formatUnits(v.ratioWhole, 0)} : 1 NFT`} />}
        {v.collectionSize !== null && <Fact k="Collection" v={`${v.collectionSize.toLocaleString("en-US")} NFTs · ${v.minted} minted`} />}
        {v.type === "burn" && <Fact k="NFT order" v="Collection order, public" />}
        {v.feeLamports !== null && <Fact k="Platform fee" v={`${formatSol(v.feeLamports)} · ${v.type === "burn" ? "burn" : "capture, re-roll"}`} ok={v.feeIsExactTier} testId="fact-fee" />}
        {v.type === "hybrid" && <Fact k="Converting back" v="No platform fee" />}
        {v.type === "burn" && <Fact k="Converting back" v="Not possible" />}
        {v.type !== "plain" && <Fact k="NFT standard" v="Metaplex Core" />}
        {v.type === "hybrid" && <Fact k="Randomness" v="Switchboard On-Demand · committed art" />}
        <Fact k="Graduation target" v={formatSol(v.graduationLamports, 2)} />
        <Fact k="Audit" v="Not yet · unaudited beta" ok={false} />
      </dl>
      {!v.feeIsExactTier && v.feeLamports !== null && (
        <p role="alert" className="text-warning mt-2 text-xs" data-testid="fee-not-tier">The stored fee is not the exact tier for this ratio. Armory blocks captures for this launch.</p>
      )}
      <p className="text-dim mt-2 text-xs">Unsold curve tokens are locked by the program, not burned{supplyFixed ? "; supply stays 1,000,000,000" : ""}.</p>
    </section>
  );
}

export function HoldingsTokensOnly() {
  return (
    <section className="card space-y-2 p-5" aria-labelledby="ho-h" data-testid="holdings-card">
      <h2 id="ho-h" className="font-semibold">Your holdings</h2>
      <p className="text-muted text-sm">Tokens only (no NFT side). Connect a wallet: your SOL and token balances show in the Trade panel.</p>
    </section>
  );
}

export function BurnHoldings() {
  return (
    <section className="card space-y-2 p-5" aria-labelledby="bh-h" data-testid="holdings-card">
      <h2 id="bh-h" className="font-semibold">Your holdings</h2>
      <p className="text-muted text-sm">Tokens, plus Burn NFTs shown as <b className="text-fg">not valued</b>: they have no token backing, so they don&apos;t count toward your total.</p>
    </section>
  );
}
