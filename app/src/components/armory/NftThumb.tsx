"use client";
/** NFT picker thumbnail: the image named by the NFT's Irys metadata (devnet), else a numbered tile. */
import { useEffect, useState } from "react";
import { resolveTokenImage } from "@/lib/meteora/tokenImage";

const cache = new Map<string, Promise<string | null>>();

export function NftThumb({ uri, index }: { uri?: string | null; index: number }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    if (!uri) return;
    let live = true;
    if (!cache.has(uri)) cache.set(uri, resolveTokenImage(uri));
    void cache.get(uri)!.then((s) => live && setSrc(s));
    return () => {
      live = false;
    };
  }, [uri]);
  return (
    <span className="bg-surface-2 rounded-chip mb-2 flex aspect-square items-center justify-center overflow-hidden" aria-hidden="true">
      {src ? (
        // eslint-disable-next-line @next/next/no-img-element
        <img src={src} alt="" loading="lazy" className="size-full object-cover" data-testid="nft-thumb" />
      ) : (
        <span className="hd text-dim text-lg">#{index}</span>
      )}
    </span>
  );
}
