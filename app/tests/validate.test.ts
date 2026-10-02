import { describe, expect, it } from "vitest";
import { Keypair, PublicKey } from "@solana/web3.js";
import { HYBRID_VAULT_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@/config/programs";
import { U64_MAX, formatAmount, parseAmount, parseMetadataUri, parsePublicKey, toGatewayUrl } from "@/lib/validate";

describe("parsePublicKey", () => {
  const wallet = Keypair.generate().publicKey.toBase58();
  const [pda] = PublicKey.findProgramAddressSync([new TextEncoder().encode("escrow")], HYBRID_VAULT_PROGRAM_ID);

  it("accepts valid keys", () => {
    const r = parsePublicKey(wallet);
    expect(r.ok && r.value.toBase58()).toBe(wallet);
    expect(parsePublicKey(TOKEN_PROGRAM_ID.toBase58()).ok).toBe(true);
  });

  it.each([
    ["non-string", 123],
    ["empty", ""],
    ["too short", "abc"],
    ["invalid chars (0,O,I,l)", "0OIl" + wallet.slice(4)],
    ["whitespace", " " + wallet],
    ["too long", wallet + "1111"],
    ["words", "notakey"],
    ["path traversal", "../../etc/passwd"],
  ])("rejects %s", (_n, input) => {
    expect(parsePublicKey(input).ok).toBe(false);
  });

  it("rejects base58 strings that do not decode to 32 bytes", () => {
    expect(parsePublicKey("z".repeat(44)).ok).toBe(false);
  });

  it("enforces on-curve / off-curve", () => {
    expect(parsePublicKey(wallet, { curve: "on" }).ok).toBe(true);
    expect(parsePublicKey(wallet, { curve: "off" }).ok).toBe(false);
    expect(parsePublicKey(pda.toBase58(), { curve: "off" }).ok).toBe(true);
    expect(parsePublicKey(pda.toBase58(), { curve: "on" }).ok).toBe(false);
  });
});

describe("parseAmount", () => {
  it("parses decimals into bigint base units", () => {
    expect(parseAmount("1.5", { decimals: 6 })).toEqual({ ok: true, value: 1_500_000n });
    expect(parseAmount("0.000001", { decimals: 6 })).toEqual({ ok: true, value: 1n });
    expect(parseAmount("42", { decimals: 0 })).toEqual({ ok: true, value: 42n });
    expect(parseAmount("1.10", { decimals: 2 })).toEqual({ ok: true, value: 110n });
  });

  it("is exact where floats would not be", () => {
    // 0.1 + 0.2 style precision issues cannot happen.
    expect(parseAmount("0.3", { decimals: 18 })).toEqual({ ok: true, value: 300_000_000_000_000_000n });
    expect(parseAmount("18446744073709551615", { decimals: 0 })).toEqual({ ok: true, value: U64_MAX });
  });

  it.each(["", "-1", "+1", "1e3", "1,000", " 1", "1 ", "01", ".5", "5.", "1.2.3", "NaN", "Infinity", "0x10", "１"])(
    "rejects %j",
    (s) => {
      expect(parseAmount(s, { decimals: 6 }).ok).toBe(false);
    },
  );

  it("rejects too many decimal places", () => {
    expect(parseAmount("1.0000001", { decimals: 6 }).ok).toBe(false);
    expect(parseAmount("1.5", { decimals: 0 }).ok).toBe(false);
  });

  it("enforces bounds (default min 1, max u64)", () => {
    expect(parseAmount("0", { decimals: 6 }).ok).toBe(false);
    expect(parseAmount("0", { decimals: 6, min: 0n }).ok).toBe(true);
    expect(parseAmount("18446744073709551616", { decimals: 0 }).ok).toBe(false);
    expect(parseAmount("5", { decimals: 0, max: 4n }).ok).toBe(false);
    expect(parseAmount("21", { decimals: 0, min: 1n, max: 20n }).ok).toBe(false);
  });

  it("rejects invalid decimals and non-strings", () => {
    expect(parseAmount("1", { decimals: -1 }).ok).toBe(false);
    expect(parseAmount("1", { decimals: 1.5 }).ok).toBe(false);
    expect(parseAmount("1", { decimals: 19 }).ok).toBe(false);
    expect(parseAmount(1 as unknown as string, { decimals: 0 }).ok).toBe(false);
  });
});

describe("formatAmount", () => {
  it("formats without floats", () => {
    expect(formatAmount(1_500_000n, 6)).toBe("1.5");
    expect(formatAmount(1n, 9)).toBe("0.000000001");
    expect(formatAmount(-5000n, 9)).toBe("-0.000005");
    expect(formatAmount(U64_MAX, 0)).toBe("18446744073709551615");
    expect(formatAmount(0n, 6)).toBe("0");
  });
});

describe("parseMetadataUri", () => {
  it.each([
    ["https://arweave.net/abc123/metadata.json", "arweave"],
    ["https://example.com/meta/1.json", "https"],
    ["ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/1.json", "ipfs"],
    ["ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi", "ipfs"],
    ["ar://bNbA3TEQVL60xlgCcqdz4ZPHFZ711cZ3hmkpGttDt_U", "arweave"],
  ])("accepts %s", (uri, kind) => {
    const r = parseMetadataUri(uri);
    expect(r.ok).toBe(true);
    if (r.ok) expect(r.value.kind).toBe(kind);
  });

  it.each([
    "http://example.com/a.json",
    "javascript:alert(1)",
    "data:application/json,{}",
    "file:///etc/passwd",
    "https://user:pass@example.com/a.json",
    "https://localhost/a.json",
    "https://127.0.0.1/a.json",
    "https://[::1]/a.json",
    "https://example.com:8443/a.json",
    "ipfs://notacid",
    "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/../x",
    "ar://short",
    "https://example.com/a b",
    "https://example.com/" + "a".repeat(200),
    "",
  ])("rejects %s", (uri) => {
    expect(parseMetadataUri(uri).ok).toBe(false);
  });

  it("supports an https host allowlist", () => {
    const opts = { allowedHttpsHosts: ["arweave.net", ".nftstorage.link"] };
    expect(parseMetadataUri("https://arweave.net/x", opts).ok).toBe(true);
    expect(parseMetadataUri("https://abc.ipfs.nftstorage.link/x", opts).ok).toBe(true);
    expect(parseMetadataUri("https://evil.com/x", opts).ok).toBe(false);
    expect(parseMetadataUri("https://evilarweave.net/x", opts).ok).toBe(false);
  });

  it("maps to gateways", () => {
    const r = parseMetadataUri("ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/1.json");
    expect(r.ok && toGatewayUrl(r.value)).toBe("https://ipfs.io/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/1.json");
  });
});
