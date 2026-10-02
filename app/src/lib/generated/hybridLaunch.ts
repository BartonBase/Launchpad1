/**
 * Typed client for `hybrid_launch`, hand-written from src/lib/generated/idl/hybrid_launch.json
 * (the DEVNET IDL from /workspace/idl/devnet, byte-identical to the deployed program, commit
 * 5cd7a7e, devnet-e2e build). Discriminators come from the generated idlMeta.ts.
 *
 * Deployed instructions: launch, register_dbc_launch. Plain and burn modes (launch_plain,
 * launch_burn, …) exist only on branch fix/modes-1-5 @ c43be58 and are NOT in the devnet IDL, so
 * this client deliberately has no builders for them (UI shows them as "pending deploy").
 */
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, HYBRID_LAUNCH_PROGRAM_ID, PLATFORM_FEE_RECIPIENT, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@/config/programs";
import { Reader, Writer, hasDiscriminator, utf8 } from "./borsh";
import { hybridLaunchAccounts, hybridLaunchIx } from "./idlMeta";
import { ataAddress } from "./spl";

const PID = HYBRID_LAUNCH_PROGRAM_ID;
const pda = (seeds: (Uint8Array | Buffer)[]) => PublicKey.findProgramAddressSync(seeds, PID)[0];

export const launchConfigPda = (mint: PublicKey) => pda([utf8("launch_config"), mint.toBuffer()]);
export const mintAuthorityPda = (launchConfig: PublicKey) => pda([utf8("mint_authority"), launchConfig.toBuffer()]);
export const launchVaultPda = (mint: PublicKey, launchConfig: PublicKey) =>
  pda([utf8("launch_vault"), mint.toBuffer(), launchConfig.toBuffer()]);
export const dbcBufferPda = () => pda([utf8("dbc_buffer")]);

export interface LaunchConfig {
  readonly kind: "hybrid";
  readonly version: number;
  readonly creator: PublicKey;
  readonly mint: PublicKey;
  readonly launchDestination: PublicKey;
  readonly launchVault: PublicKey;
  readonly decimals: number;
  readonly totalSupplyBase: bigint;
  readonly ratioWholeTokens: bigint;
  readonly ratioBase: bigint;
  readonly collectionSize: bigint;
  readonly maxTokensInNftForm: bigint;
  readonly feeLamports: bigint;
  readonly feeRecipient: PublicKey;
  readonly graduationThresholdLamports: bigint;
  readonly launchedAt: bigint;
  readonly dbcConfig: PublicKey;
  readonly dbcPool: PublicKey;
  /** dbc_pool == default => native launch: the vault can never open. */
  readonly onCurve: boolean;
}

export function decodeLaunchConfig(data: Uint8Array): LaunchConfig {
  if (!hasDiscriminator(data, hybridLaunchAccounts.LaunchConfig)) throw new Error("Not a LaunchConfig");
  const r = new Reader(data, 8);
  const version = r.u8();
  r.u8(); // bump
  r.u8(); // mint_authority_bump
  const creator = r.pubkey();
  const mint = r.pubkey();
  const launchDestination = r.pubkey();
  const launchVault = r.pubkey();
  r.u8(); // launch_vault_bump
  const decimals = r.u8();
  const totalSupplyBase = r.u64();
  const ratioWholeTokens = r.u64();
  const ratioBase = r.u64();
  const collectionSize = r.u64();
  const maxTokensInNftForm = r.u64();
  const feeLamports = r.u64();
  const feeRecipient = r.pubkey();
  const graduationThresholdLamports = r.u64();
  r.u8(); // graduation_slice_pct (always 0 since lazy minting)
  const launchedAt = r.i64();
  const dbcConfig = r.pubkey();
  const dbcPool = r.pubkey();
  return {
    kind: "hybrid", version, creator, mint, launchDestination, launchVault, decimals, totalSupplyBase, ratioWholeTokens,
    ratioBase, collectionSize, maxTokensInNftForm, feeLamports, feeRecipient, graduationThresholdLamports, launchedAt,
    dbcConfig, dbcPool, onCurve: !dbcPool.equals(PublicKey.default),
  };
}

export interface LaunchParams {
  readonly decimals: number;
  readonly ratioWholeTokens: bigint;
  readonly collectionSize: bigint;
  readonly graduationThresholdLamports: bigint;
}

function encodeLaunchParams(p: LaunchParams): Buffer {
  return new Writer().raw(hybridLaunchIx.launch).u8(p.decimals).u64(p.ratioWholeTokens).u64(p.collectionSize).u64(p.graduationThresholdLamports).toBuffer();
}

function nativeLaunchKeys(creator: PublicKey, mint: PublicKey) {
  const lc = launchConfigPda(mint);
  const lv = launchVaultPda(mint, lc);
  return [
    { pubkey: creator, isSigner: true, isWritable: true },
    { pubkey: mint, isSigner: true, isWritable: true },
    { pubkey: lc, isSigner: false, isWritable: true },
    { pubkey: mintAuthorityPda(lc), isSigner: false, isWritable: false },
    { pubkey: lv, isSigner: false, isWritable: false },
    { pubkey: ataAddress(lv, mint), isSigner: false, isWritable: true },
    { pubkey: PLATFORM_FEE_RECIPIENT, isSigner: false, isWritable: false },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
  ];
}

/**
 * `launch` (native hybrid launch, DEPLOYED). Mints exactly 1B to a PDA-owned destination,
 * revokes mint + freeze authority and writes the immutable LaunchConfig. No bonding curve:
 * the vault of a native launch can never open (graduation check fails closed). Devnet test use.
 */
export function launchIx(creator: PublicKey, mint: PublicKey, params: LaunchParams): TransactionInstruction {
  return new TransactionInstruction({ programId: PID, keys: nativeLaunchKeys(creator, mint), data: encodeLaunchParams(params) });
}


/**
 * `register_dbc_launch(params)` (ADR-014, Meteora DBC curve path). The DBC pool must already exist
 * (created in the SAME transaction by dbc.initializeVirtualPoolWithSplTokenIx); the program checks
 * the pool/config and records the immutable LaunchConfig. Accounts in IDL order.
 */
export function registerDbcLaunchIx(a: {
  creator: PublicKey; mint: PublicKey; dbcConfig: PublicKey; dbcPool: PublicKey; ratioWholeTokens: bigint; collectionSize: bigint;
}): TransactionInstruction {
  const buffer = dbcBufferPda();
  return new TransactionInstruction({
    programId: PID,
    keys: [
      { pubkey: a.creator, isSigner: true, isWritable: true },
      { pubkey: a.mint, isSigner: false, isWritable: false },
      { pubkey: a.dbcConfig, isSigner: false, isWritable: false },
      { pubkey: a.dbcPool, isSigner: false, isWritable: false },
      { pubkey: launchConfigPda(a.mint), isSigner: false, isWritable: true },
      { pubkey: buffer, isSigner: false, isWritable: false },
      { pubkey: ataAddress(buffer, a.mint), isSigner: false, isWritable: true },
      { pubkey: PLATFORM_FEE_RECIPIENT, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: new Writer().raw(hybridLaunchIx.registerDbcLaunch).u64(a.ratioWholeTokens).u64(a.collectionSize).toBuffer(),
  });
}
