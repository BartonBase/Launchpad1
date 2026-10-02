"use client";
/**
 * Hybrid token panels (design: token-graduated, token-capture, token-release, token-reroll):
 * Convert (Tokens → NFT with a review step, NFT → tokens), Your holdings, Your NFTs + re-roll,
 * Floor and rarity. Every action goes through the safe-send pipeline. Expired pending requests get a
 * permissionless expire/refund; live ones offer Reveal (Switchboard gateway, allowlisted) and Settle (mints the
 * picked NFT when the launch has a traits manifest); both go through the same safe-send preview.
 */
import { useMemo, useState } from "react";
import { PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { MINT_ESCROW_LAMPORTS, RAND_LOCK_ACCOUNT_BYTES, REQUEST_ACCOUNT_BYTES, tierFeeLamports } from "@/config/armory";
import { useChainRead } from "@/hooks/useChain";
import { buildCaptureTx, buildExpireTx, buildRerollTx, buildRevealTx, buildSettleTx, buildUnwrapTx } from "@/lib/armory/builders";
import { checkBalance, estimateRentExempt, requestCost } from "@/lib/armory/fees";
import { formatSol, formatTokens, shortAddr } from "@/lib/armory/format";
import { fetchHoldings, fetchUserRequests, type LaunchDTO } from "@/lib/armory/reads";
import { useSafeSend, type BuildFn } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";
import { CostBreakdown } from "./CostBreakdown";
import { MascotPlaceholder } from "./MascotPlaceholder";

export type HybridPanel = "capture" | "release" | "reroll" | null;

function Steps({ items }: { items: readonly (readonly [string, string])[] }) {
  return (
    <ol className="space-y-2.5">
      {items.map(([t, d], i) => (
        <li key={t} className="flex gap-3 text-sm">
          <span className={`flex size-6 shrink-0 items-center justify-center rounded-full border text-xs ${i === 0 ? "border-accent-border text-accent-text" : "border-border text-muted"}`}>{i + 1}</span>
          <span>
            <b className="font-semibold">{t}</b>
            <span className="text-muted block">{d}</span>
          </span>
        </li>
      ))}
    </ol>
  );
}

export function HybridPanels({ launch, initialPanel = null }: { launch: LaunchDTO; initialPanel?: HybridPanel }) {
  const { publicKey, connected } = useWallet();
  const safeSend = useSafeSend();
  const [tab, setTab] = useState<"capture" | "release">(initialPanel === "release" ? "release" : "capture");
  const [review, setReview] = useState(initialPanel === "capture");
  const [releasePick, setReleasePick] = useState<number | null>(null);
  const [rerollPick, setRerollPick] = useState<number | null>(null);
  const [nonce, setNonce] = useState(0);
  const user = publicKey?.toBase58() ?? null;
  const ratioBase = BigInt(launch.ratioBase);
  const ratioLabel = formatTokens(ratioBase, launch.decimals, 0);
  const name = launch.collectionName ?? "this collection";
  const vault = launch.vault ? new PublicKey(launch.vault) : null;

  const balance = useChainRead(user && `bal:${user}`, (c) => c.getBalance(new PublicKey(user!)).then(BigInt), nonce);
  const holdings = useChainRead(user && `hold:${user}:${launch.mint}`, (c) => fetchHoldings(c, new PublicKey(user!), launch), nonce);
  const requests = useChainRead(user && launch.vault ? `rq:${user}:${launch.vault}` : null, (c) => fetchUserRequests(c, new PublicKey(user!), launch.vault!), nonce);
  const slot = useChainRead(user ? "slot" : null, (c) => c.getSlot(), nonce);
  const rent = useChainRead("rent:request", async (c) => {
    const [a, b] = await Promise.all([c.getMinimumBalanceForRentExemption(REQUEST_ACCOUNT_BYTES), c.getMinimumBalanceForRentExemption(RAND_LOCK_ACCOUNT_BYTES)]);
    return BigInt(a + b);
  });
  const cost = useMemo(
    () =>
      requestCost({
        tierFeeLamports: BigInt(launch.feeLamports), // read from chain (LaunchConfig.fee_lamports)
        tempRentLamports: rent.data ?? estimateRentExempt(REQUEST_ACCOUNT_BYTES) + estimateRentExempt(RAND_LOCK_ACCOUNT_BYTES),
        depositLamports: MINT_ESCROW_LAMPORTS,
      }),
    [launch.feeLamports, rent.data],
  );
  let expectedTier: bigint | null = null;
  try {
    expectedTier = tierFeeLamports(Number(launch.ratioWholeTokens));
  } catch {
    expectedTier = null;
  }

  const nfts = holdings.data?.nftIndexes ?? [];
  const tokenBase = holdings.data?.tokenBase;
  const solShort = balance.data !== undefined && !checkBalance(balance.data, cost).ok;
  const common = !launch.vaultOpen ? (launch.state === "native" ? "This native devnet test launch has no curve, so converting can never open." : "Converting opens when the token graduates.") : !connected ? "Connect a wallet to convert." : null;
  const tierBlock = !launch.feeIsExactTier
    ? `Stored fee ${formatSol(launch.feeLamports)} isn't the exact tier for this ratio${expectedTier !== null ? ` (${formatSol(expectedTier)})` : ""}. Capturing is disabled.`
    : null;
  const captureBlock =
    common ?? tierBlock ?? (tokenBase !== undefined && tokenBase < ratioBase ? `You need ${ratioLabel} tokens to capture one NFT.` : null) ?? (solShort ? "Not enough SOL (see above)." : null);
  const releaseBlock = common ?? (releasePick === null ? "Pick one of your NFTs to release." : null);
  const rerollBlock = common ?? tierBlock ?? (rerollPick === null ? "Pick one of your NFTs to re-roll." : null) ?? (solShort ? "Not enough SOL (see above)." : null);

  const run = (b: BuildFn) => void safeSend.start(b);
  const blocked = (reason: string | null, id: string) =>
    reason ? (
      <p className="text-muted text-sm" data-testid={`${id}-blocked-reason`}>
        {reason}
      </p>
    ) : null;
  const pick = (prefix: "release" | "reroll", value: number | null, onPick: (i: number) => void) =>
    nfts.length === 0 ? (
      <p className="text-muted text-sm">{!connected ? "Connect a wallet to see your NFTs." : holdings.loading ? "Loading your NFTs…" : `No ${name} NFTs in this wallet.`}</p>
    ) : (
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
        {nfts.map((i) => (
          <label key={i} className={`rounded-panel cursor-pointer border p-2 ${value === i ? "border-accent-border bg-accent-tint" : "border-border"}`}>
            <input type="radio" name={`${prefix}-nft`} className="sr-only" checked={value === i} onChange={() => onPick(i)} data-testid={`${prefix}-nft-option-${i}`} />
            <span className="bg-surface-2 rounded-chip mb-2 flex aspect-square items-center justify-center" aria-hidden="true">
              <span className="text-dim font-mono text-xs">art</span>
            </span>
            <span className="num text-sm font-semibold">#{String(i).padStart(4, "0")}</span>
          </label>
        ))}
      </div>
    );

  return (
    <>
      <div className="grid gap-6 md:grid-cols-2">
        <section className="card space-y-4 p-5" aria-labelledby="convert-h" data-testid="convert-panel" data-panel={tab === "capture" && review ? "capture-review" : tab}>
          <div className="flex items-start justify-between gap-2">
            <div>
              <h2 id="convert-h" className="font-semibold">Convert</h2>
              <p className="text-muted text-xs">{tab === "release" ? "Release an NFT back to tokens" : review ? "Review capture" : "Tokens and NFTs, exact both ways"}</p>
            </div>
            {tab === "release" ? <span className="tag tag-ok"><i />Free</span> : launch.vaultOpen ? <span className="tag tag-ok"><i />Open</span> : <span className="tag">Locked</span>}
          </div>
          <div role="tablist" aria-label="Convert direction" className="seg">
            {(["capture", "release"] as const).map((x) => (
              <button key={x} role="tab" type="button" aria-selected={tab === x} onClick={() => { setTab(x); setReview(false); }} data-testid={`tab-${x}`}>
                {x === "capture" ? "Tokens → NFT" : "NFT → tokens"}
              </button>
            ))}
          </div>

          {!launch.vaultOpen && (
            <div className="flex items-center gap-4">
              <MascotPlaceholder size={88} />
              <p className="text-muted text-sm">{common}</p>
            </div>
          )}

          {tab === "capture" && !review && (
            <div className="space-y-3" role="tabpanel" aria-label="Tokens to NFT">
              <div className="bg-surface-2 rounded-panel border-border border p-4">
                <p className="text-muted flex justify-between text-xs">You give <span data-testid="wallet-token-balance">Balance {tokenBase !== undefined ? formatTokens(tokenBase, launch.decimals) : "—"}</span></p>
                <p className="num text-2xl font-semibold">{ratioLabel}</p>
              </div>
              <div className="bg-surface-2 rounded-panel border-border border p-4">
                <p className="text-muted text-xs">You receive</p>
                <p className="text-2xl font-semibold">1 NFT</p>
                <p className="text-dim text-xs">Picked by Switchboard VRF · takes a few seconds</p>
              </div>
              <dl>
                <div className="fact"><dt>Rate</dt><dd>{ratioLabel} = 1 NFT</dd></div>
                <div className="fact"><dt>Platform fee</dt><dd data-testid="convert-fee">{formatSol(launch.feeLamports)}</dd></div>
                <div className="fact"><dt>Mint deposit (refundable)</dt><dd>{formatSol(cost.deposit)}</dd></div>
                <div className="fact"><dt>Tokens taken</dt><dd>None</dd></div>
              </dl>
              {blocked(captureBlock === "Not enough SOL (see above)." ? null : captureBlock, "capture-entry")}
              <button type="button" className="btn btn-primary btn-lg w-full" disabled={captureBlock !== null && captureBlock !== "Not enough SOL (see above)."} onClick={() => setReview(true)} data-testid="capture-start">
                Convert to 1 NFT
              </button>
            </div>
          )}

          {tab === "capture" && review && (
            <div className="space-y-4" role="tabpanel" aria-label="Review capture">
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="bg-surface-2 rounded-panel border-border border p-4">
                  <p className="text-muted text-xs">You lock</p>
                  <p className="num text-lg font-semibold">{ratioLabel} tokens</p>
                  <p className="text-muted text-xs">Held in the vault, not spent. You get them back when you release the NFT.</p>
                </div>
                <div className="bg-surface-2 rounded-panel border-border border p-4">
                  <p className="text-muted text-xs">You get</p>
                  <p className="text-lg font-semibold">1 random NFT</p>
                  <p className="text-muted text-xs">Picked from the {launch.collectionSize - Number(launch.assetsOutside)} not held in wallets.</p>
                </div>
              </div>
              <CostBreakdown prefix="capture" cost={cost} balance={connected ? balance.data : undefined} feeContext={`Set for ${name} at launch (${ratioLabel} ratio)`} />
              <Steps items={[["Sign in your wallet", `Locks ${ratioLabel} tokens and pays the fee + deposit`], ["Randomness arrives", "Switchboard VRF picks your NFT, usually within seconds"], ["Your NFT is revealed", "Minted to your wallet if it's new; the unused deposit comes back"]]} />
              {blocked(captureBlock, "capture")}
              <div className="flex gap-3">
                <button type="button" className="btn btn-primary btn-lg flex-1" disabled={captureBlock !== null || safeSend.busy || !vault} onClick={() => run(({ connection, payer }) => buildCaptureTx(connection, payer, vault!))} data-testid="capture-submit">
                  Confirm and sign
                </button>
                <button type="button" className="btn btn-lg" onClick={() => setReview(false)} data-testid="capture-back">Back</button>
              </div>
              <p className="text-dim text-xs">Rarity is cosmetic. Whatever you get converts back for exactly {ratioLabel} tokens. The mint cost is Solana rent plus the Metaplex Core fee, not an Armory fee.</p>
            </div>
          )}

          {tab === "release" && (
            <div className="space-y-3" role="tabpanel" aria-label="NFT to tokens">
              <div className="bg-surface-2 rounded-panel border-border border p-3">
                <p className="text-muted mb-2 flex justify-between text-xs">You give <span>{nfts.length} in wallet</span></p>
                {pick("release", releasePick, setReleasePick)}
              </div>
              <div className="bg-surface-2 rounded-panel border-border border p-4">
                <p className="text-muted flex justify-between text-xs">You receive <span>Exact, whatever the traits</span></p>
                <p className="num text-2xl font-semibold">{ratioLabel}</p>
              </div>
              <dl data-testid="release-cost">
                {["Platform fee", "Mint cost", "Randomness", "Tokens taken"].map((k) => (
                  <div key={k} className="fact"><dt>{k}</dt><dd>None</dd></div>
                ))}
                <div className="fact"><dt>Network fee</dt><dd>Shown in the preview</dd></div>
              </dl>
              {blocked(releaseBlock, "release")}
              <button type="button" className="btn btn-primary btn-lg w-full" disabled={releaseBlock !== null || safeSend.busy || !vault} onClick={() => run(({ connection, payer }) => buildUnwrapTx(connection, payer, vault!, releasePick!))} data-testid="release-submit">
                {releasePick !== null ? `Release #${String(releasePick).padStart(4, "0")} for ${ratioLabel}` : "Release"}
              </button>
              <p className="text-dim text-xs"><b className="text-fg">Release is free.</b> The NFT goes back into the pool and can be picked by a future capture or re-roll.</p>
            </div>
          )}

          {safeSend.status === "error" && !safeSend.preview && <p role="alert" className="text-negative text-sm break-words" data-testid="convert-error">{safeSend.error}</p>}
          {safeSend.status === "success" && (
            <div role="status" className="tag-ok rounded-panel border p-3 text-sm" data-testid="convert-success">
              Submitted. Waiting for randomness and settle where applicable.{" "}
              <button type="button" className="underline" onClick={() => setNonce((n) => n + 1)}>Refresh</button>
            </div>
          )}
          {(requests.data?.length ?? 0) > 0 && (
            <div className="space-y-1" data-testid="pending-requests">
              <p className="eyebrow">Your pending requests</p>
              <ul className="text-sm">
                {requests.data!.map((r) => {
                  const expired = slot.data !== undefined && !r.revealed && BigInt(slot.data) > BigInt(r.deadlineSlot);
                  return (
                    <li key={r.address} className="fact" data-testid={`pending-request-${r.seq}`}>
                      <span>#{r.seq} · {r.kind === "capture" ? "Capture" : "Re-roll"} · <span className="text-muted">{r.revealed ? "Revealed, waiting for settle" : expired ? "Expired" : "Waiting for randomness"}</span></span>
                      {!expired && !r.revealed && (
                        <button type="button" className="btn btn-sm" disabled={safeSend.busy} onClick={() => run(({ connection, payer }) => buildRevealTx(connection, payer, new PublicKey(r.address)))} data-testid={`reveal-${r.seq}`}>
                          Reveal
                        </button>
                      )}
                      {r.revealed && (
                        <button type="button" className="btn btn-sm btn-primary" disabled={safeSend.busy} onClick={() => run(async ({ connection, payer }) => (await buildSettleTx(connection, payer, new PublicKey(r.address))).tx)} data-testid={`settle-${r.seq}`}>
                          Settle
                        </button>
                      )}
                      {expired && (
                        <button type="button" className="btn btn-sm" disabled={safeSend.busy} onClick={() => run(({ connection, payer }) => buildExpireTx(connection, payer, vault!, new PublicKey(r.address)))} data-testid={`expire-${r.seq}`}>
                          Expire &amp; refund {formatSol(r.mintEscrowLamports, 4)}
                        </button>
                      )}
                    </li>
                  );
                })}
              </ul>
            </div>
          )}
        </section>

        <section className="card space-y-4 p-5" aria-labelledby="hold-h" data-testid="holdings-card">
          <div>
            <h2 id="hold-h" className="font-semibold">Your holdings</h2>
            <p className="text-dim font-mono text-xs">{user ? shortAddr(user) : "Not connected"}</p>
          </div>
          <p className="hd num text-4xl" data-testid="wallet-sol-balance">{balance.data !== undefined ? formatSol(balance.data, 3) : "—"}</p>
          <dl>
            <div className="fact"><dt>Tokens</dt><dd>{tokenBase !== undefined ? formatTokens(tokenBase, launch.decimals) : "—"}</dd></div>
            <div className="fact"><dt>NFTs</dt><dd>{nfts.length} · {ratioLabel} tokens each at release</dd></div>
          </dl>
          {tokenBase !== undefined && tokenBase >= ratioBase && launch.vaultOpen && (
            <p className="border-accent-border bg-accent-tint text-accent-text rounded-panel border p-3 text-sm">Tokens available to convert: {(tokenBase / ratioBase).toString()} NFTs</p>
          )}
          <div className="grid grid-cols-2 gap-2">
            <div className="bg-surface-2 rounded-panel border-border border p-3"><p className="text-muted text-xs">Captured so far</p><p className="num text-xl font-semibold">{launch.mintedCount}</p><p className="text-dim text-xs">of {launch.collectionSize}</p></div>
            <div className="bg-surface-2 rounded-panel border-border border p-3"><p className="text-muted text-xs">Not yet minted</p><p className="num text-xl font-semibold">{launch.collectionSize - launch.mintedCount}</p><p className="text-dim text-xs">minted on capture</p></div>
          </div>
        </section>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <section className="card space-y-4 p-5" aria-labelledby="nfts-h" data-testid="your-nfts" id="reroll">
          <div>
            <h2 id="nfts-h" className="font-semibold">Your NFTs</h2>
            <p className="text-muted text-xs">{nfts.length} {name} in this wallet</p>
          </div>
          {pick("reroll", rerollPick, setRerollPick)}
          {(rerollPick !== null || initialPanel === "reroll") && (
            <div className="border-accent-border space-y-3 rounded-[var(--arm-radius-panel)] border p-4">
              <h3 className="font-semibold">Re-roll {rerollPick !== null ? `#${String(rerollPick).padStart(4, "0")}` : "an NFT"}</h3>
              <p className="text-muted text-xs">Swap it for a different random NFT. The pick can be one not minted yet; the result may be more common. It still converts back for exactly {ratioLabel} tokens.</p>
              <CostBreakdown prefix="reroll" cost={cost} balance={connected ? balance.data : undefined} feeContext="Same as a capture" />
              <Steps items={[["Sign in your wallet", "The NFT goes back to the pool and you pay the fee + deposit"], ["Randomness arrives", "Switchboard VRF picks a different NFT"], ["Your new NFT is revealed", "Minted to your wallet if it's new"]]} />
              {blocked(rerollBlock, "reroll")}
              <div className="flex gap-3">
                <button type="button" className="btn btn-primary btn-lg flex-1" disabled={rerollBlock !== null || safeSend.busy || !vault} onClick={() => run(({ connection, payer }) => buildRerollTx(connection, payer, vault!, rerollPick!))} data-testid="reroll-submit">
                  Confirm re-roll
                </button>
                <button type="button" className="btn btn-lg" onClick={() => setRerollPick(null)} data-testid="reroll-cancel">Cancel</button>
              </div>
            </div>
          )}
        </section>
        <section className="card space-y-3 p-5" aria-labelledby="floor-h" data-testid="floor-card">
          <h2 id="floor-h" className="font-semibold">Floor and rarity</h2>
          <div className="bg-surface-2 rounded-panel border-border border p-4">
            <p className="text-muted text-xs">Floor = ratio × token price</p>
            <p className="num font-mono">{ratioLabel} × token price</p>
            <p className="text-dim text-xs">Live token price comes with the Meteora DBC / DEX integration.</p>
          </div>
          <p className="text-sm"><b>Every NFT converts back for exactly {ratioLabel} tokens, whatever its rarity.</b> <span className="text-muted">Rarity is cosmetic; marketplace prices are set by buyers and sellers.</span></p>
        </section>
      </div>
      <TxPreviewModal safeSend={safeSend} />
    </>
  );
}
