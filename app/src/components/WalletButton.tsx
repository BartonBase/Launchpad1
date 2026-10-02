"use client";

import dynamic from "next/dynamic";

// The wallet button reads browser-only state (installed wallets), so render it
// on the client only to avoid hydration mismatches.
const WalletMultiButton = dynamic(
  async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
  {
    ssr: false,
    loading: () => (
      <span className="btn btn-sm text-muted">Loading wallet…</span>
    ),
  },
);

export function WalletButton() {
  return <WalletMultiButton />;
}
