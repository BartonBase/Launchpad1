import type { Metadata } from "next";
import type { ReactNode } from "react";
import "./globals.css";
import { SolanaProviders } from "@/providers/SolanaProviders";
import { SiteHeader } from "@/components/SiteHeader";
import { SiteFooter } from "@/components/SiteFooter";
import { BRAND } from "@/config/armory";
import { fontVariables } from "./fonts";

/** Chain-backed shell (authorities panel, launches): re-render at most every 30 s (ISR). */
export const revalidate = 30;

export const metadata: Metadata = {
  title: { default: BRAND.name, template: `%s · ${BRAND.name}` },
  description: "Launch a Solana meme coin on its own, or with an NFT collection built in.",
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className={fontVariables}>
      <body className="flex min-h-screen flex-col antialiased">
        <a href="#main" className="skip-link" data-testid="skip-link">
          Skip to content
        </a>
        <SolanaProviders>
          <SiteHeader />
          <main id="main" tabIndex={-1} className="flex-1 outline-none">{children}</main>
          <SiteFooter />
        </SolanaProviders>
      </body>
    </html>
  );
}
