"use client";
/**
 * Launch wizard (design: a-obsidian/launch.html, launch-plain.html, launch-burn.html; math:
 * NOTE.md "Wizard math" via lib/armory/wizardMath). Steps renumber per type: Plain 1–4, Hybrid
 * and Burn 1–5. Only Hybrid can launch on devnet (native `launch`). Plain and Burn follow their
 * designs but are "Pending deploy": no builders, launch disabled. Tax split and Raffle are
 * "Coming soon" and can't be selected.
 */
import Link from "next/link";
import { useMemo, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import type { PublicKey } from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import {
  BETA_DEPOSIT_CAP,
  DEFAULT_GRADUATION_SOL,
  LAUNCH_TYPES,
  MIN_GRADUATION_LAMPORTS,
  STATUS_LABEL,
  launchTypeStatus,
  maxCollectionSize,
  tierFeeLamports,
} from "@/config/armory";
import { buildDbcLaunchTx, buildNativeLaunchTx, fetchDbcGraduationLamports } from "@/lib/armory/builders";
import { DBC_PLATFORM_CONFIG } from "@/config/integrations";
import { useChainRead } from "@/hooks/useChain";
import { compact, formatSol, formatUnits } from "@/lib/armory/format";
import { LAUNCH_DECIMALS, validateLaunchForm, type LaunchForm } from "@/lib/armory/launchForm";
import { EXAMPLE_CURVE_SPLIT, EXAMPLE_SOL_USD, designSol, type WizardType } from "@/lib/armory/wizardMath";
import { useSafeSend } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";
import { TypeIcon } from "./TypeIcon";

const GROUPS = [
  { label: "Large collections", hint: "50K to 200K: more NFTs, each one cheaper", ratios: [50_000, 100_000, 200_000] },
  { label: "Small, scarce collections", hint: "500K to 5M: fewer NFTs, each one takes more tokens", ratios: [500_000, 1_000_000, 2_500_000, 5_000_000] },
] as const;
const TNAME: Record<WizardType, string> = { plain: "Plain", hybrid: "Hybrid", burn: "Burn" };

function steps(t: WizardType): string[] {
  return t === "plain" ? ["Launch type", "Basics", "Token & curve", "Review & launch"] : ["Launch type", "Basics", "Supply & conversion", "Art & randomness", "Review & launch"];
}

function Row({ k, val, testId }: { k: string; val: React.ReactNode; testId?: string }) {
  return (
    <div className="fact">
      <dt>{k}</dt>
      <dd data-testid={testId}>{val}</dd>
    </div>
  );
}

export function LaunchWizard({ initialType = "hybrid" }: { initialType?: WizardType }) {
  const { connected } = useWallet();
  const safeSend = useSafeSend();
  const [step, setStep] = useState(0);
  const [ack, setAck] = useState(false);
  const [form, setForm] = useState<LaunchForm>({ type: initialType, name: "", symbol: "", ratio: 1_000_000, collectionSize: "500", graduationSol: String(DEFAULT_GRADUATION_SOL) });
  const [launchedMint, setLaunchedMint] = useState<PublicKey | null>(null);
  const v = useMemo(() => validateLaunchForm(form, MIN_GRADUATION_LAMPORTS[CLUSTER.name]), [form]);
  const m = v.math;
  const set = <K extends keyof LaunchForm>(k: K, val: LaunchForm[K]) => setForm((f) => ({ ...f, [k]: val }));
  const t = form.type;
  const isPlain = t === "plain";
  const isBurn = t === "burn";
  const live = launchTypeStatus(t, CLUSTER.name) === "live";
  const names = steps(t);
  const last = names.length - 1;
  const fee = m.feeLamports;
  const ratioLabel = formatUnits(BigInt(form.ratio), 0);
  const sizeLabel = (Number(form.collectionSize.replace(/,/g, "")) || 0).toLocaleString("en-US");

  const stepOk = (i: number): boolean => {
    const name = names[i];
    if (name === "Launch type") return true;
    if (name === "Basics") return !v.errors.name && !v.errors.symbol;
    if (name === "Supply & conversion" || name === "Token & curve") return m.valid;
    return true;
  };

  const launch = () => {
    if (!v.params || t !== "hybrid") return; // plain/burn: pending deploy, no builders
    const params = v.params;
    void safeSend.start(({ payer }) => {
      const b = buildNativeLaunchTx(payer, params);
      setLaunchedMint(b.mint);
      return { tx: b.tx, signers: b.signers };
    });
  };

  // Meteora DBC path (devnet, simulate-only): the graduation target is the pinned DBC config's
  // migration_quote_threshold read from chain, not the form value.
  const dbcAvailable = t === "hybrid" && DBC_PLATFORM_CONFIG[CLUSTER.name] !== null;
  const dbcGrad = useChainRead(dbcAvailable ? "dbc:grad" : null, (c) => fetchDbcGraduationLamports(c));
  const previewDbc = () => {
    if (!v.params || !dbcAvailable) return;
    const params = v.params;
    void safeSend.start(
      async ({ connection, payer }) => {
        const b = await buildDbcLaunchTx(connection, payer, { name: form.name.trim(), symbol: form.symbol.trim(), uri: "", ratioWholeTokens: params.ratioWholeTokens, collectionSize: params.collectionSize });
        return { tx: b.tx, signers: b.signers };
      },
      { previewOnly: "the Meteora DBC launch is simulated so you can review it; sending is not enabled yet." },
    );
  };

  const err = (k: keyof typeof v.errors, show = true) =>
    show && v.errors[k] ? (
      <p id={`${k}-err`} className="text-warning mt-1 text-xs" data-testid={`launch-error-${k}`}>
        {v.errors[k]}
      </p>
    ) : null;


  return (
    <div className="grid gap-6 lg:grid-cols-[200px_minmax(0,1fr)_340px]">
      <ol className="flex gap-1 overflow-x-auto lg:flex-col" aria-label="Steps">
        {names.map((s, i) => (
          <li key={s}>
            <button
              type="button"
              onClick={() => i <= step && setStep(i)}
              aria-current={i === step ? "step" : undefined}
              className={`rounded-chip flex w-full items-center gap-2 px-3 py-2 text-left text-sm whitespace-nowrap ${i === step ? "bg-surface-2 text-fg" : "text-muted"}`}
              data-testid={`launch-step-${i + 1}`}
            >
              <span className={`flex size-6 items-center justify-center rounded-full border text-xs ${i < step ? "border-positive-text text-positive-text" : i === step ? "border-accent-border text-accent-text" : "border-border"}`}>
                {i < step ? "✓" : i + 1}
              </span>
              {s}
            </button>
          </li>
        ))}
      </ol>

      <div className="card min-w-0 space-y-6 p-5 md:p-6">
        <div className="flex items-center justify-between gap-2">
          <p className="eyebrow">Step {step + 1}</p>
          {(names[step] === "Launch type" || names[step] === "Supply & conversion" || names[step] === "Token & curve") && <span className="tag tag-demo">Permanent</span>}
        </div>

        {names[step] === "Launch type" && (
          <fieldset className="space-y-4">
            <legend className="mb-1 text-lg font-semibold">Launch type</legend>
            <p className="text-muted text-sm">The type decides what holders can do with your token. It&apos;s fixed on-chain once you launch.</p>
            <div className="grid gap-3 sm:grid-cols-3">
              {LAUNCH_TYPES.filter((x) => x.id === "plain" || x.id === "hybrid" || x.id === "burn").map((x) => {
                const st = launchTypeStatus(x.id, CLUSTER.name);
                const on = t === x.id;
                return (
                  <label key={x.id} className={`rounded-panel cursor-pointer border p-4 ${on ? "border-accent-border bg-accent-tint" : "border-border"}`} data-testid={`launch-type-option-${x.id}`} data-status={st}>
                    <input type="radio" name="type" className="sr-only" checked={on} onChange={() => set("type", x.id as WizardType)} />
                    <TypeIcon type={x.id} className="text-accent-text" />
                    <span className="mt-2 block font-semibold">{x.name}</span>
                    <span className="text-accent-text block text-xs">{x.short}</span>
                    <span className="text-muted mt-1 block text-xs">{x.description}</span>
                    <span className={`tag mt-2 ${st === "live" ? "tag-ok" : "tag-demo"}`}>{STATUS_LABEL[st]}</span>
                  </label>
                );
              })}
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              {LAUNCH_TYPES.filter((x) => x.id === "tax" || x.id === "raffle").map((x) => (
                <div key={x.id} aria-disabled="true" className="card-soon rounded-panel flex gap-3 border p-3" data-testid={`launch-type-option-${x.id}`} data-status="coming-soon">
                  <TypeIcon type={x.id} size={20} />
                  <div>
                    <p className="text-sm font-medium">
                      {x.name} <span className="tag tag-soon ml-1">Coming soon</span>
                    </p>
                    <p className="text-xs">{x.description}</p>
                  </div>
                </div>
              ))}
            </div>
            <p className="text-dim text-xs">Tax split and Raffle aren&apos;t available yet. They&apos;re listed so you know what&apos;s planned.</p>
            {!live && (
              <p className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-pending-deploy">
                {TNAME[t]} is built but not deployed to {CLUSTER.label} yet. You can walk through the setup; launching is disabled.
              </p>
            )}
          </fieldset>
        )}

        {names[step] === "Basics" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">{isPlain ? "Token basics" : "Collection basics"}</h2>
            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <label htmlFor="name" className="mb-1 flex justify-between text-sm font-medium">
                  Name <span className="text-dim font-normal">{form.name.length} / 32</span>
                </label>
                <input id="name" className="input" value={form.name} maxLength={32} onChange={(e) => set("name", e.target.value)} aria-invalid={form.name !== "" && !!v.errors.name} aria-describedby="name-err" data-testid="launch-name" />
                {err("name", form.name !== "")}
              </div>
              <div>
                <label htmlFor="symbol" className="mb-1 flex justify-between text-sm font-medium">
                  Ticker <span className="text-dim font-normal">3 to 6 letters</span>
                </label>
                <input id="symbol" className="input uppercase" value={form.symbol} maxLength={6} onChange={(e) => set("symbol", e.target.value.toUpperCase().replace(/[^A-Z]/g, ""))} aria-invalid={form.symbol !== "" && !!v.errors.symbol} aria-describedby="symbol-err" data-testid="launch-symbol" />
                {err("symbol", form.symbol !== "")}
              </div>
            </div>
            <div className="capslot" data-testid="launch-artwork-pending">
              {isPlain ? "Token image" : "Collection artwork"} upload and metadata come with the bonding-curve launch. On {CLUSTER.label} the test launch writes no metadata, so name and ticker are for review only.
            </div>
          </div>
        )}

        {(names[step] === "Supply & conversion" || names[step] === "Token & curve") && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold">{isPlain ? "Token and curve" : isBurn ? "Supply and burn rate" : "Supply and conversion"}</h2>
            <div className="grid gap-2 sm:grid-cols-2">
              {[
                ["Supply starts at 1,000,000,000", "Mint authority revoked at launch, so nobody can mint more."],
                ["Freeze authority revoked", "Nobody can freeze holders' tokens."],
                ...(isPlain ? [["No NFT side", "No ratio, collection, converting or platform fee."]] : [["Conversion rate set at launch", "Fixed in the launch record; no instruction can change it."]]),
                ["No transfer tax", "A classic SPL token. No tax on any transfer."],
              ].map(([a, b]) => (
                <div key={a} className="bg-surface-2 rounded-panel border-border border p-3 text-sm">
                  <p className="font-medium">🔒 {a}</p>
                  <p className="text-muted text-xs">{b}</p>
                </div>
              ))}
            </div>
            {!isPlain && (
              <>
                <p className="border-accent-border bg-accent-tint text-accent-text rounded-panel border p-3 text-sm">
                  {isBurn ? "Burning" : "Converting"} opens when your token graduates. NFTs are minted one at a time, the first time someone {isBurn ? "burns for them" : "captures them"}. The collector pays the small mint cost.{" "}
                  {isBurn ? "Each burn" : "Captures and re-rolls"} carry a {fee !== null ? formatSol(fee, 3) : ""} platform fee.{isBurn ? " There is no way back." : " Converting back is free."}
                </p>
                {GROUPS.map((g) => (
                  <fieldset key={g.label}>
                    <legend className="mb-2 flex w-full justify-between text-sm font-medium">
                      {g.label} <span className="text-dim text-xs font-normal">{g.hint}</span>
                    </legend>
                    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                      {g.ratios.map((r) => (
                        <label key={r} className={`rounded-panel cursor-pointer border p-3 ${form.ratio === r ? "border-accent-border bg-accent-tint" : "border-border"}`} data-testid={`ratio-${r}`}>
                          <input type="radio" name="ratio" className="sr-only" checked={form.ratio === r} onChange={() => set("ratio", r)} />
                          <span className="num block font-semibold">{formatUnits(BigInt(r), 0)}</span>
                          <span className="text-muted block text-xs">max {maxCollectionSize(r).toLocaleString("en-US")} NFTs</span>
                          <span className="text-dim block text-xs">≈ {designSol(r * 2.8e-8)} SOL per NFT (est.)</span>
                          <span className="text-accent-text block text-xs">Fee {formatSol(tierFeeLamports(r), 3)}</span>
                        </label>
                      ))}
                    </div>
                  </fieldset>
                ))}
              </>
            )}
            <div className="grid gap-4 sm:grid-cols-2">
              {!isPlain && (
                <div>
                  <label htmlFor="size" className="mb-1 block text-sm font-medium">Collection size</label>
                  <input id="size" inputMode="numeric" className="input num" value={form.collectionSize} onChange={(e) => set("collectionSize", e.target.value)} aria-invalid={!!v.errors.collectionSize} aria-describedby="collectionSize-err" data-testid="launch-collection-size" />
                  {err("collectionSize") ?? (
                    <p className="text-dim mt-1 text-xs">
                      {sizeLabel} × {ratioLabel} = {formatUnits(m.nftTokens, 0)} tokens, within the 1,000,000,000 supply.
                    </p>
                  )}
                </div>
              )}
              <div>
                <label htmlFor="grad" className="mb-1 block text-sm font-medium">Graduation target (SOL)</label>
                <input id="grad" inputMode="decimal" className="input num" value={form.graduationSol} onChange={(e) => set("graduationSol", e.target.value)} aria-invalid={!!v.errors.graduationSol} aria-describedby="graduationSol-err" data-testid="launch-graduation" />
                {err("graduationSol")}
                <p className="text-dim mt-1 text-xs" data-testid="launch-graduation-min">
                  SOL raised on the Meteora bonding curve before your token graduates. Minimum {formatSol(m.minTargetLamports, 1)} (on-chain floor on {CLUSTER.label}: {formatSol(MIN_GRADUATION_LAMPORTS[CLUSTER.name], 1)}).
                </p>
              </div>
            </div>
          </div>
        )}

        {names[step] === "Art & randomness" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Art, traits and randomness</h2>
            <p className="text-muted text-sm">
              Rarity is cosmetic. {isBurn ? "Every NFT costs the same to mint, whatever its traits, and none converts back to tokens." : `Every NFT converts back for exactly ${ratioLabel} tokens, whatever its traits.`}
            </p>
            <div className="capslot" data-testid="launch-traits-pending">Traits file (CSV or JSON) upload comes with the art-commitment step; not wired on {CLUSTER.label} yet.</div>
            <p className="text-sm font-medium">
              How NFTs are assigned <span className="text-dim text-xs font-normal">Fixed for this type, not a setting</span>
            </p>
            <ul className="space-y-2 text-sm">
              <li className="bg-surface-2 rounded-panel border-border border p-3">
                <b>Art and traits committed before launch.</b> <span className="text-muted">A fingerprint (Merkle root) of every NFT&apos;s art and traits is published at launch. Nobody can swap or edit a piece afterwards.</span>
              </li>
              {isBurn ? (
                <li className="bg-surface-2 rounded-panel border-border border p-3">
                  <b>NFTs go out in collection order.</b> <span className="text-muted">#0, #1, #2 and so on, one per burn. No randomness, so the next piece is always public.</span>
                </li>
              ) : (
                <li className="bg-surface-2 rounded-panel border-border border p-3">
                  <b>Switchboard VRF picks which NFT you get.</b> <span className="text-muted">A random number with a proof anyone can check, used at every capture and re-roll.</span>
                </li>
              )}
            </ul>
          </div>
        )}

        {names[step] === "Review & launch" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Review and launch</h2>
            <dl data-testid="launch-review">
              <Row k="Launch type" val={TNAME[t]} />
              <Row k="Token" val={`${form.name || "—"} · $${form.symbol || "—"}`} />
              <Row k="Supply at launch" val="1,000,000,000" />
              <Row k="Graduation target" val={`${form.graduationSol} SOL raised`} />
              <Row k="Bonding curve" val={t === "hybrid" ? "None on devnet test launch (Meteora DBC next)" : "Meteora DBC"} />
              <Row k="Mint authority" val="Revoked at launch" />
              <Row k="Freeze authority" val="Revoked at launch" />
              <Row k="Tax" val="No transfer tax" />
              {isPlain ? (
                <Row k="NFT collection" val="None" />
              ) : (
                <>
                  <Row k={isBurn ? "Burn rate" : "Conversion rate"} val={`${ratioLabel} tokens${isBurn ? " burned" : ""} = 1 NFT`} />
                  <Row k="Collection size" val={`${sizeLabel} NFTs (range 100 to ${m.maxSize.toLocaleString("en-US")})`} />
                  <Row k={isBurn ? "Most that can be burned" : "Max in NFT form"} val={`${formatUnits(m.nftTokens, 0)} (${m.nftPct}%)${isBurn ? `, supply ≥ ${formatUnits(m.restTokens, 0)}` : ""}`} testId="launch-review-nft-tokens" />
                  <Row k="NFT minting" val={isBurn ? "On each burn" : "On first capture"} />
                  <Row k={isBurn ? "NFT order" : "Randomness"} val={isBurn ? "Collection order, public" : "Committed art + Switchboard VRF"} />
                  <Row k="Platform fee" val={`${fee !== null ? formatSol(fee, 3) : ""} per ${isBurn ? "burn" : "capture or re-roll"}`} />
                  <Row k="Converting back" val={isBurn ? "Not possible, one-way" : "No platform fee"} />
                </>
              )}
              <Row k="Program upgrades" val="Dev key now · multisig planned" />
              <Row k="Audit" val="Not yet · unaudited beta" />
            </dl>
            <div className="capslot" data-testid="launch-capslot">
              <span className="tag tag-ua">Unaudited beta</span> Deposit cap <b>{BETA_DEPOSIT_CAP.sol} SOL per wallet</b> <span className="tag tag-demo">Example, not set</span>
            </div>
            {t === "hybrid" && (
              <p className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-native-warning">
                Devnet test launch: no bonding curve is created, so this token can never graduate and converting will never open.
              </p>
            )}
            <label className="flex items-start gap-2 text-sm">
              <input type="checkbox" className="mt-1" checked={ack} onChange={(e) => setAck(e.target.checked)} data-testid="launch-ack" />
              <span className="text-muted">
                I understand that the launch type{isPlain ? "" : ", collection size and rarity settings"} can&apos;t be changed after launch, that Armory is an unaudited beta, and that the programs can still be upgraded until they&apos;re frozen after the audit (see <Link href="/trust" className="text-accent-text">Trust &amp; security</Link>).
              </span>
            </label>
            <button
              type="button"
              className="btn btn-primary btn-lg w-full"
              disabled={!live || !ack || !connected || !v.params || safeSend.busy}
              onClick={launch}
              data-testid="launch-submit"
            >
              {live ? `Launch on ${CLUSTER.label}` : `${TNAME[t]}: pending deploy`}
            </button>
            {dbcAvailable && (
              <div className="space-y-1" data-testid="launch-dbc">
                <button type="button" className="btn w-full" disabled={!ack || !connected || !v.params || safeSend.busy} onClick={previewDbc} data-testid="launch-dbc-preview">
                  Preview Meteora DBC launch (simulate only)
                </button>
                <p className="text-dim text-xs" data-testid="launch-dbc-graduation">
                  With a bonding curve, the graduation target comes from the pinned DBC config: {dbcGrad.data != null ? `${formatSol(dbcGrad.data, 1)} raised` : "loading…"}. No metadata URI is set yet.
                </p>
              </div>
            )}
            {live && !connected && <p className="text-muted text-sm">Connect a wallet to launch.</p>}
            <p className="text-dim text-xs">Test network only. No real funds.</p>
            {safeSend.status === "error" && !safeSend.preview && (
              <p role="alert" className="text-negative text-sm" data-testid="launch-error">{safeSend.error}</p>
            )}
            {safeSend.status === "success" && launchedMint && (
              <p role="status" className="text-positive-text text-sm" data-testid="launch-success">
                Launched. <Link className="underline" href={`/t/${launchedMint.toBase58()}`}>Open the token page</Link>.
              </p>
            )}
          </div>
        )}

        <div className="flex justify-between gap-3">
          <button type="button" className="btn" disabled={step === 0} onClick={() => setStep((s) => s - 1)} data-testid="launch-back">
            Back
          </button>
          {step < last && (
            <button type="button" className="btn btn-primary" disabled={!stepOk(step)} onClick={() => setStep((s) => s + 1)} data-testid="launch-next">
              Continue
            </button>
          )}
        </div>
      </div>

      <aside className="space-y-4">
        <section className="card space-y-4 p-5" aria-label="Live math" data-testid="launch-math">
          <div className="flex items-center justify-between">
            <p className="eyebrow">Live math · {TNAME[t]}</p>
            <span className="text-dim text-xs">Updates as you edit</span>
          </div>
          {isPlain ? (
            <>
              <p className="hd text-3xl">1,000,000,000</p>
              <p className="text-muted text-sm">No NFT side. Tokens trade on the curve until it reaches {form.graduationSol || "—"} SOL, then move to a DEX pool.</p>
              <div className="progress"><span style={{ width: "80%" }} /></div>
              <dl>
                <Row k="Sold on the curve" val={`${formatUnits(EXAMPLE_CURVE_SPLIT.curve, 0)} · 80%`} />
                <Row k="Set aside for the DEX pool" val={`${formatUnits(EXAMPLE_CURVE_SPLIT.dex, 0)} · 20%`} />
                <Row k="Per-NFT fees" val="None" />
              </dl>
              <p className="text-dim text-xs">Split shown is an example; the approved Meteora curve config sets it. <span className="tag tag-demo">Example</span></p>
            </>
          ) : (
            <>
              <p className="hd text-3xl" data-testid="math-size">{m.sizeOk ? `${sizeLabel} NFTs` : "—"}</p>
              <p className="text-muted text-xs">× {ratioLabel} tokens each</p>
              <p className="text-sm" data-testid="math-lead">
                {!m.sizeOk
                  ? v.errors.collectionSize
                  : isBurn
                    ? `If every NFT is minted, ${formatUnits(m.nftTokens, 0)} tokens (${m.nftPct}%) are burned for good and supply ends at ${formatUnits(m.restTokens, 0)}.`
                    : `Up to ${formatUnits(m.nftTokens, 0)} tokens (${m.nftPct}% of the fixed supply) can be in NFT form at once.${m.restTokens > 0n ? ` At least ${formatUnits(m.restTokens, 0)} always stay as tokens.` : ""}`}
              </p>
              <div className="progress"><span style={{ width: `${m.sizeOk ? m.nftPct : 100}%`, background: m.sizeOk ? undefined : "var(--arm-color-status-warn-default)" }} /></div>
              <dl>
                <Row k={isBurn ? "Can be burned" : "Can become NFTs"} val={m.sizeOk ? `${formatUnits(m.nftTokens, 0)} · ${m.nftPct}%` : "—"} testId="math-nft-tokens" />
                <Row k={isBurn ? "Never burned" : "Always tokens"} val={m.sizeOk ? `${formatUnits(m.restTokens, 0)} · ${Math.round((100 - m.nftPct) * 10) / 10}%` : "—"} testId="math-rest-tokens" />
                <Row k="Allowed size at this ratio" val={`100 to ${m.maxSize.toLocaleString("en-US")} NFTs`} testId="math-max" />
                <Row k="Graduation target" val={`${form.graduationSol || "—"} SOL`} />
                <Row k={isBurn ? "Holder pays per burn" : "Holder pays per NFT"} val={fee !== null ? (isBurn ? `${formatSol(fee, 3)} + ≈ 0.005 SOL mint` : `${formatSol(fee, 3)} + mint deposit (refundable) if new`) : "—"} testId="math-fee" />
              </dl>
              <div className="bg-surface-2 rounded-panel border-border border p-3 text-sm">
                <p className="text-muted text-xs">{isBurn ? "Tokens burned per NFT, worth (estimate)" : "Price of 1 NFT (estimate)"} · {ratioLabel} × token price</p>
                <div className="mt-2 grid grid-cols-2 gap-2">
                  <div>
                    <p className="text-dim text-xs">At launch price</p>
                    <p className="num font-semibold" data-testid="math-price-launch">{designSol(m.nftPriceLaunchSol)} SOL</p>
                    <p className="text-dim text-xs">≈ ${(m.nftPriceLaunchSol * EXAMPLE_SOL_USD).toFixed(2)}</p>
                  </div>
                  <div>
                    <p className="text-dim text-xs">At graduation price</p>
                    <p className="num font-semibold" data-testid="math-price-grad">{designSol(m.nftPriceGraduationSol)} SOL</p>
                    <p className="text-dim text-xs">≈ ${(m.nftPriceGraduationSol * EXAMPLE_SOL_USD).toFixed(2)}</p>
                  </div>
                </div>
                <p className="text-dim mt-2 text-xs">Example curve prices (0.000000028 and ≈0.00000057 SOL per token) at $150/SOL. <span className="tag tag-demo">Example</span></p>
              </div>
            </>
          )}
          <p className={`rounded-panel border p-2 text-sm ${m.valid ? "tag-ok" : "border-warning-border bg-warning-bg text-warning"}`} data-testid="math-status">
            {m.valid ? (isPlain ? `Valid: Plain token, graduates at ${form.graduationSol} SOL.` : `Valid: ${sizeLabel} NFTs at ${ratioLabel} tokens each.`) : (v.errors.collectionSize ?? v.errors.graduationSol)}
          </p>
          <p className="text-dim text-xs">Decimals {LAUNCH_DECIMALS} · {compact(1_000_000_000)} fixed supply</p>
        </section>
      </aside>
      <TxPreviewModal safeSend={safeSend} />
    </div>
  );
}
