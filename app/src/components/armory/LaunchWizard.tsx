"use client";
/**
 * Launch wizard (design: a-obsidian/launch.html; math: lib/armory/wizardMath). Steps renumber per
 * type: Launch 1–4, Hybrid 1–5. Both types launch on Armory's Meteora DBC config with the official
 * SDK (lib/meteora/dbc), with an optional creator first buy (dev buy) in the same transaction.
 * Hybrid then registers the pool with hybrid_launch (register_dbc_launch) and commits the art
 * (init_vault) in a second transaction. Art and metadata are uploaded from the browser to Irys
 * (lib/armory/art); no server key is involved. Burn, Tax split and Raffle are "Coming soon".
 */
import Link from "next/link";
import { useMemo, useRef, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { Keypair, PublicKey } from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";
import {
  FIRST_MINT_RANGE_TEXT,
  LAUNCH_TYPES,
  MIN_GRADUATION_LAMPORTS,
  MINT_ESCROW_LAMPORTS,
  STATUS_LABEL,
  launchTypeStatus,
  maxCollectionSize,
  tierFeeLamports,
} from "@/config/armory";
import { buildHybridRegisterTx, fetchDbcCurveSplit, fetchDbcGraduationLamports } from "@/lib/armory/builders";
import { buildPlainLaunchTx, quoteDevBuy } from "@/lib/meteora/dbc";
import { launchConfigPda } from "@/lib/generated/hybridLaunch";
import { Knight } from "./Knight";
import { DBC_PLATFORM_CONFIG } from "@/config/integrations";
import { useChainRead } from "@/hooks/useChain";
import { compact, formatSol, formatUnits, shortAddr, solToLamports } from "@/lib/armory/format";
import { LAUNCH_DECIMALS, validateLaunchForm, type LaunchForm } from "@/lib/armory/launchForm";
import { EXAMPLE_SOL_USD, FALLBACK_CURVE_SPLIT, designSol, type WizardType } from "@/lib/armory/wizardMath";
import { buildCollection, uploadTokenMetadata, type ArtImage, type BuiltCollection } from "@/lib/armory/art";
import { throwawaySigner } from "@/lib/armory/irys";
import { prepareImage } from "@/lib/armory/imagePrep";
import { useSafeSend } from "@/lib/tx/useSafeSend";
import { TxPreviewModal } from "@/components/TxPreviewModal";
import { TypeIcon } from "./TypeIcon";

/** Hybrid conversion rates offered: the 0.01 SOL capture-fee tier (docs/DECISIONS.md ADR-023). */
export const HYBRID_RATIOS = [500_000, 1_000_000, 2_500_000, 5_000_000] as const;
const TNAME: Record<WizardType, string> = { plain: "Launch", hybrid: "Hybrid", burn: "Burn" };
const DEV_BUY_SLIPPAGE_BPS = 100;
const MAX_ART_IMAGES = 50;

function steps(t: WizardType): string[] {
  return t === "plain" ? ["Launch type", "Basics", "Token & curve", "Review & launch"] : ["Launch type", "Basics", "Supply & conversion", "NFT art", "Review & launch"];
}

function Row({ k, val, testId }: { k: string; val: React.ReactNode; testId?: string }) {
  return (
    <div className="fact">
      <dt>{k}</dt>
      <dd data-testid={testId}>{val}</dd>
    </div>
  );
}

interface Prepared extends ArtImage {
  readonly url: string;
}
const toPrepared = (a: ArtImage): Prepared => ({ ...a, url: URL.createObjectURL(new Blob([a.data as BlobPart], { type: a.contentType })) });

export function LaunchWizard({ initialType = "hybrid" }: { initialType?: WizardType }) {
  const { connected } = useWallet();
  const safeSend = useSafeSend();
  const [step, setStep] = useState(0);
  const [ack, setAck] = useState(false);
  const [form, setForm] = useState<LaunchForm>({ type: initialType, name: "", symbol: "", ratio: 1_000_000, collectionSize: "100", graduationSol: "85" });
  const [description, setDescription] = useState("");
  const [tokenImage, setTokenImage] = useState<Prepared | null>(null);
  const [artImages, setArtImages] = useState<Prepared[]>([]);
  const [imgErr, setImgErr] = useState<string | null>(null);
  const [art, setArt] = useState<(BuiltCollection & { key: string }) | null>(null);
  const [upload, setUpload] = useState<{ stage: string; done: number; total: number } | null>(null);
  const [uploadErr, setUploadErr] = useState<string | null>(null);
  const [devBuySol, setDevBuySol] = useState("");
  const [mintKp] = useState(() => Keypair.generate());
  const [launchedMint, setLaunchedMint] = useState<PublicKey | null>(null);
  const [action, setAction] = useState<"launch" | "register" | null>(null);
  const metaCache = useRef<{ key: string; uri: string } | null>(null);
  const [metaUri, setMetaUri] = useState<string | null>(null);

  const chainMin = MIN_GRADUATION_LAMPORTS[CLUSTER.name];
  const v = useMemo(() => validateLaunchForm(form, chainMin), [form, chainMin]);
  const split = useChainRead("dbc:split", (c) => fetchDbcCurveSplit(c)).data ?? FALLBACK_CURVE_SPLIT;
  const m = v.math;
  const set = <K extends keyof LaunchForm>(k: K, val: LaunchForm[K]) => setForm((f) => ({ ...f, [k]: val }));
  const t = form.type;
  const isPlain = t === "plain";
  const live = launchTypeStatus(t, CLUSTER.name) === "live";
  const names = steps(t);
  const last = names.length - 1;
  const fee = m.feeLamports;
  const ratioLabel = formatUnits(BigInt(form.ratio), 0);
  const size = Number(form.collectionSize.replace(/,/g, "")) || 0;
  const sizeLabel = size.toLocaleString("en-US");
  const collectionName = (form.collectionName ?? "").trim();

  const dbcConfig = DBC_PLATFORM_CONFIG[CLUSTER.name];
  const dbcGrad = useChainRead(dbcConfig !== null ? "dbc:grad" : null, (c) => fetchDbcGraduationLamports(c));
  const gradText = dbcGrad.data != null ? formatSol(dbcGrad.data, 1) : "…";

  // Dev buy: optional SOL amount, quoted on a fresh pool at the launch slot (anti-snipe fee included).
  const devLamports = devBuySol.trim() === "" ? 0n : solToLamports(devBuySol.trim());
  const devBuyErr = devLamports === null ? "Enter a SOL amount, e.g. 0.05." : dbcGrad.data != null && devLamports >= dbcGrad.data ? "A dev buy this large would fill the whole curve." : null;
  const devQ = useChainRead(dbcConfig && devLamports && devLamports > 0n && !devBuyErr ? `dbc:devbuy:${devLamports}` : null, (c) => quoteDevBuy(c, devLamports!, DEV_BUY_SLIPPAGE_BPS));
  const sched = useChainRead(dbcConfig ? "dbc:sched" : null, (c) => quoteDevBuy(c, 10_000_000n, DEV_BUY_SLIPPAGE_BPS)).data?.schedule ?? null;
  const devTokens = devQ.data ? formatUnits(devQ.data.tokensOut / 10n ** BigInt(devQ.data.decimals), 0) : null;
  const devBuyText = devLamports && devLamports > 0n ? `${formatSol(devLamports, 4)}${devTokens ? ` → ≈ ${devTokens} ${form.symbol ? "$" + form.symbol : "tokens"}` : ""}` : "None";
  const antiSnipeText = sched
    ? sched.antiSnipe
      ? `Anti-snipe fee: ${sched.startPct}% at launch, falling to ${sched.endPct}%${sched.window ? ` over ${sched.window}` : ""}. Your dev buy lands at the launch moment, so it pays ${devQ.data ? devQ.data.feePct : sched.startPct}%.`
      : `This curve has no anti-snipe surcharge: your dev buy pays the normal ${sched.endPct}% trade fee, like everyone else.`
    : "Loading the curve's fee schedule…";

  const artKey = `${mintKp.publicKey.toBase58()}|${collectionName}|${size}|${form.symbol}|${artImages.map((a) => a.name + a.data.length).join(",")}`;
  const artReady = art !== null && art.key === artKey;

  const stepOk = (i: number): boolean => {
    const name = names[i];
    if (name === "Launch type") return true;
    if (name === "Basics") return !v.errors.name && !v.errors.symbol;
    if (name === "Supply & conversion") return m.valid && (HYBRID_RATIOS as readonly number[]).includes(form.ratio);
    if (name === "Token & curve") return m.valid && !devBuyErr;
    if (name === "NFT art") return artReady && !devBuyErr;
    return true;
  };

  const pickImage = async (files: FileList | null) => {
    setImgErr(null);
    const f = files?.[0];
    if (!f) return;
    try {
      setTokenImage(toPrepared(await prepareImage(f)));
    } catch (e) {
      setImgErr(e instanceof Error ? e.message : String(e));
    }
  };
  const pickArt = async (files: FileList | null) => {
    setUploadErr(null);
    const list = Array.from(files ?? []).slice(0, MAX_ART_IMAGES);
    try {
      setArtImages(await Promise.all(list.map(async (f) => toPrepared(await prepareImage(f)))));
    } catch (e) {
      setUploadErr(e instanceof Error ? e.message : String(e));
    }
  };
  const uploadArt = async () => {
    setUploadErr(null);
    if (artImages.length === 0 || artImages.length > size) return;
    try {
      const built = await buildCollection({
        collectionName,
        symbol: form.symbol.trim(),
        description: description.trim(),
        images: artImages,
        size,
        launchConfig: launchConfigPda(mintKp.publicKey),
        signer: throwawaySigner(),
        onProgress: (stage, done, total) => setUpload({ stage, done, total }),
      });
      setArt({ ...built, key: artKey });
    } catch (e) {
      setUploadErr(e instanceof Error ? e.message : String(e));
    } finally {
      setUpload(null);
    }
  };

  const formOk = Object.keys(v.errors).length === 0;
  const launch = () => {
    if (!formOk || !dbcConfig || devBuyErr) return;
    if (t === "hybrid" && !artReady) return;
    const p = { name: form.name.trim(), symbol: form.symbol.trim(), description: description.trim() };
    const img = tokenImage;
    const lamports = devLamports ?? 0n;
    setAction("launch");
    void safeSend.start(async ({ connection, payer }) => {
      const key = `${p.name}|${p.symbol}|${p.description}|${img?.url ?? ""}`;
      if (metaCache.current?.key !== key) metaCache.current = { key, uri: await uploadTokenMetadata({ ...p, image: img, signer: throwawaySigner() }) };
      const uri = metaCache.current.uri;
      setMetaUri(uri);
      const devBuy = lamports > 0n ? { lamports, minimumAmountOut: (await quoteDevBuy(connection, lamports, DEV_BUY_SLIPPAGE_BPS)).minimumAmountOut } : null;
      const b = await buildPlainLaunchTx(connection, payer, { name: p.name, symbol: p.symbol, uri }, devBuy, mintKp);
      setLaunchedMint(b.mint);
      return { tx: b.tx, signers: b.signers };
    });
  };
  const register = () => {
    if (!launchedMint || !art || !v.params) return;
    const params = v.params, mint = launchedMint, a = art;
    setAction("register");
    void safeSend.start(async ({ connection, payer }) => {
      const b = await buildHybridRegisterTx(connection, payer, {
        mint,
        ratioWholeTokens: params.ratioWholeTokens,
        collectionSize: params.collectionSize,
        art: { collectionName, collectionUri: a.collectionUri, traitRoot: a.traitRoot, traitSchemaHash: a.schemaHash },
      });
      return { tx: b.tx, signers: b.signers };
    });
  };
  const launchedOk = launchedMint !== null && (action !== "launch" || safeSend.status === "success");
  const registered = action === "register" && safeSend.status === "success";

  const err = (k: keyof typeof v.errors, show = true) =>
    show && v.errors[k] ? (
      <p id={`${k}-err`} className="text-warning mt-1 text-xs" data-testid={`launch-error-${k}`}>
        {v.errors[k]}
      </p>
    ) : null;

  const devBuyField = (
    <div className="space-y-1" data-testid="launch-devbuy">
      <label htmlFor="devbuy" className="mb-1 flex justify-between text-sm font-medium">
        Dev buy (optional) <span className="text-dim font-normal">SOL you spend on your own token at launch</span>
      </label>
      <input id="devbuy" inputMode="decimal" className="input num" placeholder="0" value={devBuySol} onChange={(e) => setDevBuySol(e.target.value)} aria-invalid={!!devBuyErr} data-testid="launch-devbuy-sol" />
      {devBuyErr ? (
        <p className="text-warning text-xs" data-testid="launch-devbuy-error">{devBuyErr}</p>
      ) : devLamports && devLamports > 0n ? (
        <p className="text-sm" data-testid="launch-devbuy-estimate">
          {devQ.data ? (
            <>
              ≈ <b className="num">{devTokens}</b> tokens for {formatSol(devLamports, 4)} (fee {formatSol(devQ.data.feeLamports, 5)}, {devQ.data.feePct}%). You receive at least {formatUnits(devQ.data.minimumAmountOut / 10n ** BigInt(devQ.data.decimals), 0)} or the launch fails.
            </>
          ) : devQ.error ? (
            <span className="text-warning">Couldn&apos;t estimate: {devQ.error}</span>
          ) : (
            "Estimating…"
          )}
        </p>
      ) : null}
      <p className="text-dim text-xs" data-testid="launch-devbuy-antisnipe">
        Bought in the same transaction that creates the curve, before anyone else can trade. {antiSnipeText}
      </p>
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
                return (
                  <label key={x.id} className={`rounded-panel cursor-pointer border p-4 ${on ? "border-accent-border bg-accent-tint" : "border-border"}`} data-testid={`launch-type-option-${x.id}`} data-status={st}>
                    <input type="radio" name="type" className="sr-only" checked={on} onChange={() => set("type", x.id as WizardType)} />
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
            <p className="text-dim text-xs">Burn, Tax split and Raffle aren&apos;t available yet. They&apos;re listed so you know what&apos;s planned.</p>
            {!live && (
              <p className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid="launch-pending-deploy">
                {TNAME[t]} launches aren&apos;t open yet. You can walk through the setup; launching is disabled.
              </p>
            )}
          </fieldset>
        )}

        {names[step] === "Basics" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Token basics</h2>
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
            <div>
              <label htmlFor="desc" className="mb-1 flex justify-between text-sm font-medium">
                Description <span className="text-dim font-normal">Optional · {description.length} / 280</span>
              </label>
              <textarea id="desc" className="input min-h-20" maxLength={280} value={description} onChange={(e) => setDescription(e.target.value)} data-testid="launch-description" />
            </div>
            <div className="flex items-center gap-4">
              <div className="bg-surface-2 border-border flex size-20 shrink-0 items-center justify-center overflow-hidden rounded-full border">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                {tokenImage ? <img src={tokenImage.url} alt="Token image preview" className="size-full object-cover" data-testid="launch-image-preview" /> : <span className="text-dim text-xs">No image</span>}
              </div>
              <div className="space-y-1">
                <label htmlFor="token-image" className="text-sm font-medium">Token image</label>
                <input id="token-image" type="file" accept="image/png,image/jpeg,image/webp,image/gif" className="block text-sm" onChange={(e) => void pickImage(e.target.files)} data-testid="launch-image" />
                <p className="text-dim text-xs">PNG, JPEG, WebP or GIF. Large images are resized in your browser. Uploaded with the token&apos;s metadata when you launch.</p>
                {imgErr && <p className="text-warning text-xs">{imgErr}</p>}
              </div>
            </div>
          </div>
        )}

        {(names[step] === "Supply & conversion" || names[step] === "Token & curve") && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold">{isPlain ? "Token and curve" : "Supply and conversion"}</h2>
            <div className="grid gap-2 sm:grid-cols-2">
              {[
                ["Supply: fixed at 1,000,000,000 tokens", "Mint authority revoked at launch, so nobody can mint more."],
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
                  Converting opens when your token graduates. NFTs are minted one at a time, the first time someone captures them. The collector&apos;s refundable deposit covers the small mint cost. Captures and re-rolls carry a {fee !== null ? formatSol(fee, 3) : ""} platform fee. Converting back is free.
                </p>
                <fieldset>
                  <legend className="mb-2 flex w-full justify-between text-sm font-medium">
                    Tokens per NFT <span className="text-dim text-xs font-normal">Fewer NFTs, each one takes more tokens</span>
                  </legend>
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                    {HYBRID_RATIOS.map((r) => (
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
                <div>
                  <label htmlFor="size" className="mb-1 block text-sm font-medium">Collection size</label>
                  <input id="size" inputMode="numeric" className="input num" value={form.collectionSize} onChange={(e) => set("collectionSize", e.target.value)} aria-invalid={!!v.errors.collectionSize} aria-describedby="collectionSize-err" data-testid="launch-collection-size" />
                  {err("collectionSize") ?? (
                    <p className="text-dim mt-1 text-xs">
                      {sizeLabel} × {ratioLabel} = {formatUnits(m.nftTokens, 0)} tokens, within the 1,000,000,000 supply.
                    </p>
                  )}
                </div>
              </>
            )}
            <div data-testid="launch-graduation-fixed">
              <p className="mb-1 text-sm font-medium">Graduation target</p>
              <p className="input num flex items-center">{gradText}</p>
              <p className="text-dim mt-1 text-xs">
                Fixed by Armory&apos;s curve settings, read from chain. When the curve raises it, the token graduates and its liquidity moves to a locked trading pool.
              </p>
            </div>
            {isPlain && devBuyField}
          </div>
        )}

        {names[step] === "NFT art" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">NFT art</h2>
            <p className="text-muted text-sm">
              Name the collection and add its images. Armory writes one metadata file per NFT, uploads everything to Arweave (via Irys) from your browser, and locks a fingerprint of the whole set when the vault is created. Every NFT converts back for exactly {ratioLabel} tokens, whatever its art.
            </p>
            <div>
              <label htmlFor="collectionName" className="mb-1 flex justify-between text-sm font-medium">
                Collection name <span className="text-dim font-normal">{collectionName.length} / 32</span>
              </label>
              <input id="collectionName" className="input" maxLength={32} value={form.collectionName ?? ""} placeholder={form.name || "Low Orbit"} onChange={(e) => set("collectionName", e.target.value)} data-testid="launch-collectionName" />
            </div>
            <div className="space-y-1">
              <label htmlFor="art-images" className="text-sm font-medium">Images <span className="text-dim font-normal">1 to {Math.min(MAX_ART_IMAGES, size || MAX_ART_IMAGES)}</span></label>
              <input id="art-images" type="file" multiple accept="image/png,image/jpeg,image/webp,image/gif" className="block text-sm" onChange={(e) => void pickArt(e.target.files)} data-testid="launch-art-images" />
              <p className="text-dim text-xs">
                {sizeLabel} NFTs share these images in turn (NFT #0 gets image 1, #1 gets image 2, …). Each image is resized in your browser to under 96 KB.
              </p>
            </div>
            {artImages.length > 0 && (
              <div className="grid grid-cols-5 gap-2 sm:grid-cols-8" data-testid="launch-art-thumbs">
                {artImages.map((a, i) => (
                  // eslint-disable-next-line @next/next/no-img-element
                  <img key={a.url} src={a.url} alt={`Art ${i + 1}`} className="rounded-chip border-border aspect-square w-full border object-cover" />
                ))}
              </div>
            )}
            <div className="flex flex-wrap items-center gap-3">
              <button type="button" className="btn btn-primary" disabled={!collectionName || artImages.length === 0 || artImages.length > size || !m.valid || upload !== null || artReady} onClick={() => void uploadArt()} data-testid="launch-art-upload">
                {artReady ? "Art uploaded ✓" : upload ? "Uploading…" : "Upload art"}
              </button>
              {upload && (
                <span className="text-muted text-sm" data-testid="launch-art-progress">
                  {upload.stage}: {upload.done} / {upload.total}
                </span>
              )}
              {artReady && art && (
                <a className="text-accent-text text-sm underline" href={art.collectionUrl} target="_blank" rel="noreferrer" data-testid="launch-art-link">
                  Collection metadata
                </a>
              )}
            </div>
            {uploadErr && <p role="alert" className="text-negative text-sm" data-testid="launch-art-error">{uploadErr}</p>}
            {devBuyField}
          </div>
        )}

        {names[step] === "Review & launch" && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Review and launch</h2>
            <dl data-testid="launch-review">
              <Row k="Launch type" val={TNAME[t]} testId="launch-review-type" />
              <Row k="Token" val={`${form.name || "—"} · $${form.symbol || "—"}`} testId="launch-review-token" />
              <Row
                k="Image"
                testId="launch-review-image"
                val={
                  // eslint-disable-next-line @next/next/no-img-element
                  tokenImage ? <img src={tokenImage.url} alt="Token" className="inline-block size-10 rounded-full object-cover" /> : "None"
                }
              />
              {!isPlain && (
                <>
                  <Row k="NFT collection" val={collectionName || "—"} testId="launch-review-collection" />
                  <Row k="Collection size" val={`${sizeLabel} NFTs`} testId="launch-review-size" />
                  <Row k="Tokens per NFT" val={`${ratioLabel} tokens = 1 NFT`} testId="launch-review-ratio" />
                  <Row k="Max in NFT form" val={`${formatUnits(m.nftTokens, 0)} tokens (${m.nftPct}%) if every NFT is minted`} testId="launch-review-nft-tokens" />
                </>
              )}
              <Row k="Dev buy" val={devBuyText} testId="launch-review-devbuy" />
            </dl>
            <details className="bg-surface-2 rounded-panel border-border border p-3 text-sm" data-testid="launch-tech">
              <summary className="cursor-pointer font-medium">Technical details</summary>
              <dl className="mt-2">
                <Row k="Supply" val="Fixed at 1,000,000,000 tokens" />
                <Row k="Decimals" val={String(LAUNCH_DECIMALS)} />
                <Row k="Graduation target" val={`${gradText} raised, then a locked trading pool`} />
                <Row k="Bonding curve" val={`Armory curve · settings ${dbcConfig ? shortAddr(dbcConfig.toBase58(), 4) : "—"} · ${sched ? `${sched.endPct}% per trade` : "…"}`} />
                <Row k="Dev buy fee" val={antiSnipeText} />
                <Row k="Token metadata" val={metaUri ? <span className="font-mono text-xs">{metaUri}</span> : "Uploaded to Arweave (Irys) at launch"} />
                <Row k="Mint authority" val="Revoked at launch" />
                <Row k="Freeze authority" val="Revoked at launch" />
                {!isPlain && (
                  <>
                    <Row k="Art commitment" val={art ? <span className="font-mono text-xs">{art.collectionUri} · root {Array.from(art.traitRoot.slice(0, 4), (b) => b.toString(16).padStart(2, "0")).join("")}…</span> : "—"} />
                    <Row k="Randomness" val="Committed art + Switchboard On-Demand" />
                    <Row k="Platform fee" val={`${fee !== null ? formatSol(fee, 3) : ""} per capture or re-roll`} />
                    <Row k="Mint deposit" val={`${formatSol(MINT_ESCROW_LAMPORTS, 4)}, refunded except ${FIRST_MINT_RANGE_TEXT} if the NFT is minted new`} />
                    <Row k="Converting back" val="No platform fee" />
                  </>
                )}
                <Row k="Launch fee" val="None · you pay Solana rent" />
                <Row k="Program upgrades" val="Not locked yet · multisig + 7-day delay planned for mainnet" />
              </dl>
            </details>
            <label className="flex items-start gap-2 text-sm">
              <input type="checkbox" className="mt-1" checked={ack} onChange={(e) => setAck(e.target.checked)} data-testid="launch-ack" />
              <span className="text-muted">I understand that the launch type{isPlain ? "" : ", collection size and art"} can&apos;t be changed after launch.</span>
            </label>
            {!launchedOk && (
              <button
                type="button"
                className="btn btn-primary btn-lg w-full"
                disabled={!live || !ack || !connected || !formOk || !dbcConfig || !!devBuyErr || (!isPlain && !artReady) || safeSend.busy}
                onClick={launch}
                data-testid="launch-submit"
              >
                {live ? (isPlain ? "Launch token" : "Launch Hybrid (step 1 of 2)") : `${TNAME[t]}: ${STATUS_LABEL[launchTypeStatus(t, CLUSTER.name)].toLowerCase()}`}
              </button>
            )}
            {!isPlain && launchedOk && !registered && (
              <div className="space-y-1">
                <button type="button" className="btn btn-primary btn-lg w-full" disabled={!connected || !artReady || safeSend.busy} onClick={register} data-testid="launch-register">
                  Create the NFT vault (step 2 of 2)
                </button>
                <p className="text-dim text-xs">Your token is live on the curve. This records the conversion rate and locks your art into the collection vault.</p>
              </div>
            )}
            {live && !connected && <p className="text-muted text-sm">Connect a wallet to launch.</p>}
            {safeSend.status === "error" && !safeSend.preview && (
              <p role="alert" className="text-negative text-sm" data-testid="launch-error">{safeSend.error}</p>
            )}
            {launchedOk && launchedMint && (isPlain || registered) && (
              <div role="status" className="bg-surface-2 rounded-panel border-border flex items-end gap-4 border px-4 pt-2" data-testid="launch-success">
                <Knight k="c" glow sizes="96px" className="w-[84px] shrink-0 sm:w-[96px]" />
                <p className="text-positive-text pb-4 text-sm">
                  <span className="text-fg block text-base font-semibold">Launched.</span>
                  <Link className="underline" href={`/t/${launchedMint.toBase58()}`}>Open the token page</Link>.
                </p>
              </div>
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
                <Row k="Unsold buffer, locked" val={`${formatUnits(split.buffer, 0)} · ${split.pct[2]}%`} />
                <Row k="Dev buy" val={devBuyText} />
              </dl>
            </>
          ) : (
            <>
              <p className="hd text-3xl" data-testid="math-size">{m.sizeOk ? `${sizeLabel} NFTs` : "—"}</p>
              <p className="text-muted text-xs">× {ratioLabel} tokens each</p>
              <p className="text-sm" data-testid="math-lead">
                {!m.sizeOk ? v.errors.collectionSize : `Up to ${formatUnits(m.nftTokens, 0)} tokens (${m.nftPct}% of the fixed supply) can be in NFT form at once.${m.restTokens > 0n ? ` At least ${formatUnits(m.restTokens, 0)} always stay as tokens.` : ""}`}
              </p>
              <div className="progress"><span style={{ width: `${m.sizeOk ? m.nftPct : 100}%`, background: m.sizeOk ? undefined : "var(--arm-color-status-warn-default)" }} /></div>
              <dl>
                <Row k="Can become NFTs" val={m.sizeOk ? `${formatUnits(m.nftTokens, 0)} · ${m.nftPct}%` : "—"} testId="math-nft-tokens" />
                <Row k="Always tokens" val={m.sizeOk ? `${formatUnits(m.restTokens, 0)} · ${Math.round((100 - m.nftPct) * 10) / 10}%` : "—"} testId="math-rest-tokens" />
                <Row k="Allowed size at this ratio" val={`100 to ${m.maxSize.toLocaleString("en-US")} NFTs`} testId="math-max" />
                <Row k="Graduation target" val={gradText} />
                <Row k="Holder pays per NFT" val={fee !== null ? `${formatSol(fee, 3)} + ${formatSol(MINT_ESCROW_LAMPORTS, 4)} deposit, mostly refunded` : "—"} testId="math-fee" />
                <Row k="Dev buy" val={devBuyText} />
              </dl>
              <div className="bg-surface-2 rounded-panel border-border border p-3 text-sm">
                <p className="text-muted text-xs">Price of 1 NFT (estimate) · {ratioLabel} × token price</p>
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
              </div>
            </>
          )}
          <p className={`rounded-panel border p-2 text-sm ${m.valid ? "tag-ok" : "border-warning-border bg-warning-bg text-warning"}`} data-testid="math-status">
            {m.valid ? (isPlain ? `Valid: graduates at ${gradText}.` : `Valid: ${sizeLabel} NFTs at ${ratioLabel} tokens each.`) : (v.errors.collectionSize ?? v.errors.graduationSol)}
          </p>
          <p className="text-dim text-xs">Decimals {LAUNCH_DECIMALS} · {compact(1_000_000_000)} fixed supply</p>
        </section>
      </aside>
      <TxPreviewModal safeSend={safeSend} />
    </div>
  );
}
