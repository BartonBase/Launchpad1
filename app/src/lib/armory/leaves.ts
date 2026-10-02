/**
 * Leaf sources for settle-with-mint. Minting the picked NFT needs its committed leaf preimage
 * (traits, salt, image/json hashes, URI) and a Merkle proof against the vault's trait_root. Those
 * come from the creator's traits manifest. Real launches will publish one (not wired yet); the
 * devnet E2E launch uses the deterministic scheme from scripts/devnet-e2e/e2e.cjs, mirrored here
 * and checked against the on-chain trait_root in test:devnet.
 * Leaf/merkle scheme: programs/hybrid_vault/src/merkle.rs (leaf v2, domain-separated nodes).
 */
import type { PublicKey } from "@solana/web3.js";
import type { MintArgs } from "@/lib/generated/hybridVault";
import { u32le, utf8 } from "@/lib/generated/borsh";

async function sha(...parts: (Uint8Array | number[] | string)[]): Promise<Uint8Array> {
  const bytes = parts.map((p) => (typeof p === "string" ? utf8(p) : Uint8Array.from(p)));
  const all = new Uint8Array(bytes.reduce((n, b) => n + b.length, 0));
  let o = 0;
  for (const b of bytes) {
    all.set(b, o);
    o += b.length;
  }
  return new Uint8Array(await globalThis.crypto.subtle.digest("SHA-256", all));
}

export interface LeafSource {
  readonly label: string;
  readonly size: number;
  mintArgs(launchConfig: PublicKey, index: number): Promise<MintArgs>;
  root(launchConfig: PublicKey): Promise<Uint8Array>;
}

/** Deterministic E2E collection (scripts/devnet-e2e/e2e.cjs leafPre/leafHash/merkle). */
export function e2eLeafSource(n: number): LeafSource {
  const pre = (i: number) => ({
    traitValues: Array.from({ length: 8 }, (_, k) => (i * 7 + k * 3) % 11),
    salt: new Uint8Array(32).fill(i & 255),
    imageSha256: new Uint8Array(32).fill((i + 1) & 255),
    jsonSha256: new Uint8Array(32).fill((i + 2) & 255),
    uri: `ipfs://bafydevnete2e${i}`,
  });
  const cache = new Map<string, Promise<Uint8Array[][]>>();
  const levels = (lc: PublicKey) => {
    const key = lc.toBase58();
    let p = cache.get(key);
    if (!p) {
      p = (async () => {
        const schema = await sha("mintmark-devnet-e2e-schema");
        const leaves = await Promise.all(
          Array.from({ length: n }, async (_, i) => {
            const l = pre(i);
            const tvb = new Uint8Array(16);
            l.traitValues.forEach((v, k) => { tvb[2 * k] = v & 255; tvb[2 * k + 1] = v >> 8; });
            const th = await sha(schema, tvb);
            const ah = await sha(l.salt, l.imageSha256, l.jsonSha256, utf8(l.uri));
            return sha([0], "mintmark-leaf-v2", lc.toBytes(), u32le(i), th, ah);
          }),
        );
        const lv: Uint8Array[][] = [leaves];
        while (lv[lv.length - 1]!.length > 1) {
          const c = lv[lv.length - 1]!;
          const nx: Uint8Array[] = [];
          for (let j = 0; j < c.length; j += 2) nx.push(await sha([1], c[j]!, c[j + 1] ?? c[j]!));
          lv.push(nx);
        }
        return lv;
      })();
      cache.set(key, p);
    }
    return p;
  };
  return {
    label: "Devnet E2E deterministic test collection",
    size: n,
    async root(lc) {
      const lv = await levels(lc);
      return lv[lv.length - 1]![0]!;
    },
    async mintArgs(lc, index) {
      if (index < 0 || index >= n) throw new RangeError("Leaf index out of range");
      const lv = await levels(lc);
      const proof: Uint8Array[] = [];
      let i = index;
      for (const level of lv.slice(0, -1)) {
        proof.push(level[i ^ 1] ?? level[i]!);
        i >>= 1;
      }
      return { leaf: pre(index), proof };
    },
  };
}

/** Launch config (base58) -> leaf source, per cluster. Unknown launches have no manifest yet. */
const SOURCES: Record<string, LeafSource> = {
  FKAzFsdDGShgtiMntihnM6BUd8D6fDNMYsUut6en2kYD: e2eLeafSource(100), // devnet E2E launch (mint 3GC9zF…)
};
export function leafSourceFor(launchConfig: PublicKey): LeafSource | null {
  return SOURCES[launchConfig.toBase58()] ?? null;
}
