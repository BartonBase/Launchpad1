/**
 * Typed client for `hybrid_vault`, hand-written from src/lib/generated/idl/hybrid_vault.json (the
 * DEVNET IDL from /workspace/idl/devnet, byte-identical to the deployed program, commit 5cd7a7e)
 * and docs/lazy-mint-interface.md. Discriminators come from the generated idlMeta.ts.
 * The deployed vault predates QA-FEE-03: it never raises 6061 FeeNotTier, so the client checks the
 * exact tier fee itself (src/lib/armory/fees.ts).
 */
import { PublicKey, TransactionInstruction, type AccountMeta } from "@solana/web3.js";
import {
  ADDRESS_LOOKUP_TABLE_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  HYBRID_VAULT_PROGRAM_ID,
  MPL_CORE_PROGRAM_ID,
  PLATFORM_FEE_RECIPIENT,
  SLOT_HASHES_SYSVAR,
  SWITCHBOARD_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  WRAPPED_SOL_MINT,
} from "@/config/programs";
import { Reader, Writer, hasDiscriminator, u32le, u64le, utf8 } from "./borsh";
import { hybridVaultAccounts, hybridVaultErrors, hybridVaultIx } from "./idlMeta";
import { ataAddress } from "./spl";

const PID = HYBRID_VAULT_PROGRAM_ID;
const pda = (seeds: (Uint8Array | Buffer)[]) => PublicKey.findProgramAddressSync(seeds, PID)[0];

export const vaultPda = (launchConfig: PublicKey) => pda([utf8("vault"), launchConfig.toBuffer()]);
export const vaultAuthorityPda = (vault: PublicKey) => pda([utf8("vault_authority"), vault.toBuffer()]);
export const randomnessAuthorityPda = (vault: PublicKey) => pda([utf8("randomness_authority"), vault.toBuffer()]);
export const vaultTokensPda = (vault: PublicKey) => pda([utf8("vault_tokens"), vault.toBuffer()]);
export const collectionPda = (vault: PublicKey) => pda([utf8("collection"), vault.toBuffer()]);
export const requestPda = (vault: PublicKey, seq: bigint) => pda([utf8("request"), vault.toBuffer(), u64le(seq)]);
export const mintEscrowPda = (vault: PublicKey, seq: bigint) => pda([utf8("mint_escrow"), vault.toBuffer(), u64le(seq)]);
export const randLockPda = (randomness: PublicKey) => pda([utf8("rand_lock"), randomness.toBuffer()]);
export const assetPda = (vault: PublicKey, index: number) => pda([utf8("asset"), vault.toBuffer(), u32le(index)]);

export interface Vault {
  readonly version: number;
  readonly open: boolean;
  readonly launchConfig: PublicKey;
  readonly mint: PublicKey;
  readonly creator: PublicKey;
  readonly collection: PublicKey;
  readonly pool: PublicKey;
  readonly vaultTokens: PublicKey;
  readonly sbQueue: PublicKey;
  /** Merkle root of the committed leaves (settle-with-mint proofs are checked against it). */
  readonly traitRoot: Uint8Array;
  readonly mintedCount: number;
  readonly assetsOutside: bigint;
  readonly pendingCaptures: bigint;
  readonly pendingRerolls: bigint;
  readonly nextSeq: bigint;
  readonly nextSettleSeq: bigint;
  readonly openedAtSlot: bigint;
  readonly totalFeeLamports: bigint;
  readonly totalCaptures: bigint;
  readonly totalRerolls: bigint;
  readonly totalUnwraps: bigint;
  readonly totalRecommits: bigint;
  readonly totalExpired: bigint;
}

