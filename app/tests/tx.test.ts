import { describe, expect, it } from "vitest";
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
  type Connection,
} from "@solana/web3.js";
import { resolveCluster } from "@/config/cluster";
import {
  HYBRID_LAUNCH_PROGRAM_ID,
  HYBRID_VAULT_PROGRAM_ID,
  MEMO_PROGRAM_ID,
  MPL_CORE_PROGRAM_ID,
  SWITCHBOARD_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  isAllowedProgram,
} from "@/config/programs";
import { validateInstructions } from "@/lib/tx/validate";
import { parseInvokedPrograms, parseTokenAccount, simulate } from "@/lib/tx/simulate";
import { buildTxPreview } from "@/lib/tx/preview";
import { viewMessage } from "@/lib/tx/message";

const payer = Keypair.generate().publicKey;
const ix = (programId: PublicKey) =>
  new TransactionInstruction({ programId, keys: [{ pubkey: payer, isSigner: true, isWritable: true }], data: Buffer.from([1]) });
const legacy = (...ixs: TransactionInstruction[]) => {
  const tx = new Transaction();
  if (ixs.length > 0) tx.add(...ixs);
  tx.feePayer = payer;
  return tx;
};

describe("program allowlist", () => {
  it("allows the pinned top-level programs incl. the Armory devnet programs", () => {
    expect(HYBRID_LAUNCH_PROGRAM_ID.toBase58()).toBe("9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8");
    expect(HYBRID_VAULT_PROGRAM_ID.toBase58()).toBe("BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG");
    for (const p of [TOKEN_PROGRAM_ID, MEMO_PROGRAM_ID, HYBRID_LAUNCH_PROGRAM_ID, HYBRID_VAULT_PROGRAM_ID]) {
      expect(isAllowedProgram(p, "localnet")).toBe(true);
      expect(isAllowedProgram(p, "devnet")).toBe(true);
    }
    expect(isAllowedProgram(SystemProgram.programId)).toBe(true);
    expect(isAllowedProgram(ComputeBudgetProgram.programId)).toBe(true);
  });

  it("does not allow placeholders or random programs", () => {
    // CPI-only programs are labelled but never a top-level target from this app.
    for (const p of [TOKEN_2022_PROGRAM_ID, MPL_CORE_PROGRAM_ID, SWITCHBOARD_PROGRAM_ID]) expect(isAllowedProgram(p, "devnet")).toBe(false);
    expect(isAllowedProgram(Keypair.generate().publicKey)).toBe(false);
    expect(isAllowedProgram("not-a-key")).toBe(false);
  });
});

describe("validateInstructions", () => {
  it("accepts allowlisted-only transactions", () => {
    const tx = legacy(
      ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
      SystemProgram.transfer({ fromPubkey: payer, toPubkey: Keypair.generate().publicKey, lamports: 1 }),
      ix(MEMO_PROGRAM_ID),
      ix(HYBRID_VAULT_PROGRAM_ID),
    );
    const r = validateInstructions(tx, "localnet");
    expect(r.errors).toEqual([]);
    expect(r.ok).toBe(true);
    expect(r.instructions.map((i) => i.programName)).toEqual([
      "Compute Budget",
      "System Program",
      "SPL Memo",
      "Armory hybrid_vault",
    ]);
  });

  it("rejects a transaction containing any unknown program", () => {
    const evil = Keypair.generate().publicKey;
    const r = validateInstructions(legacy(ix(MEMO_PROGRAM_ID), ix(evil)), "devnet");
    expect(r.ok).toBe(false);
    expect(r.instructions[1]).toMatchObject({ allowed: false, programId: evil.toBase58(), programName: null });
    expect(r.errors.join()).toContain(evil.toBase58());
  });

  it("rejects empty and malformed transactions", () => {
    expect(validateInstructions(legacy(), "localnet").ok).toBe(false);
    const noPayer = new Transaction().add(ix(MEMO_PROGRAM_ID));
    expect(validateInstructions(noPayer, "localnet").ok).toBe(false);
  });

  it("works for versioned transactions", () => {
    const good = new VersionedTransaction(
      new TransactionMessage({ payerKey: payer, recentBlockhash: PublicKey.default.toBase58(), instructions: [ix(HYBRID_LAUNCH_PROGRAM_ID)] }).compileToV0Message(),
    );
    expect(validateInstructions(good, "localnet").ok).toBe(true);
    const bad = new VersionedTransaction(
      new TransactionMessage({
        payerKey: payer,
        recentBlockhash: PublicKey.default.toBase58(),
        instructions: [ix(Keypair.generate().publicKey)],
      }).compileToV0Message(),
    );
    expect(validateInstructions(bad, "localnet").ok).toBe(false);
  });
});

