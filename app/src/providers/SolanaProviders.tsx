"use client";

/**
 * Wallet + connection providers.
 *
 * - Endpoint comes ONLY from src/config/cluster.ts (NEXT_PUBLIC_SOLANA_CLUSTER /
 *   NEXT_PUBLIC_SOLANA_RPC_URL); requests retry on 429 with backoff (src/lib/rpc.ts).
 * - wallets: explicit Phantom + Solflare adapters, so both are always listed in
 *   the modal (with their install page if the extension isn't there: picking a
 *   wallet that isn't detected opens its site in a new tab). Any installed
 *   Wallet Standard wallet (Phantom, Solflare, Backpack, ...) is auto-detected
 *   and replaces the adapter of the same name, so nothing is listed twice.
 *   Solflare without the extension is treated as "not detected" (opens its
 *   install page) instead of the SDK's web-wallet iframe, which our CSP
 *   (frame-src 'none') blocks and which connect.solflare.com refuses to be framed
 *   anyway (X-Frame-Options: sameorigin). On iOS the adapter's deep link into the
 *   Solflare app is kept; on Android, Mobile Wallet Adapter is added automatically.
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
import { WalletAdapterNetwork, WalletNotReadyError, WalletReadyState, isIosAndRedirectable, type Adapter, type WalletError } from "@solana/wallet-adapter-base";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-phantom";
import { SolflareWalletAdapter } from "@solana/wallet-adapter-solflare";
import { CLUSTER } from "@/config/cluster";
import { connectionConfig } from "@/lib/rpc";

/** Solflare adapter that never falls back to the web-wallet iframe on desktop (see header). */
class SolflareExtensionAdapter extends SolflareWalletAdapter {
  override get readyState(): WalletReadyState {
    const rs = super.readyState;
    return rs === WalletReadyState.Loadable && !isIosAndRedirectable() ? WalletReadyState.NotDetected : rs;
  }
}

function ConnectOnSelect() {
  const { wallet, connected, connecting, connect, select } = useWallet();
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
    const adapter = wallet.adapter;
    connect().catch((e: unknown) => {
      // Not installed (e.g. Phantom without the extension): open its install page in a new tab
      // (still within the click's user activation) and go back to "Select Wallet".
      // Other rejections are reported through WalletProvider onError.
      if (e instanceof WalletNotReadyError) {
        window.open(adapter.url, "_blank", "noopener,noreferrer");
        select(null);
      }
    });
  }, [visible, name, wallet, connected, connecting, connect, select]);
  return null;
}

export function SolanaProviders({ children }: { children: ReactNode }) {
  const wallets = useMemo<Adapter[]>(
    () => [new PhantomWalletAdapter(), new SolflareExtensionAdapter({ network: CLUSTER.isMainnet ? WalletAdapterNetwork.Mainnet : WalletAdapterNetwork.Devnet })],
    [],
  );
  const config = useMemo(() => connectionConfig(CLUSTER.wsUrl), []);
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
