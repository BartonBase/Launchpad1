import type { Metadata } from "next";
import { PortfolioView } from "@/components/armory/PortfolioView";

export const metadata: Metadata = { title: "Portfolio", description: "Your Armory tokens, NFTs and pending requests." };

export default function PortfolioPage() {
  return (
    <div className="mx-auto max-w-(--container-site) space-y-8 px-4 py-10">
      <div className="space-y-2">
        <h1 className="hd text-4xl md:text-5xl">Portfolio</h1>
        <p className="text-muted max-w-xl">Everything in this wallet across Armory: tokens, NFTs you can release, and pending requests.</p>
      </div>
      <PortfolioView />
    </div>
  );
}