export function decodeVault(data: Uint8Array): Vault {
  if (!hasDiscriminator(data, hybridVaultAccounts.Vault)) throw new Error("Not a Vault");
  const r = new Reader(data, 8);
  const version = r.u8();
  for (let i = 0; i < 5; i++) r.u8(); // bumps
  const open = r.bool();
  const launchConfig = r.pubkey();
  const mint = r.pubkey();
  const creator = r.pubkey();
  const collection = r.pubkey();
  const pool = r.pubkey();
  const vaultTokens = r.pubkey();
  const sbQueue = r.pubkey();
  const traitRoot = r.bytes(32);
  r.bytes(32); // trait_schema_hash
  const mintedCount = r.u32();
  return {
    version, open, launchConfig, mint, creator, collection, pool, vaultTokens, sbQueue, traitRoot, mintedCount,
    assetsOutside: r.u64(), pendingCaptures: r.u64(), pendingRerolls: r.u64(), nextSeq: r.u64(), nextSettleSeq: r.u64(),
    openedAtSlot: r.u64(), totalFeeLamports: r.u64(), totalCaptures: r.u64(), totalRerolls: r.u64(), totalUnwraps: r.u64(),
    totalRecommits: r.u64(), totalExpired: r.u64(),
  };
}

export const NO_HANDED_IN = 0xffff_ffff;

export interface Request {
  readonly kind: number;
  readonly vault: PublicKey;
  readonly seq: bigint;
  readonly user: PublicKey;
  readonly randomness: PublicKey;
  readonly seedSlot: bigint;
  readonly deadlineSlot: bigint;
  readonly commits: number;
  readonly revealed: boolean;
  readonly handedInIndex: number;
  readonly mintEscrowLamports: bigint;
}
/** Offset of Request.user (for getProgramAccounts memcmp): 8 disc + bump + kind + vault + seq. */
export const REQUEST_USER_OFFSET = 8 + 1 + 1 + 32 + 8;

export function decodeRequest(data: Uint8Array): Request {
  if (!hasDiscriminator(data, hybridVaultAccounts.Request)) throw new Error("Not a Request");
  const r = new Reader(data, 8);
  r.u8();
  const kind = r.u8();
  const vault = r.pubkey();
  const seq = r.u64();
  const user = r.pubkey();
  const randomness = r.pubkey();
  const seedSlot = r.u64();
  const deadlineSlot = r.u64();
  const commits = r.u8();
  r.bytes(128); // oracles[4]
  const revealed = r.bool();
  r.bytes(32);
  const handedInIndex = r.u32();
  const mintEscrowLamports = r.u64();
  return { kind, vault, seq, user, randomness, seedSlot, deadlineSlot, commits, revealed, handedInIndex, mintEscrowLamports };
}

/** Raw zero-copy pool header + minted bitmap (layout v2, "hvpool02"). */
export interface PoolView {
  readonly capacity: number;
  readonly poolLen: number;
  readonly incomingLen: number;
  isMinted(index: number): boolean;
  mintedIndexes(): number[];
}
const POOL_MAGIC = utf8("hvpool02");
export function decodePool(data: Uint8Array): PoolView {
  for (let i = 0; i < 8; i++) if (data[i] !== POOL_MAGIC[i]) throw new Error("Not a v2 pool account");
  const v = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const capacity = v.getUint32(40, true);
  const bitmapOff = 64 + 16 * capacity;
  if (data.length < bitmapOff + Math.ceil(capacity / 8)) throw new RangeError("Pool account too short");
  const isMinted = (i: number) => i >= 0 && i < capacity && ((data[bitmapOff + (i >> 3)]! >> (i & 7)) & 1) === 1;
  return {
    capacity,
    poolLen: v.getUint32(44, true),
    incomingLen: v.getUint32(52, true),
    isMinted,
    mintedIndexes: () => Array.from({ length: capacity }, (_, i) => i).filter(isMinted),
  };
}

export function decodeVaultError(code: number): string | null {
  const e = hybridVaultErrors[code];
  return e ? `${e.name}: ${e.msg}` : null;
}

const ro = (pubkey: PublicKey): AccountMeta => ({ pubkey, isSigner: false, isWritable: false });
const rw = (pubkey: PublicKey): AccountMeta => ({ pubkey, isSigner: false, isWritable: true });

export interface CaptureAccounts {
  readonly user: PublicKey;
  readonly vault: PublicKey;
  readonly vaultData: Vault;
  readonly randomness: PublicKey;
  readonly sbOracle: PublicKey;
  /** Optional stale-oracle proofs (M-04), passed as remaining accounts. */
  readonly staleProofs?: readonly PublicKey[];
}

