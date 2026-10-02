import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";
import { AddressLookupTableAccount, Keypair, PublicKey, Transaction, TransactionInstruction, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import { HYBRID_VAULT_PROGRAM_ID, SWITCHBOARD_PROGRAM_ID } from "@/config/programs";
import { SWITCHBOARD_GATEWAY_RE } from "@/config/integrations";
import { customErrorCode, nextOracleStep, rightKeyFromLogs } from "@/lib/armory/oracle";
import { checkGatewayUrl } from "@/lib/armory/reveal";
import { checkLookups } from "@/lib/tx/lookup";
import { validateInstructions } from "@/lib/tx/validate";
import { Writer } from "@/lib/generated/borsh";
import { encodeMintArgs } from "@/lib/generated/hybridVault";
import { DBC_INIT_POOL_SPL_DISC } from "@/lib/generated/dbc";

const pk = () => Keypair.generate().publicKey;

describe("oracle retry steps", () => {
  const a = pk(), b = pk(), c = pk();
  it("parses custom codes and the Right: key", () => {
    expect(customErrorCode({ InstructionError: [1, { Custom: 6050 }] })).toBe(6050);
    expect(customErrorCode({ InstructionError: [0, "MissingRequiredSignature"] })).toBeNull();
    expect(rightKeyFromLogs(["Program log: Left:", "Program log: x", "Program log: Right:", `Program log: ${b.toBase58()}`])?.equals(b)).toBe(true);
    expect(rightKeyFromLogs(["Program log: Right:", "Program log: not-a-key"])).toBeNull();
  });
  it("6050 retries with the named oracle; 6053 rotates and records stale", () => {
    const wrong = nextOracleStep({ err: { InstructionError: [1, { Custom: 6050 }] }, logs: ["Program log: Right:", `Program log: ${b.toBase58()}`] }, { oracle: a, stale: [] }, [a, b, c]);
    expect(wrong.kind === "retry" && wrong.next.oracle.equals(b)).toBe(true);
    const same = nextOracleStep({ err: { InstructionError: [1, { Custom: 6050 }] }, logs: ["Program log: Right:", `Program log: ${a.toBase58()}`] }, { oracle: a, stale: [] }, [a]);
    expect(same.kind).toBe("stop");
    const stale = nextOracleStep({ err: { InstructionError: [1, { Custom: 6053 }] }, logs: [] }, { oracle: a, stale: [] }, [a, b]);
    expect(stale.kind === "retry" && stale.next.oracle.equals(b) && stale.next.stale[0]!.equals(a)).toBe(true);
    expect(nextOracleStep({ err: { InstructionError: [1, { Custom: 6053 }] }, logs: [] }, { oracle: b, stale: [a] }, [a, b]).kind).toBe("stop");
    expect(nextOracleStep({ err: null, logs: [] }, { oracle: a, stale: [] }, [a]).kind).toBe("done");
    // 6053 with a Right: key follows the program's answer (skipping candidate order).
    const rightStale = nextOracleStep({ err: { InstructionError: [1, { Custom: 6053 }] }, logs: ["Program log: Right:", `Program log: ${c.toBase58()}`] }, { oracle: a, stale: [] }, [a, b, c]);
    expect(rightStale.kind === "retry" && rightStale.next.oracle.equals(c) && rightStale.next.stale[0]!.equals(a)).toBe(true);
  });
});

describe("Switchboard gateway allowlist", () => {
  it("accepts devnet xip gateways only", () => {
    expect(checkGatewayUrl("https://141.95.35.110.xip.switchboard-oracles.xyz/devnet", "devnet")).toBeTruthy();
    for (const bad of [
      "http://141.95.35.110.xip.switchboard-oracles.xyz/devnet",
      "https://evil.com/141.95.35.110.xip.switchboard-oracles.xyz/devnet",
      "https://141.95.35.110.xip.switchboard-oracles.xyz.evil.com/devnet",
      "https://141.95.35.110.xip.switchboard-oracles.xyz/mainnet",
      "https://a.b.c.d.xip.switchboard-oracles.xyz/devnet",
    ]) expect(() => checkGatewayUrl(bad, "devnet")).toThrow(/allowlist/);
    expect(SWITCHBOARD_GATEWAY_RE.localnet).toBeNull();
  });
});

describe("address lookup tables", () => {
  const payer = pk(), table = pk(), k0 = pk(), k1 = pk();
  const msg = (addresses: PublicKey[], authority?: PublicKey) => {
    const alt = new AddressLookupTableAccount({ key: table, state: { deactivationSlot: 18446744073709551615n, lastExtendedSlot: 0, lastExtendedSlotStartIndex: 0, authority, addresses } });
    const ix = new TransactionInstruction({ programId: HYBRID_VAULT_PROGRAM_ID, keys: [{ pubkey: k0, isSigner: false, isWritable: true }, { pubkey: k1, isSigner: false, isWritable: false }], data: Buffer.from([1]) });
    const m = new TransactionMessage({ payerKey: payer, recentBlockhash: PublicKey.default.toBase58(), instructions: [ix] }).compileToV0Message([alt]);
    return { m, alt };
  };
  it("resolves a pinned, frozen, active table", () => {
    const { m, alt } = msg([k0, k1]);
    const r = checkLookups(m, new Map([[table.toBase58(), alt]]), [table]);
    expect(r.errors).toEqual([]);
    expect(r.writable[0]!.equals(k0) && r.readonly[0]!.equals(k1)).toBe(true);
  });
  it("blocks unpinned, mutable, missing tables and bad indexes", () => {
    const { m, alt } = msg([k0, k1], pk());
    const r = checkLookups(m, new Map([[table.toBase58(), alt]]), []);
    expect(r.errors.join(" ")).toMatch(/not pinned.*authority/);
    expect(checkLookups(m, new Map(), [table]).errors.join(" ")).toMatch(/not found/);
    const short = new AddressLookupTableAccount({ key: table, state: { ...alt.state, authority: undefined, addresses: [k0] } });
    expect(checkLookups(m, new Map([[table.toBase58(), short]]), [table]).errors.join(" ")).toMatch(/no entry #1/);
  });
  it("accepts a table with an authority only when its contents equal the pinned list", () => {
    const { m, alt } = msg([k0, k1], pk());
    const key = table.toBase58();
    const ok = checkLookups(m, new Map([[key, alt]]), [table], { [key]: [k0.toBase58(), k1.toBase58()] });
    expect(ok.errors).toEqual([]);
    expect(ok.tables[0]!.verified).toBe(true);
    const bad = checkLookups(m, new Map([[key, alt]]), [table], { [key]: [k1.toBase58(), k0.toBase58()] });
    expect(bad.errors.join(" ")).toMatch(/doesn't match its pinned contents/);
    const extra = checkLookups(m, new Map([[key, alt]]), [table], { [key]: [k0.toBase58()] });
    expect(extra.errors.join(" ")).toMatch(/pinned contents/);
  });
});

describe("encoders", () => {
  it("encodes MintArgs as Option<(8 x u16, 3 x [u8;32], string, Vec<[u8;32]>)>", () => {
    expect([...encodeMintArgs(new Writer(), null).toBuffer()]).toEqual([0]);
    const z = new Uint8Array(32);
    const buf = encodeMintArgs(new Writer(), { leaf: { traitValues: [1, 2, 3, 4, 5, 6, 7, 258], salt: z, imageSha256: z, jsonSha256: z, uri: "ab" }, proof: [z, z] }).toBuffer();
    expect(buf.length).toBe(1 + 16 + 96 + 4 + 2 + 4 + 64);
    expect([...buf.subarray(15, 17)]).toEqual([2, 1]);
    expect(() => encodeMintArgs(new Writer(), { leaf: { traitValues: [1], salt: z, imageSha256: z, jsonSha256: z, uri: "" }, proof: [] })).toThrow();
  });
  it("DBC discriminator is sha256('global:initialize_virtual_pool_with_spl_token')[0..8]", () => {
    const d = createHash("sha256").update("global:initialize_virtual_pool_with_spl_token").digest().subarray(0, 8);
    expect([...d]).toEqual([...DBC_INIT_POOL_SPL_DISC]);
  });
});

describe("validate: CPI-only programs", () => {
  it("rejects a top-level call to a CPI-only program", () => {
    const payer = pk();
    const tx = new Transaction({ feePayer: payer, recentBlockhash: PublicKey.default.toBase58() }).add(
      new TransactionInstruction({ programId: SWITCHBOARD_PROGRAM_ID, keys: [{ pubkey: payer, isSigner: true, isWritable: true }], data: Buffer.from([1]) }),
    );
    const r = validateInstructions(tx, "devnet");
    expect(r.ok).toBe(false);
    expect(r.errors.join(" ")).toMatch(/CPI-only/);
    expect(validateInstructions(new VersionedTransaction(new TransactionMessage({ payerKey: payer, recentBlockhash: PublicKey.default.toBase58(), instructions: [new TransactionInstruction({ programId: HYBRID_VAULT_PROGRAM_ID, keys: [], data: Buffer.from([1]) })] }).compileToV0Message()), "devnet").ok).toBe(true);
  });
});
