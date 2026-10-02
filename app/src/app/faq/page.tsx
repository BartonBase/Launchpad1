import type { Metadata } from "next";
import type { ReactNode } from "react";
import Link from "next/link";
import { BETA_DEPOSIT_CAP, FIRST_MINT_RANGE_TEXT, MIN_GRADUATION_LAMPORTS, MINT_ESCROW_LAMPORTS, PROGRAM_UPGRADES_COPY, mintDepositText } from "@/config/armory";
import { CLUSTER } from "@/config/cluster";
import { formatSol } from "@/lib/armory/format";
import { DocLayout, DocSection } from "@/components/armory/DocLayout";

export const metadata: Metadata = { title: "FAQ", description: "Plain-language answers about Armory." };

// Copy follows design a-obsidian/faq.html; answers that disagree with the devnet programs are
// corrected to the chain (Plain live on Meteora DBC, Burn coming soon, refundable deposit, no randomness fee, no launch fee).
const GROUPS: readonly { id: string; title: string; qa: readonly (readonly [string, ReactNode])[] }[] = [
  { id: "basics", title: "Basics", qa: [
    ["What is Armory?", <>A place to launch Solana memecoins. A launch can be <b className="text-fg">Plain</b> (just the coin) or <b className="text-fg">Hybrid</b> (the coin is also an NFT collection, both ways), with <b className="text-fg">Burn</b> (burn coins to mint an NFT) coming soon. Every market runs on Meteora: a Dynamic Bonding Curve first, then a DAMM v2 pool. It&apos;s an unaudited demo on devnet with test tokens.</>],
    ["What's a bonding curve?", "A formula that sets the price: it goes up as people buy and down as they sell. Armory uses Meteora's Dynamic Bonding Curve (DBC) with the platform's approved curve settings, and you can buy and sell on it from the token page. A native devnet test launch has no curve, so it can never graduate and converting never opens."],
    ["What does \"graduation\" mean?", `When the curve has raised its target (85 SOL by default; the programs accept ${formatSol(MIN_GRADUATION_LAMPORTS[CLUSTER.name], 1)} or more on ${CLUSTER.label}, 10 SOL on the production build; on the curve path the approved curve settings fix it), the token graduates. The SOL raised and the tokens set aside migrate into a Meteora DAMM v2 pool (LP locked), and trading continues there, from the same Trade panel. Unsold curve tokens stay locked by the program.`],
    ["Which launch types are live?", "Plain and Hybrid, on devnet. Plain launches straight on Meteora DBC. Burn, Tax split and Raffle are coming soon."],
  ] },
  { id: "hybrid", title: "Hybrid", qa: [
    ["What is SPL-404?", "A standard Solana token (SPL) paired with an NFT collection, so a fixed number of tokens and one NFT can be swapped for each other. Hybrid launches work this way."],
    ["How do I get an NFT?", `After graduation, lock the collection's ratio of tokens (for example 1,000,000) and you get one NFT. You pay a flat SOL platform fee set by the ratio. ${mintDepositText(MINT_ESCROW_LAMPORTS)}. Your tokens are held, not spent.`],
    ["Which NFT do I get?", "A random one, picked by Switchboard: Switchboard On-Demand randomness on devnet, which comes with a proof the program checks on-chain, picks it from the art committed when the collection vault was created. Anyone can trigger the reveal and settle steps; nobody can choose a piece or see the next one."],
    ["Can I get my tokens back?", "Yes. Release an NFT and you get exactly the ratio back, whatever its traits. Release has no platform fee; you only pay the Solana network fee."],
    ["What's a re-roll?", "Swapping your NFT for a different random one, again picked by Switchboard. It costs the same platform fee as a capture plus the same mint deposit, and the result may be more common than what you had. Re-rolls are built in; creators don't configure them."],
    ["Does rarity change what I get back?", "No. Rarity is cosmetic. Every NFT releases for the same number of tokens. Any marketplace price is set by buyers and sellers, not by Armory."],
  ] },
  { id: "burn", title: "Plain and Burn (coming soon)", qa: [
    ["How does a Burn launch work?", `After graduation, burn the ratio of tokens to mint the next NFT in the collection. The tokens are destroyed, so the supply goes down. You pay the platform fee and ${FIRST_MINT_RANGE_TEXT} mint cost directly; there's no deposit (Burn is pending deploy).`],
    ["Can I undo a burn?", "No. Burn NFTs can't be converted back to tokens or re-rolled. They have no token backing; their price is whatever buyers pay."],
    ["Which NFT do I get from a burn?", "The next one in collection order. The order is public, so you can see what's next."],
    ["What's a Plain launch?", "Just the coin: 1,000,000,000 tokens on a Meteora bonding curve, then a Meteora DAMM v2 pool. No NFTs and no per-NFT fees, just the 1% curve trade fee. It's live on devnet."],
    ["Why is Burn \"coming soon\"?", "Its program code is written but not deployed to devnet yet, so Burn launches can't be created. The example page shows how it will look."],
  ] },
  { id: "fees", title: "Fees", qa: [
    ["What does it cost?", "Platform fees are flat SOL by ratio: 0.002 SOL (50K), 0.005 SOL (100K and 200K) or 0.01 SOL (500K and up), per capture, re-roll or burn. Release is free. The kept part of the mint deposit (Burn: the mint cost) is Solana rent plus the Metaplex fee, not an Armory fee. There's no launch fee on chain today, and no separate randomness fee."],
    ["Do you take tokens as a fee?", "No. Fees are paid in SOL only, and go to a platform address fixed in the program."],
  ] },
  { id: "safety", title: "Safety", qa: [
    ["Is Armory audited?", "Not yet. There have been internal security reviews, and a paid third-party audit is required before mainnet. Until then, everything is labeled \"Unaudited beta\"."],
    ["What's the deposit cap?", `A per-wallet limit planned while the programs are unaudited. It isn't enforced yet; "${BETA_DEPOSIT_CAP.sol} SOL per wallet" on the site is an example.`],
    ["Can anyone freeze my tokens or mint more?", "No. Freeze authority is never set and mint authority is revoked at launch. Token names and images can't be edited either."],
    ["Can anyone pause the app?", "No key or multisig can pause the vault, so captures, releases, re-rolls and burns can't be halted by us."],
    ["Who can change the programs?", `${PROGRAM_UPGRADES_COPY.status}. ${PROGRAM_UPGRADES_COPY.today} ${PROGRAM_UPGRADES_COPY.planned} ${PROGRAM_UPGRADES_COPY.after}`],
    ["Does an AI hold any keys?", "No AI agent holds mainnet keys. Mainnet keys are held by humans only."],
    ["How do I report a bug?", <>Privately, through the <Link href="/bug-bounty" className="text-accent-text">bug bounty page</Link>. Rewards are to be announced.</>],
  ] },
  { id: "soon", title: "Coming soon", qa: [
    ["Why aren't Tax split and Raffle live?", "They're on hold until there's a public way to buy those tokens and two fixes for stuck funds are done. Raffle also needs a legal check first."],
  ] },
];

export default function FaqPage() {
  return (
    <DocLayout
      title="FAQ"
      intro="Plain-language answers about Armory. Every answer here matches how the programs actually work today."
      chips={<span className="tag tag-ua"><i />Unaudited beta</span>}
      toc={GROUPS.map((g) => [g.id, g.title] as const)}
    >
      {GROUPS.map((g) => (
        <DocSection key={g.id} id={g.id} title={<span className="text-dim font-mono text-xs tracking-wide uppercase">{g.title}</span>}>
          <div className="divide-border -mt-2 divide-y" data-testid={`faq-${g.id}`}>
            {g.qa.map(([q, a]) => (
              <div key={q} className="py-4 first:pt-2 last:pb-0">
                <h3 className="font-semibold">{q}</h3>
                <p className="text-muted mt-1 text-sm">{a}</p>
              </div>
            ))}
          </div>
        </DocSection>
      ))}
    </DocLayout>
  );
}
