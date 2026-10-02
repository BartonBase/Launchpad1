"use client";
/**
 * Launch wizard (design: a-obsidian/launch.html, launch-plain.html, launch-burn.html; math:
 * NOTE.md "Wizard math" via lib/armory/wizardMath). Steps renumber per type: Plain 1–4, Hybrid
 * 1–5. Hybrid launches natively on devnet (hybrid_launch `launch`, no curve). Plain launches straight
 * on Meteora DBC with the official SDK (lib/meteora/dbc buildPlainLaunchTx) against the platform
 * config. Burn, Tax split and Raffle are "Coming soon" and can't be selected. Step 4 "Art commitment" feeds init_vault (InitVaultParams);
 * the token metadata URI feeds the DBC pool (curve path). Native `launch` has no URI field.
 */
import Link from "next/link";
import { useMemo, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import type { PublicKey } from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import {
  BURN_MINT_TEXT,
  DEFAULT_GRADUATION_SOL,
  FIRST_MINT_RANGE_TEXT,
  LAUNCH_TYPES,
  MIN_GRADUATION_LAMPORTS,
  MINT_ESCROW_LAMPORTS,
  STATUS_LABEL,
  launchTypeStatus,
  maxCollectionSize,
  tierFeeLamports,
} from "@/config/armory";
import { buildDbcLaunchTx, buildInitVaultTx, buildNativeLaunchTx, fetchDbcCurveSplit, fetchDbcGraduationLamports } from "@/lib/armory/builders";
import { NO_CURVE_MESSAGE } from "@/lib/armory/errors";
import { buildPlainLaunchTx } from "@/lib/meteora/dbc";
import { shortAddr } from "@/lib/armory/format";
import { Knight } from "./Knight";
import { DBC_PLATFORM_CONFIG } from "@/config/integrations";
import { useChainRead } from "@/hooks/useChain";
import { compact, formatSol, formatUnits } from "@/lib/armory/format";
import { LAUNCH_DECIMALS, validateArt, validateLaunchForm, type LaunchForm } from "@/lib/armory/launchForm";
import { EXAMPLE_SOL_USD, FALLBACK_CURVE_SPLIT, designSol, type WizardType } from "@/lib/armory/wizardMath";
import { useSafeSend } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";
import { TypeIcon } from "./TypeIcon";
import { MetadataUpload } from "./MetadataUpload";
import { fetchPlatformTerms } from "@/lib/meteora/dbc";
import { PLANNED_MAINNET_TERMS } from "@/lib/meteora/terms";
import { launchDisclosure } from "@/config/launchTerms";
import { armoryProgramsEnabled } from "@/config/programs";

const GROUPS = [
  { label: "Large collections", hint: "50K to 200K: more NFTs, each one cheaper", ratios: [50_000, 100_000, 200_000] },
  { label: "Small, scarce collections", hint: "500K to 5M: fewer NFTs, each one takes more tokens", ratios: [500_000, 1_000_000, 2_500_000, 5_000_000] },
] as const;
const TNAME: Record<WizardType, string> = { plain: "Launch", hybrid: "Hybrid", burn: "Burn" };

function steps(t: WizardType): string[] {
  return t === "plain" ? ["Launch type", "Basics", "Token & curve", "Review & launch"] : ["Launch type", "Basics", "Supply & conversion", "Art commitment", "Review & launch"];
}

function Row({ k, val, testId }: { k: string; val: React.ReactNode; testId?: string }) {
  return (
    <div className="fact">
      <dt>{k}</dt>
      <dd data-testid={testId}>{val}</dd>
    </div>
  );
}

export function LaunchWizard({ initialType: requested = "hybrid" }: { initialType?: WizardType }) {
  // A type that isn't open on this cluster (Hybrid on mainnet) can't be the starting point.
  const initialType: WizardType = launchTypeStatus(requested, CLUSTER.name) === "coming-soon" ? "plain" : requested;
  const { connected } = useWallet();
  const safeSend = useSafeSend();
  const [step, setStep] = useState(0);
  const [ack, setAck] = useState(false);
  const [form, setForm] = useState<LaunchForm>({ type: initialType, name: "", symbol: "", ratio: 1_000_000, collectionSize: "500", graduationSol: String(DEFAULT_GRADUATION_SOL) });
  const [launchedMint, setLaunchedMint] = useState<PublicKey | null>(null);
  const v = useMemo(() => validateLaunchForm(form, MIN_GRADUATION_LAMPORTS[CLUSTER.name]), [form]);
  const artV = useMemo(() => validateArt(form), [form]);
  const errs = { ...v.errors, ...artV.errors };
  const chainMin = MIN_GRADUATION_LAMPORTS[CLUSTER.name];
  const split = useChainRead("dbc:split", (c) => fetchDbcCurveSplit(c)).data ?? FALLBACK_CURVE_SPLIT;
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
    if (name === "Art commitment") return artV.art !== null;
    return true;
  };

  const formOk = Object.keys(v.errors).length === 0;
  const launch = () => {
    if (t === "plain") {
      if (!formOk || !plainReady) return;
      const p = { name: form.name.trim(), symbol: form.symbol.trim(), uri: (form.metadataUri ?? "").trim() };
      void safeSend.start(async ({ connection, payer }) => {
        const b = await buildPlainLaunchTx(connection, payer, p);
        setLaunchedMint(b.mint);
        return { tx: b.tx, signers: b.signers };
      });
      return;
    }
    if (!v.params || t !== "hybrid") return; // burn: coming soon, no builders
    const params = v.params;
    void safeSend.start(({ payer }) => {
      const b = buildNativeLaunchTx(payer, params);
      setLaunchedMint(b.mint);
      return { tx: b.tx, signers: b.signers };
    });
  };

  // Meteora DBC path (devnet, simulate-only): the graduation target is the pinned DBC config's
  // migration_quote_threshold read from chain, not the form value.
  const dbcConfig = DBC_PLATFORM_CONFIG[CLUSTER.name];
  const dbcAvailable = t === "hybrid" && dbcConfig !== null;
  const plainAvailable = t === "plain" && dbcConfig !== null;
  const dbcGrad = useChainRead(dbcConfig !== null ? "dbc:grad" : null, (c) => fetchDbcGraduationLamports(c));
  // Live launch terms (fee, creator share, anti-snipe, graduation) + the config safety check (mainnet fee wallet).
  const platform = useChainRead(dbcConfig !== null ? "dbc:terms" : null, (c) => fetchPlatformTerms(c));
  const terms = platform.data?.terms ?? PLANNED_MAINNET_TERMS;
  const configBlocked = (platform.data?.problems.length ?? 0) > 0;
  const plainReady = plainAvailable && !configBlocked;
  const disclosure = launchDisclosure(terms);
  const hasPrograms = armoryProgramsEnabled(CLUSTER.name);
  const pctText = (bps: number) => `${(bps / 100).toLocaleString("en-US", { maximumFractionDigits: 2 })}%`;
  const gradText = isPlain ? (dbcGrad.data != null ? formatSol(dbcGrad.data, 1) : dbcConfig ? "…" : `${PLANNED_MAINNET_TERMS.graduationSol} SOL (planned)`) : `${form.graduationSol || "—"} SOL`;
  const previewDbc = () => {
    if (!v.params || !dbcAvailable) return;
    const params = v.params;
    void safeSend.start(
      async ({ connection, payer }) => {
        const b = await buildDbcLaunchTx(connection, payer, { name: form.name.trim(), symbol: form.symbol.trim(), uri: (form.metadataUri ?? "").trim(), ratioWholeTokens: params.ratioWholeTokens, collectionSize: params.collectionSize });
        return { tx: b.tx, signers: b.signers };
      },
      { previewOnly: "the bonding-curve launch is simulated so you can review it; sending is not enabled yet." },
    );
  };

  // After a native launch: commit the art (init_vault) for the new mint, through the same preview.
  const commitArt = () => {
    if (!launchedMint || !artV.art || !v.params) return;
    const art = artV.art, mint = launchedMint, size = Number(v.params.collectionSize);
    void safeSend.start(async ({ connection, payer }) => {
      const b = await buildInitVaultTx(connection, payer, mint, size, art);
      return { tx: b.tx, signers: b.signers };
    });
  };
  const fillExampleArt = () =>
    setForm((f) => ({
      ...f,
      collectionName: f.collectionName || f.name || "My collection",
      collectionUri: "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
      traitRoot: "7f3a" + "0".repeat(56) + "c91e",
      schemaHash: "5c4e" + "0".repeat(56) + "a1b2",
    }));

  const err = (k: keyof typeof errs, show = true) =>
    show && errs[k] ? (
      <p id={`${k}-err`} className="text-warning mt-1 text-xs" data-testid={`launch-error-${k}`}>
        {errs[k]}
      </p>
    ) : null;
  const field = (k: "metadataUri" | "collectionName" | "collectionUri" | "traitRoot" | "schemaHash", label: string, hint: string, ph: string) => (
    <div>
      <label htmlFor={k} className="mb-1 flex justify-between text-sm font-medium">
        {label} <span className="text-dim font-normal">{hint}</span>
      </label>
      <input id={k} className="input font-mono text-xs" value={form[k] ?? ""} placeholder={ph} spellCheck={false} onChange={(e) => set(k, e.target.value)} aria-invalid={!!form[k] && !!errs[k]} aria-describedby={`${k}-err`} data-testid={`launch-${k}`} />
      {err(k, !!form[k])}
    </div>
  );


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
        </div>

        {names[step] === "Launch type" && (
          <fieldset className="space-y-4">
            <legend className="mb-1 text-lg font-semibold">Launch type</legend>
            <p className="text-muted text-sm">The type decides what holders can do with your token. It&apos;s fixed on-chain once you launch.</p>
            <div className="grid gap-3 sm:grid-cols-2">
              {LAUNCH_TYPES.filter((x) => x.id === "plain" || x.id === "hybrid").map((x) => {
                const st = launchTypeStatus(x.id, CLUSTER.name);
                const on = t === x.id;
                const soon = st === "coming-soon";
                return (
                  <label key={x.id} aria-disabled={soon || undefined} className={`rounded-panel border p-4 ${soon ? "card-soon" : "cursor-pointer"} ${on ? "border-accent-border bg-accent-tint" : "border-border"}`} data-testid={`launch-type-option-${x.id}`} data-status={st}>
                    <input type="radio" name="type" className="sr-only" checked={on} disabled={soon} onChange={() => set("type", x.id as WizardType)} />
                    <TypeIcon type={x.id} className="text-accent-text" />
                    <span className="mt-2 block font-semibold">{x.name}</span>
                    <span className="text-accent-text block text-xs">{x.short}</span>
                    <span className="text-muted mt-1 block text-xs">{x.description}</span>
                    {st === "live" ? <span className="tag tag-ok mt-2"><i />{STATUS_LABEL[st]}</span> : <span className="tag tag-pd mt-2"><i />{STATUS_LABEL[st]}</span>}
                  </label>
                );
              })}
            </div>
            <div className="grid gap-3 sm:grid-cols-3">
              {LAUNCH_TYPES.filter((x) => x.id === "burn" || x.id === "tax" || x.id === "raffle").map((x) => (
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
            <p className="text-dim text-xs">{hasPrograms ? "Burn, Tax split and Raffle aren't available yet." : "Hybrid, Burn, Tax split and Raffle aren't available on mainnet yet."} They&apos;re listed so you know what&apos;s planned.</p>
            {!live && (
              <p className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-pending-deploy">
                {TNAME[t]} launches aren&apos;t open yet. You can walk through the setup; launching is disabled.
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
            {isPlain && <MetadataUpload name={form.name} symbol={form.symbol} onUploaded={(uri) => set("metadataUri", uri)} />}
            {field("metadataUri", "Token metadata URI", isPlain ? "Filled by the upload, or paste your own" : "Hosted off-chain", "https://arweave.net/… · ipfs://… · ar://…")}
            <p className="text-dim text-xs" data-testid="launch-metadata-note">
              Optional. Wallets and explorers read the token&apos;s image and description from this metadata file. It&apos;s set when the curve pool is created and can&apos;t be edited afterwards. {isPlain ? "Upload above (Arweave, public and permanent) or paste a file you host yourself. Armory doesn't store your files." : "Host it yourself (IPFS or Arweave). A Hybrid launch without a curve has no metadata field, so it's only used on the curve path."}
            </p>
          </div>
        )}

        {(names[step] === "Supply & conversion" || names[step] === "Token & curve") && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold">{isPlain ? "Token and curve" : isBurn ? "Supply and burn rate" : "Supply and conversion"}</h2>
            <div className="grid gap-2 sm:grid-cols-2">
              {[
                [isBurn ? "Supply: capped at 1,000,000,000 tokens" : "Supply: fixed at 1,000,000,000 tokens", isBurn ? "Mint authority revoked at launch, so nobody can mint more; burns only lower it." : "Mint authority revoked at launch, so nobody can mint more."],
                ["Freeze authority revoked", "Nobody can freeze holders' tokens."],
                ...(isPlain ? [["No NFT side", "No ratio, collection or converting. Just the coin."]] : [["Conversion rate set at launch", "Fixed in the launch record; no instruction can change it."]]),
                ["No transfer tax", "A classic SPL token. No tax on any transfer."],
              ].map(([a, b]) => (
                <div key={a} className="bg-surface-2 rounded-panel border-border border p-3 text-sm">
                  <p className="font-medium">🔒 {a}</p>
                  <p className="text-muted text-xs">{b}</p>
                </div>
              ))}
            </div>
            {isPlain && (
              <div className="border-accent-border bg-accent-tint rounded-panel border p-3 text-sm" data-testid="launch-disclosure">
                <p className="text-accent-text mb-1 font-medium">Fees and what you earn</p>
                <ul className="text-muted list-disc space-y-1 pl-5 text-xs">
                  {disclosure.map((l) => <li key={l}>{l}</li>)}
                </ul>
              </div>
            )}
            {!isPlain && (
              <>
                <p className="border-accent-border bg-accent-tint text-accent-text rounded-panel border p-3 text-sm">
                  {isBurn ? "Burning" : "Converting"} opens when your token graduates. NFTs are minted one at a time, the first time someone {isBurn ? "burns for them. The burner pays the mint cost." : "captures them. The collector's refundable deposit covers the small mint cost."}{" "}
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
              {isPlain ? (
                <div data-testid="launch-graduation-fixed">
                  <p className="mb-1 text-sm font-medium">Graduation target</p>
                  <p className="input num flex items-center">{gradText}</p>
                  <p className="text-dim mt-1 text-xs">
                    Fixed by Armory&apos;s curve settings ({dbcConfig ? shortAddr(dbcConfig.toBase58(), 4) : "none"}), read from chain. When the curve raises it, the token graduates and its liquidity moves to a locked trading pool.
                  </p>
                </div>
              ) : (
              <div>
                <label htmlFor="grad" className="mb-1 flex justify-between text-sm font-medium">
                  Graduation target (SOL){" "}
                  <span className="text-dim font-normal" data-testid="launch-graduation-hint">
                    Default {DEFAULT_GRADUATION_SOL} SOL · minimum {formatSol(chainMin, 1)}
                  </span>
                </label>
                <input id="grad" inputMode="decimal" className="input num" value={form.graduationSol} onChange={(e) => set("graduationSol", e.target.value)} aria-invalid={!!v.errors.graduationSol} aria-describedby="graduationSol-err" data-testid="launch-graduation" />
                {err("graduationSol")}
                <p className="text-dim mt-1 text-xs" data-testid="launch-graduation-min">
                  SOL raised on the bonding curve before your token graduates. Default {DEFAULT_GRADUATION_SOL} SOL; the programs accept {formatSol(chainMin, 1)} or more. On the curve path, the platform&apos;s approved curve settings fix the target{dbcGrad.data != null ? ` (${formatSol(dbcGrad.data, 1)})` : ""}.
                </p>
              </div>
              )}
            </div>
            {!isPlain && (
              <div className="bg-surface-2 rounded-panel border-border border p-3 text-sm" data-testid="launch-mint-cost">
                <p className="text-muted text-xs">{isBurn ? "Mint cost per NFT · paid by the burner" : "Mint deposit per capture · paid by the collector"}</p>
                <p className="font-medium">{isBurn ? `About ${FIRST_MINT_RANGE_TEXT.replace("≈ ", "")}, every burn` : `${formatSol(MINT_ESCROW_LAMPORTS, 4)}, mostly refunded`}</p>
                <p className="text-dim text-xs">
                  {isBurn
                    ? "Every burn mints a new NFT, so the burner pays Solana rent and the Metaplex Core fee directly. No deposit. Nothing comes out of your proceeds."
                    : `Each capture or re-roll puts up a ${formatSol(MINT_ESCROW_LAMPORTS, 4)} deposit, refunded except ${FIRST_MINT_RANGE_TEXT} if the NFT is minted for the first time (Solana rent + Metaplex fee). Nothing comes out of your graduation proceeds.`}
                </p>
              </div>
            )}
          </div>
        )}

        {names[step] === "Art commitment" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Art commitment and randomness</h2>
            <p className="text-muted text-sm">
              Your art is prepared off-chain. Here you enter its addresses and fingerprints, which are locked in for good. Rarity is cosmetic.{" "}
              {isBurn ? "Every NFT costs the same to mint, whatever its traits, and none converts back to tokens." : `Every NFT converts back for exactly ${ratioLabel} tokens, whatever its traits.`}
            </p>
            <ol className="text-muted list-decimal space-y-1 pl-5 text-sm" data-testid="launch-art-prep">
              <li>Upload each NFT&apos;s image and metadata JSON to IPFS or Arweave. Armory has no upload step.</li>
              <li>Build a Merkle tree with one leaf per NFT: its 8 trait values, a salt, the SHA-256 of its image and JSON, and its metadata URI.</li>
              <li>Enter the results below. Keep the full leaf list available: each {isBurn ? "burn" : "capture"} is settled with that NFT&apos;s leaf and proof.</li>
            </ol>
            <div className="grid gap-4 sm:grid-cols-2" data-testid="launch-art-fields">
              {field("collectionName", "Collection name", "Up to 32 characters", "Low Orbit")}
              {field("collectionUri", "Collection metadata URI", "ipfs:// or ar:// only", "ipfs://…")}
              {field("traitRoot", "Trait root", "Merkle root, 32 bytes (hex)", "0x…")}
              {field("schemaHash", "Trait schema hash", "32 bytes (hex)", "0x…")}
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <button type="button" className="btn btn-sm" onClick={fillExampleArt} data-testid="launch-art-example">Fill sample values</button>
              <span className="text-dim text-xs">These four values are committed when the collection vault is created, before {isBurn ? "burning" : "converting"} opens. No instruction can change them afterwards.</span>
            </div>
            <p className="text-sm font-medium">
              How NFTs are assigned <span className="text-dim text-xs font-normal">Fixed for this type, not a setting</span>
            </p>
            <ul className="space-y-2 text-sm">
              <li className="bg-surface-2 rounded-panel border-border border p-3">
                <b>Art and traits committed before {isBurn ? "burning" : "converting"} opens.</b> <span className="text-muted">A fingerprint (Merkle root) of every NFT&apos;s art and traits is published when the collection vault is created. Nobody can swap or edit a piece afterwards.</span>
              </li>
              {isBurn ? (
                <li className="bg-surface-2 rounded-panel border-border border p-3">
                  <b>NFTs go out in collection order.</b> <span className="text-muted">#0, #1, #2 and so on, one per burn. No randomness, so the next piece is always public.</span>
                </li>
              ) : (
                <li className="bg-surface-2 rounded-panel border-border border p-3">
                  <b>Switchboard picks which NFT you get.</b> <span className="text-muted">Switchboard On-Demand randomness comes with a proof the program checks on-chain, at every capture and re-roll. Anyone can trigger the reveal and settle steps; nobody, including you, can choose a piece.</span>
                </li>
              )}
            </ul>
            {!isBurn && (
              <p className="text-muted text-sm" data-testid="launch-reroll-note">
                <b className="text-fg">Re-rolls are built in. There&apos;s nothing to set.</b> A re-roll costs this collection&apos;s platform fee ({fee !== null ? formatSol(fee, 3) : "—"}, the same as a capture) plus the refundable {formatSol(MINT_ESCROW_LAMPORTS, 4)} mint deposit.
              </p>
            )}
          </div>
        )}

        {names[step] === "Review & launch" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Review and launch</h2>
            <dl data-testid="launch-review">
              <Row k="Launch type" val={TNAME[t]} />
              <Row k="Token" val={`${form.name || "—"} · $${form.symbol || "—"}`} />
              <Row k="Supply" val={isBurn ? "Capped at 1,000,000,000 tokens" : "Fixed at 1,000,000,000 tokens"} />
              <Row k="Graduation target" val={isPlain ? `${gradText} raised, then a locked trading pool` : `${form.graduationSol} SOL raised`} />
              <Row k="Bonding curve" val={t === "hybrid" ? "None: native launch (curve launch: preview only)" : `Armory curve · settings ${dbcConfig ? shortAddr(dbcConfig.toBase58(), 4) : "—"}`} testId="launch-review-curve" />
              <Row k="Decimals" val={String(LAUNCH_DECIMALS)} testId="launch-review-decimals" />
              <Row k="Token metadata URI" val={(form.metadataUri ?? "").trim() || (isPlain ? "None" : "None (curve path only)")} />
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
                  <Row k="Art commitment" val={artV.art ? `${artV.art.collectionName} · ${artV.art.collectionUri.slice(0, 18)}… · root 0x${form.traitRoot!.replace(/^0x/i, "").slice(0, 4)}…` : "—"} testId="launch-review-art" />
                  <Row k={isBurn ? "NFT order" : "Randomness"} val={isBurn ? "Collection order, public" : "Committed art + Switchboard On-Demand"} />
                  <Row k="Platform fee" val={`${fee !== null ? formatSol(fee, 3) : ""} per ${isBurn ? "burn" : "capture or re-roll"}`} />
                  <Row k={isBurn ? "Mint cost, every burn" : "Mint deposit"} val={isBurn ? BURN_MINT_TEXT : `${formatSol(MINT_ESCROW_LAMPORTS, 4)}, refunded except ${FIRST_MINT_RANGE_TEXT} if the NFT is minted new`} testId="launch-review-deposit" />
                  <Row k="Converting back" val={isBurn ? "Not possible, one-way" : "No platform fee"} />
                </>
              )}
              <Row k="Launch fee" val={isPlain ? `None · you pay about ${terms.creatorCostSol.toFixed(4)} SOL network rent and fees` : "None on chain today"} testId="launch-review-launch-fee" />
              {isPlain && (
                <>
                  <Row k="Curve trade fee" val={`${pctText(terms.tradingFeeBps)} per trade, in SOL`} testId="launch-review-trade-fee" />
                  <Row k="Your share" val={`${terms.creatorFeePercent}% of the fee after Meteora's cut · claim in Portfolio`} testId="launch-review-creator-share" />
                  {terms.antiSnipeStartBps !== null && terms.antiSnipeSeconds !== null && <Row k="Anti-snipe" val={`Fee starts at ${pctText(terms.antiSnipeStartBps)}, falls to ${pctText(terms.tradingFeeBps)} over ${terms.antiSnipeSeconds} s`} testId="launch-review-antisnipe" />}
                </>
              )}
              {hasPrograms ? <Row k="Program upgrades" val="Not locked yet · 1 dev key per program · multisig + 7-day delay planned for mainnet" /> : <Row k="Programs" val="Meteora's audited bonding curve and DAMM v2 only. No Armory program." testId="launch-review-programs" />}
            </dl>
            {t === "hybrid" && (
              <p className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-native-warning">
                Native launch: no bonding curve is created. {NO_CURVE_MESSAGE.replace("This launch has no bonding curve, so it", "The token")}
              </p>
            )}
            <label className="flex items-start gap-2 text-sm">
              <input type="checkbox" className="mt-1" checked={ack} onChange={(e) => setAck(e.target.checked)} data-testid="launch-ack" />
              <span className="text-muted">
                {hasPrograms
                  ? <>I understand that the launch type{isPlain ? "" : ", collection size and committed art"} can&apos;t be changed after launch, and that the programs can still be upgraded until they&apos;re frozen after the audit.</>
                  : <>I understand the name, ticker and metadata can&apos;t be changed after launch, and I&apos;ve read the fees above, including the anti-snipe fee on early trades.</>}
              </span>
            </label>
            <button
              type="button"
              className="btn btn-primary btn-lg w-full"
              disabled={!live || !ack || !connected || (isPlain ? !formOk || !plainReady : !v.params) || safeSend.busy}
              onClick={launch}
              data-testid="launch-submit"
            >
              {live ? (isPlain ? (plainAvailable ? "Launch token" : "Launches opening soon") : "Launch Hybrid") : `${TNAME[t]}: ${STATUS_LABEL[launchTypeStatus(t, CLUSTER.name)].toLowerCase()}`}
            </button>
            {isPlain && !plainAvailable && (
              <p className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-config-missing">
                Launches open as soon as Armory&apos;s {CLUSTER.displayName.toLowerCase()} curve settings are published. You can walk through the setup now.
              </p>
            )}
            {isPlain && configBlocked && (
              <p role="alert" className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-config-problem">
                Launching is paused: {platform.data?.problems.join(" ")}
              </p>
            )}
            {dbcAvailable && (
              <div className="space-y-1" data-testid="launch-dbc">
                <button type="button" className="btn w-full" disabled={!ack || !connected || !v.params || safeSend.busy} onClick={previewDbc} data-testid="launch-dbc-preview">
                  Preview bonding-curve launch (simulate only)
                </button>
                <p className="text-dim text-xs" data-testid="launch-dbc-graduation">
                  With a bonding curve, the graduation target comes from Armory&apos;s curve settings: {dbcGrad.data != null ? `${formatSol(dbcGrad.data, 1)} raised` : "loading…"}. {(form.metadataUri ?? "").trim() ? "Uses your token metadata URI." : "No token metadata URI set (step 2)."}
                </p>
              </div>
            )}
            {live && !connected && <p className="text-muted text-sm">Connect a wallet to launch.</p>}
            {safeSend.status === "error" && !safeSend.preview && (
              <p role="alert" className="text-negative text-sm" data-testid="launch-error">{safeSend.error}</p>
            )}
            {safeSend.status === "success" && launchedMint && (
              <div role="status" className="bg-surface-2 rounded-panel border-border flex items-end gap-4 border px-4 pt-2" data-testid="launch-success">
                <Knight k="c" glow sizes="96px" className="w-[84px] shrink-0 sm:w-[96px]" />
                <p className="text-positive-text pb-4 text-sm">
                  <span className="text-fg block text-base font-semibold">Launched.</span>
                  <Link className="underline" href={`/t/${launchedMint.toBase58()}`}>Open the token page</Link>.
                </p>
              </div>
            )}
            {launchedMint && t === "hybrid" && (
              <button type="button" className="btn w-full" disabled={!connected || !artV.art || safeSend.busy} onClick={commitArt} data-testid="launch-commit-art">
                Commit art (create the collection vault)
              </button>
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
              <p className="text-muted text-sm">Supply fixed at 1,000,000,000 tokens. No NFT side. Tokens trade on the bonding curve until it raises {gradText}, then the token graduates to a locked trading pool.</p>
              <div className="progress"><span style={{ width: `${split.pct[0]}%` }} /></div>
              <dl data-testid="math-split">
                <Row k="Sold on the curve" val={`${formatUnits(split.curve, 0)} · ${split.pct[0]}%`} />
                <Row k="Set aside for the trading pool" val={`${formatUnits(split.dex, 0)} · ${split.pct[1]}%`} />
                {split.buffer > 0n && <Row k={hasPrograms ? "Unsold buffer, locked" : "Unsold leftover, burned"} val={`${formatUnits(split.buffer, 0)} · ${split.pct[2]}%`} />}
                <Row k="Per-NFT fees" val="None" />
              </dl>
              <p className="text-dim text-xs">{dbcConfig ? "Split from Armory's curve settings (read from chain); the approved settings at launch decide it." : "Example split. The curve settings, once published, decide it."}</p>
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
                <Row k={isBurn ? "Holder pays per burn" : "Holder pays per NFT"} val={fee !== null ? (isBurn ? `${formatSol(fee, 3)} + ${FIRST_MINT_RANGE_TEXT} mint` : `${formatSol(fee, 3)} + ${formatSol(MINT_ESCROW_LAMPORTS, 4)} deposit, mostly refunded`) : "—"} testId="math-fee" />
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
                <p className="text-dim mt-2 text-xs">Estimated from typical curve prices (0.000000028 and ≈0.00000057 SOL per token) at $150/SOL.</p>
              </div>
            </>
          )}
          <p className={`rounded-panel border p-2 text-sm ${m.valid ? "tag-ok" : "border-warning-border bg-warning-bg text-warning"}`} data-testid="math-status">
            {m.valid ? (isPlain ? `Valid: graduates at ${gradText}.` : `Valid: ${sizeLabel} NFTs at ${ratioLabel} tokens each.`) : (errs.collectionSize ?? errs.graduationSol)}
          </p>
          <p className="text-dim text-xs">Decimals {LAUNCH_DECIMALS} · {compact(1_000_000_000)} fixed supply</p>
        </section>
      </aside>
      <TxPreviewModal safeSend={safeSend} />
    </div>
  );
}
