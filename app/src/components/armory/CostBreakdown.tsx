"use client";
/**
 * Itemized capture / re-roll cost with the refundable lazy-mint deposit called out. Stable
 * data-testids: `${prefix}-cost-*`, `${prefix}-deposit-explainer`, `${prefix}-insufficient-balance`.
 */
import { mintDepositText } from "@/config/armory";
import { checkBalance, type RequestCost } from "@/lib/armory/fees";
import { formatSol } from "@/lib/armory/format";

function Row({ label, sub, value, testId, strong }: { label: string; sub?: string; value: string; testId: string; strong?: boolean }) {
  return (
    <div className="flex items-start justify-between gap-4 py-2.5">
      <div>
        <p className={strong ? "font-semibold" : ""}>{label}</p>
        {sub && <p className="text-muted text-xs">{sub}</p>}
      </div>
      <p className={`num text-right whitespace-nowrap ${strong ? "text-accent-text font-semibold" : ""}`} data-testid={testId}>
        {value}
      </p>
    </div>
  );
}

export function CostBreakdown({
  prefix,
  cost,
  balance,
  feeContext,
}: {
  prefix: "capture" | "reroll";
  cost: RequestCost;
  balance: bigint | undefined;
  feeContext: string;
}) {
  const check = balance === undefined ? null : checkBalance(balance, cost);
  return (
    <div className="space-y-3">
      <div className="border-border rounded-panel divide-border divide-y border px-4 text-sm" data-testid={`${prefix}-cost`}>
        <p className="eyebrow pt-3 pb-2">Itemized cost, paid in SOL</p>
        <Row label="Platform fee" sub={`${feeContext}. Not refunded.`} value={formatSol(cost.tierFee)} testId={`${prefix}-cost-tier-fee`} />
        <Row
          label="Mint deposit"
          sub={`${mintDepositText(cost.deposit, cost.firstMintRange)}. Held while Switchboard picks your NFT; refunded in full if the NFT already exists or the request expires.`}
          value={formatSol(cost.deposit)}
          testId={`${prefix}-cost-deposit`}
        />
        <Row label="Temporary account rent" sub="For the request record and the randomness lock. Refunded when it settles." value={formatSol(cost.tempRent)} testId={`${prefix}-cost-rent`} />
        <Row
          label="Randomness account setup"
          sub={cost.setup > 0n ? "This collection has no free Switchboard randomness account, so one is created with your request. One-time and not refunded; it stays with the collection for later requests." : "None: a free Switchboard randomness account is reused."}
          value={formatSol(cost.setup)}
          testId={`${prefix}-cost-setup`}
        />
        <Row label="Network fee" sub="Solana transaction fee (exact amount shown in the preview)." value={`≈ ${formatSol(cost.networkFee)}`} testId={`${prefix}-cost-network`} />
        <Row label="Needed in your wallet" value={formatSol(cost.requiredBalance)} testId={`${prefix}-cost-total`} strong />
      </div>
      <p className="text-muted text-xs" data-testid={`${prefix}-deposit-explainer`}>
        You actually spend between <span className="text-fg num">{formatSol(cost.minNetCost, 6)}</span> (the NFT was minted
        before) and about <span className="text-fg num">{formatSol(cost.maxNetCost, 6)}</span> (first mint: about{" "}
        {formatSol(cost.firstMintRange[0], 4).replace(" SOL", "")}–{formatSol(cost.firstMintRange[1], 4)} of the deposit pays
        Solana rent and the Metaplex Core fee; the rest comes back). The mint cost is not an Armory fee.
      </p>
      {check && !check.ok && (
        <p role="alert" className="border-warning-border bg-warning-bg text-warning rounded-panel border p-3 text-sm" data-testid={`${prefix}-insufficient-balance`}>
          Not enough SOL. You have {formatSol(balance!)} and need {formatSol(cost.requiredBalance)} (short by{" "}
          {formatSol(check.shortfall)}). Most of it is the refundable deposit; add SOL to your wallet and try again.
        </p>
      )}
    </div>
  );
}
