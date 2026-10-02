import { describe, it, expect } from "vitest";
import { Keypair, PublicKey } from "@solana/web3.js";
import { parseHash32, validateArt, type LaunchForm } from "@/lib/armory/launchForm";
import { initVaultIx, vaultPoolBytes } from "@/lib/generated/hybridVault";
import { friendlyProgramError, instructionCustomError, NO_CURVE_MESSAGE } from "@/lib/armory/errors";
import { decodeDbcConfig, DBC_CONFIG_DISCRIMINATOR, DBC_CONFIG_LEN } from "@/lib/generated/dbc";
import { HYBRID_VAULT_PROGRAM_ID } from "@/config/programs";

const H = "ab".repeat(32);
const form = (o: Partial<LaunchForm>) => ({ collectionName: "Knights", collectionUri: "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi", traitRoot: H, schemaHash: "0x" + H, ...o }) as LaunchForm;

describe("art commitment (init_vault)", () => {
  it("parseHash32 takes 64 hex chars with optional 0x", () => {
    expect(parseHash32(H)?.length).toBe(32);
    expect(parseHash32("0x" + H)![0]).toBe(0xab);
    expect(parseHash32("ab".repeat(31))).toBeNull();
    expect(parseHash32("zz".repeat(32))).toBeNull();
  });
  it("validateArt mirrors init_vault limits", () => {
    expect(validateArt(form({})).art).not.toBeNull();
    expect(validateArt(form({ collectionName: "x".repeat(33) })).errors.collectionName).toBeTruthy();
    expect(validateArt(form({ collectionUri: "https://example.com/c.json" })).errors.collectionUri).toBeTruthy();
    expect(validateArt(form({ collectionUri: "ar://" + "a".repeat(43) })).art).not.toBeNull();
    expect(validateArt(form({ traitRoot: "" })).errors.traitRoot).toBeTruthy();
  });
  it("initVaultIx layout: disc + root + schema + name + uri + sb_queue, 12 accounts", () => {
    const q = Keypair.generate().publicKey;
    const art = validateArt(form({})).art!;
    const raw = initVaultIx({
      creator: Keypair.generate().publicKey, launchConfig: Keypair.generate().publicKey, mint: Keypair.generate().publicKey, pool: Keypair.generate().publicKey,
      params: { traitRoot: art.traitRoot, traitSchemaHash: art.traitSchemaHash, collectionName: art.collectionName, collectionUri: art.collectionUri, sbQueue: q },
    });
    const ix = { ...raw, data: Buffer.from(raw.data) };
    expect(ix.programId.equals(HYBRID_VAULT_PROGRAM_ID)).toBe(true);
    expect(ix.keys).toHaveLength(12);
    expect([...ix.data.subarray(0, 8)]).toEqual([77, 79, 85, 150, 33, 217, 52, 106]);
    expect(ix.data[8]).toBe(0xab);
    expect(ix.data.readUInt32LE(72)).toBe(7);
    expect(ix.data.subarray(76, 83).toString()).toBe("Knights");
    expect(ix.data.length).toBe(8 + 64 + 4 + 7 + 4 + art.collectionUri.length + 32);
    expect(new PublicKey(ix.data.subarray(ix.data.length - 32)).equals(q)).toBe(true);
    expect(vaultPoolBytes(10)).toBe(64 + 160 + 2);
  });
});

describe("friendly program errors", () => {
  it("parses InstructionError custom codes", () => {
    expect(instructionCustomError({ InstructionError: [1, { Custom: 6037 }] })).toEqual({ index: 1, code: 6037 });
    expect(instructionCustomError('{"InstructionError":[0,{"Custom":3}]}')).toEqual({ index: 0, code: 3 });
    expect(instructionCustomError("AccountNotFound")).toBeNull();
  });
  it("maps vault 6037 to the no-curve message only for the vault program", () => {
    expect(friendlyProgramError(HYBRID_VAULT_PROGRAM_ID.toBase58(), 6037)).toBe(NO_CURVE_MESSAGE);
    expect(friendlyProgramError(Keypair.generate().publicKey.toBase58(), 6037)).toBeNull();
  });
});

describe("DBC config curve split offsets", () => {
  it("reads swap base / migration base / pre-migration supply", () => {
    const d = new Uint8Array(DBC_CONFIG_LEN);
    d.set(DBC_CONFIG_DISCRIMINATOR, 0);
    const v = new DataView(d.buffer);
    v.setBigUint64(256, 549_999_753_000_000n, true);
    v.setBigUint64(264, 100_000_000n, true);
    v.setBigUint64(272, 200_000_066_000_000n, true);
    v.setBigUint64(344, 1_000_000_000_000_000n, true);
    const c = decodeDbcConfig(d);
    expect(c.swapBaseAmount).toBe(549_999_753_000_000n);
    expect(c.migrationQuoteThreshold).toBe(100_000_000n);
    expect(c.migrationBaseThreshold).toBe(200_000_066_000_000n);
    expect(c.preMigrationTokenSupply).toBe(1_000_000_000_000_000n);
  });
});