/** `request_capture()` — tokens -> NFT request (tier fee + refundable mint deposit). */
export function requestCaptureIx(a: CaptureAccounts): TransactionInstruction {
  const v = a.vaultData;
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.user, isSigner: true, isWritable: true },
      rw(a.vault),
      ro(v.launchConfig),
      ro(v.pool),
      ro(v.mint),
      rw(ataAddress(a.user, v.mint)),
      rw(vaultTokensPda(a.vault)),
      rw(PLATFORM_FEE_RECIPIENT),
      rw(requestPda(a.vault, v.nextSeq)),
      rw(mintEscrowPda(a.vault, v.nextSeq)),
      rw(randLockPda(a.randomness)),
      rw(a.randomness),
      ro(randomnessAuthorityPda(a.vault)),
      ro(v.sbQueue),
      rw(a.sbOracle),
      ro(SLOT_HASHES_SYSVAR),
      ro(SWITCHBOARD_PROGRAM_ID),
      ro(TOKEN_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
      ...(a.staleProofs ?? []).map(ro),
    ],
    data: new Writer().raw(hybridVaultIx.requestCapture).toBuffer(),
  });
}

/** `request_reroll(index)` — hand in NFT #index for a new random one (tier fee + deposit). */
export function requestRerollIx(a: CaptureAccounts & { readonly index: number }): TransactionInstruction {
  const v = a.vaultData;
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.user, isSigner: true, isWritable: true },
      rw(a.vault),
      ro(v.launchConfig),
      ro(v.pool),
      ro(vaultTokensPda(a.vault)),
      rw(PLATFORM_FEE_RECIPIENT),
      ro(vaultAuthorityPda(a.vault)),
      rw(assetPda(a.vault, a.index)),
      rw(v.collection),
      ro(MPL_CORE_PROGRAM_ID),
      rw(requestPda(a.vault, v.nextSeq)),
      rw(mintEscrowPda(a.vault, v.nextSeq)),
      rw(randLockPda(a.randomness)),
      rw(a.randomness),
      ro(randomnessAuthorityPda(a.vault)),
      ro(v.sbQueue),
      rw(a.sbOracle),
      ro(SLOT_HASHES_SYSVAR),
      ro(SWITCHBOARD_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
      ...(a.staleProofs ?? []).map(ro),
    ],
    data: new Writer().raw(hybridVaultIx.requestReroll).u32(a.index).toBuffer(),
  });
}

/** `unwrap(index)` — release: NFT -> exactly `ratio` tokens. Free (no SOL fee). */
export function unwrapIx(a: { user: PublicKey; vault: PublicKey; vaultData: Vault; index: number }): TransactionInstruction {
  const v = a.vaultData;
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.user, isSigner: true, isWritable: true },
      rw(a.vault),
      ro(v.launchConfig),
      rw(v.pool),
      ro(v.mint),
      ro(vaultAuthorityPda(a.vault)),
      rw(vaultTokensPda(a.vault)),
      rw(ataAddress(a.user, v.mint)),
      rw(assetPda(a.vault, a.index)),
      rw(v.collection),
      ro(MPL_CORE_PROGRAM_ID),
      ro(TOKEN_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
    ],
    data: new Writer().raw(hybridVaultIx.unwrap).u32(a.index).toBuffer(),
  });
}

/** `expire_request()` — permissionless refund of an unrevealed request after its deadline. */
export function expireRequestIx(a: { caller: PublicKey; vault: PublicKey; vaultData: Vault; request: PublicKey; requestData: Request }): TransactionInstruction {
  const v = a.vaultData;
  const rq = a.requestData;
  const asset = rq.handedInIndex === NO_HANDED_IN ? assetPda(a.vault, 0) : assetPda(a.vault, rq.handedInIndex);
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.caller, isSigner: true, isWritable: true },
      rw(a.vault),
      ro(v.launchConfig),
      ro(v.pool),
      rw(a.request),
      rw(randLockPda(rq.randomness)),
      ro(rq.randomness),
      rw(rq.user),
      rw(ataAddress(rq.user, v.mint)),
      ro(v.mint),
      ro(vaultAuthorityPda(a.vault)),
      rw(vaultTokensPda(a.vault)),
      rw(mintEscrowPda(a.vault, rq.seq)),
      rw(asset),
      rw(v.collection),
      ro(MPL_CORE_PROGRAM_ID),
      ro(TOKEN_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
    ],
    data: new Writer().raw(hybridVaultIx.expireRequest).toBuffer(),
  });
}