describe("simulation helpers", () => {
  it("parses invoked programs incl. CPI depth", () => {
    const logs = [
      `Program ${HYBRID_VAULT_PROGRAM_ID.toBase58()} invoke [1]`,
      `Program ${MPL_CORE_PROGRAM_ID.toBase58()} invoke [2]`,
      `Program ${MPL_CORE_PROGRAM_ID.toBase58()} success`,
      `Program ${HYBRID_VAULT_PROGRAM_ID.toBase58()} success`,
    ];
    expect(parseInvokedPrograms(logs)).toEqual([
      { programId: HYBRID_VAULT_PROGRAM_ID.toBase58(), depth: 1 },
      { programId: MPL_CORE_PROGRAM_ID.toBase58(), depth: 2 },
    ]);
  });

  it("parses SPL token accounts and rejects non-token data", () => {
    const mint = Keypair.generate().publicKey;
    const data = new Uint8Array(165);
    data.set(mint.toBytes(), 0);
    data.set(payer.toBytes(), 32);
    data.set([0x39, 0x30], 64); // 12345 LE
    const t = parseTokenAccount(TOKEN_PROGRAM_ID, data);
    expect(t?.mint.equals(mint)).toBe(true);
    expect(t?.owner.equals(payer)).toBe(true);
    expect(t?.amount).toBe(12345n);
    expect(parseTokenAccount(SystemProgram.programId, data)).toBeNull();
    expect(parseTokenAccount(TOKEN_PROGRAM_ID, new Uint8Array(82))).toBeNull();
    const ext = new Uint8Array(200);
    ext.set(data, 0);
    ext[165] = 2;
    expect(parseTokenAccount(TOKEN_2022_PROGRAM_ID, ext)?.amount).toBe(12345n);
    ext[165] = 1; // a mint, not an account
    expect(parseTokenAccount(TOKEN_2022_PROGRAM_ID, ext)).toBeNull();
  });

  it("simulate() diffs SOL balances and builds a sendable preview", async () => {
    const to = Keypair.generate().publicKey;
    const tx = legacy(SystemProgram.transfer({ fromPubkey: payer, toPubkey: to, lamports: 1_000 }));
    tx.recentBlockhash = PublicKey.default.toBase58();
    const calls: unknown[] = [];
    const fake = {
      getMultipleAccountsInfo: async (keys: PublicKey[]) =>
        keys.map((k) => (k.equals(payer) ? { lamports: 10_000_000, owner: SystemProgram.programId, data: Buffer.alloc(0), executable: false } : null)),
      simulateTransaction: async (_tx: VersionedTransaction, config: unknown) => {
        calls.push(config);
        return {
          context: { slot: 1 },
          value: {
            err: null,
            logs: [`Program ${SystemProgram.programId.toBase58()} invoke [1]`, `Program ${SystemProgram.programId.toBase58()} success`],
            unitsConsumed: 150,
            accounts: [
              { lamports: 10_000_000 - 1_000 - 5_000, owner: SystemProgram.programId.toBase58(), data: ["", "base64"], executable: false, rentEpoch: 0 },
              { lamports: 1_000, owner: SystemProgram.programId.toBase58(), data: ["", "base64"], executable: false, rentEpoch: 0 },
            ],
          },
        };
      },
    } as unknown as Connection;

    const sim = await simulate(fake, tx, { signer: payer });
    expect(calls[0]).toMatchObject({ sigVerify: false, replaceRecentBlockhash: true, accounts: { encoding: "base64" } });
    expect(sim.ok).toBe(true);
    expect(sim.unitsConsumed).toBe(150);
    expect(sim.solChanges.map((c) => c.deltaLamports)).toEqual([-6_000n, 1_000n]);

    const preview = buildTxPreview({
      cluster: resolveCluster({}),
      view: viewMessage(tx),
      fingerprint: "00",
      validation: validateInstructions(tx, "localnet"),
      simulation: sim,
      feeLamports: 5_000n,
      signer: payer.toBase58(),
    });
    expect(preview.canSend).toBe(true);
    expect(preview.programs[0]).toMatchObject({ name: "System Program", allowlisted: true });
    expect(preview.solChanges[0]?.display).toBe("-0.000006 SOL");
    expect(preview.solChanges[1]?.display).toBe("+0.000001 SOL");
    expect(preview.feeDisplay).toBe("0.000005 SOL");
  });

  it("preview blocks sending on failed simulation or foreign fee payer", async () => {
    const tx = legacy(ix(MEMO_PROGRAM_ID));
    const view = viewMessage(tx);
    const base = {
      cluster: resolveCluster({}),
      view,
      fingerprint: "00",
      validation: validateInstructions(tx, "localnet"),
      feeLamports: null,
    };
    const failed = buildTxPreview({
      ...base,
      signer: payer.toBase58(),
      simulation: {
        ok: false, error: "InstructionError", rawError: "InstructionError", logs: [], unitsConsumed: null,
        solChanges: [], tokenChanges: [], invokedPrograms: [], diffedAccounts: 1, usesLookupTables: false, slot: 1,
      },
    });
    expect(failed.canSend).toBe(false);
    const foreign = buildTxPreview({ ...base, signer: Keypair.generate().publicKey.toBase58(), simulation: null });
    expect(foreign.canSend).toBe(false);
    expect(foreign.errors.join()).toContain("is not the connected wallet");
  });
});
