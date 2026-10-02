"use client";

/**
 * Wallet + connection providers.
 *
 * - Endpoint comes ONLY from src/config/cluster.ts (localnet/devnet; mainnet
 *   throws at import time).
 * - wallets = [] : no per-wallet adapter packages. Wallets that implement the
 *   Wallet Standard (Phantom, Solflare, Backpack, ...) are auto-detected.
 * - autoConnect = false : connecting is always an explicit user action. On a
 *   launchpad the wallet should never be silently re-attached when a page
 *   loads (e.g. after following a link to a collection page); the small UX
 *   cost of clicking "Select Wallet" per visit is worth the explicitness.
 */
import { useCallback, useMemo, type ReactNode } from "react";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import type { Adapter, WalletError } from "@solana/wallet-adapter-base";
import { CLUSTER } from "@/config/cluster";

export function SolanaProviders({ children }: { children: ReactNode }) {
  const wallets = useMemo<Adapter[]>(() => [], []);
  const config = useMemo(() => ({ commitment: "confirmed" as const, wsEndpoint: CLUSTER.wsUrl }), []);
  const onError = useCallback((error: WalletError) => {
    // Never log payloads/keys; the error name + message is enough.
    console.warn(`[wallet] ${error.name}: ${error.message}`);
  }, []);

  return (
    <ConnectionProvider endpoint={CLUSTER.rpcUrl} config={config}>
      <WalletProvider wallets={wallets} autoConnect={false} onError={onError}>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