// ---------------------------------------------------------------------------------------------
// Randomness lifecycle: init_randomness / reveal_randomness / settle_capture / settle_reroll
// (accounts in IDL order, src/lib/generated/idl/hybrid_vault.json)
// ---------------------------------------------------------------------------------------------

/** `init_randomness(recent_slot)`: permissionless; creates a vault-owned Switchboard randomness
 * account. `randomness` is a fresh keypair that must co-sign. */
export function initRandomnessIx(a: {
  payer: PublicKey; vault: PublicKey; randomness: PublicKey; sbQueue: PublicKey; recentSlot: bigint;
  sbProgramState: PublicKey; sbLutSigner: PublicKey; sbLut: PublicKey;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.payer, isSigner: true, isWritable: true },
      ro(a.vault),
      { pubkey: a.randomness, isSigner: true, isWritable: true },
      ro(randomnessAuthorityPda(a.vault)),
      rw(ataAddress(a.randomness, WRAPPED_SOL_MINT)),
      rw(a.sbQueue),
      ro(SYSTEM_PROGRAM_ID),
      ro(TOKEN_PROGRAM_ID),
      ro(ASSOCIATED_TOKEN_PROGRAM_ID),
      ro(WRAPPED_SOL_MINT),
      ro(a.sbProgramState),
      ro(a.sbLutSigner),
      rw(a.sbLut),
      ro(ADDRESS_LOOKUP_TABLE_PROGRAM_ID),
      ro(SWITCHBOARD_PROGRAM_ID),
    ],
    data: new Writer().raw(hybridVaultIx.initRandomness).u64(a.recentSlot).toBuffer(),
  });
}

export interface RevealArgs {
  readonly signature: Uint8Array; // 64
  readonly recoveryId: number;
  readonly value: Uint8Array; // 32
}

/** `reveal_randomness(args)`: permissionless; forwards the gateway-signed reveal to Switchboard. */
export function revealRandomnessIx(a: {
  payer: PublicKey; vault: PublicKey; request: PublicKey; randomness: PublicKey; sbOracle: PublicKey; sbQueue: PublicKey;
  sbStats: PublicKey; sbProgramState: PublicKey; args: RevealArgs;
}): TransactionInstruction {
  if (a.args.signature.length !== 64 || a.args.value.length !== 32) throw new Error("Malformed reveal payload");
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.payer, isSigner: true, isWritable: true },
      ro(a.vault),
      rw(a.request),
      ro(randLockPda(a.randomness)),
      rw(a.randomness),
      ro(randomnessAuthorityPda(a.vault)),
      ro(a.sbOracle),
      ro(a.sbQueue),
      rw(a.sbStats),
      ro(SLOT_HASHES_SYSVAR),
      ro(SYSTEM_PROGRAM_ID),
      rw(ataAddress(a.randomness, WRAPPED_SOL_MINT)),
      ro(TOKEN_PROGRAM_ID),
      ro(WRAPPED_SOL_MINT),
      ro(a.sbProgramState),
      ro(SWITCHBOARD_PROGRAM_ID),
    ],
    data: new Writer().raw(hybridVaultIx.revealRandomness).raw(a.args.signature).u8(a.args.recoveryId).raw(a.args.value).toBuffer(),
  });
}

export interface LeafPreimage {
  readonly traitValues: readonly number[]; // 8 x u16
  readonly salt: Uint8Array; // 32
  readonly imageSha256: Uint8Array; // 32
  readonly jsonSha256: Uint8Array; // 32
  readonly uri: string;
}
export interface MintArgs {
  readonly leaf: LeafPreimage;
  readonly proof: readonly Uint8Array[]; // 32 each
}

