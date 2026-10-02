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
 *   ConnectOnSelect: picking a wallet in the modal IS that explicit action, so
 *   it connects right away (without it, the adapter only selects the wallet and
 *   the user has to press "Connect" a second time).
 */
import { useCallback, useEffect, useMemo, useRef, type ReactNode } from "react";
import { ConnectionProvider, WalletProvider, useWallet } from "@solana/wallet-adapter-react";
import { WalletModalProvider, useWalletModal } from "@solana/wallet-adapter-react-ui";
import type { Adapter, WalletError } from "@solana/wallet-adapter-base";
import { CLUSTER } from "@/config/cluster";

function ConnectOnSelect() {
  const { wallet, connected, connecting, connect } = useWallet();
  const { visible } = useWalletModal();
  const atOpen = useRef<string | null | undefined>(undefined); // wallet name when the modal opened
  const pending = useRef(false);
  const name = wallet?.adapter.name ?? null;
  useEffect(() => {
    if (visible) {
      if (atOpen.current === undefined) atOpen.current = name;
      return;
    }
    if (atOpen.current !== undefined) {
      // Modal just closed: connect only if the user picked a (different) wallet in it.
      pending.current = name !== null && name !== atOpen.current;
      atOpen.current = undefined;
    }
    if (!pending.current || !wallet || connected || connecting) return;
    pending.current = false;
    connect().catch(() => {
      /* rejection is reported through WalletProvider onError */
    });
  }, [visible, name, wallet, connected, connecting, connect]);
  return null;
}

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
        <WalletModalProvider>
          <ConnectOnSelect />
          {children}
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