export function encodeMintArgs(w: Writer, m: MintArgs | null): Writer {
  if (!m) return w.u8(0);
  const { leaf } = m;
  if (leaf.traitValues.length !== 8 || leaf.salt.length !== 32 || leaf.imageSha256.length !== 32 || leaf.jsonSha256.length !== 32) throw new Error("Malformed leaf");
  w.u8(1);
  for (const t of leaf.traitValues) w.u16(t);
  w.raw(leaf.salt).raw(leaf.imageSha256).raw(leaf.jsonSha256).string(leaf.uri);
  w.u32(m.proof.length);
  for (const p of m.proof) {
    if (p.length !== 32) throw new Error("Malformed proof node");
    w.raw(p);
  }
  return w;
}

/** `settle_capture(mint)` / `settle_reroll(mint)`: permissionless; delivers (or lazily mints with
 * `mint`) the picked NFT to the request's user. */
export function settleIx(a: {
  kind: "capture" | "reroll"; settler: PublicKey; vault: PublicKey; vaultData: Vault; request: PublicKey; seq: bigint;
  randomness: PublicKey; user: PublicKey; assetIndex: number; mint: MintArgs | null;
}): TransactionInstruction {
  const v = a.vaultData;
  const disc = a.kind === "reroll" ? hybridVaultIx.settleReroll : hybridVaultIx.settleCapture;
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.settler, isSigner: true, isWritable: true },
      rw(a.vault),
      ro(v.launchConfig),
      rw(v.pool),
      rw(a.request),
      rw(randLockPda(a.randomness)),
      ro(a.randomness),
      ro(vaultAuthorityPda(a.vault)),
      rw(mintEscrowPda(a.vault, a.seq)),
      ro(vaultTokensPda(a.vault)),
      rw(assetPda(a.vault, a.assetIndex)),
      rw(v.collection),
      rw(a.user),
      ro(MPL_CORE_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
    ],
    data: encodeMintArgs(new Writer().raw(disc), a.mint).toBuffer(),
  });
}

/** InitVaultParams (IDL order): trait_root, trait_schema_hash, collection_name, collection_uri, sb_queue. */
export interface InitVaultParams {
  readonly traitRoot: Uint8Array; // 32
  readonly traitSchemaHash: Uint8Array; // 32
  readonly collectionName: string; // <= 32 bytes
  readonly collectionUri: string; // ipfs:// or ar://, <= 200 bytes
  readonly sbQueue: PublicKey;
}
/** Bytes for the pre-created pool account (e2e.cjs: 64 + 16·N + ceil(N/8)). */
export const vaultPoolBytes = (collectionSize: number) => 64 + 16 * collectionSize + Math.ceil(collectionSize / 8);

export function initVaultIx(a: { creator: PublicKey; launchConfig: PublicKey; mint: PublicKey; pool: PublicKey; params: InitVaultParams }): TransactionInstruction {
  const p = a.params;
  if (p.traitRoot.length !== 32 || p.traitSchemaHash.length !== 32) throw new Error("Malformed art commitment");
  const vault = vaultPda(a.launchConfig);
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.creator, isSigner: true, isWritable: true },
      ro(a.launchConfig),
      ro(a.mint),
      rw(vault),
      rw(vaultAuthorityPda(vault)),
      ro(randomnessAuthorityPda(vault)),
      rw(vaultTokensPda(vault)),
      rw(a.pool),
      rw(collectionPda(vault)),
      ro(MPL_CORE_PROGRAM_ID),
      ro(TOKEN_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
    ],
    data: new Writer().raw(hybridVaultIx.initVault).raw(p.traitRoot).raw(p.traitSchemaHash).string(p.collectionName).string(p.collectionUri).raw(p.sbQueue.toBytes()).toBuffer(),
  });
}

/**
 * `open_vault()` (permissionless, one-way): opens converting once the launch's recorded DBC pool has
 * migrated (graduation_proof = that DBC pool). Accounts in IDL order.
 */
export function openVaultIx(a: { caller: PublicKey; vault: PublicKey; vaultData: Vault; dbcPool: PublicKey }): TransactionInstruction {
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.caller, isSigner: true, isWritable: false },
      rw(a.vault),
      ro(a.vaultData.launchConfig),
      ro(a.vaultData.pool),
      ro(vaultTokensPda(a.vault)),
      ro(a.vaultData.collection),
      ro(a.dbcPool),
    ],
    data: new Writer().raw(hybridVaultIx.openVault).toBuffer(),
  });
}
