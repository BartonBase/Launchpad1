from common import *
import re
import os
BRAND = "Armory"  # final name (Barton 2026-10-01 5:13 PM MT; formerly Fuze/Mintmark); local override, common.py untouched
MPH = '<figure class="mph {cls}"><img src="../../placeholder/knight-placeholder.svg" alt="Placeholder art, mascot coming soon" width="400" height="400"><figcaption>Placeholder art</figcaption></figure>'  # TEMPORARY knight placeholder (design/placeholder/) until Barton's Higgsfield mascot
OUT = os.path.join(os.path.dirname(__file__), "..", "a-obsidian")
os.makedirs(OUT, exist_ok=True)

HEAD = """<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>%TITLE%</title>
<meta name="description" content="%DESC%">
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wdth,wght@12..96,75..100,700..800&family=Geist:wght@400;500;600;700&family=Geist+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>
:root{--bg:#08090a;--s1:#0e0f11;--s2:#131417;--s3:#1a1b1f;--line:rgba(255,255,255,.07);--line2:rgba(255,255,255,.12);
--ink:#eceef1;--mute:#a0a5ae;--dim:#858a94;--acc:#ff6a2b;--acc2:#ff9a62;--accp:#e85a1e;--accbg:rgba(255,106,43,.12);--accglow:rgba(255,106,43,.45);--up:#3ecf8e;--down:#f25c7a;
--r:14px;--sans:"Geist","Inter",ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif;--mono:"Geist Mono",ui-monospace,"SF Mono",Menlo,monospace;--hd:"Bricolage Grotesque","Geist",ui-sans-serif,sans-serif}
*{box-sizing:border-box}html{background:var(--bg)}
body{margin:0;background:var(--bg);color:var(--ink);font:400 14px/1.5 var(--sans);-webkit-font-smoothing:antialiased;letter-spacing:-.005em}
a{color:inherit;text-decoration:none}svg{display:block}
.wrap{max-width:1312px;margin:0 auto;padding:0 32px}
.mono{font-family:var(--mono);font-feature-settings:"zero" 0}.num{font-variant-numeric:tabular-nums}
.mute{color:var(--mute)}.dim{color:var(--dim)}.up{color:var(--up)}.down{color:var(--down)}.acc{color:var(--acc2)}
sub{font-size:.62em;vertical-align:-.25em;line-height:0;margin:0 .04em;color:var(--mute)}
/* nav */
.nav{position:sticky;top:0;z-index:10;background:rgba(8,9,10,.72);backdrop-filter:blur(14px);border-bottom:1px solid var(--line)}
.nav .wrap{display:flex;align-items:center;height:64px;gap:28px}
.brand{display:flex;align-items:center;gap:10px;font-family:var(--hd);font-weight:800;font-stretch:82%;font-variation-settings:"opsz" 96;font-size:22px;letter-spacing:-.015em;line-height:1}.brand .tagwn{font-stretch:100%;font-variation-settings:normal;letter-spacing:.04em}
.tagwn{font:500 10.5px/1 var(--mono);color:var(--dim);border:1px solid var(--line2);border-radius:99px;padding:4px 7px;text-transform:uppercase;letter-spacing:.04em}
.tagwn.beta{color:var(--acc2);border-color:rgba(255,106,43,.35);background:var(--accbg)}
.links{display:flex;gap:4px}.links a{color:var(--mute);padding:7px 12px;border-radius:8px;font-weight:500}.links a.on,.links a:hover{color:var(--ink);background:rgba(255,255,255,.04)}
.navr{margin-left:auto;display:flex;align-items:center;gap:10px}
.search{display:flex;align-items:center;gap:8px;width:260px;height:36px;border:1px solid var(--line);background:var(--s1);border-radius:10px;padding:0 10px;color:var(--dim)}
.search kbd{margin-left:auto;font:500 11px var(--mono);border:1px solid var(--line2);border-radius:5px;padding:1px 5px;color:var(--mute)}
.pill{display:inline-flex;align-items:center;gap:6px;font:500 12px/1 var(--sans);color:var(--mute);border:1px solid var(--line);border-radius:99px;padding:6px 10px;white-space:nowrap}
.pill i{width:6px;height:6px;border-radius:50%;background:#e8b04b;display:block}
.btn{display:inline-flex;align-items:center;justify-content:center;gap:8px;height:40px;padding:0 16px;border-radius:10px;font:600 14px var(--sans);border:1px solid transparent;cursor:pointer;white-space:nowrap}
.btn.p{background:var(--ink);color:#0b0c0e}.btn.a{background:var(--acc);color:#120805;box-shadow:0 0 0 1px rgba(255,255,255,.08) inset,0 8px 24px -8px rgba(255,106,43,.6)}
a:focus-visible,button:focus-visible,input:focus-visible,textarea:focus-visible,summary:focus-visible,[tabindex]:focus-visible{outline:2px solid var(--acc2);outline-offset:2px}
.in:focus-visible{outline-offset:1px}
.skip{position:absolute;left:12px;top:-60px;z-index:20;padding:10px 14px;border-radius:10px;background:var(--acc);color:#120805;font-weight:600}.skip:focus{top:8px}
button.search{appearance:none;font:inherit;text-align:left;cursor:pointer;color:var(--dim)}
.menub{display:none;place-items:center;width:40px;height:40px;border-radius:10px;border:1px solid var(--line2);background:var(--s1);color:var(--ink);cursor:pointer}
.sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
.btn.a:active{background:var(--accp)}.btn.p:active{background:#d4d7dc}
.btn.g{background:rgba(255,255,255,.03);border-color:var(--line2);color:var(--ink)}.btn.g:active{background:rgba(255,255,255,.07)}
.btn.gh{background:transparent;color:var(--mute)}.btn.gh:hover{color:var(--ink);background:rgba(255,255,255,.04)}
.btn.d{background:rgba(242,92,122,.1);border-color:rgba(242,92,122,.35);color:#ff9fb2}.btn.d:active{background:rgba(242,92,122,.18)}
.btn.off,.btn[disabled]{opacity:.45;pointer-events:none;box-shadow:none}
.bbar{background:#15100c;border-bottom:1px solid rgba(255,106,43,.22);font-size:12.5px;color:#d9c6ba}.bbar .wrap{display:flex;align-items:center;gap:12px;height:38px}
.bbar b{color:var(--ink);font-weight:600}.bbar a{margin-left:auto;color:var(--acc2);font-weight:500}.bbar .sep{color:#5a4a40}
.chip.ok2{background:rgba(62,207,142,.08);border-color:rgba(62,207,142,.25);color:#8fe0b9}.chip.ok2 i,.chip.acc i{width:6px;height:6px;border-radius:50%;background:currentColor;display:block}
.chip.soon{background:rgba(255,255,255,.03);border-style:dashed;border-color:var(--line2);color:var(--dim)}.chip.pd{background:transparent;border-style:dashed;border-color:rgba(232,176,75,.4);color:#e8c27a}.chip.pd i{width:6px;height:6px;border-radius:50%;border:1.5px solid currentColor;background:transparent;display:inline-block}
.chip.ua{background:rgba(232,176,75,.08);border-color:rgba(232,176,75,.28);color:#e8c27a}
.chip.bt{background:var(--accbg);border-color:rgba(255,106,43,.3);color:var(--acc2)}
.tchip{display:inline-flex;align-items:center;gap:6px;height:24px;padding:0 9px 0 7px;border-radius:7px;background:rgba(255,255,255,.05);border:1px solid var(--line2);font-size:12px;color:var(--ink);font-weight:500;white-space:nowrap}.tchip svg{width:14px;height:14px;color:var(--acc2)}
.capslot{display:flex;align-items:center;gap:8px;flex-wrap:wrap;padding:10px 12px;border-radius:10px;border:1px dashed var(--line2);background:rgba(255,255,255,.015);font-size:12px;color:var(--mute)}.capslot b{color:var(--ink);font-weight:500}.btn.sm{height:36px;padding:0 14px;font-size:13px}.btn.lg{height:48px;padding:0 22px;font-size:15px;border-radius:12px}
.demo{font:500 10.5px/1 var(--mono);text-transform:uppercase;letter-spacing:.06em;color:#e8b04b;background:rgba(232,176,75,.08);border:1px solid rgba(232,176,75,.2);border-radius:6px;padding:4px 7px;white-space:nowrap}
.card{background:linear-gradient(180deg,rgba(255,255,255,.025),rgba(255,255,255,.01));border:1px solid var(--line);border-radius:var(--r)}
.eyebrow{font:500 12px/1 var(--mono);text-transform:uppercase;letter-spacing:.08em;color:var(--mute)}
h1,h2,h3{margin:0;letter-spacing:-.035em;font-weight:600}
.chip{display:inline-flex;align-items:center;gap:6px;height:24px;padding:0 9px;border-radius:7px;background:rgba(255,255,255,.05);border:1px solid var(--line);font-size:12px;color:var(--mute);font-weight:500;white-space:nowrap}
.chip.acc{background:var(--accbg);border-color:rgba(255,106,43,.25);color:var(--acc2)}
.hd,.hero h1,.shead h2,.trust h2,.feat h2,.band h3,.tname h1,.grad h4{font-family:var(--hd);font-weight:800;font-stretch:82%;font-variation-settings:"opsz" 96;letter-spacing:-.02em}
.mph{position:relative;display:block;flex:none;margin:0;container-type:size}.mph img{display:block;width:100%;height:calc(100% - 14px);object-fit:contain}.mph figcaption{position:absolute;left:0;right:0;bottom:0;text-align:center;font:500 9.5px/1.2 var(--mono);color:var(--dim);letter-spacing:.04em}@container (max-height:119px){.mph img{height:100%}.mph figcaption{display:none}}

/* progress bar: clean track, accent fill, still accent tip. No animation. */
.bar{position:relative;height:4px;border-radius:99px;background:rgba(255,255,255,.07)}.bar i{position:relative;display:block;height:100%;border-radius:99px;background:linear-gradient(90deg,rgba(255,106,43,.45),var(--acc))}.bar i:after{content:"";position:absolute;right:-4px;top:50%;width:8px;height:8px;margin-top:-4px;border-radius:50%;background:var(--acc);box-shadow:0 0 0 2px rgba(255,106,43,.25),0 0 10px var(--accglow)}
.bar.done{background:rgba(255,255,255,.07)}.bar.done i{background:var(--up)}.bar.done i:after{display:none}
.footer{border-top:1px solid var(--line);margin-top:120px;padding:56px 0 40px;color:var(--mute)}
.fgrid{display:grid;grid-template-columns:1.4fr repeat(4,1fr);gap:32px}
.fgrid h4{margin:0 0 14px;font:500 12px var(--mono);text-transform:uppercase;letter-spacing:.08em;color:var(--dim)}
.fgrid a{display:block;padding:4px 0;color:var(--mute)}.fgrid a.brand{display:flex;padding:0}
.risk{margin-top:44px;padding:18px 20px;border:1px solid var(--line);border-radius:12px;background:var(--s1);display:flex;gap:14px;align-items:flex-start;font-size:13px;line-height:1.6}
.risk b{color:var(--ink);font-weight:600}
.legal{display:flex;justify-content:space-between;margin-top:24px;font-size:12px;color:var(--dim)}
%CSS%
</style></head><body>"""

MARK = ""  # no symbol: plain text wordmark in the headline font

SUPPLY_LINE = "Fixed at 1,000,000,000. No one can mint more."
def fee_for(ratio):  # tiered SOL platform fee, fixed per collection at launch
    return 0.002 if ratio <= 50_000 else 0.005 if ratio <= 200_000 else 0.01
FEE_TIERS = [("50K tokens per NFT", 0.002), ("100K and 200K", 0.005), ("500K, 1M, 2.5M and 5M", 0.01)]
FEE_REST = ("No setting can raise it. The programs themselves can still be upgraded while in beta: today on devnet with one development key per program, "
            "and on mainnet only through a 3-of-5 multisig with a public 7-day delay, until they're frozen after the audit.")
UPG_SHORT = "Dev key now · multisig planned"
UPG_LONG = ("Today, on devnet, each program has one development upgrade key (a test key, not a mainnet key). Before mainnet: a 3-of-5 multisig "
            "with a public 7-day delay. After the audit and a stabilization period, upgrades are switched off for good.")
UA_LINE = '<div class="ualine"><span class="chip ua">Unaudited beta</span><span>The programs haven\'t been audited. Only use amounts you can afford to lose.</span></div>'
def fee_copy(x):
    return f"Platform fee: {x} per capture (tokens to NFT) or re-roll, set for this collection at launch based on its ratio. Converting back is free. " + FEE_REST
# lazy minting (Barton, 2026-09-25 5:13 PM MT): nothing is minted at graduation; each NFT is minted on its first capture
# Chain (fix/modes-1-5): hybrid_vault MINT_ESCROW_LAMPORTS = 6,338,100 per capture/re-roll request, spent only if the pick
# was never minted (rent + Core fee, measured 0.0031-0.0044 SOL, DECISIONS CD Q1), rest refunded at settle. Burn mints in
# wrap_permanent and the user pays rent + Core fee directly (create_asset_user_pays): no deposit. No randomness fee exists.
MINT_DEP = "0.0063 SOL"
MINT_SPEND = "≈ 0.003–0.004 SOL"
MINT_DEP_LINE = "Mint deposit: 0.0063 SOL, refunded except about 0.003–0.004 SOL if your NFT is minted for the first time (Solana rent + Metaplex fee)."
CAP_NOW = "0.0163 SOL"          # Nocturnes (1M): 0.01 fee + 0.0063 refundable deposit, paid at request
CAP_NET = "0.01–≈ 0.014 SOL"    # net after the settle refund: 0.01 if the NFT exists, ≈ 0.0131–0.0144 if minted new
BURN_MINT = "≈ 0.003–0.004 SOL" # paid directly per burn, no deposit
BURN_TOTAL = "≈ 0.013–0.014 SOL"
MINT_COST = MINT_SPEND
MINT_DISC = f"The kept part of the deposit is Solana rent plus the Metaplex Core fee, not a {BRAND} fee."
LAZY_LINE = "NFTs are minted one at a time, the first time someone captures them. The collector's refundable deposit covers the small mint cost."
# Deployed on devnet (STATUS.md: Hybrid deployed 2026-09-25; Modes 1 and 3 "Not deployed. Local LiteSVM")
STATUS = {"plain": "pending", "hybrid": "live", "burn": "pending", "tax": "soon", "raffle": "soon"}
def status_chip(k, h=None):
    st = STATUS[k]; hs = f' style="height:{h}px"' if h else ""
    return {"live": f'<span class="chip ok2"{hs}><i></i>Live on devnet</span>',
            "pending": f'<span class="chip pd"{hs}><i></i>Pending deploy</span>',
            "soon": f'<span class="chip soon"{hs}>Coming soon</span>'}[st]
FEE_COPY = fee_copy("0.01 SOL")  # Nocturnes (1M) and Monolith (5M)
FEE_ADDR_H = 'Fee address: <span class="mono">7xKp…3fQa</span> <span class="dim">(example)</span>'
GRAD_LINE = "Collections are revealed and converting opens when a token graduates."

TIERS = [("Common", "#9aa3ae", 60), ("Uncommon", "#86c5a6", 25), ("Rare", "#a697ff", 12), ("Legendary", "#d6b67c", 3)]
TINT = {n: c for n, c, _ in TIERS}
NFT_TIER = {142: "Common", 655: "Rare", 412: "Uncommon", 871: "Common", 93: "Common", 588: "Uncommon", 21: "Common", 22: "Uncommon", 23: "Common", 302: "Rare", 731: "Legendary"}
SKY = ["Indigo", "Violet", "Teal", "Ember", "Slate"]
def tier(name, small=False):
    c = TINT[name]
    return f'<span class="tier{" sm" if small else ""}" style="--t:{c}"><i></i>{name}</span>'
def tdot(name):
    return f'<i class="tdot" style="--t:{TINT[name]}" title="{name}"></i>'
RARITY_CSS = """
.tier{display:inline-flex;align-items:center;gap:5px;height:20px;padding:0 7px;border-radius:6px;font:500 11px/1 var(--sans);color:color-mix(in srgb,var(--t) 88%,#fff);background:color-mix(in srgb,var(--t) 13%,transparent);border:1px solid color-mix(in srgb,var(--t) 28%,transparent);white-space:nowrap}
.tier i{width:5px;height:5px;border-radius:50%;background:var(--t);display:block}
.tier.sm{height:18px;font-size:10.5px;padding:0 6px}
.tdot{position:absolute;right:3px;bottom:3px;width:7px;height:7px;border-radius:50%;background:var(--t);box-shadow:0 0 0 2px rgba(8,9,10,.85);display:block}
.trait{display:inline-flex;align-items:center;height:20px;padding:0 7px;border-radius:6px;font-size:11px;color:var(--mute);background:rgba(255,255,255,.04);border:1px solid var(--line);white-space:nowrap}
.trait b{color:var(--ink);font-weight:500;margin-left:4px}
"""

def nav(active):
    items = [("Explore", "explore", "explore.html"), ("Launch", "launch", "launch.html"), ("Portfolio", "portfolio", "portfolio.html"), ("Trust", "trust", "trust.html"), ("FAQ", "faq", "faq.html")]
    links = "".join(f'<a href="{h}" class="{"on" if a==active else ""}">{n}</a>' for n, a, h in items)
    return f"""<header class="nav"><div class="wrap">
<a class="brand" href="home.html">{MARK}{BRAND}<span class="tagwn beta">Beta</span></a>
<nav class="links" aria-label="Main">{links}</nav>
<div class="navr"><button type="button" class="search" aria-label="Search launches"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>Search launches<kbd>⌘K</kbd></button>
<span class="pill"><i></i>Devnet preview</span><a class="btn p sm" href="#">Connect wallet</a><button type="button" class="menub" aria-label="Open menu" aria-expanded="false"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 7h16M4 12h16M4 17h16"/></svg></button></div></div></header>"""

RISK_H = ("Memecoins and NFTs are highly volatile. You can lose everything you put in. Nothing here is financial advice. "
          "The programs are unaudited devnet software. Availability may be restricted in your country.")
FOOT = f"""<footer class="footer"><div class="wrap">
<div class="fgrid"><div><a class="brand" href="home.html" style="color:var(--ink)">{MARK}{BRAND}</a><p style="max-width:300px;margin:14px 0 0;line-height:1.6">Launch a Solana memecoin on its own, or with an NFT collection that converts both ways or burns in one way. Unaudited beta on devnet.</p></div>
<div><h2 class="fh">Product</h2><a href="explore.html">Explore launches</a><a href="launch.html">Launch a token</a><a href="portfolio.html">Portfolio</a></div>
<div><h2 class="fh">Learn</h2><a href="home.html#types">Launch types</a><a href="faq.html">FAQ</a><a href="faq.html#curve">Bonding curves</a><a href="trust.html#randomness">Verifying randomness</a></div>
<div><h2 class="fh">Trust</h2><a href="trust.html">Trust &amp; security</a><a href="trust.html#audit">Audit status</a><a href="trust.html#keys">Key policy</a><a href="bug-bounty.html">Bug bounty</a></div>
<div><h2 class="fh">Company</h2><a>About</a><a>Terms</a><a>Privacy</a><a>Contact</a></div></div>
<div class="risk"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#e8b04b" stroke-width="1.8" style="flex:none;margin-top:1px"><path d="M12 3 2 20h20L12 3z"/><path d="M12 10v4M12 17.5v.01"/></svg>
<div><b>Risk disclosure.</b> %RISK%</div></div>
<div class="legal"><span>© 2026 {BRAND}. Devnet preview with demo data. No real funds.</span><span class="mono">SOL/USD assumed at $150 for display</span></div>
</div></footer></body></html>"""

DESC = "Launch a Solana memecoin on its own, as a hybrid token and NFT collection, or with a burn-to-mint collection. Unaudited beta on devnet."
# deposit cap: no value and no on-chain cap in fix/modes-1-5 (app armory.ts: 'No cap exists on-chain yet') -> 'not enforced yet' (flagged)
DEP_CAP = "10 SOL per wallet"
BETA_BAR = f'''<div class="bbar" role="note"><div class="wrap"><span class="chip ua" style="height:22px">Unaudited beta</span><span class="bx">Programs are <b>not audited</b> yet. Devnet only, demo data.</span><span class="sep bx">·</span><span>Deposit cap <b>not enforced yet</b></span><a href="trust.html"><span class="bx">How {BRAND} is secured </span>→ Trust</a></div></div>'''
def cap_slot(extra=""):  # step 7 hook: deposit cap + Unaudited beta label in wizard and trade panels
    return f'<div class="capslot"><span class="chip ua" style="height:22px">Unaudited beta</span><span>Deposit cap <b>{DEP_CAP}</b></span><span class="demo" style="padding:3px 6px">Example · not enforced yet</span>{extra}</div>'
# launch types (ADR-020 + fix/modes-1-5). icons: 24x24, 1.75 stroke, currentColor
TICONS = {
 "plain": '<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="4.5"/>',
 "hybrid": '<circle cx="7.5" cy="7.5" r="4"/><rect x="13" y="12" width="7.5" height="9" rx="1.6"/><path d="M13.5 5.5h3.5a2 2 0 0 1 2 2V10"/><path d="m17 8.5 2 2 2-2"/><path d="M10.5 18.5H7a2 2 0 0 1-2-2V14"/><path d="m7 15.5-2-2-2 2"/>',
 "burn": '<path d="M12 21c-3.9 0-6.5-2.6-6.5-6.2 0-3.3 2.3-5.4 3.6-7.9.4 1.6 1.3 2.7 2.4 3.2.1-3.1 1.3-5.5 3.5-7.1-.3 2.9.7 4.9 2 6.6 1 1.4 1.5 2.8 1.5 4.7 0 3.9-2.6 6.7-6.5 6.7Z"/><path d="M12 21c-1.6 0-2.7-1.1-2.7-2.6 0-1.4.9-2.3 1.6-3.5.6.8 1.3 1 2 .6.4 1 1.8 1.7 1.8 3.1 0 1.4-1.1 2.4-2.7 2.4Z"/>',
 "tax": '<circle cx="12" cy="12" r="8.5"/><path d="M12 3.5V12l6 6"/><path d="M12 12l-6.4 5.6"/>',
 "raffle": '<path d="M3.5 9V7a1 1 0 0 1 1-1h15a1 1 0 0 1 1 1v2a3 3 0 0 0 0 6v2a1 1 0 0 1-1 1h-15a1 1 0 0 1-1-1v-2a3 3 0 0 0 0-6Z"/><path d="M14.5 6.5v1.5M14.5 11.25v1.5M14.5 16v1.5"/>',
}
def ticon(k, s=24):
    return f'<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{TICONS[k]}</svg>'
TYPES = [  # key, name, live, short, one-liner
 ("plain", "Plain", True, "Just the coin", "A classic Solana memecoin on a bonding curve. No NFT collection."),
 ("hybrid", "Hybrid", True, "Coin and NFT, both ways", "Lock a fixed number of tokens to get a random NFT, and return the NFT for exactly those tokens, any time after graduation."),
 ("burn", "Burn", True, "Burn coins to mint an NFT", "Burn a fixed number of tokens to mint the next NFT in the collection. One-way: the tokens are gone and the NFT can't be turned back."),
 ("tax", "Tax split", False, "Coming soon", "A fee on every transfer, fixed at launch, buys back tokens that are shared equally across minted NFTs."),
 ("raffle", "Raffle", False, "Coming soon", "A fee on every transfer, fixed at launch, builds a pot. Each round, verifiable randomness picks one minted NFT to receive it."),
]
TNAME = {k: n for k, n, *_ in TYPES}
def tchip(k):
    return f'<span class="tchip">{ticon(k, 14)}{TNAME[k]}</span>'
_ICD = os.path.join(os.path.dirname(__file__), "..", "..", "system", "icons"); os.makedirs(_ICD, exist_ok=True)
for _k, _n, *_ in TYPES:
    open(os.path.join(_ICD, f"{_k if _k != 'tax' else 'tax-split'}.svg"), "w").write(f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><title>{BRAND} launch type: {_n}</title>{TICONS[_k]}</svg>\n')
def gloss(kind):
    g = ["<b>Bonding curve</b>: the formula that sets the price before graduation; it rises as people buy.",
         "<b>Graduation</b>: when the curve raises its SOL target, trading moves to a public DEX pool" + ("." if kind == "plain" else " and the NFT side opens.")]
    if kind == "hybrid":
        g.append("<b>SPL-404</b>: a token paired with an NFT collection at a fixed rate, convertible both ways.")
    return '<p class="gl">' + " ".join(g) + "</p>"
GL_CSS = ".gl{margin:10px 0 0;max-width:640px;font-size:12.5px;line-height:1.5;color:var(--mute)}.gl b{color:var(--ink);font-weight:600}"
def h_up(body):  # card titles become h2 on pages whose only section heading is the h1 (keeps heading order 1-2-3)
    return body.replace("<h3", "<h2").replace("</h3>", "</h2>").replace("<h4>", "<h3>").replace("</h4>", "</h3>")
H_UP_CSS = GL_CSS + ".ch h2{font-size:15px;letter-spacing:-.015em}.grad h3{margin:0;font-size:20px;font-family:var(--hd);font-weight:800;font-stretch:82%;letter-spacing:-.02em}.card>h2{font-size:15px}"
def a11y(html):
    """Static a11y pass: decorative SVGs hidden, href-less anchors focusable, labels tied to inputs, footer headings in order."""
    html = re.sub(r'<svg(?![^>]*\b(?:role|aria-hidden|aria-label)=)', '<svg aria-hidden="true" focusable="false"', html)
    html = re.sub(r'<a (?![^>]*\bhref=)([^>]*aria-disabled="true"[^>]*)>', r'<a tabindex="-1" \1>', html)
    html = re.sub(r'<a(?=[ >])(?![^>]*\b(?:href|tabindex)=)', '<a href="#"', html)
    n = [0]
    def lab(m):
        n[0] += 1; ltxt, gap, tag, attrs = m.group(1), m.group(2), m.group(3), m.group(4)
        mid = re.search(r'\bid="([^"]+)"', attrs)
        fid = mid.group(1) if mid else f"fld{n[0]}"
        if not mid: attrs = f' id="{fid}"' + attrs
        return f'<label for="{fid}">{ltxt}</label>{gap}<{tag}{attrs}>'
    html = re.sub(r'<label>((?:(?!</?label).)*?)</label>((?:(?!<label|class="maxline"|class="upl"|class="drop"|class="tiers").){0,240}?)<(input|textarea)([^>]*)>', lab, html, flags=re.S)
    return html
def page(title, css, body, active="home", risk=None, desc=DESC):
    return a11y(HEAD.replace("%TITLE%", title).replace("%DESC%", desc).replace("%CSS%", css + RARITY_CSS + RESP_CSS).replace("<body>", '<body><a class="skip" href="#main">Skip to content</a>', 1)
                + BETA_BAR + nav(active) + '<main id="main">' + body + '</main>' + FOOT.replace("%RISK%", risk or RISK_H))

# ------------------------------------------------------------------ HOME
RESP_CSS = """
/* Responsive: tokens.json breakpoints (sm 640, md 900, lg 1200, xl 1440). Desktop-first: max-width = next breakpoint - .02px. No motion. */
.fgrid h2.fh,.cl h2.fh,.fh{margin:0 0 14px;font:500 12px var(--mono);text-transform:uppercase;letter-spacing:.08em;color:var(--dim)}
@media (max-width:1199.98px){
 .layout{grid-template-columns:minmax(0,1fr) 330px}.lgrid{grid-template-columns:minmax(0,1fr) 340px}.rail{display:none}
 .grid{grid-template-columns:repeat(2,1fr)}.hgrid{gap:40px}.hero h1{font-size:52px}.switch{left:-24px}
 .doc{grid-template-columns:minmax(0,1fr)}.toc{display:none}.sumg{grid-template-columns:repeat(2,1fr)}.fgrid{grid-template-columns:repeat(3,1fr)}
 .stats{grid-template-columns:repeat(3,1fr)}.fc{grid-template-columns:repeat(2,1fr)}.sw2{grid-template-columns:repeat(4,1fr)}
}
@media (max-width:899.98px){
 .wrap{padding:0 20px}.nav .wrap{gap:12px}.links,.search,.navr .pill{display:none}.menub{display:grid}
 .hgrid,.feat,.explain,.trust,.layout,.lgrid,.two,.psum,.cols2,.mkt,.pv,.soonrow,.sg,.cdemo,.tl3,.gwrap{grid-template-columns:minmax(0,1fr)}
 .psum>div+div{border-left:0;border-top:1px solid var(--line)}.aside,.rail,.toc{position:static}
 .mosaic{height:440px}.switch{left:12px;bottom:12px}
 .steps{grid-template-columns:1fr 1fr}.step{border-bottom:1px solid var(--line)}
 .cmp,.pt,.at,.card table{display:block;overflow-x:auto;max-width:100%}.cmp th,.cmp td{min-width:150px}
 .lhead,.phd,.shead,.band,.sysh{flex-direction:column;align-items:flex-start}.band{gap:18px}
 .trow{flex-wrap:wrap}.tpr{margin-left:0;text-align:left;width:100%}.kst{grid-template-columns:1fr 1fr}.fgrid{grid-template-columns:1fr 1fr}
 .types{grid-template-columns:minmax(0,1fr)}.types2{grid-template-columns:minmax(0,1fr)}
 .auth>div{grid-template-columns:1fr auto}.auth p{grid-column:1/-1}.mph.curve-m{display:none}.curve{grid-template-columns:auto minmax(0,1fr)}
 .gsteps,.lst,.fnfts{grid-template-columns:1fr 1fr}.srch{min-width:0;width:100%}
}
@media (max-width:639.98px){
 main{overflow-x:clip}.fa{overflow:hidden}.fctas{flex-wrap:wrap}.fctas .chip{white-space:normal}.shead>div{display:flex;flex-wrap:wrap;gap:10px}.tabs{max-width:100%;overflow-x:auto}.ch{flex-wrap:wrap;gap:10px}.mkb .btn{white-space:normal;height:auto;min-height:40px;padding-top:9px;padding-bottom:9px}.mkb .btn>*{min-width:0}
 .wrap{padding:0 16px}.bbar .bx,.bbar .demo{display:none}.bbar .wrap{gap:8px;font-size:12px}
 .nav .wrap{height:58px}.navr .btn.sm{padding:0 10px}
 .hero{padding:40px 0 48px}.hero h1{font-size:40px}.mph.hero-m{width:96px;height:96px}.lead{font-size:16px}
 .ctas{flex-wrap:wrap}.ctas .btn{flex:1}.hstats{gap:20px;flex-wrap:wrap;margin-top:32px}
 .mosaic{display:block;height:auto}.mosaic .tile:not(.big){display:none}.tile.big{height:360px}.switch{position:relative;left:0;bottom:0;width:auto;margin-top:12px}
 .feat .fa{min-height:300px}.feat .fb{padding:22px}.feat h2{font-size:32px}.fstats{grid-template-columns:1fr 1fr}.fnfts{grid-template-columns:repeat(3,1fr)}
 .shead h2,.trust h2{font-size:28px}.phd h1,.lhead h1{font-size:34px}.explain{padding:22px}.dgm{grid-template-columns:1fr}.dgm .ar{flex-direction:row;justify-content:center}
 .grid,.steps,.locks,.gsteps,.fc,.sumg,.row2,.sizew,.gradw,.rfx,.lgrid .facts,.review,.cgive,.rtiles,.rtiles.c3,.est .p,.stat3,.owned,.lst,.icx,.bpx,.fgrid,.explain{grid-template-columns:minmax(0,1fr)}
 .stats{grid-template-columns:1fr 1fr}.stats div{border-bottom:1px solid var(--line)}.sw2{grid-template-columns:repeat(2,1fr)}
 .step{border-right:0}.band{padding:26px}.dsec,.sec{padding:20px}.pad{padding:18px}.ti .v b{font-size:24px}.tname h1{font-size:30px}
 .fbar{gap:8px}.fg{overflow-x:auto;max-width:100%}.drop{grid-template-columns:1fr}.tiers .th,.tiers .tr,.tiers .tf2{grid-template-columns:1fr 84px 70px}
 .auth{margin:0 -18px}.auth>div{padding:12px 18px}.stats{margin:18px -18px -18px}.locks{margin:0 -18px -18px}.lk{padding:14px 18px}.locks:has(>.lk.card){margin:0}
 .types2>div{flex-wrap:wrap}.lbtn{flex-wrap:wrap}.psum>div{padding:20px}.pv2{font-size:32px}
}
"""
HOME_CSS = """
.hero{position:relative;padding:88px 0 96px;overflow:hidden;border-bottom:1px solid var(--line)}
.hero:before{content:"";position:absolute;inset:0;background:radial-gradient(700px 420px at 72% 38%,rgba(255,106,43,.16),transparent 70%),radial-gradient(600px 300px at 10% 0%,rgba(255,255,255,.04),transparent 70%);pointer-events:none}
.hero:after{content:"";position:absolute;inset:0;background-image:linear-gradient(var(--line) 1px,transparent 1px),linear-gradient(90deg,var(--line) 1px,transparent 1px);background-size:64px 64px;mask-image:radial-gradient(800px 500px at 70% 40%,#000 10%,transparent 70%);opacity:.5;pointer-events:none}
.hgrid{position:relative;z-index:1;display:grid;grid-template-columns:1fr 1.02fr;gap:72px;align-items:center}
.hrow{display:flex;align-items:center;gap:8px;margin:18px 0 18px}.mph.hero-m{width:168px;height:168px;margin-left:auto}
.hero h1{font-size:64px;line-height:1;margin:0;background:linear-gradient(180deg,#fff 30%,#a9adb6);-webkit-background-clip:text;background-clip:text;color:transparent}
.lead{font-size:18px;line-height:1.6;color:var(--mute);max-width:520px;margin:0 0 32px}
.lead b{color:var(--ink);font-weight:500;white-space:nowrap}
.ctas{display:flex;gap:10px}
.hstats{display:flex;gap:40px;margin-top:52px;padding-top:28px;border-top:1px solid var(--line);align-items:flex-end}
.hstats div b{display:block;font-size:24px;font-weight:600;letter-spacing:-.03em}.hstats div span{font-size:13px;color:var(--mute)}
/* collection mosaic */
.mosaic{position:relative;display:grid;grid-template-columns:repeat(3,1fr);grid-template-rows:repeat(3,1fr);gap:10px;height:560px}
.tile{position:relative;border-radius:14px;overflow:hidden;border:1px solid var(--line2);background:var(--s1)}
.tile svg{width:100%;height:100%}.tile.big{grid-column:2/4;grid-row:1/3;border-radius:18px;box-shadow:0 40px 90px -30px rgba(0,0,0,.9)}
.tile .tl{position:absolute;left:8px;bottom:8px;right:8px;display:flex;align-items:center;gap:6px;background:rgba(10,11,14,.55);backdrop-filter:blur(10px);border:1px solid rgba(255,255,255,.1);border-radius:8px;padding:5px 8px;font-size:11px;color:#d8dae0}
.tile .tl b{color:#fff;font-weight:600}.tile .tl .tdot{position:static;margin-left:auto}
.tile.big .tl{left:14px;right:14px;bottom:14px;padding:10px 12px;border-radius:11px;font-size:12px;flex-wrap:wrap}
.tile.big .tl .trs{display:flex;gap:6px;width:100%;margin-top:6px}
.switch{position:absolute;left:-64px;bottom:34px;width:350px;padding:14px 16px;border-radius:16px;background:rgba(17,18,21,.9);backdrop-filter:blur(18px);border:1px solid var(--line2);box-shadow:0 30px 80px -20px rgba(0,0,0,.85)}
.switch .h{display:flex;justify-content:space-between;align-items:center;font-size:12px;color:var(--mute)}
.switch .row{display:flex;align-items:center;justify-content:space-between;margin-top:10px}
.switch .row b{white-space:nowrap;font-size:16px;font-weight:600;letter-spacing:-.02em}
.switch .row .ic{width:26px;height:26px;border-radius:50%;overflow:hidden;flex:none}.switch .row .ic.sq{border-radius:7px}
.switch .row .a{display:flex;align-items:center;gap:8px}
.switch .sw2{width:28px;height:28px;border-radius:9px;border:1px solid var(--line2);display:grid;place-items:center;color:var(--acc2);background:var(--s2)}
.switch p{margin:10px 0 0;font-size:11.5px;color:var(--mute);line-height:1.5}
/* featured */
.feat{display:grid;grid-template-columns:1.05fr 1fr;gap:0;overflow:hidden;border-radius:22px}
.feat .fa{position:relative;min-height:600px}.feat .fa svg{position:absolute;inset:0;width:100%;height:100%}
.feat .fa .tl{position:absolute;left:18px;bottom:18px;display:flex;gap:8px;align-items:center;background:rgba(10,11,14,.55);backdrop-filter:blur(10px);border:1px solid rgba(255,255,255,.1);border-radius:10px;padding:8px 11px;font-size:12px;color:#d8dae0}
.feat .fb{padding:40px 44px}
.feat h2{font-size:44px;letter-spacing:-.04em;line-height:1.05;display:flex;align-items:baseline;gap:12px}
.feat .desc{color:var(--mute);font-size:15.5px;line-height:1.65;margin:14px 0 24px;max-width:520px}
.fstats{display:grid;grid-template-columns:repeat(4,1fr);border:1px solid var(--line);border-radius:14px;overflow:hidden}
.fstats div{padding:14px 16px;border-right:1px solid var(--line)}.fstats div:last-child{border-right:0}
.fstats span{display:block;font-size:12px;color:var(--mute)}.fstats b{display:block;font-size:18px;font-weight:600;letter-spacing:-.02em;margin-top:4px}
.ftraits{margin-top:22px;display:flex;flex-direction:column;gap:10px}
.ftraits .tr2{display:flex;align-items:center;gap:8px;flex-wrap:wrap}.ftraits .tr2>span:first-child{width:70px;font-size:12px;color:var(--mute)}
.fnfts{display:grid;grid-template-columns:repeat(5,1fr);gap:8px;margin-top:22px}
.fnfts .n{border:1px solid var(--line);border-radius:12px;padding:5px;background:var(--s1)}
.fnfts .n .im{aspect-ratio:1;border-radius:8px;overflow:hidden}.fnfts .n .im svg{width:100%;height:100%}
.fnfts .n .m{display:flex;flex-direction:column;align-items:flex-start;gap:5px;padding:7px 2px 2px;font:500 11px var(--mono);color:var(--mute)}
.fctas{display:flex;gap:10px;margin-top:26px;align-items:center}
/* product shot */
.shot{position:relative;height:560px}
.artcard{position:absolute;right:0;top:0;width:470px;padding:10px;border-radius:22px}
.artcard .art{border-radius:14px;overflow:hidden;height:520px;position:relative}
.artcard .art svg{width:100%;height:100%}
.artcard .lbl{position:absolute;left:14px;bottom:14px;right:14px;display:flex;justify-content:flex-end}
.glass{background:rgba(10,11,14,.55);backdrop-filter:blur(12px);border:1px solid rgba(255,255,255,.1);border-radius:10px;padding:8px 11px;font-size:12px;color:#d8dae0}
.glass b{color:#fff;font-weight:600}
.conv{position:absolute;left:-24px;top:80px;width:330px;padding:18px;border-radius:18px;background:rgba(17,18,21,.88);backdrop-filter:blur(18px);border:1px solid var(--line2);box-shadow:0 30px 80px -20px rgba(0,0,0,.85)}
.conv .hd{display:flex;justify-content:space-between;align-items:center;margin-bottom:14px}
.conv .hd b{font-size:14px;font-weight:600}
.side{background:var(--s2);border:1px solid var(--line);border-radius:12px;padding:12px 14px}
.side .l{display:flex;justify-content:space-between;font-size:12px;color:var(--mute)}
.side .v{display:flex;justify-content:space-between;align-items:center;margin-top:6px}
.side .v b{font-size:22px;font-weight:600;letter-spacing:-.03em}
.tok{display:flex;align-items:center;gap:8px;background:var(--s3);border:1px solid var(--line);border-radius:99px;padding:4px 10px 4px 4px;font-weight:600;font-size:13px}
.tok .ic{width:22px;height:22px;border-radius:50%;overflow:hidden}
.tok .ic.sq{border-radius:6px}
.swapic{display:flex;justify-content:center;margin:-8px 0;position:relative;z-index:1}
.swapic span{width:30px;height:30px;border-radius:9px;background:var(--s1);border:1px solid var(--line2);display:grid;place-items:center;color:var(--mute)}
.frow{display:flex;justify-content:space-between;font-size:12.5px;color:var(--mute);margin-top:12px}
.frow b{color:var(--ink);font-weight:500}
.conv .btn{width:100%;margin-top:14px}
.conv .ex{font-size:12px;color:var(--mute);margin:10px 0 0;line-height:1.5}
.hold{position:absolute;right:-20px;top:34px;width:232px;padding:14px 16px;border-radius:16px;background:rgba(17,18,21,.86);backdrop-filter:blur(18px);border:1px solid var(--line2);box-shadow:0 20px 60px -20px rgba(0,0,0,.8)}
.hold .t{display:flex;justify-content:space-between;font-size:12px;color:var(--mute)}
.hold .val{font-size:26px;font-weight:600;letter-spacing:-.035em;margin:4px 0 2px}
.thumbs{display:flex;gap:6px;margin-top:10px;align-items:center}
.thumbs .th{width:34px;height:34px;border-radius:8px;overflow:hidden;border:1px solid var(--line2);flex:none}
/* sections */
section.s{padding-top:104px}
.shead{display:flex;align-items:flex-end;justify-content:space-between;margin-bottom:28px;gap:24px}
.shead h2{font-size:36px;line-height:1.1}
.shead p{margin:10px 0 0;color:var(--mute);font-size:16px;max-width:560px}
.tabs{display:flex;gap:2px;padding:3px;border:1px solid var(--line);border-radius:10px;background:var(--s1)}
.tabs a{padding:6px 12px;border-radius:7px;color:var(--mute);font-weight:500;font-size:13px}.tabs a.on{background:var(--s3);color:var(--ink);box-shadow:0 0 0 1px var(--line) inset}
.grid{display:grid;grid-template-columns:repeat(3,1fr);gap:20px}
.lc{padding:8px;border-radius:16px}
.lc .art{position:relative;border-radius:11px;overflow:hidden;aspect-ratio:4/3}
.lc .art svg{width:100%;height:100%}
.lc .ov{position:absolute;left:8px;top:8px;right:8px;display:flex;justify-content:space-between}
.lc .ov span{font:500 11px/1 var(--sans);padding:5px 7px;border-radius:6px;background:rgba(8,9,10,.62);backdrop-filter:blur(8px);color:#e9eaee;border:1px solid rgba(255,255,255,.08)}
.lc .bd{padding:14px 8px 8px}
.lc .ovb{position:absolute;left:8px;bottom:8px;display:flex;gap:5px;align-items:center}
.lc .ovb .tier{background:rgba(8,9,10,.62);backdrop-filter:blur(8px)}
.glass2{font:500 11px/1 var(--mono);padding:4px 6px;border-radius:6px;background:rgba(8,9,10,.62);backdrop-filter:blur(8px);color:#e9eaee;border:1px solid rgba(255,255,255,.08)}
.lc .t{display:flex;justify-content:space-between;align-items:baseline}
.lc .t b{font-size:17px;font-weight:600;letter-spacing:-.02em}
.strip{display:flex;align-items:center;gap:6px;margin-top:12px}.strip .sth{position:relative;width:34px;height:34px;border-radius:8px;overflow:hidden;border:1px solid var(--line2);flex:none}.strip .sth svg{width:100%;height:100%}.strip .more{margin-left:auto;font-size:12px;color:var(--mute)}
.lc .mc{display:flex;justify-content:space-between;align-items:baseline;margin:14px 0 12px}
.lc .mc b{font-size:18px;font-weight:600;letter-spacing:-.02em}
.lc .pl{display:flex;justify-content:space-between;font-size:12px;color:var(--mute);margin-top:8px}
.lc .meta{display:flex;justify-content:space-between;border-top:1px solid var(--line);margin-top:14px;padding-top:12px;font-size:12px;color:var(--mute)}
.lc .meta b{color:var(--ink);font-weight:500}.lc .meta{align-items:center}
.explain{display:grid;grid-template-columns:1.05fr 1fr;gap:40px;align-items:center;padding:36px;margin-bottom:20px}
.dgm{display:grid;grid-template-columns:1fr auto 1fr;gap:18px;align-items:center}
.dgm .f{padding:18px;border-radius:14px;background:var(--s2);border:1px solid var(--line2);text-align:center}
.dgm .f small{display:block;font:500 11px var(--mono);color:var(--dim);text-transform:uppercase;letter-spacing:.06em;margin-bottom:10px}
.dgm .f b{display:block;font-size:17px;font-weight:600;letter-spacing:-.02em;margin-top:10px}.dgm .f span{font-size:12px;color:var(--mute)}
.dgm .coin{width:64px;height:64px;border-radius:50%;margin:0 auto;overflow:hidden;border:1px solid var(--line2)}.dgm .coin svg,.dgm .nft svg{width:100%;height:100%;display:block}
.dgm .nft{width:64px;height:76px;border-radius:10px;margin:0 auto;overflow:hidden;border:1px solid var(--line2)}
.dgm .ar{display:flex;flex-direction:column;gap:10px;font:500 11.5px var(--mono);color:var(--mute)}.dgm .ar div{display:flex;align-items:center;gap:6px;white-space:nowrap}.dgm .ar svg{color:var(--acc2)}
.explain h3{font-family:var(--hd);font-weight:800;font-stretch:82%;font-size:30px;letter-spacing:-.02em;line-height:1.1;margin:12px 0 14px}
.explain p{color:var(--mute);font-size:15px;line-height:1.65;margin:0 0 12px}.explain p b{color:var(--ink);font-weight:500}
.cmp{width:100%;border-collapse:separate;border-spacing:0;border:1px solid var(--line);border-radius:18px;overflow:hidden;font-size:13px}
.cmp th,.cmp td{padding:14px 16px;text-align:left;vertical-align:top;border-bottom:1px solid var(--line);border-right:1px solid var(--line);line-height:1.5}
.cmp tr:last-child td{border-bottom:0}.cmp th:last-child,.cmp td:last-child{border-right:0}
.cmp thead th{background:var(--s1);padding:18px 16px}.cmp thead th .tn{display:flex;align-items:center;gap:8px;font-size:15px;font-weight:600;color:var(--ink);letter-spacing:-.01em}
.cmp thead th .tn svg{color:var(--acc2)}.cmp thead th p{margin:6px 0 10px;font-weight:400;font-size:12px;color:var(--mute);line-height:1.45}
.cmp td:first-child{color:var(--mute);width:150px;font-size:12.5px}.cmp td{color:var(--ink)}
.cmp .so{background:rgba(255,255,255,.012)}.cmp .so,.cmp .so .tn{color:var(--mute)}.cmp thead th.so .tn svg{color:var(--dim)}
.tlink{display:inline-flex;align-items:center;gap:6px;margin-top:6px;color:var(--acc2);font-weight:500;font-size:14px}
.styles{display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-top:14px}
.sty{display:flex;gap:12px;align-items:flex-start;padding:16px 18px;border:1px solid var(--line);border-radius:14px;background:var(--s1)}
.sty b{display:block;font-size:14px;font-weight:600;letter-spacing:-.01em}.sty p{margin:4px 0 0;font-size:12.5px;color:var(--mute);line-height:1.5}
.sty .st{margin-left:auto;flex:none}.sty.off{opacity:.72}
.steps{display:grid;grid-template-columns:repeat(4,1fr);border:1px solid var(--line);border-radius:18px;overflow:hidden}
.step{display:flex;flex-direction:column;padding:28px 26px 30px;border-right:1px solid var(--line);background:linear-gradient(180deg,rgba(255,255,255,.02),transparent)}.step:last-child{border-right:0}
.step .ico{width:40px;height:40px;border-radius:11px;background:var(--s2);border:1px solid var(--line2);display:grid;place-items:center;color:var(--acc2);margin-bottom:22px}
.step .n{font:500 12px var(--mono);color:var(--dim);margin-bottom:8px}
.step h3{font-size:18px;letter-spacing:-.02em;margin-bottom:8px}
.step p{margin:0 0 20px;color:var(--mute);line-height:1.6}
.step .eg{margin-top:auto;align-self:flex-start;font:500 12px/1.5 var(--mono);color:var(--acc2);background:var(--accbg);border-radius:8px;padding:8px 10px}
.trust{display:grid;grid-template-columns:.9fr 1.6fr;gap:56px;align-items:start}
.trust h2{font-size:40px;line-height:1.08}
.trust .intro p{color:var(--mute);font-size:16px;line-height:1.65;margin:18px 0 26px}
.kv{display:flex;flex-direction:column;border-top:1px solid var(--line)}
.kv div{display:flex;justify-content:space-between;padding:12px 0;border-bottom:1px solid var(--line);font-size:13px}.kv span{color:var(--mute)}
.locks{display:grid;grid-template-columns:1fr 1fr;gap:12px}
.lk{padding:20px 20px 22px;display:flex;gap:14px}
.lk .ck{flex:none;width:28px;height:28px;border-radius:8px;display:grid;place-items:center;background:rgba(62,207,142,.1);color:var(--up)}
.lk b{display:block;font-size:15px;font-weight:600;letter-spacing:-.015em;margin:3px 0 5px}
.lk p{margin:0;color:var(--mute);font-size:13px;line-height:1.55}
.ph{display:inline-flex;align-items:center;gap:6px}.ph i{width:6px;height:6px;border-radius:50%;display:block}
.lc .ov span.ph.g{color:#9fe6c3;border-color:rgba(62,207,142,.3)}.lc .ov span.ph.g i{background:var(--up)}
.lc .ov span.ph.c{color:#ffd3bd;border-color:rgba(255,106,43,.35)}.lc .ov span.ph.c i{background:var(--acc)}
.strip .lkd{width:34px;height:34px;border-radius:8px;border:1px dashed var(--line2);display:grid;place-items:center;color:var(--dim);flex:none;background:rgba(255,255,255,.02)}
.gline{display:flex;align-items:center;gap:8px;margin-top:10px;font-size:14px;color:var(--mute)}
.band{margin-top:104px;padding:44px 48px;border-radius:22px;display:flex;justify-content:space-between;align-items:center;background:radial-gradient(500px 200px at 85% 50%,rgba(255,106,43,.2),transparent 70%),var(--s1);border:1px solid var(--line)}
.band h3{font-size:28px;letter-spacing:-.03em}.band p{margin:8px 0 0;color:var(--mute);font-size:15px}
"""

def icon(name):
    P = {
        "curve": '<path d="M3 20c6 0 8-4 10-8s4-8 8-8"/><path d="M3 20h18" opacity=".4"/>',
        "convert": '<path d="M4 8h13l-3-3M20 16H7l3 3"/>',
        "back": '<path d="M9 14 4 9l5-5"/><path d="M4 9h11a5 5 0 0 1 0 10h-3"/>',
        "grid": '<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/>',
        "grad": '<path d="M4 17 10 11l4 4 6-7"/><path d="M15 8h5v5"/>',
        "check": '<path d="m5 12 4.5 4.5L19 7"/>',
    }[name]
    return f'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">{P}</svg>'

SAMPLE_TIERS = ["Uncommon", "Common", "Rare", "Common", "Common", "Legendary", "Uncommon", "Common", "Rare"]

import copy
A_LAUNCHES = copy.deepcopy(LAUNCHES)
A_LAUNCHES.append(dict(key="mono", name="Monolith", sym="MONO", price=0.000000206, prog=100, ratio=5_000_000, tax=0.0, holders=412, ch=+9.3, vol=64.8, age="1d"))
for L in A_LAUNCHES:
    if L["key"] == "noct": L["price"] = 0.000000612  # graduated Nocturnes (matches token-graduated.html)
    L["ratio"] = {"glass": 200_000, "note": 2_500_000}.get(L["key"], L["ratio"])  # only the 7 approved ratios
    L["pieces"] = SUPPLY // L["ratio"]; L["mc_sol"] = round(L["price"] * SUPPLY, 4); L["mc_usd"] = L["mc_sol"] * SOL_USD; L["nft_sol"] = L["price"] * L["ratio"]
    L["tax"] = 0.0  # every launch is an SPL-404 hybrid: classic SPL token, no transfer tax
    # phase: graduated (collection minted, converting open) vs on curve (token only)
    L["prog"] = {"noct": 100, "note": 100, "mono": 100, "glass": 100, "low": 64, "ferro": 100}.get(L["key"], L["prog"])
    L["type"] = {"salt": "plain", "kite": "plain", "ferro": "burn", "quiet": "burn"}.get(L["key"], "hybrid")
    L["supply"] = SUPPLY
    if L["key"] == "ferro": L["pieces"] = 500
    if L["key"] == "ferro":  # Example: 312 NFTs burned in -> 312,000,000 FERRO destroyed
        L["minted"] = 312; L["supply"] = SUPPLY - 312 * L["ratio"]; L["mc_sol"] = round(L["price"] * L["supply"], 4); L["mc_usd"] = L["mc_sol"] * SOL_USD
TOK_HREF = {"noct": "token-graduated.html", "salt": "token-plain.html", "ferro": "token-burn.html"}

def ratio_short(r):  # local override: 2,500,000 -> 2.5M
    return f"{r/1_000_000:g}M" if r >= 1_000_000 else f"{r//1000}K"

def floor_s(L):
    v = L["ratio"] * L["price"]
    return f"{v:.3g}"

LOCK_S = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>'
def launch_card(L, k):
    g = L["prog"] >= 100
    prog = ('<div class="bar done"><i style="width:100%"></i></div><div class="pl"><span class="up">Graduated</span><span>Trading on a DEX</span></div>'
            if g else
            f'<div class="bar"><i style="width:{L["prog"]}%"></i></div><div class="pl"><span>Bonding curve</span><span class="num">{L["prog"]}% filled</span></div>')
    ch = f'<span class="{"up" if L["ch"]>=0 else "down"} num" style="font-size:13px;font-weight:500">{L["ch"]:+.1f}%</span>'
    chip = ('<span class="ph g"><i></i>Graduated</span>' if g else f'<span class="ph c"><i></i>On curve · {L["prog"]}%</span>')
    href = TOK_HREF.get(L["key"], {"plain": "token-plain.html", "burn": "token-burn.html"}.get(L["type"], "token.html"))
    ty = L["type"]
    strip_t = ["Common", "Uncommon", "Common", "Rare"]
    if ty == "plain":
        ovb = ""
        meta = f'{tchip(ty)}<span>Supply <b class="num">1B</b> fixed · no NFTs</span>'
        strip = '<span class="more" style="margin-left:0">A plain token. No NFT collection, nothing to convert.</span>'
    elif ty == "burn" and g:
        ovb = ""
        mn = L.get("minted", 0)
        meta = f'{tchip(ty)}<span><b class="num">{ratio_short(L["ratio"])}</b> burned = 1 NFT · <b class="num">{fmt(mn)}</b>/{fmt(L["pieces"])} minted</span>'
        strip = "".join(f'<span class="sth">{art(L["key"], f"s{k}{j}", 300 + j)}</span>' for j in range(4)) + '<span class="more">Minted in collection order</span>'
    elif ty == "burn":
        ovb = ""
        meta = f'{tchip(ty)}<span><b class="num">{ratio_short(L["ratio"])}</b> burned = 1 NFT · <b class="num">{fmt(L["pieces"])}</b> NFTs</span>'
        strip = "".join(f'<span class="lkd">{LOCK_S}</span>' for _ in range(4)) + '<span class="more">Burning opens at graduation</span>'
    elif g:
        ovb = f'<div class="ovb"><span class="glass2">#{(k*137+412)%L["pieces"]:04d}</span>{tier(SAMPLE_TIERS[k], True)}</div>'
        meta = f'{tchip(ty)}<span><b class="num">{ratio_short(L["ratio"])}</b> = 1 NFT · <b class="num">{fmt(L["pieces"])}</b> NFTs</span>'
        seeds = [142, 588, 871, 655] if L["key"] == "noct" else [k * 31 + j * 7 + 3 for j in range(4)]
        strip = "".join(f'<span class="sth">{art(L["key"], f"s{k}{j}", sd)}{tdot(NFT_TIER.get(sd, strip_t[j]) if L["key"] == "noct" else strip_t[(j + k) % 4])}</span>' for j, sd in enumerate(seeds))
        strip += '<span class="more">Cosmetic rarity · 4 tiers</span>'
    else:
        ovb = ""
        meta = f'{tchip(ty)}<span><b class="num">{ratio_short(L["ratio"])}</b> = 1 NFT · <b class="num">{fmt(L["pieces"])}</b> NFTs</span>'
        strip = "".join(f'<span class="lkd">{LOCK_S}</span>' for _ in range(4)) + '<span class="more">Art revealed at graduation</span>'
    return f"""<a class="lc card" href="{href}"><div class="art">{art(L['key'], f'c{k}', 412 if k == 0 else k)}<div class="ov">{chip}<span>{L['age']}</span></div>{ovb}</div>
<div class="bd"><div class="t"><b>{L['name']}</b><span class="mono dim" style="font-size:12px">{L['sym']}</span></div>
<div class="strip">{strip}</div>
<div class="mc"><div><b class="num">{usd(L['mc_usd'])}</b> <span class="dim num" style="font-size:12px">{fmt(L['mc_sol'])} SOL</span></div>{ch}</div>
{prog}
<div class="meta">{meta}</div></div></a>"""

def home():
    a_big = art("noct", "hero", 412, 0)
    th = lambda s, n: f'<div class="th" style="position:relative">{art("noct", "t"+str(n), n)}{tdot(NFT_TIER[n])}</div>'
    def tile(key, name, n, seed, t, cls=""):
        return f'<div class="tile {cls}">{art(key, "mz"+str(seed), seed)}<div class="tl"><b>{name}</b><span class="num">#{n:04d}</span>{tdot(t)}</div></div>'
    shot = f"""<div class="mosaic">
{tile("glass", "Glasshouse", 823, 823, "Common")}
<div class="tile big">{a_big}<div class="tl"><b>Nocturnes #0412</b>{tier("Uncommon", True)}<span style="margin-left:auto" class="mute">1 of 1,000</span><div class="trs"><span class="trait">Sky<b>Teal</b></span><span class="trait">Phase<b>Crescent</b></span><span class="trait">Horizon<b>Low</b></span></div></div></div>
{tile("noct", "Nocturnes", 871, 871, "Common")}
{tile("glass", "Glasshouse", 214, 214, "Rare")}
{tile("mono", "Monolith", 117, 117, "Rare")}
{tile("note", "Field Notes", 97, 97, "Legendary")}
<div class="switch"><div class="h"><span>Switch any time</span><span class="demo">Demo</span></div>
<div class="row"><span class="a"><span class="ic">{art('noct','i1',1)}</span><b class="num">1,000,000 NOCT</b></span><span class="sw2"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 8h13l-3-3M20 16H7l3 3"/></svg></span><span class="a"><b>1 NFT</b><span class="ic sq">{art('noct','i2',7)}</span></span></div>
<p>Exact both ways, no tokens taken. Nocturnes: 0.01 SOL platform fee to capture, free to convert back; network fees apply. Opens when a token graduates.</p></div>
</div>"""
    cards = "".join(launch_card(L, k) for k, L in enumerate(A_LAUNCHES))
    steps = [
        ("curve", "01", "Buy the token", "Every collection starts with its token on a bonding curve, where it trades as a plain token. When the curve fills, the token graduates and converting opens.", "Example: 1 SOL ≈ 2,578,125 tokens on the curve"),
        ("convert", "02", "Convert to an NFT", "After graduation, turn the collection's fixed number of tokens into an NFT, picked at random. A small SOL platform fee set per collection, plus the mint cost if the NFT is new; no tokens taken.", "1,000,000 NOCT → 1 Nocturne"),
        ("grid", "03", "Collect, show or list it", "Keep it in your wallet or list it on an NFT marketplace such as Tensor or Magic Eden. Rarity is cosmetic.", "Traits, tiers, listings"),
        ("back", "04", "Switch back any time", "Return an NFT and get exactly the collection's ratio in tokens, whatever its traits. No platform fee. The token keeps trading as usual.", "1 Nocturne → 1,000,000 NOCT"),
    ]
    steps_h = "".join(f'<div class="step"><div class="ico">{icon(i)}</div><div class="n">{n}</div><h3>{t}</h3><p>{p}</p><div class="eg">{e}</div></div>' for i, n, t, p, e in steps)
    glocks = [
        ("Nobody can mint more", "Every token starts at 1,000,000,000 and mint authority is revoked at launch. Burn launches can only shrink supply."),
        ("No one can freeze your tokens", "Freeze authority is revoked at launch, for every token on the platform."),
        ("No transfer tax", "Plain, Hybrid and Burn tokens are standard Solana tokens. No tax on transfers, buys or sells."),
        ("Hybrid: converting back is exact", "1 NFT always converts back for exactly the collection's ratio in tokens, with no platform fee. Tokens are never taken; fees are paid in SOL."),
        ("Rarity is cosmetic", "Every NFT in a collection converts back for the same number of tokens, whatever its traits."),
        ("Randomness you can check", "Hybrid NFTs and re-rolls use Switchboard verifiable randomness. Burn hands out NFTs in collection order, published before you burn."),
    ]
    th = "".join(f'<th class="{"" if live else "so"}"><div class="tn">{ticon(k, 20)}{n}</div><p>{one}</p>{status_chip(k)}</th>' for k, n, live, _, one in TYPES)
    rows = [("NFT collection", "None", "100 to 10,000 NFTs", "100 to 10,000 NFTs", "Yes", "Yes"),
            ("Getting an NFT", "—", "Lock tokens (kept, not spent)", "Burn tokens (gone for good)", "Lock tokens", "Lock tokens"),
            ("Back to tokens", "—", "Yes, exactly the ratio, free", "No. One-way", "Details at launch", "Details at launch"),
            ("Which NFT you get", "—", "Random, picked by Switchboard VRF (verifiable randomness)", "The next one in collection order", "Next in collection order", "Next in collection order"),
            ("SOL fee per NFT", "None", "0.002 to 0.01 SOL by ratio, plus a 0.0063 SOL mint deposit (mostly refunded)", "0.002 to 0.01 SOL by ratio, plus ≈ 0.003–0.004 SOL mint cost", "Details at launch", "Details at launch"),
            ("Token supply", "1B, fixed", "1B, fixed", "Starts at 1B, only goes down", "1B, fixed", "1B, fixed"),
            ("Fee on transfers", "None", "None", "None", "Set at launch, then locked", "Set at launch, then locked")]
    trs = "".join("<tr>" + "".join(f'<td class="{"so" if c > 3 else ""}">{v}</td>' for c, v in enumerate(r)) + "</tr>" for r in rows)
    compare_h = f'<table class="cmp"><thead><tr><th></th>{th}</tr></thead><tbody>{trs}</tbody></table><p class="dim" style="font-size:12.5px;margin:12px 2px 0">Hybrid: each capture or re-roll puts up a 0.0063 SOL mint deposit, refunded except about 0.003–0.004 SOL if the NFT is minted for the first time. Burn pays that mint cost directly. Both are Solana rent plus the Metaplex Core fee, not an {BRAND} fee. Plain and Burn are pending deploy. Tax split and Raffle are shown for reference and can\'t be launched yet.</p>'
    locks_h = "".join(f'<div class="lk card"><div class="ck">{icon("check")}</div><div><b>{t}</b><p>{p}</p></div></div>' for t, p in glocks)
    body = f"""
<section class="hero"><div class="wrap hgrid"><div>
<span class="pill"><i></i>Unaudited beta · Devnet</span>
<div class="hrow"><h1>Trade the meme.<br>Collect the art.</h1>{MPH.format(cls="hero-m")}</div>
<p class="lead">Launch a Solana memecoin on its own, or with an NFT collection built in. With a <b>Hybrid</b> launch the coin is also an NFT: once it graduates (its bonding curve, the formula that sets the launch price, reaches its SOL target and trading moves to a DEX), 1,000,000 NOCT converts into one Nocturne, and one Nocturne always converts back to 1,000,000 NOCT.</p>
<div class="ctas"><a class="btn a lg" href="explore.html">Explore launches</a><a class="btn g lg" href="launch.html">Launch a token</a></div>
<div class="hstats"><div><b class="num">128</b><span>Launches live</span></div><div><b class="num">3,412</b><span>NFTs converted</span></div><div><b class="num">1,907</b><span>Converted back</span></div><span class="demo" style="margin-bottom:4px">Demo data</span></div>
</div>{shot}</div></section>

<div class="wrap">
<section class="s"><div class="card feat"><div class="fa">{art("noct", "feat", 655)}<div class="tl"><b style="color:#fff">Nocturnes #0655</b>{tier("Rare", True)}</div></div>
<div class="fb"><div style="display:flex;justify-content:space-between;align-items:center"><div class="eyebrow">Featured collection</div><span class="demo">Demo data</span></div>
<h2 style="margin-top:14px">Nocturnes <span class="mono mute" style="font-size:15px;letter-spacing:0">NOCT</span></h2>
<p class="desc">1,000 generative night skies by 5Gh2…aP9s. Hold NOCT, or convert 1,000,000 NOCT into a Nocturne picked at random, and convert it back for exactly 1,000,000 NOCT whenever you like.</p>
<div class="fstats"><div><span>Floor</span><b class="num">0.612 SOL</b></div><div><span>Collection</span><b class="num">1,000 NFTs</b></div><div><span>Per NFT</span><b class="num">1M NOCT</b></div><div><span>Held as NFTs</span><b class="num">412</b></div></div>
<div class="ftraits"><div class="tr2"><span>Sky</span><span class="trait">Teal</span><span class="trait">Indigo</span><span class="trait">Violet</span><span class="trait">Ember</span><span class="trait">Slate</span></div>
<div class="tr2"><span>Phase</span><span class="trait">Crescent</span><span class="trait">Half</span><span class="trait">Eclipse</span><span class="trait">Full</span></div>
<div class="tr2"><span>Rarity</span>{"".join(tier(n, True) for n, _, _ in TIERS)}<span class="mute" style="font-size:12px">Cosmetic only</span></div></div>
<div class="fnfts">{"".join(f'<div class="n"><div class="im">{art("noct", "fn"+str(x), x)}</div><div class="m"><span>#{x:04d}</span>{tier(NFT_TIER[x], True)}</div></div>' for x in (93, 142, 412, 588, 871))}</div>
<div class="fctas">{tchip("hybrid")}<a class="btn p" href="token-graduated.html">View collection</a><a class="btn g" href="token-graduated.html">Buy NOCT</a><span class="chip" style="margin-left:6px;color:#9fe6c3;border-color:rgba(62,207,142,.3);background:rgba(62,207,142,.08)">Graduated · converting open</span></div></div></div></section>

<section class="s" id="live"><div class="shead"><div><div class="eyebrow">Live now</div><h2 style="margin-top:12px">Trending launches</h2><p>Plain tokens trade as coins only. Hybrid and Burn collections open when a token graduates.</p></div>
<div style="display:flex;gap:12px;align-items:center"><span class="demo">Demo data</span><div class="tabs"><a class="on">Trending</a><a>New</a><a>Near graduation</a><a>Graduated</a></div><a class="btn g sm" href="explore.html">View all</a></div></div>
<div class="grid">{cards}</div></section>

<section class="s" id="how"><div class="card explain"><div><div class="eyebrow">How it works</div><h3>A coin that's also an NFT</h3>
<p>Most launches give you one thing: a coin. A <b>Hybrid</b> launch gives you a coin and an art collection that are the <b>same asset</b> in two forms.</p>
<p>Hold the coin and trade it like any memecoin. Once it graduates, lock a fixed number of coins to get one NFT from the collection. Hand the NFT back and you get exactly those coins back. Nothing is spent except a small SOL fee to capture.</p>
<p class="dim" style="font-size:13px">Prefer just a coin? Choose <b>Plain</b>. Want collecting to be permanent? Choose <b>Burn</b>.</p></div>
<div class="dgm"><div class="f"><small>Coin form</small><div class="coin">{art("noct","dg1",1)}</div><b class="num">1,000,000 NOCT</b><span>Trade it on the curve or a DEX</span></div>
<div class="ar"><div>Lock coins <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14m-4-4 4 4-4 4"/></svg></div><div><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5m4-4-4 4 4 4"/></svg> Return NFT</div></div>
<div class="f"><small>NFT form</small><div class="nft">{art("noct","dg2",412)}</div><b>Nocturne #0412</b><span>Keep it, show it or list it</span></div></div></div>
<div class="steps">{steps_h}</div></section>

<section class="s" id="types"><div class="shead"><div><div class="eyebrow">Launch types</div><h2 style="margin-top:12px">Five ways to launch. One is live on devnet.</h2><p>Pick the type when you launch. It's fixed on-chain after that, and every token page shows which type it is.</p></div></div>
{compare_h}</section>

<section class="s"><div class="trust"><div class="intro"><div class="eyebrow">Locked rules</div><h2 style="margin-top:12px">What nobody can change. Not the creator, not us.</h2>
<p>Every launch type has fixed rules you can check on-chain. Where a setting can change, we say who controls it. {BRAND} is in beta and hasn't been audited yet, so deposits are capped.</p>
<div class="kv"><div><span>Smart contract audit</span><b style="color:#e8b04b;font-weight:500">Not yet, unaudited beta</b></div><div><span>Authorities</span><b style="font-weight:500">Mint and freeze revoked</b></div><div><span>Program upgrades</span><b style="font-weight:500">{UPG_SHORT}</b></div><div><span>Deposit cap</span><b style="font-weight:500">Planned · not enforced yet</b></div><div><span>Network</span><b style="font-weight:500">Solana devnet</b></div></div>
<a class="tlink" href="trust.html">Read trust &amp; security →</a></div>
<div class="locks">{locks_h}</div></div></section>

<div class="band"><div><h3>Launch your meme, with or without a collection</h3><p>Plain, Hybrid or Burn. Set it up in a few steps and see every number before you sign.</p></div><a class="btn a lg" href="launch.html">Launch a token</a></div>
</div>"""
    return page(f"{BRAND}: memecoins, with or without a collection", HOME_CSS, body)

open(os.path.join(OUT, "home.html"), "w").write(home())
print("home ok")

# ------------------------------------------------------------------ TOKEN
TOKEN_CSS = """
.thead{padding:28px 0 24px;border-bottom:1px solid var(--line)}
.crumb{font-size:13px;color:var(--dim);margin-bottom:18px}.crumb a{color:var(--mute)}
.trow{display:flex;align-items:center;gap:18px}
.tav{width:64px;height:64px;border-radius:16px;overflow:hidden;border:1px solid var(--line2)}
.tname{display:flex;align-items:baseline;gap:10px}.tname h1{font-size:30px;letter-spacing:-.035em}
.chips{display:flex;gap:6px;margin-top:8px;flex-wrap:wrap}
.tpr{margin-left:auto;text-align:right}
.tpr .p{font-size:32px;font-weight:600;letter-spacing:-.035em}.tpr .p sub{font-size:.5em}
.addr{display:flex;gap:6px;margin-top:6px;justify-content:flex-end}
.addr span{font:500 11.5px var(--mono);color:var(--mute);border:1px solid var(--line);border-radius:7px;padding:4px 8px}
.layout{display:grid;grid-template-columns:minmax(0,1fr) 372px;gap:24px;padding-top:24px;align-items:start}
.col{display:flex;flex-direction:column;gap:24px;min-width:0}
.pad{padding:22px 24px}
.ch{display:flex;justify-content:space-between;align-items:center;margin-bottom:16px}
.ch h3{font-size:15px;letter-spacing:-.015em}
.ch .sub2{font-size:12.5px;color:var(--mute);margin-top:3px}
.tf{display:flex;gap:2px;padding:3px;border:1px solid var(--line);border-radius:9px;background:var(--s1)}
.tf a{padding:4px 10px;border-radius:6px;font:500 12px var(--mono);color:var(--mute)}.tf a.on{background:var(--s3);color:var(--ink)}
.stats{display:grid;grid-template-columns:repeat(5,1fr);border-top:1px solid var(--line);margin:18px -24px -22px}
.stats div{padding:16px 24px;border-right:1px solid var(--line)}.stats div:last-child{border-right:0}
.stats span{display:block;font-size:12px;color:var(--mute);margin-bottom:4px}
.stats b{font-size:17px;font-weight:600;letter-spacing:-.02em}.stats small{display:block;font-size:12px;color:var(--dim);margin-top:2px}
.curve{display:grid;grid-template-columns:auto 1fr auto;gap:28px;align-items:center}.mph.curve-m{width:104px;height:104px}
.curve .big{font-size:44px;font-weight:600;letter-spacing:-.045em;line-height:1}
.curve .bar{height:6px;margin:4px 0 12px}.curve .bar i:after{width:12px;height:12px;margin-top:-6px;right:-6px}
.cm{display:flex;justify-content:space-between;font-size:12.5px;color:var(--mute)}.cm b{color:var(--ink);font-weight:500}
.expl{margin:16px 0 0;padding-top:16px;border-top:1px solid var(--line);color:var(--mute);font-size:13px;line-height:1.6}
.two{display:grid;grid-template-columns:1fr 1fr;gap:24px}
/* convert */
.seg2{display:flex;gap:2px;padding:3px;border:1px solid var(--line);border-radius:10px;background:var(--s1)}
.seg2 a{flex:1;text-align:center;padding:7px 10px;border-radius:7px;font-weight:500;font-size:13px;color:var(--mute)}.seg2 a.on{background:var(--s3);color:var(--ink);box-shadow:0 0 0 1px var(--line) inset}
.side{background:var(--s2);border:1px solid var(--line);border-radius:12px;padding:12px 14px}
.side .l{display:flex;justify-content:space-between;font-size:12px;color:var(--mute)}
.side .v{display:flex;justify-content:space-between;align-items:center;margin-top:6px}
.side .v b{font-size:24px;font-weight:600;letter-spacing:-.03em}
.side .s{font-size:12px;color:var(--dim);margin-top:2px}
.tok{display:flex;align-items:center;gap:8px;background:var(--s3);border:1px solid var(--line);border-radius:99px;padding:4px 10px 4px 4px;font-weight:600;font-size:13px;white-space:nowrap}
.tok .ic{width:22px;height:22px;border-radius:50%;overflow:hidden}.tok .ic.sq{border-radius:6px}
.stack3{display:flex}.stack3 .ic{width:22px;height:22px;border-radius:6px;overflow:hidden;border:1.5px solid var(--s3);margin-left:-8px}.stack3 .ic:first-child{margin-left:0}
.swapic{display:flex;justify-content:center;margin:-9px 0;position:relative;z-index:1}
.swapic span{width:30px;height:30px;border-radius:9px;background:var(--s1);border:1px solid var(--line2);display:grid;place-items:center;color:var(--mute)}
.rows{margin-top:14px;display:flex;flex-direction:column;gap:8px}
.rows div{display:flex;justify-content:space-between;font-size:12.5px;color:var(--mute)}.rows b{color:var(--ink);font-weight:500}
.cta{width:100%;margin-top:16px;height:44px}
.note{margin:12px 0 0;font-size:12.5px;color:var(--mute);line-height:1.55}
.note b{color:var(--ink);font-weight:500}
.fn{font-size:11px;color:var(--dim);margin-top:10px;line-height:1.5}
/* holdings */
.hv{font-size:34px;font-weight:600;letter-spacing:-.04em;line-height:1.1}
.hl{display:flex;flex-direction:column;margin-top:18px;border-top:1px solid var(--line)}
.hl>div{display:flex;align-items:center;gap:12px;padding:14px 0;border-bottom:1px solid var(--line)}
.hl .ic{width:36px;height:36px;border-radius:50%;overflow:hidden;flex:none}
.hl .nm{font-weight:500}.hl .nm small{display:block;color:var(--mute);font-size:12px;font-weight:400}
.hl .rt{margin-left:auto;text-align:right}.hl .rt small{display:block;color:var(--mute);font-size:12px}
.nfts{display:flex;gap:8px}.nfts .n{width:36px;height:36px;border-radius:9px;overflow:hidden;border:1px solid var(--line2);flex:none}
.avail{margin-top:14px;padding:14px;border-radius:12px;background:var(--accbg);border:1px solid rgba(255,106,43,.22);display:flex;justify-content:space-between;align-items:center}
.avail span{font-size:12px;color:var(--acc2)}.avail b{display:block;font-size:15px;font-weight:600;color:var(--ink);margin-top:2px}
.mini{background:var(--s2);border:1px solid var(--line);border-radius:12px;padding:12px 14px}
.mini span{display:block;font-size:12px;color:var(--mute)}.mini b{display:block;font-size:16px;font-weight:600;letter-spacing:-.02em;margin-top:4px}.mini small{font-size:12px;color:var(--dim)}
.stat3{display:grid;grid-template-columns:repeat(3,1fr);gap:8px;margin-top:4px}
/* locks */
.locks{display:grid;grid-template-columns:1fr 1fr;border-top:1px solid var(--line);margin:0 -24px -22px}
.lk{display:flex;gap:12px;padding:16px 24px;border-bottom:1px solid var(--line)}
.lk:nth-child(odd){border-right:1px solid var(--line)}.lk:nth-last-child(-n+2){border-bottom:0}
.lk .ck{flex:none;width:24px;height:24px;border-radius:7px;display:grid;place-items:center;background:rgba(62,207,142,.1);color:var(--up)}
.lk b{display:block;font-size:13.5px;font-weight:600;margin:2px 0 3px;letter-spacing:-.01em}.lk p{margin:0;color:var(--mute);font-size:12.5px;line-height:1.5}
/* activity */
table{width:100%;border-collapse:collapse;font-size:13px}
th{text-align:left;font:500 11.5px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);padding:0 0 10px}
td{padding:11px 0;border-top:1px solid var(--line)}
th:not(:first-child),td:not(:first-child){text-align:right}
.ty{display:inline-flex;align-items:center;gap:8px;font-weight:500}.ty i{width:7px;height:7px;border-radius:50%;display:block}
/* trade */
.trade{padding:18px}
.ti{margin-top:14px;background:var(--s2);border:1px solid var(--line);border-radius:12px;padding:14px}
.ti .l{display:flex;justify-content:space-between;font-size:12px;color:var(--mute)}
.ti .v{display:flex;justify-content:space-between;align-items:center;margin-top:8px}
.ti .v b{font-size:28px;font-weight:600;letter-spacing:-.035em}
.quick{display:flex;gap:6px;margin-top:10px}.quick a{flex:1;text-align:center;font:500 12px var(--mono);padding:6px 0;border-radius:7px;border:1px solid var(--line);color:var(--mute)}
.recv{display:flex;justify-content:space-between;align-items:center;margin-top:10px;padding:14px;border-radius:12px;border:1px solid var(--line)}
.recv span{font-size:12px;color:var(--mute)}.recv b{display:block;font-size:20px;font-weight:600;letter-spacing:-.03em;margin-top:2px}
.facts{padding:6px 20px}
.facts div{display:flex;justify-content:space-between;padding:11px 0;border-bottom:1px solid var(--line);font-size:13px}.facts div:last-child{border-bottom:0}
.facts span{color:var(--mute)}.facts b{font-weight:500}
.owned{display:grid;grid-template-columns:1fr 1fr;gap:10px}
.mkt{display:grid;grid-template-columns:minmax(0,.8fr) minmax(0,1.2fr);gap:28px;align-items:start}.mkb{display:flex;flex-direction:column;gap:8px;margin-top:16px}.mkb .btn{display:flex;align-items:center;justify-content:space-between;gap:8px;height:42px}
.lst{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:8px}.li{min-width:0}.li{border:1px solid var(--line);border-radius:12px;padding:5px;background:var(--s1)}.li .im{aspect-ratio:1;border-radius:8px;overflow:hidden}.li .im svg{width:100%;height:100%}
.li .m1{display:flex;flex-direction:column;align-items:flex-start;gap:5px;padding:7px 3px 0;font:500 11.5px var(--mono);color:var(--mute)}.li .m2{display:flex;flex-direction:column;gap:1px;padding:5px 3px 2px}.li .m2 b{font-size:13px;font-weight:600}.li .m2 span{font-size:11px;color:var(--dim)}
.fc{display:grid;grid-template-columns:repeat(3,1fr);gap:10px}.fc>div{background:var(--s2);border:1px solid var(--line);border-radius:12px;padding:14px}.fc span{display:block;font-size:12px;color:var(--mute)}.fc b{display:block;font-size:14.5px;font-weight:600;letter-spacing:-.015em;margin-top:6px}.fc p{margin:6px 0 0;font-size:12px;color:var(--mute);line-height:1.5}
.owned .on{border:1px solid var(--line);border-radius:12px;padding:6px;background:var(--s1)}
.owned .on.sel{border-color:rgba(255,106,43,.5);box-shadow:0 0 0 3px rgba(255,106,43,.1)}
.owned .im{border-radius:8px;overflow:hidden;aspect-ratio:1}
.owned .md{padding:9px 4px 3px}.owned .t1{display:flex;justify-content:space-between;align-items:center}.owned .t1 b{font-weight:600;font-size:13px}
.owned .trs{display:flex;gap:5px;margin-top:7px;flex-wrap:wrap}
.rr{margin-top:14px;padding-top:14px;border-top:1px solid var(--line)}
.rrh{display:flex;flex-direction:column;gap:2px}.rrh b{font-weight:600;font-size:14px}
.flr{padding:16px;border-radius:12px;background:var(--s1);border:1px solid var(--line);display:flex;flex-direction:column;gap:6px}
.flr .fx{font:500 14px var(--mono);white-space:nowrap;color:var(--ink)}.flr .fx b{color:var(--acc2);font-weight:500}
.dist{display:flex;gap:2px;height:6px;border-radius:99px;overflow:hidden;margin:16px 0 10px}.dist i{display:block;height:100%;opacity:.8}
.dl div{display:grid;grid-template-columns:1fr auto 44px;align-items:center;gap:10px;padding:7px 0;border-bottom:1px solid var(--line);font-size:12.5px}.dl div:last-child{border-bottom:0}.dl span.dim{text-align:right}.dl .tier{justify-self:start}
.ok{display:inline-flex;align-items:center;gap:6px;color:var(--up)}.ok svg{display:inline-block}
"""

def chart(end_price=None, gid="ag", seed=None):
    vals = price_series(end=(end_price or T["price"]) * 1e9, seed=seed or (3 if end_price is None else 11))
    W, H, G = 804, 250, 58
    d, pts, lo, hi = line_path(vals, W - G, H, 14)
    area = d + f" L{W-G} {H} L0 {H} Z"
    ex, ey = pts[-1]
    grid = ""
    for k in range(5):
        y = 14 + k * (H - 28) / 4
        v = hi - k * (hi - lo) / 4
        grid += f'<line x1="0" x2="{W-G+8}" y1="{y:.0f}" y2="{y:.0f}" stroke="rgba(255,255,255,.05)"/><text x="{W}" y="{y+4:.0f}" text-anchor="end" fill="#858a94" font-size="11" font-family="Geist Mono, monospace">${v*SOL_USD/1000:.1f}K</text>'
    xl = "".join(f'<text x="{x}" y="{H+18}" fill="#858a94" font-size="11" font-family="Geist Mono, monospace" text-anchor="{a}">{t}</text>' for x, t, a in [(0, "Sep 21" if end_price else "Sep 22", "start"), ((W-G)/2, "Sep 23", "middle"), (W-G, "Sep 24 · now", "end")])
    return f"""<svg viewBox="0 0 {W} {H+24}" width="100%" style="overflow:visible" role="img" aria-label="Price chart, example data"><defs><linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff6a2b" stop-opacity=".28"/><stop offset="1" stop-color="#ff6a2b" stop-opacity="0"/></linearGradient></defs>
{grid}<path d="{area}" fill="url(#{gid})"/><path d="{d}" fill="none" stroke="#ff9a62" stroke-width="1.8" stroke-linejoin="round"/>
<line x1="{ex:.0f}" x2="{ex:.0f}" y1="0" y2="{H}" stroke="rgba(255,255,255,.12)" stroke-dasharray="3 3"/><circle cx="{ex:.1f}" cy="{ey:.1f}" r="4.5" fill="#fff" stroke="#ff6a2b" stroke-width="3"/>{xl}</svg>"""

import math as _m
TRADE_FEE = 0.01
POOL_FEE = 0.0025                        # example DEX pool fee after graduation
G_PRICE = 0.000000612                    # Nocturnes price after graduation (demo)
H_BUY = lambda sol, px=None: round(sol * (1 - TRADE_FEE) / (px or T["price"]))
H_SELL = lambda tk: round(tk * T["price"] * (1 - TRADE_FEE), 3)
G_BUY = lambda sol: round(sol * (1 - POOL_FEE) / G_PRICE)
G_SELL = lambda tk: round(tk * G_PRICE * (1 - POOL_FEE), 3)
H_BUY_NET = H_BUY(1.0)                                   # 2,578,125
ACT_CURVE = [("Buy", "7fQa…m2Lx", H_BUY(2.5), 2.5, "12s", ""), ("Sell", "C2vN…t8Rb", 4_000_000, H_SELL(4_000_000), "1m", ""),
             ("Buy", "9mTe…Ka71", H_BUY(0.5), 0.5, "2m", ""), ("Buy", "4LxW…eR0c", H_BUY(1.2), 1.2, "5m", ""),
             ("Sell", "Fz1k…uY6n", 750_000, H_SELL(750_000), "7m", ""), ("Buy", "Qw2r…Hn5t", H_BUY(3.0), 3.0, "9m", "")]
ACT_GRAD = [("Buy", "7fQa…m2Lx", G_BUY(2.5), 2.5, "12s", ""), ("Wrap", "Hk3P…9wQe", 1_000_000, None, "48s", "Nocturnes #0588"),
            ("Sell", "C2vN…t8Rb", 4_000_000, G_SELL(4_000_000), "1m", ""), ("Reroll", "Gw4d…Lr8e", None, None, "2m", "#0301 → #0777"),
            ("Buy", "9mTe…Ka71", G_BUY(0.5), 0.5, "2m", ""), ("Unwrap", "Bq8s…Zp4D", 1_000_000, None, "4m", "Nocturnes #0231"),
            ("Buy", "4LxW…eR0c", G_BUY(1.2), 1.2, "5m", ""), ("Sell", "Fz1k…uY6n", 750_000, G_SELL(750_000), "7m", "")]

def h_locked(grad):
    return [
        ("Supply is fixed", SUPPLY_LINE),
        ("Nobody can freeze your tokens", "Freeze authority is revoked at launch."),
        ("No transfer tax", "NOCT is a standard Solana token. There is no tax on transfers, buys, sells or converts."),
        ("Tokens are never taken", "1,000,000 NOCT converts to exactly one Nocturne, and back to exactly 1,000,000 NOCT. Fees are paid in SOL; converting back is free."),
        ("Rarity is cosmetic", "Every Nocturne converts back for exactly 1,000,000 NOCT, whatever its traits."),
        ("NFTs are picked at random", "Converting and re-rolling pick with Switchboard verifiable randomness. Nobody can choose one or peek at the next."),
        ("Art and traits are fixed", "Committed before converting opens and can't be edited." + ("" if grad else " Art is revealed at graduation.")),
        ("This collection's fee can't go up", "0.01 SOL per capture or re-roll, set for this collection at launch based on its ratio. The mint cost is set by Solana and Metaplex, not us."),
    ]

AUTH_CSS = """
.auth{display:flex;flex-direction:column;border-top:1px solid var(--line);margin:0 -24px}
.auth>div{display:grid;grid-template-columns:170px 132px 1fr;gap:14px;align-items:start;padding:13px 24px;border-bottom:1px solid var(--line);font-size:13px}
.auth b{font-weight:600}.auth p{margin:0;color:var(--mute);font-size:12.5px;line-height:1.5}
.st2{display:inline-flex;align-items:center;gap:6px;font-size:12px;font-weight:500;white-space:nowrap}.st2 i{width:7px;height:7px;border-radius:50%;background:var(--up)}.st2.ok{color:#8fe0b9}.st2.wn{color:#e8c27a}.st2.wn i{background:#e8b04b}
.ualine{display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-top:12px;font-size:12px;color:var(--mute)}
.subh{font:500 11px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);margin:20px 0 4px}
"""
def auth_rows(kind, sym="NOCT", ratio="1,000,000"):
    ok, wn = "ok", "wn"
    r = [("Mint authority", "Revoked", ok, f"Nobody can mint more {sym}. Checked on-chain when the launch is registered."),
         ("Freeze authority", "Never set", ok, "Nobody can freeze tokens in your wallet."),
         ("Token metadata", "Immutable", ok, "The token's name, ticker and image can't be edited. The curve must be set up with no metadata update authority."),
         ("Launch settings", "Locked", ok, {"plain": "The launch record is written once. There's no instruction to update or close it.",
                                           "hybrid": "Ratio, collection size and fee tier are written once. There's no instruction to update or close them.",
                                           "burn": "Burn rate, collection size and fee tier are written once. There's no instruction to update or close them."}[kind])]
    if kind == "hybrid":
        r += [("NFT collection", "Locked", ok, "Update authority is a program address, with no update instruction once the art is committed."),
              ("Vault", "No withdrawals", ok, f"Only a release moves tokens out, and only exactly {ratio} {sym} to the NFT's holder."),
              ("Fee recipient", "Fixed in code", ok, "Fees can only go to the platform address written into the program."),
              ("Pause switch", "None", ok, "No key or multisig can halt captures, re-rolls or releases.")]
    elif kind == "burn":
        r += [("NFT collection", "Program-held", ok, "Update authority is a program address. Each NFT must match the art committed before converting opened."),
              ("Fee recipient", "Fixed in code", ok, "Fees can only go to the platform address written into the program."),
              ("Pause switch", "None", ok, "No key or multisig can halt burns.")]
    else:
        r += [("Pause switch", f"None in {BRAND}", ok, f"{BRAND}'s programs have no pause instruction. Trading runs on the third-party curve and DEX programs, which {BRAND} doesn't control.")]
    r += [("Program upgrades", "Not locked yet", wn, UPG_LONG)]
    return r
def auth_panel(kind, sym="NOCT", ratio="1,000,000"):
    rows = "".join(f'<div><b>{a}</b><span class="st2 {c}"><i></i>{st}</span><p>{d}</p></div>' for a, st, c, d in auth_rows(kind, sym, ratio))
    return f"""<div class="card pad"><div class="ch"><div><h3>Locked authorities</h3><div class="sub2">Who can change what on this token, checked on-chain</div></div><span class="chip ua">Unaudited beta</span></div>
<div class="auth">{rows}</div><p class="fn" style="margin:12px 0 0">More on keys, upgrades and the audit plan: <a class="acc" href="trust.html">Trust &amp; security</a>.</p></div>"""

LISTINGS = [(871, "0.618", "Tensor"), (93, "0.624", "Magic Eden"), (588, "0.615", "Tensor"), (302, "0.621", "Magic Eden")]
EXT = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 4h6v6M20 4l-9 9"/><path d="M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"/></svg>'
LOCK_I = '<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>'
TRAITS_ODDS = [("Sky", [("Indigo", 30), ("Violet", 25), ("Teal", 20), ("Ember", 15), ("Slate", 10)]),
               ("Phase", [("Crescent", 40), ("Half", 30), ("Eclipse", 20), ("Full", 10)]),
               ("Horizon", [("Low", 50), ("Mid", 35), ("High", 15)])]

TOKEN_CSS += """
.ccost{margin-top:14px;padding:14px 16px;border-radius:12px;border:1px solid var(--line);background:var(--s1)}
.ccost .ct{display:flex;justify-content:space-between;align-items:baseline;gap:12px;margin-bottom:4px}.ccost .ct b{font-size:13.5px;font-weight:600}.ccost .ct span{font-size:12px;color:var(--mute)}
.ccost .cr{display:flex;justify-content:space-between;gap:16px;padding:8px 0;border-top:1px solid var(--line);font-size:13px}.ccost .cr span{color:var(--mute)}.ccost .cr span small{display:block;font-size:11.5px;color:var(--dim);margin-top:1px}.ccost .cr b{font-weight:500;white-space:nowrap;text-align:right}
.ccost .cr.tot b{font-weight:600;color:var(--ink)}
.ccost .fn{margin:8px 0 0;font-size:11.5px;color:var(--dim);line-height:1.5}
.chip.ok2{background:rgba(62,207,142,.08);border-color:rgba(62,207,142,.25);color:#8fe0b9}
.chip.ok2 i,.chip.acc i{width:6px;height:6px;border-radius:50%;background:currentColor;display:block}
.gsteps{display:grid;grid-template-columns:repeat(3,1fr);gap:10px}.gwrap{display:grid;grid-template-columns:132px 1fr;gap:14px;align-items:center}.mph.grad-m{width:132px;height:132px}
.gsteps>div{padding:14px 16px;border-radius:12px;background:var(--s1);border:1px solid var(--line)}
.gsteps .n{font:500 11.5px var(--mono);color:var(--dim)}.gsteps b{display:block;font-size:13.5px;font-weight:600;margin:6px 0 4px;letter-spacing:-.01em}.gsteps p{margin:0;font-size:12.5px;color:var(--mute);line-height:1.5}
.gsteps.done>div .n{color:var(--up)}
.lockp{position:relative;display:flex;flex-direction:column;align-items:flex-start;gap:10px;padding:18px;border-radius:12px;background:var(--s2);border:1px solid var(--line)}
.lockp .li2{width:36px;height:36px;border-radius:10px;display:grid;place-items:center;background:var(--s3);border:1px solid var(--line2);color:var(--acc2)}
.lph{display:flex;justify-content:space-between;align-items:flex-start;width:100%}.mph.conv-m{width:96px;height:96px}
.lockp b{font-size:17px;font-weight:600;letter-spacing:-.02em}.lockp p{margin:0;font-size:13px;color:var(--mute);line-height:1.6}
.btn.off{opacity:.5;pointer-events:none;box-shadow:none}
.pv{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1.15fr);gap:28px;align-items:start}
.pvart{display:grid;grid-template-columns:1fr 1fr;gap:8px}
.pvt{position:relative;aspect-ratio:1;border-radius:12px;overflow:hidden;border:1px solid var(--line2);background:var(--s1)}
.pvt>svg{width:100%;height:100%;filter:blur(18px) saturate(.8);transform:scale(1.25)}
.pvt .ov2{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;background:rgba(8,9,10,.35);color:#d8dae0;font-size:12px;font-weight:500}
.pvt .ov2 span{display:grid;place-items:center;width:30px;height:30px;border-radius:9px;background:rgba(10,11,14,.6);border:1px solid rgba(255,255,255,.12)}
.odds{display:flex;flex-direction:column;gap:14px}
.odds .ot{font:500 11.5px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);margin-bottom:6px}
.odds .ov3{display:flex;flex-wrap:wrap;gap:6px}
.odds .trait b{color:var(--mute);font-weight:500}
.grad{display:flex;align-items:center;gap:18px}
.grad .gi{width:48px;height:48px;border-radius:14px;display:grid;place-items:center;background:rgba(62,207,142,.1);border:1px solid rgba(62,207,142,.28);color:var(--up);flex:none}
.grad h4{margin:0;font-size:20px;letter-spacing:-.025em}.grad p{margin:3px 0 0;color:var(--mute);font-size:13px}
.feeline{margin-top:14px;padding:12px 14px;border-radius:12px;border:1px solid var(--line);background:var(--s1);font-size:12.5px;color:var(--mute);line-height:1.6}
.feeline b{color:var(--ink);font-weight:500}
.feeaddr{display:flex;align-items:center;gap:8px;margin-top:8px;font-size:12px;color:var(--mute)}.feeaddr .mono{color:var(--ink);border:1px solid var(--line);border-radius:6px;padding:2px 6px}
"""

def _side_trade(sym, icon_art, price, net, rows, btn, note="You'll review the exact amounts in your wallet before signing."):
    return f"""<div class="card trade"><div class="seg2"><a class="on">Buy</a><a>Sell</a></div>
<div class="ti"><div class="l"><span>You pay</span><span class="num">Balance 12.40 SOL</span></div><div class="v"><b class="num">1.00</b><span class="tok"><span class="ic" style="background:linear-gradient(135deg,#9945ff,#14f195)"></span>SOL</span></div>
<div class="quick"><a>0.1</a><a>0.5</a><a>1</a><a>5</a><a>Max</a></div></div>
<div class="recv"><div><span>You receive (estimate)</span><b class="num">{fmt(net)} {sym}</b></div><span class="tok"><span class="ic">{icon_art}</span>{sym}</span></div>
<div class="rows"><div><span>Price</span><b class="num">{sub_price(price)} SOL</b></div>{rows}<div><span>Max slippage</span><b>1%</b></div></div>
<div style="margin-top:14px">{cap_slot()}</div>
<a class="btn a cta" style="height:48px;font-size:15px">{btn}</a>
<p class="note" style="text-align:center">{note}</p></div>"""

def _risk_card(text):
    return f'<div class="card pad" style="padding:18px 20px"><div style="display:flex;gap:10px;font-size:12.5px;color:var(--mute);line-height:1.6"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#e8b04b" stroke-width="1.8" style="flex:none;margin-top:2px"><path d="M12 3 2 20h20L12 3z"/><path d="M12 10v4M12 17.5v.01"/></svg><span>{text}</span></div></div>'

def _act_table(rows_html, tabs, sub="Latest on-chain events"):
    return f"""<div class="card pad"><div class="ch"><div><h3>Activity</h3><div class="sub2">{sub}</div></div><div class="tf">{tabs}</div></div>
<table><thead><tr><th>Type</th><th>Account</th><th>Amount</th><th>Value</th><th>Time</th></tr></thead><tbody>{rows_html}</tbody></table></div>"""

def token_page(grad=False, state=None):
    t = T
    px = G_PRICE if grad else t["price"]
    mc_sol = px * SUPPLY; mc_usd = mc_sol * SOL_USD
    floor = T["ratio"] * px
    ck = icon("check")
    nft = lambda n, s: art("noct", f"n{s}", s)
    colors = {"Buy": "var(--up)", "Sell": "var(--down)", "Wrap": "var(--acc)", "Unwrap": "var(--acc2)", "Reroll": "#a0a5ae"}
    labels = {"Buy": "Buy", "Sell": "Sell", "Wrap": "Converted to NFT", "Unwrap": "Converted to tokens", "Reroll": "Re-rolled NFT"}
    rows = ""
    for k, w, tk, sv, a, e in (ACT_GRAD if grad else ACT_CURVE):
        if k == "Reroll":
            amt, val = e, '<span class="mute">0.01 SOL fee</span>'
        elif k == "Wrap":
            amt, val = f"{fmt(tk)} NOCT", f'<span class="mute">{e} · 0.01 SOL fee</span>'
        elif k == "Unwrap":
            amt, val = f"{fmt(tk)} NOCT", f'<span class="mute">{e} · no platform fee</span>'
        else:
            amt, val = f"{fmt(tk)} NOCT", f"{sv:.3f} SOL"
        rows += f'<tr><td><span class="ty"><i style="background:{colors[k]}"></i>{labels[k]}</span></td><td class="mono mute">{w}</td><td class="num">{amt}</td><td class="num">{val}</td><td class="mute num">{a} ago</td></tr>'
    locks = "".join(f'<div class="lk"><div class="ck">{ck}</div><div><b>{a}</b><p>{b}</p></div></div>' for a, b in h_locked(grad))
    counts = {"Common": 600, "Uncommon": 250, "Rare": 120, "Legendary": 30}
    dist_bar = "".join(f'<i style="flex:{counts[n]};background:{c}"></i>' for n, c, _ in TIERS)
    dist_rows = "".join(f'<div>{tier(n, True)}<span class="num">{counts[n]:,} NFTs</span><span class="num dim">{counts[n]/10:g}%</span></div>' for n, c, _ in TIERS)
    if grad:
        hl = [("DEX pool", "200,000,000+", "21.4%"), ("3xTr…Qp2m", "31,200,000", "3.12%"), ("9mTe…Ka71", "24,650,000", "2.47%"), ("Gw4d…Lr8e", "19,800,000", "1.98%"), ("7fQa…m2Lx (you)", "5,750,000", "0.58%")]
    else:
        hl = [("Bonding curve", "288,000,000", "28.8%"), ("3xTr…Qp2m", "31,200,000", "3.12%"), ("9mTe…Ka71", "24,650,000", "2.47%"), ("Gw4d…Lr8e", "19,800,000", "1.98%"), ("7fQa…m2Lx (you)", "3,750,000", "0.38%")]
    holders = "".join(f'<div><span class="{"" if n in ("Bonding curve", "DEX pool") else "mono"}">{n}</span><b class="num">{p}</b></div>' for n, v, p in hl)
    fee_addr = f'<div class="feeaddr">{FEE_ADDR_H}</div>'

    # ---------- header
    phase_chip = ('<span class="chip ok2"><i></i>Graduated</span><span class="chip">Trading on a DEX</span><span class="chip">Converting open</span>'
                  if grad else '<span class="chip acc"><i></i>On curve · 64%</span><span class="chip">Converting opens at graduation</span>')
    coll_addr = '<span>Collection NoCC…4fW1</span>' if grad else '<span>Collection address: at graduation</span>'
    head = f"""<div class="thead"><div class="crumb"><a href="home.html">Explore</a> &nbsp;/&nbsp; Nocturnes</div>
<div class="trow"><div class="tav">{art('noct','av',412)}</div>
<div><div class="tname"><h1>Nocturnes</h1><span class="mono mute">NOCT</span></div>
<div class="chips">{tchip("hybrid")}{phase_chip}<span class="chip">1,000 NFTs · 1M NOCT each</span><span class="chip">No transfer tax</span><span class="demo" style="align-self:center">Demo data</span></div>{gloss("hybrid")}</div>
<div class="tpr"><div class="p num">{sub_price(px)} <span style="font-size:18px;color:var(--mute);font-weight:500">SOL</span></div>
<div class="mute num" style="font-size:13px">${px*SOL_USD:.7f} per token · <span class="up">{"+4.8" if grad else "+12.4"}% (24h)</span></div>
<div class="addr"><span>Mint {TOK['mint']}</span>{coll_addr}</div></div></div></div>"""

    market = f"""<div class="card pad"><div class="ch"><div><h3>Market cap</h3><div class="sub2 num"><b style="color:var(--ink);font-size:22px;font-weight:600;letter-spacing:-.03em">{usd(mc_usd)}</b> &nbsp;{fmt(mc_sol)} SOL ≈ {sub_price(px)} SOL × supply</div></div>
<div style="display:flex;gap:10px;align-items:center"><span class="demo">Example figures</span><div class="tf"><a>1H</a><a>6H</a><a>1D</a><a class="on">ALL</a></div></div></div>
{chart(G_PRICE if grad else None, "gg" if grad else "ag")}
<div class="stats"><div><span>Price</span><b class="num">{sub_price(px)}</b><small>SOL per token</small></div>
<div><span>Market cap</span><b class="num">{usd(mc_usd)}</b><small class="num">{fmt(mc_sol)} SOL</small></div>
<div><span>Holders</span><b class="num">{fmt(1618 if grad else t['holders'])}</b><small>Top 10 hold {TOK['top10']}%</small></div>
<div><span>24h volume</span><b class="num">{"486.2" if grad else t['vol']} SOL</b><small class="num">{usd((486.2 if grad else t['vol'])*SOL_USD)}</small></div>
<div><span>Supply</span><b class="num">1,000,000,000</b><small style="color:var(--up)">No one can mint more</small></div></div></div>"""

    if grad:
        phase = f"""<div class="card pad"><div class="grad"><div class="gi">{icon("grad").replace('18','22')}</div><div><h4>Graduated</h4><p>Graduated at 85 SOL · 412 of 1,000 captured so far <span class="demo" style="margin-left:4px;vertical-align:1px">Example</span><br>The bonding curve filled on Sep 24; NOCT now trades in a public DEX pool.</p></div><span class="chip ok2" style="margin-left:auto"><i></i>Graduated</span></div>
<div class="gsteps done" style="margin-top:18px"><div><span class="n">✓ Liquidity</span><b>Moved to a DEX pool</b><p>The curve's SOL and 200,000,000 NOCT went into a public pool.</p></div>
<div><span class="n">✓ Converting</span><b>Open, art revealed</b><p>Swap 1,000,000 NOCT for one NFT and back. Each Nocturne is minted the first time someone captures it.</p></div>
<div><span class="n">✓ NFT trading</span><b>Live on Tensor and Magic Eden</b><p>Marketplaces list the Nocturnes captured so far, and more appear as they're captured.</p></div></div></div>"""
    else:
        phase = f"""<div class="card pad"><div class="ch"><div><h3>Bonding curve</h3><div class="sub2">NOCT trades as a plain token until the curve fills</div></div><span class="chip acc"><i></i>Graduates at 85 SOL</span></div>
<div class="curve"><div class="big num">64%</div><div><div class="bar" role="img" aria-label="Curve 64% filled"><i style="width:64%"></i></div>
<div class="cm"><span><b class="num">54.4</b> of <b class="num">85 SOL</b> raised</span><span>Graduation target <b class="num">85 SOL</b></span><span><b class="num">200M NOCT</b> set aside for the DEX pool</span></div></div>{MPH.format(cls="curve-m")}</div>
<p class="expl">While on the curve, the price moves with every buy and sell. No NFTs exist yet and converting is closed. You can preview the traits and rarity odds below.</p>
<div style="margin-top:18px"><div class="eyebrow" style="font-size:11px;margin-bottom:10px">What happens at graduation</div>
<div class="gwrap">{MPH.format(cls="grad-m")}<div class="gsteps"><div><span class="n">01</span><b>Liquidity moves to a DEX</b><p>The curve's SOL and 200,000,000 NOCT go into a public pool.</p></div>
<div><span class="n">02</span><b>Converting opens</b><p>Swap NOCT for NFTs and back. Art is revealed. Each Nocturne is minted the first time someone captures it.</p></div>
<div><span class="n">03</span><b>NFT trading follows</b><p>Captured Nocturnes can be listed on Tensor and Magic Eden. More appear as they're captured.</p></div></div></div></div></div>"""

    # ---------- convert + holdings
    if grad:
        convert = f"""<div class="card pad"><div class="ch"><div><h3>Convert</h3><div class="sub2">Tokens and NFTs, exact both ways</div></div><span class="chip ok2"><i></i>Open</span></div>
<div class="seg2"><a class="on">Tokens → NFT</a><a>NFT → tokens</a></div>
<div class="side" style="margin-top:14px"><div class="l"><span>You give</span><span class="num">Balance 5,750,000</span></div>
<div class="v"><b class="num">1,000,000</b><span class="tok"><span class="ic">{art('noct','ti1',1)}</span>NOCT</span></div><div class="s num">≈ {floor:.3f} SOL · {usd(floor*SOL_USD,2)}</div></div>
<div class="swapic"><span><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 4v16M3 16l4 4 4-4M17 20V4M13 8l4-4 4 4"/></svg></span></div>
<div class="side"><div class="l"><span>You receive</span><span>Nocturnes</span></div>
<div class="v"><b class="num">1 NFT</b><span class="tok"><span class="stack3"><span class="ic">{nft('',21)}</span><span class="ic">{nft('',22)}</span><span class="ic">{nft('',23)}</span></span>Random</span></div><div class="s">Picked by Switchboard VRF · takes a few seconds</div></div>
<div class="rows"><div><span>Rate</span><b class="num">1,000,000 NOCT = 1 NFT</b></div><div><span>Platform fee</span><b class="num">0.01 SOL</b></div><div><span>Mint deposit <span class="dim">(refundable)</span></span><b class="num">{MINT_DEP}</b></div><div><span>Paid now</span><b class="num">{CAP_NOW}</b></div><div><span>Net after refund</span><b class="num">{CAP_NET}</b></div><div><span>Tokens taken</span><b>None</b></div></div>
<a class="btn a cta">Convert to 1 NFT</a>
<p class="note"><b>Convert back any time: you get back exactly 1,000,000 NOCT.</b> No platform fee and no mint cost, since the NFT already exists. Network fees apply (a small Solana transaction fee).</p>
<p class="fn" style="margin-top:8px">The 0.0063 SOL mint deposit comes back in full when the picked NFT has been minted before; otherwise about 0.003–0.004 SOL of it is kept. {MINT_DISC}</p>{UA_LINE}</div>"""
        wt, wn = 5_750_000, 2
        tok_sol, nft_sol = wt * px, wn * floor
        holdings = f"""<div class="card pad"><div class="ch"><div><h3>Your holdings</h3><div class="sub2 mono">7fQa…m2Lx</div></div><span class="demo">Demo wallet</span></div>
<div class="hv num">{tok_sol+nft_sol:.3f} SOL</div><div class="mute num" style="font-size:13px;margin-top:4px">{usd((tok_sol+nft_sol)*SOL_USD,2)} total value</div>
<div class="hl"><div><span class="ic">{art('noct','h1',1)}</span><div class="nm">NOCT tokens<small class="num">{fmt(wt)}</small></div><div class="rt num">{tok_sol:.3f} SOL<small>{usd(tok_sol*SOL_USD,2)}</small></div></div>
<div><div class="nfts">{''.join(f'<span class="n" style="position:relative">{nft(x,int(x[1:]))}{tdot(NFT_TIER[int(x[1:])])}</span>' for x in WALLET['nfts'])}</div><div class="nm">2 NFTs<small>Common, Rare</small></div><div class="rt num">{nft_sol:.3f} SOL<small>1M NOCT each</small></div></div></div>
<div class="avail"><div><span>Tokens available to convert</span><b class="num">5 NFTs · {5*floor:.3f} SOL</b></div><span class="mute num">{usd(5*floor*SOL_USD,2)}</span></div>
<p class="note" style="margin-top:14px">Your 2 NFTs convert back for exactly 2,000,000 NOCT whenever you like.</p>
<div class="stat3" style="margin-top:16px"><div class="mini"><span>Captured so far</span><b class="num">412</b><small>of 1,000 · example</small></div><div class="mini"><span>Not yet minted</span><b class="num">588</b><small>minted on capture</small></div><div class="mini"><span>Floor</span><b class="num">{floor:.3f}</b><small>SOL per NFT</small></div></div></div>"""
    else:
        convert = f"""<div class="card pad"><div class="ch"><div><h3>Convert</h3><div class="sub2">Tokens and NFTs, exact both ways</div></div><span class="chip">Closed on the curve</span></div>
<div class="lockp"><div class="lph"><span class="li2">{LOCK_I.format(s=18)}</span>{MPH.format(cls="conv-m")}</div><b>Converting opens at graduation</b>
<p>When the curve fills, you can swap every 1,000,000 NOCT for one NFT, and convert back for free. {LAZY_LINE}</p></div>
<div class="rows"><div><span>Rate</span><b class="num">1,000,000 NOCT = 1 NFT</b></div><div><span>Capture once open</span><b class="num">0.01 SOL + {MINT_DEP} deposit</b></div><div><span>Converting back</span><b>No platform fee</b></div><div><span>Tokens taken</span><b>None</b></div></div>
<a class="btn g cta off" aria-disabled="true">Converting opens at graduation</a>
<p class="note">Until then, NOCT trades as a plain token on the curve.</p>{UA_LINE}</div>"""
        wt = WALLET["tokens"]; tok_sol = wt * px
        holdings = f"""<div class="card pad"><div class="ch"><div><h3>Your holdings</h3><div class="sub2 mono">7fQa…m2Lx</div></div><span class="demo">Demo wallet</span></div>
<div class="hv num">{tok_sol:.3f} SOL</div><div class="mute num" style="font-size:13px;margin-top:4px">{usd(tok_sol*SOL_USD,2)} total value</div>
<div class="hl"><div><span class="ic">{art('noct','h1',1)}</span><div class="nm">NOCT tokens<small class="num">{fmt(wt)}</small></div><div class="rt num">{tok_sol:.3f} SOL<small>{usd(tok_sol*SOL_USD,2)}</small></div></div>
<div><span class="ic" style="display:grid;place-items:center;background:var(--s2);border:1px solid var(--line2);color:var(--dim)">{LOCK_I.format(s=15)}</span><div class="nm">NFTs<small>None yet. Converting opens at graduation.</small></div><div class="rt num">0<small>NFTs</small></div></div></div>
<div class="avail"><div><span>Convertible once converting opens</span><b class="num">3 NFTs · 3,000,000 NOCT</b></div><span class="mute num">750,000 left</span></div>
<p class="note" style="margin-top:14px">Values use the current curve price. Prices move with trading.</p>
<div class="stat3" style="margin-top:16px"><div class="mini"><span>Curve filled</span><b class="num">64%</b><small>graduates when full</small></div><div class="mini"><span>Collection</span><b class="num">1,000</b><small>NFTs, minted on capture</small></div><div class="mini"><span>Holders</span><b class="num">{fmt(t['holders'])}</b><small>wallets</small></div></div></div>"""

    if state == "capture":
        convert = f"""<div class="card pad stc"><div class="ch"><div><h3>Review capture</h3><div class="sub2">Tokens → NFT · check the cost, then sign</div></div><span class="chip acc"><i></i>Step 1 of 3</span></div>
<div class="cgive"><div><span>You lock</span><b class="num">1,000,000 NOCT</b><small>Held in the vault, not spent. You get them back when you release the NFT.</small></div>
<div><span>You get</span><b>1 random Nocturne</b><small>Picked from the 588 not yet captured or released back <span class="demo">Example</span></small></div></div>
<div class="itm"><div class="ih">Itemized cost, paid in SOL</div>
<div><span>Platform fee<small>Set for Nocturnes at launch (1M ratio)</small></span><b class="num">0.01 SOL</b></div>
<div><span>Mint deposit<small>Refunded when the request settles, except about 0.003–0.004 SOL if your NFT is minted for the first time (Solana rent + Metaplex fee)</small></span><b class="num">{MINT_DEP}</b></div>
<div><span>Network fees<small>Solana transaction fees and temporary account rent, shown in your wallet. The request's account rent comes back at settle</small></span><b>Network</b></div>
<div class="tot"><span>Paid now<small>Net after the refund: 0.01 SOL if your NFT already exists, ≈ 0.013–0.014 SOL if it's minted new</small></span><b class="num">{CAP_NOW}</b></div></div>
<ol class="cst"><li class="on"><b>Sign in your wallet</b><span>Locks 1,000,000 NOCT and pays the fee</span></li><li><b>Randomness arrives</b><span>Switchboard VRF picks your NFT, usually within seconds</span></li><li><b>Your NFT is revealed</b><span>Minted to your wallet if it's new</span></li></ol>
<div style="display:flex;gap:10px;margin-top:16px"><a class="btn a cta" style="flex:1;margin:0">Confirm and sign</a><a class="btn g" style="height:46px">Back</a></div>
<p class="fn" style="margin-top:10px">Rarity is cosmetic. Whatever you get converts back for exactly 1,000,000 NOCT. {MINT_DISC}</p>{UA_LINE}</div>"""
    if state == "release":
        convert = f"""<div class="card pad stc"><div class="ch"><div><h3>Convert</h3><div class="sub2">Release an NFT back to tokens</div></div><span class="chip ok2"><i></i>Free</span></div>
<div class="seg2"><a>Tokens → NFT</a><a class="on">NFT → tokens</a></div>
<div class="side" style="margin-top:14px"><div class="l"><span>You give</span><span>2 Nocturnes in wallet</span></div>
<div class="relp"><div class="rp on"><span class="im">{nft('',142)}</span><b>#0142</b>{tier('Common', True)}</div><div class="rp"><span class="im">{nft('',655)}</span><b>#0655</b>{tier('Rare', True)}</div></div></div>
<div class="swapic"><span><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 4v16M3 16l4 4 4-4M17 20V4M13 8l4-4 4 4"/></svg></span></div>
<div class="side"><div class="l"><span>You receive</span><span>Exact, whatever the traits</span></div>
<div class="v"><b class="num">1,000,000</b><span class="tok"><span class="ic">{art('noct','ti1',1)}</span>NOCT</span></div><div class="s num">≈ {floor:.3f} SOL at the current price</div></div>
<div class="rows"><div><span>Platform fee</span><b>None</b></div><div><span>Mint cost</span><b>None</b></div><div><span>Randomness</span><b>None</b></div><div><span>Network fee</span><b>Shown in your wallet</b></div><div><span>Tokens taken</span><b>None</b></div></div>
<a class="btn a cta">Release #0142 for 1,000,000 NOCT</a>
<p class="note"><b>Release is free.</b> #0142 goes back into the pool and can be picked by a future capture or re-roll. Rare or common, every Nocturne releases for the same 1,000,000 NOCT.</p>{UA_LINE}</div>"""

    # ---------- collection section
    if grad:
        coll = f"""<div class="two">
<div class="card pad"><div class="ch"><div><h3>Your NFTs</h3><div class="sub2">2 Nocturnes in this wallet</div></div><span class="demo">Demo wallet</span></div>
<div class="owned">
<div class="on sel"><div class="im">{nft('#0142',142)}</div><div class="md"><div class="t1"><b>#0142</b>{tier('Common', True)}</div><div class="trs"><span class="trait">Sky<b>Teal</b></span><span class="trait">Phase<b>Crescent</b></span></div></div></div>
<div class="on"><div class="im">{nft('#0655',655)}</div><div class="md"><div class="t1"><b>#0655</b>{tier('Rare', True)}</div><div class="trs"><span class="trait">Sky<b>Indigo</b></span><span class="trait">Phase<b>Eclipse</b></span></div></div></div></div>
<div class="rr{' rrs' if state == 'reroll' else ''}"><div class="rrh"><b>Re-roll #0142</b><span class="mute" style="font-size:12px">Swap it for a different random Nocturne</span></div>
<div class="rows" style="margin-top:10px"><div><span>Platform fee</span><b class="num">0.01 SOL</b></div><div><span>Mint deposit <span class="dim">(refundable)</span></span><b class="num">{MINT_DEP}</b></div><div><span>Paid now</span><b class="num">{CAP_NOW}</b></div><div><span>Net after refund</span><b class="num">{CAP_NET}</b></div><div><span>Tokens taken</span><b>None</b></div></div>
<p class="note" style="margin-top:10px">The new Nocturne is picked with verifiable randomness from those not held in wallets, which can include ones not minted yet. That's when the mint cost applies. The result may be more common than #0142. Either way, the NFT still converts back for exactly 1,000,000 NOCT.</p>{UA_LINE}
{'<ol class="cst" style="margin-top:12px"><li class="on"><b>Sign in your wallet</b><span>#0142 goes back to the pool and you pay the fee</span></li><li><b>Randomness arrives</b><span>Switchboard VRF picks a different Nocturne</span></li><li><b>Your new NFT is revealed</b><span>Minted to your wallet if it&#39;s new</span></li></ol><div style="display:flex;gap:10px;margin-top:12px"><a class="btn a" style="flex:1;height:42px">Confirm re-roll</a><a class="btn g" style="height:42px">Cancel</a></div>' if state == 'reroll' else '<a class="btn g cta" style="height:40px;margin-top:12px">Re-roll #0142</a>'}</div></div>

<div class="card pad"><div class="ch"><div><h3>Floor and rarity</h3><div class="sub2">What sets the price of one NFT</div></div></div>
<div class="flr"><span class="mute" style="font-size:12.5px">Floor = ratio × token price</span><div class="fx num">1,000,000 × {sub_price(px)} SOL = <b>{floor:.3f} SOL</b></div><span class="mute num" style="font-size:12.5px">≈ {usd(floor*SOL_USD,2)} at the current DEX price</span></div>
<p class="note" style="margin-top:14px"><b>Every Nocturne converts back for exactly 1,000,000 NOCT, whatever its rarity.</b> Rarity is cosmetic only. Marketplace prices are set by buyers and sellers.</p>
<div class="dist">{dist_bar}</div>
<div class="dl">{dist_rows}</div>
<p class="note" style="margin-top:12px">Traits were committed before converting opened and matched to NFTs with verifiable randomness, so nobody can choose a specific NFT when converting.</p></div>
</div>"""
        mkt = f"""<div class="card pad"><div class="ch"><div><h3>Trade NFTs</h3><div class="sub2">Nocturnes are live on NFT marketplaces</div></div><span class="chip">External links</span></div>
<div class="mkt"><div class="mkl"><p class="note" style="margin:0;font-size:13.5px;line-height:1.65"><b>{BRAND} handles launching and converting.</b> NFT listings and sales happen on Tensor and Magic Eden, where you can buy or list a specific Nocturne. Nocturnes are minted when first captured, so marketplaces show those captured so far (412 of 1,000 in this example).</p>
<div class="mkb"><a class="btn g" href="https://www.tensor.trade/" target="_blank" rel="noopener noreferrer">Trade on Tensor {EXT}</a><a class="btn g" href="https://magiceden.io/" target="_blank" rel="noopener noreferrer">Trade on Magic Eden {EXT}</a></div>
<p class="fn" style="margin-top:12px">External sites, opened in a new tab. {BRAND} doesn't run these marketplaces or control their fees. Collection page links are placeholders.</p></div>
<div class="mkr"><div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:10px"><b style="font-size:13.5px;font-weight:600">Listings</b><span class="demo">Demo data</span></div>
<div class="lst">{"".join(f'<div class="li"><div class="im">{art("noct", "ls"+str(x), x)}</div><div class="m1"><span class="num">#{x:04d}</span>{tier(NFT_TIER[x], True)}</div><div class="m2"><b class="num">{pr} SOL</b><span>{mk}</span></div></div>' for x, pr, mk in LISTINGS)}</div>
<p class="fn" style="margin-top:10px">Listing prices are set by sellers. Rarity is cosmetic; every Nocturne converts back for exactly 1,000,000 NOCT ({floor:.3f} SOL at the current price).</p></div></div></div>"""
    else:
        tiles = "".join(f'<div class="pvt">{art("noct", "pv"+str(s), s)}<div class="ov2"><span>{LOCK_I.format(s=14)}</span>Revealed at graduation</div></div>' for s in (412, 655, 93, 871))
        odds = "".join(f'<div><div class="ot">{name}</div><div class="ov3">{"".join(f"<span class=trait>{v}<b>{p}%</b></span>" for v, p in vals)}</div></div>' for name, vals in TRAITS_ODDS)
        coll = f"""<div class="card pad"><div class="ch"><div><h3>Collection preview</h3><div class="sub2">1,000 Nocturnes · traits and rarity odds</div></div><span class="chip">Art revealed at graduation</span></div>
<div class="pv"><div><div class="pvart">{tiles}</div><p class="fn" style="margin-top:10px">Art is committed before converting opens and revealed at graduation.</p></div>
<div><div class="odds">{odds}<div><div class="ot">Rarity tiers</div><div class="dist" style="margin:4px 0 8px">{dist_bar}</div><div class="dl">{dist_rows}</div></div></div>
<p class="note" style="margin-top:12px"><b>Rarity is cosmetic only.</b> Every Nocturne will convert to and from exactly 1,000,000 NOCT, whatever its traits. Traits are matched to NFTs with verifiable randomness.</p></div></div></div>"""
        mkt = f"""<div class="card pad"><div class="ch"><div><h3>Trade NFTs</h3><div class="sub2">Collection goes live on Tensor and Magic Eden at graduation</div></div><span class="chip">At graduation</span></div>
<div class="mkt" style="grid-template-columns:minmax(0,1.2fr) minmax(0,.8fr);align-items:center"><p class="note" style="margin:0;font-size:13.5px;line-height:1.65"><b>No NFTs exist yet.</b> After graduation, each Nocturne is minted when it's first captured, and captured Nocturnes can be bought or listed on Tensor and Magic Eden. {BRAND} handles launching and converting.</p>
<div class="mkb" style="margin:0"><a class="btn g off" aria-disabled="true">Tensor · at graduation {EXT}</a><a class="btn g off" aria-disabled="true">Magic Eden · at graduation {EXT}</a></div></div></div>"""

    fee_title = "Fees and control" if grad else "Fees once converting opens"
    fee_sub = "Everything you pay, and who can change what" if grad else "Nothing to pay here until graduation, except trading on the curve"
    trade_fee = ('<div><span>Trading on the DEX</span><b>Pool fee set by the DEX</b><p>Example: 0.25% per swap, charged by the pool, not by us.</p></div>' if grad else
                 '<div><span>Trade fee on the curve</span><b>1%</b><p>Example value, taken in SOL on each buy and sell.</p></div>')
    ccost = f"""<div class="ccost"><div class="ct"><b>Capture cost (tokens to NFT)</b><span>Nocturnes · 1M ratio</span></div>
<div class="cr"><span>Platform fee</span><b class="num">0.01 SOL</b></div>
<div class="cr"><span>Mint deposit<small>Refunded at settle, except about 0.003–0.004 SOL if the NFT is minted for the first time</small></span><b class="num">{MINT_DEP}</b></div>
<div class="cr tot"><span>Paid now<small>Net after refund: 0.01 SOL, or ≈ 0.013–0.014 SOL if the NFT is minted new</small></span><b class="num">{CAP_NOW}</b></div>
<p class="fn">{MINT_DISC} Picks use Switchboard VRF (verifiable random function); there's no separate randomness fee. Re-rolls cost the same, since the new pick can be an NFT not minted yet. Converting back has no platform fee and no deposit.</p></div>"""
    fees = f"""<div class="card pad"><div class="ch"><div><h3>{fee_title}</h3><div class="sub2">{fee_sub}</div></div><span class="chip ua">Unaudited beta</span></div>
<div class="fc">
<div><span>This collection's platform fee</span><b>0.01 SOL per capture or re-roll</b><p>Tokens to NFT, or re-roll{"" if grad else ", once converting opens"}. Converting back is free.</p></div>
<div><span>Tokens taken</span><b>None</b><p>1,000,000 NOCT converts to exactly one NFT and back to exactly 1,000,000 NOCT.</p></div>
<div><span>Network fees</span><b>Network fees apply</b><p>A small Solana transaction fee, shown in your wallet.</p></div>
<div><span>Tax</span><b>No transfer tax</b><p>Standard Solana token (SPL Token). No tax on transfers, buys, sells or converts.</p></div>
{trade_fee}
<div><span>Who can change the program</span><b>{UPG_SHORT}</b><p>One dev key on devnet today. Planned for mainnet: 3-of-5 multisig, 7-day public delay, then frozen after the audit.</p></div>
</div>{ccost}<div class="feeline">{FEE_COPY}{fee_addr}</div></div>"""

    if grad:
        trade_rows = f'<div><span>Route</span><b>Public DEX pool</b></div><div><span>Pool fee 0.25% <span class="dim">(example)</span></span><b class="num">0.0025 SOL</b></div><div><span>Tax</span><b>No transfer tax</b></div>'
        side = _side_trade("NOCT", art('noct','ti2',1), px, G_BUY(1.0), trade_rows, "Buy NOCT", "Routed through the public DEX pool. You'll review exact amounts in your wallet.")
    else:
        trade_rows = f'<div><span>Trade fee 1% <span class="dim">(example)</span></span><b class="num">0.01 SOL</b></div><div><span>Tax</span><b>No transfer tax</b></div>'
        side = _side_trade("NOCT", art('noct','ti2',1), px, H_BUY_NET, trade_rows, "Buy NOCT")
    ok = ck.replace('18','13')
    facts = f"""<div class="card facts"><div><span>Launch type</span><b>Hybrid · SPL-404 (token + NFTs)</b></div>
<div><span>Phase</span><b>{'<span style="color:var(--up)">Graduated</span> · DEX' if grad else 'Bonding curve · 64%'}</b></div>
<div><span>Bonding curve</span><b>{'Filled at 85 SOL' if grad else 'Target 85 SOL'}</b></div>
<div><span>Supply</span><b class="num">1,000,000,000 · fixed</b></div>
<div><span>Mint authority</span><b class="ok">{ok}Revoked</b></div>
<div><span>Freeze authority</span><b class="ok">{ok}Revoked</b></div>
<div><span>Tax</span><b>No transfer tax</b></div>
<div><span>Convert rate</span><b class="num">1,000,000 : 1 NFT</b></div>
<div><span>Collection</span><b class="num">{'1,000 NFTs · 412 captured' if grad else '1,000 NFTs · minted on capture'}</b></div>
<div><span>Converting</span><b>{'Open' if grad else 'Opens at graduation'}</b></div>
<div><span>Platform fee</span><b class="num">0.01 SOL · capture, re-roll</b></div>
<div><span>Capture cost</span><b class="num">0.01 SOL + {MINT_DEP} refundable deposit</b></div>
<div><span>Converting back</span><b>No platform fee</b></div>
<div><span>NFT standard</span><b>Metaplex Core</b></div>
<div><span>Rarity</span><b>Cosmetic · 4 tiers</b></div>
<div><span>Randomness</span><b>Committed art + Switchboard VRF</b></div>
<div><span>Audit</span><b style="color:#e8b04b">Not yet · unaudited beta</b></div>
<p class="fn" style="margin:0;padding:10px 0 12px">Unsold curve tokens are locked by the program, not burned; supply stays 1,000,000,000.</p></div>"""
    about_chips = '<span class="chip">Creator 5Gh2…aP9s</span><span class="chip">Launched Sep 22</span>' + ('<span class="chip ok2">Graduated Sep 24</span>' if grad else '')
    about_txt = ("1,000 generative night skies, launched on " + BRAND + " as one SPL-404 asset. Each NFT converts to and from exactly 1,000,000 NOCT." if grad else
                 "1,000 generative night skies, launched on " + BRAND + " as one SPL-404 asset. Converting opens at graduation; each NFT is minted the first time it's captured and converts to and from exactly 1,000,000 NOCT.")
    body = f"""
<div class="wrap">
{head}
<div class="layout"><div class="col">
{market}
{phase}
<div class="two" id="st-conv">
{convert}
{holdings}
</div>
{coll.replace('<div class="two">', '<div class="two" id="st-coll">', 1)}
{mkt}
{fees}
{auth_panel("hybrid")}
<div class="card pad"><div class="ch"><div><h3>Rules for this token</h3><div class="sub2">Set at launch. Not the creator, not us.</div></div></div>
<div class="locks">{locks}</div></div>
{_act_table(rows, '<a class="on">All</a><a>Trades</a><a>Converts</a>' if grad else '<a class="on">All</a><a>Buys</a><a>Sells</a>', "Latest on-chain events" if grad else "Trades on the bonding curve")}
</div>
<aside class="col">
{side}
{facts}
<div class="card pad" style="padding:18px 20px"><div class="ch" style="margin-bottom:6px"><div><h3>Top holders</h3><div class="sub2">{"1,618" if grad else "1,286"} holders · Top 10 hold 18.4%</div></div></div>
<div class="facts" style="padding:0">{holders}</div></div>
<div class="card pad" style="padding:18px 20px"><h3 style="font-size:15px;letter-spacing:-.015em">About Nocturnes</h3><p class="note" style="margin-top:8px">{about_txt}</p>
<div class="chips" style="margin-top:12px">{about_chips}</div></div>
{_risk_card("Memecoins and NFTs are highly volatile. You can lose everything you put in. Nothing here is financial advice.")}
</aside></div></div>"""
    title = f"Nocturnes (NOCT){' · Graduated' if grad else ''}{' · ' + state.capitalize() if state else ''} · {BRAND}"
    desc = ("Nocturnes (NOCT) after graduation: 1,000 generative night skies as one SPL-404 asset. 1,000,000 NOCT converts to 1 NFT and back, exactly. Devnet preview." if grad else
            "Nocturnes (NOCT) on the bonding curve: preview the traits and rarity odds. Converting opens at graduation, and each NFT is minted when first captured. Devnet preview.")
    return page(title, TOKEN_CSS + STATE_CSS + AUTH_CSS + H_UP_CSS, h_up(body), active="explore", risk=RISK_H, desc=desc)

STATE_CSS = """
.stc{box-shadow:0 0 0 1px rgba(255,106,43,.35) inset}
.cgive{display:grid;grid-template-columns:1fr 1fr;gap:10px}.cgive div{padding:14px;border-radius:12px;background:var(--s2);border:1px solid var(--line)}
.cgive span{display:block;font-size:12px;color:var(--mute)}.cgive b{display:block;font-size:16px;font-weight:600;margin:4px 0 6px;letter-spacing:-.02em}.cgive small{display:block;font-size:11.5px;color:var(--mute);line-height:1.45}
.itm{margin-top:14px;border:1px solid var(--line);border-radius:12px;padding:4px 14px}.itm .ih{font:500 11px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);padding:10px 0 4px}
.itm div:not(.ih){display:flex;justify-content:space-between;gap:14px;padding:10px 0;border-top:1px solid var(--line);font-size:13px}.itm div span{color:var(--ink)}.itm small{display:block;color:var(--mute);font-size:11.5px;margin-top:2px;line-height:1.4}
.itm div b{font-weight:500;white-space:nowrap}.itm .tot span,.itm .tot b{font-weight:600}.itm .tot b{color:var(--acc2)}
.cst{list-style:none;margin:16px 0 0;padding:0;display:flex;flex-direction:column;gap:8px;counter-reset:c}
.cst li{counter-increment:c;display:grid;grid-template-columns:24px 1fr;column-gap:10px;font-size:13px}.cst li:before{content:counter(c);grid-row:span 2;width:22px;height:22px;border-radius:50%;display:grid;place-items:center;border:1px solid var(--line2);font:500 11px var(--mono);color:var(--mute)}
.cst li.on:before{border-color:var(--acc);color:var(--acc2);background:var(--accbg)}.cst li span{color:var(--mute);font-size:12px}
.relp{display:flex;gap:8px;margin-top:10px}.rp{flex:1;display:flex;align-items:center;gap:8px;padding:8px;border-radius:10px;border:1px solid var(--line2);font-size:13px}.rp.on{box-shadow:0 0 0 1px var(--acc) inset;background:var(--accbg)}
.rp .im{width:34px;height:34px;border-radius:7px;overflow:hidden;flex:none}.rp .im svg{width:100%;height:100%;display:block}
.rrs{box-shadow:0 0 0 1px rgba(255,106,43,.35) inset}
"""
open(os.path.join(OUT, "token.html"), "w").write(token_page(False))
open(os.path.join(OUT, "token-graduated.html"), "w").write(token_page(True))
for _st in ("capture", "release", "reroll"):
    open(os.path.join(OUT, f"token-{_st}.html"), "w").write(token_page(True, _st))
print("token ok")


# ------------------------------------------------------------------ PLAIN + BURN token pages (fix/modes-1-5: Mode 1 plain, Mode 3 burn)
def mode_page(kind):
    L = next(x for x in A_LAUNCHES if x["key"] == ("salt" if kind == "plain" else "ferro"))
    sym, name, key, px = L["sym"], L["name"], L["key"], L["price"]
    plain = kind == "plain"
    ck = icon("check"); ok = ck.replace('18', '13')
    supply = L["supply"]; mc_sol = px * supply; mc_usd = mc_sol * SOL_USD
    colors = {"Buy": "var(--up)", "Sell": "var(--down)", "Burn": "var(--acc)"}
    if plain:
        acts = [("Buy", "7fQa…m2Lx", 1_920_000, 0.99, "9s"), ("Buy", "Hk3P…9wQe", 4_810_000, 2.48, "41s"), ("Sell", "C2vN…t8Rb", 2_000_000, 1.01, "1m"), ("Buy", "9mTe…Ka71", 970_000, 0.5, "3m"), ("Sell", "Gw4d…Lr8e", 6_200_000, 3.14, "6m")]
        rows = "".join(f'<tr><td><span class="ty"><i style="background:{colors[k]}"></i>{k}</span></td><td class="mono mute">{w}</td><td class="num">{fmt(tk)} {sym}</td><td class="num">{v:.3f} SOL</td><td class="mute num">{a} ago</td></tr>' for k, w, tk, v, a in acts)
    else:
        acts = [("Burn", "7fQa…m2Lx", "Minted Ferro #0311", "18s"), ("Buy", "Hk3P…9wQe", f"{fmt(3_700_000)} FERRO", "52s"), ("Burn", "C2vN…t8Rb", "Minted Ferro #0310", "2m"), ("Sell", "9mTe…Ka71", f"{fmt(1_500_000)} FERRO", "4m"), ("Burn", "Gw4d…Lr8e", "Minted Ferro #0309", "7m")]
        rows = "".join(f'<tr><td><span class="ty"><i style="background:{colors[k]}"></i>{"Burned to mint" if k == "Burn" else k}</span></td><td class="mono mute">{w}</td><td class="num">{"1,000,000 FERRO" if k == "Burn" else amt}</td><td class="num">{"<span class=mute>" + amt + " · 0.01 SOL fee</span>" if k == "Burn" else ("1.236 SOL" if k == "Buy" else "0.496 SOL")}</td><td class="mute num">{a} ago</td></tr>' for k, w, amt, a in acts)

    phase_chip = ('<span class="chip acc"><i></i>On curve · 88%</span><span class="chip">No NFT collection</span>' if plain else
                  '<span class="chip ok2"><i></i>Graduated</span><span class="chip">Trading on a DEX</span><span class="chip">Burning open</span><span class="chip">500 NFTs · 1M FERRO each</span>')
    head = f"""<div class="thead"><div class="crumb"><a href="explore.html">Explore</a> &nbsp;/&nbsp; {name}</div>
<div class="trow"><div class="tav">{art(key,'av',7)}</div>
<div><div class="tname"><h1>{name}</h1><span class="mono mute">{sym}</span></div>
<div class="chips">{tchip(kind)}{status_chip(kind)}{phase_chip}<span class="chip">No transfer tax</span><span class="demo" style="align-self:center">Demo data</span></div>{gloss(kind)}</div>
<div class="tpr"><div class="p num">{sub_price(px)} <span style="font-size:18px;color:var(--mute);font-weight:500">SOL</span></div>
<div class="mute num" style="font-size:13px">${px*SOL_USD:.7f} per token · <span class="up">+{L['ch']}% (24h)</span></div>
<div class="addr"><span>Mint {"SaLT…7kQ2" if plain else "FeRR…2mXa"}</span>{"<span>No collection</span>" if plain else "<span>Collection FrCC…9qL3</span>"}</div></div></div></div>"""

    sup_cell = ('<div><span>Supply</span><b class="num">1,000,000,000</b><small style="color:var(--up)">Fixed. No one can mint more</small></div>' if plain else
                f'<div><span>Supply now</span><b class="num">{fmt(supply)}</b><small>{fmt(SUPPLY - supply)} burned · <span style="color:#e8b04b">Example</span></small></div>')
    market = f"""<div class="card pad"><div class="ch"><div><h3>Market cap</h3><div class="sub2 num"><b style="color:var(--ink);font-size:22px;font-weight:600;letter-spacing:-.03em">{usd(mc_usd)}</b> &nbsp;{fmt(mc_sol)} SOL ≈ {sub_price(px)} SOL × {"supply" if plain else "current supply"}</div></div>
<div style="display:flex;gap:10px;align-items:center"><span class="demo">Example figures</span><div class="tf"><a>1H</a><a>6H</a><a>1D</a><a class="on">ALL</a></div></div></div>
{chart(px, "pg" if plain else "bg", seed=5 if plain else 8)}
<div class="stats"><div><span>Price</span><b class="num">{sub_price(px)}</b><small>SOL per token</small></div>
<div><span>Market cap</span><b class="num">{usd(mc_usd)}</b><small class="num">{fmt(mc_sol)} SOL</small></div>
<div><span>Holders</span><b class="num">{fmt(L['holders'])}</b><small>Top 10 hold {"16.2" if plain else "21.7"}%</small></div>
<div><span>24h volume</span><b class="num">{L['vol']} SOL</b><small class="num">{usd(L['vol']*SOL_USD)}</small></div>
{sup_cell}</div></div>"""

    if plain:
        phase = f"""<div class="card pad"><div class="ch"><div><h3>Bonding curve</h3><div class="sub2">{sym} trades on the curve until it fills</div></div><span class="chip acc"><i></i>Graduates at 85 SOL</span></div>
<div class="curve"><div class="big num">88%</div><div><div class="bar" role="img" aria-label="Curve 88% filled"><i style="width:88%"></i></div>
<div class="cm"><span><b class="num">74.8</b> of <b class="num">85 SOL</b> raised</span><span>Graduation target <b class="num">85 SOL</b></span><span><b class="num">200M {sym}</b> set aside for the DEX pool</span></div></div>{MPH.format(cls="curve-m")}</div>
<p class="expl">The price moves with every buy and sell. When the curve fills, the SOL raised and the tokens set aside go into a public DEX pool and {sym} keeps trading there. That's the whole lifecycle: a Plain launch has no NFT side.</p>
<div class="gsteps" style="margin-top:18px"><div><span class="n">Now</span><b>Trading on the curve</b><p>Buy and sell {sym} here. Price follows the curve.</p></div>
<div><span class="n">At 85 SOL</span><b>Liquidity moves to a DEX</b><p>The curve's SOL and 200,000,000 {sym} go into a public pool.</p></div>
<div><span class="n">After</span><b>Trades like any token</b><p>Unsold curve tokens stay locked by the program, not burned.</p></div></div></div>"""
    else:
        phase = f"""<div class="card pad"><div class="grad"><div class="gi">{icon("grad").replace('18','22')}</div><div><h4>Graduated</h4><p>Graduated at 85 SOL · 312 of 500 NFTs minted so far <span class="demo" style="margin-left:4px;vertical-align:1px">Example</span><br>FERRO now trades in a public DEX pool, and burning is open.</p></div></div>
<div class="gsteps done" style="margin-top:18px"><div><span class="n">✓ Liquidity</span><b>Moved to a DEX pool</b><p>The curve's SOL and 200,000,000 FERRO went into a public pool.</p></div>
<div><span class="n">✓ Burning</span><b>Open, art revealed</b><p>Burn 1,000,000 FERRO to mint the next Ferro in collection order.</p></div>
<div><span class="n">✓ NFT trading</span><b>On Tensor and Magic Eden</b><p>Minted Ferros can be listed there. Burned tokens never come back.</p></div></div></div>"""

    if plain:
        wt = 2_400_000; tv = wt * px
        left = f"""<div class="card pad"><div class="ch"><div><h3>What a Plain launch is</h3><div class="sub2">Just the coin, nothing to convert</div></div>{tchip("plain")}</div>
<div class="facts" style="padding:0">
<div><span>Token</span><b>Classic SPL token</b></div><div><span>Supply</span><b class="num">1,000,000,000 · fixed</b></div>
<div><span>Mint authority</span><b class="ok">{ok}Revoked</b></div><div><span>Freeze authority</span><b class="ok">{ok}Revoked</b></div>
<div><span>Launch settings</span><b>Can't be edited</b></div><div><span>NFT collection</span><b>None</b></div>
<div><span>Converting, burning, re-rolls</span><b>Not available</b></div><div><span>Per-NFT platform fee</span><b>None, there are no NFTs</b></div></div>
<p class="note" style="margin-top:12px">Plain launches use the same bonding curve and graduation as every {BRAND} launch. They can't add an NFT collection later.</p></div>"""
        right = f"""<div class="card pad"><div class="ch"><div><h3>Your holdings</h3><div class="sub2 mono">7fQa…m2Lx</div></div><span class="demo">Demo wallet</span></div>
<div class="hv num">{tv:.3f} SOL</div><div class="mute num" style="font-size:13px;margin-top:4px">{usd(tv*SOL_USD,2)} total value</div>
<div class="hl"><div><span class="ic">{art(key,'h1',7)}</span><div class="nm">{sym} tokens<small class="num">{fmt(wt)}</small></div><div class="rt num">{tv:.3f} SOL<small>{usd(tv*SOL_USD,2)}</small></div></div></div>
<p class="note" style="margin-top:14px">Values use the current curve price. Prices move with trading.</p>
<div class="stat3" style="margin-top:16px"><div class="mini"><span>Curve filled</span><b class="num">88%</b><small>graduates when full</small></div><div class="mini"><span>Your share</span><b class="num">0.24%</b><small>of supply</small></div><div class="mini"><span>Holders</span><b class="num">{fmt(L['holders'])}</b><small>wallets</small></div></div></div>"""
        coll = ""
    else:
        wt = 4_200_000; tv = wt * px; nb = round(1_000_000 * px, 3)
        left = f"""<div class="card pad"><div class="ch"><div><h3>Burn to mint</h3><div class="sub2">Tokens → NFT · one-way</div></div><span class="chip ok2"><i></i>Open</span></div>
<div class="side"><div class="l"><span>You burn</span><span class="num">Balance 4,200,000</span></div>
<div class="v"><b class="num">1,000,000</b><span class="tok"><span class="ic">{art(key,'ti1',7)}</span>FERRO</span></div><div class="s num">≈ {nb:.3f} SOL at the current price · destroyed for good</div></div>
<div class="swapic"><span><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 4v16M6 14l6 6 6-6"/></svg></span></div>
<div class="side"><div class="l"><span>You receive</span><span>Next in collection order</span></div>
<div class="v"><b>Ferro #0312</b><span class="tok"><span class="ic">{art(key,'nx',312)}</span>{tier("Uncommon", True)}</span></div><div class="s">Minted to your wallet in the same transaction. No randomness.</div></div>
<div class="rows"><div><span>Rate</span><b class="num">1,000,000 FERRO = 1 NFT</b></div><div><span>Platform fee</span><b class="num">0.01 SOL</b></div><div><span>Mint cost <span class="dim">(paid directly, no deposit)</span></span><b class="num">{BURN_MINT}</b></div><div><span>Total</span><b class="num">{BURN_TOTAL}</b></div><div><span>Tokens</span><b>1,000,000 burned, permanently</b></div></div>
<a class="btn a cta">Burn 1,000,000 FERRO</a>
<p class="note"><b>There's no way back.</b> A Ferro can't be converted into FERRO again, and nobody can re-roll it. It has no token backing: its price is whatever buyers pay on a marketplace.</p>{UA_LINE}</div>"""
        right = f"""<div class="card pad"><div class="ch"><div><h3>Your holdings</h3><div class="sub2 mono">7fQa…m2Lx</div></div><span class="demo">Demo wallet</span></div>
<div class="hv num">{tv:.3f} SOL</div><div class="mute num" style="font-size:13px;margin-top:4px">{usd(tv*SOL_USD,2)} in tokens</div>
<div class="hl"><div><span class="ic">{art(key,'h1',7)}</span><div class="nm">FERRO tokens<small class="num">{fmt(wt)}</small></div><div class="rt num">{tv:.3f} SOL<small>{usd(tv*SOL_USD,2)}</small></div></div>
<div><span class="ic">{art(key,'h2',201)}</span><div class="nm">1 NFT<small>Ferro #0201 · Common</small></div><div class="rt">Not valued<small>No token backing</small></div></div></div>
<div class="avail"><div><span>Enough to burn for</span><b class="num">4 NFTs · 4,000,000 FERRO</b></div><span class="mute num">200,000 left</span></div>
<p class="note" style="margin-top:14px">Burned NFTs aren't counted in your total: they can't be converted back, so only a marketplace sale sets their price.</p>
<div class="stat3" style="margin-top:16px"><div class="mini"><span>Minted so far</span><b class="num">312</b><small>of 500 · example</small></div><div class="mini"><span>Burned so far</span><b class="num">312M</b><small>FERRO · example</small></div><div class="mini"><span>Most that can burn</span><b class="num">500M</b><small>if all 500 mint</small></div></div></div>"""
        order = "".join(f'<div class="oq{" nx" if i == 0 else ""}"><span class="im">{art(key, "oq"+str(n), n)}</span><b class="num">#{n:04d}</b>{tier(t, True)}<span class="dim" style="margin-left:auto;font-size:12px">{"Next" if i == 0 else f"In {i}"}</span></div>' for i, (n, t) in enumerate([(312, "Uncommon"), (313, "Common"), (314, "Common"), (315, "Rare"), (316, "Common")]))
        coll = f"""<div class="two">
<div class="card pad"><div class="ch"><div><h3>Collection order</h3><div class="sub2">Who gets what is public before anyone burns</div></div><span class="chip">No randomness</span></div>
<div class="bar" role="img" aria-label="312 of 500 minted"><i style="width:62.4%"></i></div><div class="pl2"><span><b class="num">312</b> of 500 minted</span><span class="demo">Example</span></div>
<div class="oql">{order}</div>
<p class="note" style="margin-top:12px">Art and traits were committed before burning opened. Each burn mints the next NFT in that fixed order, so anyone can see which Ferro comes next, including rare ones.</p></div>
<div class="card pad"><div class="ch"><div><h3>Supply and rarity</h3><div class="sub2">What burning does to FERRO</div></div></div>
<div class="flr"><span class="mute" style="font-size:12.5px">Supply now = 1,000,000,000 − burned</span><div class="fx num">1,000,000,000 − 312,000,000 = <b>688,000,000</b></div><span class="mute" style="font-size:12.5px">Example. Supply only goes down</span></div>
<p class="note" style="margin-top:14px"><b>Rarity is cosmetic only.</b> Traits change how a Ferro looks. They don't change what it costs to mint, and no Ferro converts back to tokens.</p>
<div class="dist">{"".join(f'<i style="flex:{c};background:{TINT[n]}"></i>' for n, c in (("Common", 300), ("Uncommon", 125), ("Rare", 60), ("Legendary", 15)))}</div>
<div class="dl">{"".join(f'<div>{tier(n, True)}<span class="num">{c:,} NFTs</span><span class="num dim">{c/5:g}%</span></div>' for n, c in (("Common", 300), ("Uncommon", 125), ("Rare", 60), ("Legendary", 15)))}</div></div></div>"""

    if plain:
        fc = f"""<div><span>Trade fee on the curve</span><b>1%</b><p>Example value, taken in SOL on each buy and sell.</p></div>
<div><span>After graduation</span><b>Pool fee set by the DEX</b><p>Charged by the pool, not by us.</p></div>
<div><span>Per-NFT fees</span><b>None</b><p>Plain launches have no NFTs, so no capture, burn or mint costs.</p></div>
<div><span>Tax</span><b>No transfer tax</b><p>Standard Solana token (SPL Token).</p></div>
<div><span>Network fees</span><b>Network fees apply</b><p>A small Solana transaction fee, shown in your wallet.</p></div>
<div><span>Who can change the program</span><b>{UPG_SHORT}</b><p>One dev key on devnet today. Planned for mainnet: 3-of-5 multisig, 7-day public delay, then frozen after the audit.</p></div>"""
        extra = ""
    else:
        fc = f"""<div><span>This collection's platform fee</span><b>0.01 SOL per burn</b><p>Set at launch from the 1M ratio. Goes to the platform fee wallet.</p></div>
<div><span>Mint cost</span><b>{MINT_COST} per burn</b><p>Paid directly by the burner: Solana rent plus the Metaplex Core fee.</p></div>
<div><span>Randomness</span><b>None</b><p>NFTs go out in collection order.</p></div>
<div><span>Tokens</span><b>Burned, not held</b><p>Each burn destroys exactly 1,000,000 FERRO. Nothing to release later.</p></div>
<div><span>Trading on the DEX</span><b>Pool fee set by the DEX</b><p>Example: 0.25% per swap, charged by the pool.</p></div>
<div><span>Who can change the program</span><b>{UPG_SHORT}</b><p>One dev key on devnet today. Planned for mainnet: 3-of-5 multisig, 7-day public delay, then frozen after the audit.</p></div>"""
        extra = f"""<div class="ccost"><div class="ct"><b>Cost to mint one Ferro</b><span>1M ratio</span></div>
<div class="cr"><span>Tokens burned</span><b class="num">1,000,000 FERRO</b></div>
<div class="cr"><span>Platform fee</span><b class="num">0.01 SOL</b></div>
<div class="cr"><span>Mint cost<small>Every burn mints a new NFT: Solana rent + Metaplex fee, paid directly</small></span><b class="num">{BURN_MINT}</b></div>
<div class="cr tot"><span>Total in SOL<small>No deposit and nothing refunded later</small></span><b class="num">{BURN_TOTAL}</b></div>
<p class="fn">{MINT_DISC}</p></div>"""
    fees = f"""<div class="card pad"><div class="ch"><div><h3>Fees and control</h3><div class="sub2">Everything you pay, and who can change what</div></div><span class="chip ua">Unaudited beta</span></div>
<div class="fc">{fc}</div>{extra}<div class="feeline">{fee_copy("0.01 SOL") if not plain else "No platform fee per NFT. Trade fees shown are examples."}</div></div>"""

    lk = ([("Supply is fixed", SUPPLY_LINE), ("Nobody can freeze your tokens", "Freeze authority is revoked at launch."), ("No transfer tax", f"{sym} is a standard Solana token."),
           ("Launch settings can't be edited", "The program has no instruction to change a Plain launch after it's created."), ("Unsold curve tokens are locked", "Held by the program, not burned and not sold later."), ("No NFT side, ever", "A Plain launch can't add a collection later.")]
          if plain else
          [("Nobody can mint more", "Mint authority is revoked. Supply can only go down, through burns."), ("Nobody can freeze your tokens", "Freeze authority is revoked at launch."),
           ("Burns are exact and final", "Each burn destroys exactly 1,000,000 FERRO and mints one NFT. There's no unwrap and no re-roll."), ("Art and order are fixed", "Committed before burning opens as a Merkle root. Each NFT must match it."),
           ("This collection's fee can't go up", "0.01 SOL per burn, set at launch from the ratio."), ("Rarity is cosmetic", "Traits don't change the burn rate or the fees.")])
    locks = "".join(f'<div class="lk"><div class="ck">{ck}</div><div><b>{a}</b><p>{b}</p></div></div>' for a, b in lk)
    if plain:
        tr = '<div><span>Trade fee 1% <span class="dim">(example)</span></span><b class="num">0.01 SOL</b></div><div><span>Tax</span><b>No transfer tax</b></div>'
        side = _side_trade(sym, art(key, 'ti2', 7), px, round(0.99 / px), tr, f"Buy {sym}")
    else:
        tr = '<div><span>Route</span><b>Public DEX pool</b></div><div><span>Pool fee 0.25% <span class="dim">(example)</span></span><b class="num">0.0025 SOL</b></div><div><span>Tax</span><b>No transfer tax</b></div>'
        side = _side_trade(sym, art(key, 'ti2', 7), px, round(0.9975 / px), tr, f"Buy {sym}", "Routed through the public DEX pool. You'll review exact amounts in your wallet.")
    if plain:
        fl = [("Launch type", "Plain · SPL token"), ("Phase", "Bonding curve · 88%"), ("Bonding curve", "Target 85 SOL"), ("Supply", "1,000,000,000 · fixed"),
              ("Mint authority", f'<span class="ok">{ok}Revoked</span>'), ("Freeze authority", f'<span class="ok">{ok}Revoked</span>'), ("Tax", "No transfer tax"), ("NFT collection", "None"), ("Audit", '<span style="color:#e8b04b">Not yet · unaudited beta</span>')]
    else:
        fl = [("Launch type", "Burn · one-way"), ("Phase", '<span style="color:var(--up)">Graduated</span> · DEX'), ("Bonding curve", "Filled at 85 SOL"), ("Supply", "1B at launch · 688M now"),
              ("Mint authority", f'<span class="ok">{ok}Revoked</span>'), ("Freeze authority", f'<span class="ok">{ok}Revoked</span>'), ("Tax", "No transfer tax"), ("Burn rate", "1,000,000 : 1 NFT"),
              ("Collection", "500 NFTs · 312 minted"), ("NFT order", "Collection order, public"), ("Platform fee", "0.01 SOL per burn"), ("Converting back", "Not possible"), ("NFT standard", "Metaplex Core"),
              ("Audit", '<span style="color:#e8b04b">Not yet · unaudited beta</span>')]
    facts = '<div class="card facts">' + "".join(f'<div><span>{a}</span><b>{b}</b></div>' for a, b in fl) + '<p class="fn" style="margin:0;padding:10px 0 12px">Unsold curve tokens are locked by the program, not burned.</p></div>'
    hl = ([("Bonding curve", "12.0%"), ("3xTr…Qp2m", "2.81%"), ("9mTe…Ka71", "2.12%"), ("7fQa…m2Lx (you)", "0.24%")] if plain else
          [("DEX pool", "24.6%"), ("3xTr…Qp2m", "3.40%"), ("Gw4d…Lr8e", "2.05%"), ("7fQa…m2Lx (you)", "0.61%")])
    holders = "".join(f'<div><span class="{"" if n in ("Bonding curve", "DEX pool") else "mono"}">{n}</span><b class="num">{v}</b></div>' for n, v in hl)
    about = (f"Salt Flats is a Plain launch: a classic Solana memecoin on a bonding curve, with no NFT collection." if plain else
             "Ferro is a Burn launch: 500 forged-metal pieces. Burn 1,000,000 FERRO to mint the next one. Burns are permanent.")
    body = f"""
<div class="wrap">
{head}
<div class="layout"><div class="col">
{market}
{phase}
<div class="two">
{left}
{right}
</div>
{coll}
{fees}
{auth_panel(kind, sym)}
<div class="card pad"><div class="ch"><div><h3>Rules for this token</h3><div class="sub2">Set at launch. Not the creator, not us.</div></div></div>
<div class="locks">{locks}</div></div>
{_act_table(rows, '<a class="on">All</a><a>Buys</a><a>Sells</a>' if plain else '<a class="on">All</a><a>Trades</a><a>Burns</a>', "Trades on the bonding curve" if plain else "Latest on-chain events")}
</div>
<aside class="col">
{side}
{facts}
<div class="card pad" style="padding:18px 20px"><div class="ch" style="margin-bottom:6px"><div><h3>Top holders</h3><div class="sub2">{fmt(L['holders'])} holders</div></div></div>
<div class="facts" style="padding:0">{holders}</div></div>
<div class="card pad" style="padding:18px 20px"><h3 style="font-size:15px;letter-spacing:-.015em">About {name}</h3><p class="note" style="margin-top:8px">{about}</p>
<div class="chips" style="margin-top:12px"><span class="chip">Creator 8Rt2…Wq4n</span><span class="chip">Launched {"Sep 27" if plain else "Sep 19"}</span></div></div>
{_risk_card("Memecoins and NFTs are highly volatile. You can lose everything you put in. Nothing here is financial advice.")}
</aside></div></div>"""
    css = TOKEN_CSS + STATE_CSS + AUTH_CSS + """
.pl2{display:flex;justify-content:space-between;align-items:center;font-size:12.5px;color:var(--mute);margin:8px 0 14px}.pl2 b{color:var(--ink);font-weight:500}
.oql{display:flex;flex-direction:column;gap:6px}.oq{display:flex;align-items:center;gap:10px;padding:7px 10px 7px 7px;border-radius:10px;border:1px solid var(--line);font-size:13px}
.oq.nx{box-shadow:0 0 0 1px rgba(255,106,43,.4) inset;background:var(--accbg)}.oq .im{width:30px;height:30px;border-radius:7px;overflow:hidden;flex:none}.oq .im svg{width:100%;height:100%;display:block}"""
    title = f"{name} ({sym}) · {TNAME[kind]} launch · {BRAND}"
    desc = (f"{name} ({sym}), a Plain launch on {BRAND}: a classic Solana memecoin on a bonding curve. No NFT collection. Unaudited beta on devnet." if plain else
            f"{name} ({sym}), a Burn launch on {BRAND}: burn 1,000,000 FERRO to mint the next NFT in collection order. One-way. Unaudited beta on devnet.")
    return page(title, css + H_UP_CSS, h_up(body), active="explore", risk=RISK_H, desc=desc)

open(os.path.join(OUT, "token-plain.html"), "w").write(mode_page("plain"))
open(os.path.join(OUT, "token-burn.html"), "w").write(mode_page("burn"))
print("plain/burn ok")

# ------------------------------------------------------------------ LAUNCH (create a collection)
LAUNCH_CSS = """
.lhead{padding:44px 0 32px;border-bottom:1px solid var(--line);display:flex;justify-content:space-between;align-items:flex-end;gap:32px}
.lhead h1{font-size:44px;letter-spacing:-.04em;margin:14px 0 10px}
.lhead p{margin:0;color:var(--mute);font-size:16px;max-width:620px;line-height:1.6}
.lgrid{display:grid;grid-template-columns:208px minmax(0,1fr) 372px;gap:28px;padding-top:32px;align-items:start}
.rail{position:sticky;top:96px;display:flex;flex-direction:column;gap:2px}
.rail a{display:flex;align-items:center;gap:12px;padding:10px 12px;border-radius:10px;color:var(--mute);font-weight:500}
.rail a.on{background:rgba(255,255,255,.04);color:var(--ink);box-shadow:0 0 0 1px var(--line) inset}
.rail .n{width:24px;height:24px;border-radius:50%;display:grid;place-items:center;font:500 11.5px var(--mono);border:1px solid var(--line2);color:var(--mute);flex:none}
.rail a.on .n{border-color:var(--acc);color:var(--acc2);background:var(--accbg)}
.rail a.done .n{background:rgba(62,207,142,.12);border-color:rgba(62,207,142,.35);color:var(--up)}
.rail .hint{margin:18px 12px 0;padding-top:16px;border-top:1px solid var(--line);font-size:12.5px;color:var(--dim);line-height:1.55}
.form{display:flex;flex-direction:column;gap:24px;min-width:0}
.sec{padding:26px 28px 28px}
.sh{display:flex;justify-content:space-between;align-items:flex-start;gap:16px;margin-bottom:22px}
.sh .k{font:500 12px var(--mono);color:var(--dim);letter-spacing:.06em;text-transform:uppercase}
.sh h2{font-size:21px;letter-spacing:-.025em;margin:6px 0 4px}
.sh p{margin:0;color:var(--mute);font-size:13.5px;line-height:1.55;max-width:560px}
.row2{display:grid;grid-template-columns:1fr 1fr;gap:16px}
.f{display:flex;flex-direction:column;gap:7px;margin-bottom:18px}
.f label{font-size:13px;font-weight:500;color:var(--ink);display:flex;justify-content:space-between}
.f label span{color:var(--dim);font-weight:400}
.in{height:44px;border-radius:11px;background:var(--s1);border:1px solid var(--line2);color:var(--ink);font:500 14px var(--sans);padding:0 14px;outline:none;width:100%}
.in:focus{border-color:rgba(255,106,43,.6);box-shadow:0 0 0 3px rgba(255,106,43,.15)}
.pre{position:relative}.pre b{position:absolute;left:14px;top:12px;color:var(--dim);font-weight:500}.pre .in{padding-left:28px}
textarea.in{height:88px;padding:12px 14px;resize:none;line-height:1.55;font-weight:400}
.help{font-size:12.5px;color:var(--mute);line-height:1.5}
.help.warn{color:#e8b04b}
.in.warn{border-color:rgba(232,176,75,.6);box-shadow:0 0 0 3px rgba(232,176,75,.12)}
.drop{display:grid;grid-template-columns:120px 1fr;gap:18px;align-items:center;padding:16px;border:1px dashed var(--line2);border-radius:14px;background:rgba(255,255,255,.015)}
.drop .pv{width:120px;height:120px;border-radius:12px;overflow:hidden;border:1px solid var(--line2)}
.drop b{font-weight:600;display:block;margin-bottom:4px}
.drop .sm{font-size:12.5px;color:var(--mute);line-height:1.55}
.facts{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:24px}
.fact{padding:12px 13px;border-radius:12px;background:var(--s1);border:1px solid var(--line)}
.fact{display:grid;grid-template-columns:28px 1fr;column-gap:10px}.fact .ic{color:var(--up);grid-row:span 2;margin-top:1px}
.fact b{display:block;font-size:12.5px;font-weight:600;line-height:1.35}
.fact span{display:block;font-size:11.5px;color:var(--mute);margin-top:3px;line-height:1.4}
.rgroups{display:flex;flex-direction:column;gap:14px}
.rgh{display:flex;align-items:baseline;justify-content:space-between;gap:10px;margin-bottom:8px}.rgh b{font-size:13px;font-weight:600}.rgh span{font-size:12px;color:var(--mute)}
.rtiles{display:grid;grid-template-columns:1fr 1fr;gap:8px}
.rtiles button{all:unset;cursor:pointer;padding:11px 14px;border-radius:12px;background:var(--s1);border:1px solid var(--line);display:grid;grid-template-columns:auto 1fr;align-items:baseline;row-gap:3px}
.rtiles button b{font:600 16px var(--sans);letter-spacing:-.02em;color:var(--ink)}
.rtiles button .mx{font-size:12px;color:var(--mute);text-align:right}
.rtiles button .es{grid-column:1/3;font:500 11px var(--mono);color:var(--dim)}
.rtiles button .es.fe{color:var(--mute)}.rtiles button.on .es.fe{color:var(--acc2)}
.rtiles.c3{grid-template-columns:repeat(3,1fr)}.rtiles.c3 button{grid-template-columns:1fr}.rtiles.c3 button .mx{text-align:left}.rtiles.c3 button .es{grid-column:1}
.gradw{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-top:22px;align-items:start}
.lazybox{margin-top:27px;padding:12px 14px;border-radius:11px;border:1px solid var(--line);background:var(--s1);display:flex;flex-direction:column;gap:5px}
.lazybox .rl{display:flex;justify-content:space-between;font-size:12px;color:var(--mute)}.lazybox .rl .mono{font-size:11px}
.lazybox b{font-size:13px;font-weight:600;color:var(--ink);line-height:1.4}.lazybox p{margin:0;font-size:12px;color:var(--mute);line-height:1.5}
.stepnav{display:flex;justify-content:space-between;align-items:center;gap:16px;margin-top:20px;padding-top:18px;border-top:1px solid var(--line)}
.btn.a[aria-disabled=true]{opacity:.45;pointer-events:none;box-shadow:none}
.rtiles button:hover{border-color:var(--line2)}
.rtiles button.on{background:var(--s3);border-color:rgba(255,106,43,.55);box-shadow:0 6px 20px -10px rgba(255,106,43,.5)}
.rtiles button.on .mx{color:var(--acc2)}
.explain{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-top:14px}
.explain div{padding:12px 14px;border-radius:12px;border:1px solid var(--line);font-size:12.5px;color:var(--mute);line-height:1.55}
.explain b{color:var(--ink);font-weight:600;display:block;margin-bottom:2px;font-size:13px}
.sizew{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-top:22px;align-items:start}
.suffix{position:relative}.suffix em{position:absolute;right:14px;top:12px;font-style:normal;color:var(--dim);font-size:13px}
.maxline{display:flex;justify-content:space-between;align-items:center;height:44px;padding:0 14px;border-radius:11px;border:1px solid var(--line);background:rgba(255,255,255,.02);font-size:13px;color:var(--mute)}
.maxline b{color:var(--ink);font-weight:600}
.ref{margin-top:14px;padding:12px 14px;border:1px dashed rgba(232,176,75,.35);border-radius:12px;display:grid;grid-template-columns:auto 1fr;gap:14px;align-items:center}
.ref .lbl{font:500 10.5px var(--mono);letter-spacing:.06em;text-transform:uppercase;color:var(--dim);grid-column:1/-1;margin-bottom:-4px}
.ref .in{height:38px;width:130px;font-size:13px}
.toggle{display:flex;align-items:center;justify-content:space-between;padding:14px 16px;border-radius:12px;border:1px solid var(--line);background:var(--s1);margin-bottom:18px}
.toggle b{font-weight:600}.toggle span{display:block;font-size:12.5px;color:var(--mute);margin-top:2px}
.sw{width:40px;height:24px;border-radius:99px;background:var(--acc);position:relative;flex:none}.sw i{position:absolute;right:3px;top:3px;width:18px;height:18px;border-radius:50%;background:#fff}
.seg{display:flex;gap:4px;padding:4px;border-radius:12px;background:var(--s1);border:1px solid var(--line)}
.seg button{all:unset;cursor:pointer;flex:1;text-align:center;padding:9px 0;border-radius:8px;font-weight:600;font-size:13.5px;color:var(--mute)}
.seg button.on{background:var(--s3);color:var(--ink);box-shadow:0 0 0 1px rgba(255,106,43,.45) inset}
.rule{margin-top:6px;padding:16px 18px;border-radius:14px;background:var(--s1);border:1px solid var(--line)}
.rule .fx{font:500 14px var(--mono);color:var(--ink)}
.rule .fx em{font-style:normal;color:var(--acc2)}
.rule ul{margin:12px 0 0;padding:0;list-style:none;display:flex;flex-direction:column;gap:8px}
.rule li{display:flex;gap:10px;font-size:13px;color:var(--mute);line-height:1.5}.rule li svg{color:var(--up);flex:none;margin-top:1px}
.rule li b{color:var(--ink);font-weight:500}
.fn{font-size:11.5px;color:var(--dim);margin-top:14px;line-height:1.5}
.quiet{display:flex;gap:18px;align-items:center;padding:18px;border-radius:14px;border:1px dashed var(--line2)}
.quiet .ico{width:44px;height:44px;border-radius:12px;display:grid;place-items:center;background:var(--s2);border:1px solid var(--line2);color:var(--mute);flex:none}
.quiet b{font-weight:600}.quiet p{margin:3px 0 0;font-size:12.5px;color:var(--mute);line-height:1.55}
.review{display:grid;grid-template-columns:1fr 1fr;gap:0 28px;border-top:1px solid var(--line)}
.review div{display:flex;justify-content:space-between;padding:11px 0;border-bottom:1px solid var(--line);font-size:13px}
.review span{color:var(--mute)}.review b{font-weight:500;text-align:right}
.rrseg button span{display:block;font:500 10.5px var(--mono);color:var(--dim);margin-top:2px}.rrseg{display:grid!important;grid-template-columns:repeat(3,1fr)}.rrseg button{height:auto;padding:9px 12px;line-height:1.3;white-space:nowrap}
.fees{margin-top:18px;padding:14px 16px;border-radius:12px;background:var(--s1);border:1px solid var(--line)}
.fees .t{display:flex;justify-content:space-between;align-items:center;margin-bottom:6px}
.fees div.r{display:flex;justify-content:space-between;padding:6px 0;font-size:13px}.fees div.r span{color:var(--mute)}
.ack{display:flex;gap:10px;align-items:flex-start;margin-top:18px;font-size:13px;color:var(--mute);line-height:1.5}
.ack i{width:18px;height:18px;border-radius:5px;background:var(--acc);display:grid;place-items:center;flex:none;margin-top:1px}
.lbtn{display:flex;gap:12px;align-items:center;margin-top:20px}
.lbtn .btn{height:48px;padding:0 26px;font-size:15px;border-radius:12px}
.btn.a[aria-disabled=true]{opacity:.45;pointer-events:none;box-shadow:none}
.riskin{margin-top:20px;display:flex;gap:10px;padding:14px 16px;border-radius:12px;border:1px solid rgba(232,176,75,.22);background:rgba(232,176,75,.05);font-size:12.5px;color:var(--mute);line-height:1.6}
.riskin b{color:var(--ink);font-weight:600}
.cos{display:flex;gap:10px;padding:14px 16px;border-radius:12px;background:var(--accbg);border:1px solid rgba(255,106,43,.25);font-size:13px;color:#ffd3bd;line-height:1.55}
.cos b{color:#fff;font-weight:600}
.upl{display:flex;gap:14px;align-items:flex-start;padding:14px 16px;border-radius:12px;border:1px solid var(--line2);background:var(--s1)}
.upl .fi{width:40px;height:40px;border-radius:10px;display:grid;place-items:center;background:var(--s2);border:1px solid var(--line2);color:var(--mute);flex:none}
.upl b{font-weight:600}.trs{display:flex;gap:5px;flex-wrap:wrap}
.offc{padding:14px 16px;border:1px solid var(--line2);border-radius:12px;background:var(--s1);font-size:13px;line-height:1.55;color:var(--mute)}.offc b{color:var(--ink);font-weight:600}.offc ol{margin:8px 0 0;padding-left:18px}.offc li{margin:3px 0}.tiers{border:1px solid var(--line);border-radius:12px;overflow:hidden}
.tiers .th,.tiers .tr,.tiers .tf2{display:grid;grid-template-columns:1fr 120px 100px;align-items:center;padding:9px 14px;gap:12px}
.tiers .th{font:500 11px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);background:var(--s1);border-bottom:1px solid var(--line)}
.tiers .tr{border-bottom:1px solid var(--line)}.tiers .tr b,.tiers .tf2 b{text-align:right;font-weight:500}
.tiers .th span:last-child{text-align:right}
.tiers .tf2{background:var(--s1)}
.in.sm2{height:32px;font-size:13px;padding:0 10px;width:84px;border-radius:8px}
.dist{display:flex;gap:2px;height:6px;border-radius:99px;overflow:hidden}.dist i{display:block;height:100%;opacity:.8}
/* live math */
.aside{position:sticky;top:96px;display:flex;flex-direction:column;gap:16px}
.math{padding:22px}
.cl{padding:18px 20px}.cl h4{margin:0 0 10px;font-size:14px;font-weight:600;letter-spacing:-.015em}
.cl div{display:flex;align-items:center;gap:10px;padding:8px 0;border-top:1px solid var(--line);font-size:13px;color:var(--mute)}.cl div b{margin-left:auto;font-weight:500;font-size:12px}
.cl .c{width:18px;height:18px;border-radius:50%;display:grid;place-items:center;flex:none;border:1px solid var(--line2)}
.cl .c.y{background:rgba(62,207,142,.12);border-color:rgba(62,207,142,.35);color:var(--up)}
.eg2 div{display:block}.eg2 div b{margin:0;display:block;color:var(--ink);font-size:13px;font-weight:600}.eg2 div span{display:block;font-size:12.5px;line-height:1.5;margin-top:2px}
.math .k{font:500 12px var(--mono);color:var(--dim);letter-spacing:.06em;text-transform:uppercase}
.math .eq{font-size:30px;font-weight:600;letter-spacing:-.04em;margin:12px 0 2px;line-height:1.1}
.math .eq span{display:block;font-size:14px;letter-spacing:-.01em;color:var(--mute);font-weight:500;margin-top:4px}
.math .lead2{margin-top:12px!important}
.math .lead2{font-size:13.5px;color:var(--mute);line-height:1.55;margin:0}
.math .lead2 b{color:var(--ink);font-weight:600}
.alloc{display:flex;height:6px;border-radius:99px;overflow:hidden;margin:18px 0 12px;background:rgba(255,255,255,.07);gap:2px}
.alloc i{display:block;height:100%;background:linear-gradient(90deg,#c94a16,var(--acc2));border-radius:99px}
.alloc s{display:block;height:100%;flex:1;background:rgba(255,255,255,.14);border-radius:99px}
.lg{display:flex;flex-direction:column;gap:8px}
.lg div{display:flex;align-items:center;gap:10px;font-size:13px}
.lg .dot{width:8px;height:8px;border-radius:2px;flex:none}
.lg b{margin-left:auto;font-weight:600}.lg small{color:var(--dim);font-size:12px;width:40px;text-align:right}
.mrows{margin-top:18px;border-top:1px solid var(--line)}
.mrows div{display:flex;justify-content:space-between;padding:10px 0;border-bottom:1px solid var(--line);font-size:13px}
.mrows span{color:var(--mute)}.mrows b{font-weight:500}
.status{margin-top:14px;display:flex;gap:9px;align-items:flex-start;padding:11px 13px;border-radius:11px;font-size:12.5px;line-height:1.5}
.status.ok{background:rgba(62,207,142,.07);border:1px solid rgba(62,207,142,.22);color:#a8e8c9}
.status.bad{background:rgba(232,176,75,.07);border:1px solid rgba(232,176,75,.3);color:#f0c877}
.status svg{flex:none;margin-top:1px}
.est{margin-top:16px;padding:14px;border-radius:12px;background:var(--accbg);border:1px solid rgba(255,106,43,.22)}
.est .t{display:flex;justify-content:space-between;align-items:center}
.est .t span{font-size:12px;color:var(--acc2);font-weight:500}
.est .p{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-top:10px}
.est .p div span{display:block;font-size:11.5px;color:var(--mute)}.est .p div b{display:block;font-size:16px;font-weight:600;letter-spacing:-.02em;margin-top:2px}.est .p div small{font-size:11.5px;color:var(--dim)}
.est p{margin:10px 0 0;font-size:11.5px;color:var(--mute);line-height:1.5}
.forholders{margin-top:16px;font-size:12.5px;color:var(--mute);line-height:1.6}
.forholders b{color:var(--ink);font-weight:600}
"""

LAUNCH_CSS += """
.types{display:grid;grid-template-columns:repeat(3,1fr);gap:10px}
.types button{all:unset;box-sizing:border-box;cursor:pointer;display:flex;flex-direction:column;gap:6px;padding:16px;border-radius:14px;border:1px solid var(--line2);background:var(--s1)}
.types button svg{color:var(--acc2);margin-bottom:6px}.types button b{font-size:15px;font-weight:600;letter-spacing:-.01em}.types button em{font-style:normal;font-size:12px;color:var(--acc2);font-weight:500}
.types button span{font-size:12px;color:var(--mute);line-height:1.45}.types button.on{box-shadow:0 0 0 1px var(--acc) inset;background:var(--accbg);border-color:transparent}
.types2{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:10px;margin-top:10px}.types2>div>div{min-width:0;flex:1;display:block;padding:0;border:0}.types2>div>svg{flex:none}
.types2>div{display:flex;gap:12px;align-items:flex-start;padding:12px 14px;border-radius:12px;border:1px dashed var(--line2);color:var(--dim)}
.types2>div b{display:block;font-size:13.5px;color:var(--mute);font-weight:600}.types2>div span{font-size:12px;line-height:1.45}.types2 .chip{margin-left:auto;flex:none}
.rfx{display:grid;grid-template-columns:1fr 1fr;gap:10px}.rfx .fact .ic{color:var(--acc2)}
.both{margin-top:10px;padding:12px 14px;border-radius:12px;background:var(--s1);border:1px solid var(--line);font-size:12.5px;color:var(--mute);line-height:1.55}.both b{color:var(--ink);font-weight:600}
.plainbox{padding:14px 16px;border-radius:12px;background:var(--s1);border:1px solid var(--line);font-size:13px;color:var(--mute);line-height:1.6;margin-bottom:18px}.plainbox b{color:var(--ink);font-weight:600}
"""
RATIOS = [50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000]
MIN_SIZE = 100
MAX_SIZE = 10_000
RATIO_GROUPS = [("Large collections", "50K to 200K · more NFTs, each one cheaper", RATIOS[:3]), ("Small, scarce collections", "500K to 5M · fewer NFTs, each one takes more tokens", RATIOS[3:])]
LAUNCH_PX, GRAD_PX = 0.000000028, 0.00000057  # example curve prices (SOL per token)

LAUNCH_JS = """
<script>
const SUPPLY=1e9, LP=%LP%, GP=%GP%, SOLUSD=150, MIN=%MIN%, CAP=%CAP%, TMIN=10, TDEF=85;
const S={type:'%T0%',ratio:1000000,size:500,target:85};
const TN={plain:'Plain',hybrid:'Hybrid',burn:'Burn'};
function vis(){document.querySelectorAll('[data-t]').forEach(e=>{e.style.display=e.dataset.t.split(' ').includes(S.type)?'':'none'});
  document.querySelectorAll('#types button').forEach(b=>b.classList.toggle('on',b.dataset.type===S.type));
  let i=0;document.querySelectorAll('section.sec').forEach(x=>{if(x.style.display!=='none'){i++;x.querySelector('.k').textContent='Step '+i}});
  i=0;document.querySelectorAll('.rail a').forEach(x=>{if(x.style.display!=='none'){i++;const n=x.querySelector('.n');if(!n.querySelector('svg'))n.textContent=i}});}const TP=[60,25,12,3];
const f=n=>Math.round(n).toLocaleString('en-US');
const s1=n=>(+n.toFixed(2)).toLocaleString('en-US',{maximumFractionDigits:2});
const sol=n=>n>=1?n.toFixed(2):n>=0.01?n.toFixed(3):n.toPrecision(2);
const $=id=>document.getElementById(id);
function render(){
  vis(); const P=S.type==='plain', B=S.type==='burn';
  const max=Math.min(CAP,SUPPLY/S.ratio), size=S.size, low=!(size>=MIN), high=size>max, sizeOk=Number.isInteger(size)&&!low&&!high;
  const T=S.target, tLow=!(T>=TMIN), ok=(P||sizeOk)&&!tLow;
  document.querySelectorAll('#ratios button').forEach(b=>b.classList.toggle('on',+b.dataset.r===S.ratio));
  $('maxv').textContent=`${f(MIN)} to ${f(max)} NFTs`;
  const nftTok=Math.min(size*S.ratio,SUPPLY), pct=nftTok/SUPPLY*100;
  $('sizeIn').value=size?f(size):''; $('tgtIn').value=T||'';
  $('sizeIn').classList.toggle('warn',!sizeOk);
  $('sizeHelp').className='help'+(sizeOk?'':' warn');
  $('sizeHelp').textContent= sizeOk ? `${f(size)} × ${f(S.ratio)} = ${f(size*S.ratio)} tokens, within the 1,000,000,000 supply.`
    : high ? `The maximum for this ratio is ${f(max)} NFTs${max===CAP?' (collections are capped at 10,000)':''}.`
    : `Collections need at least ${f(MIN)} NFTs. Choose any size from ${f(MIN)} to ${f(max)}.`;
  // graduation target (NFTs are minted on first capture, so size does not depend on it)
  $('tgtIn').classList.toggle('warn',tLow);
  $('tgtHelp').className='help'+(tLow?' warn':'');
  $('tgtHelp').textContent=tLow?`The graduation target must be at least ${TMIN} SOL (the programs' minimum).`:`SOL raised on the bonding curve before your token graduates. Default ${TDEF} SOL; the programs accept ${TMIN} SOL or more. It must match one of the platform's approved curve settings.`;
  $('nextBtn').setAttribute('aria-disabled',ok?'false':'true');
  $('mEq').innerHTML=`${f(size)} NFTs <span>× ${f(S.ratio)} tokens each</span>`;
  $('mLead').innerHTML= sizeOk ? (B ? `If every NFT is minted, <b>${f(nftTok)}</b> tokens (<b>${+pct.toFixed(2)}%</b>) are burned for good and supply ends at <b>${f(SUPPLY-nftTok)}</b>. Each burn is one-way.`
    : `Up to <b>${f(nftTok)}</b> tokens (<b>${+pct.toFixed(2)}%</b> of the fixed supply) can be in NFT form at once. ${nftTok<SUPPLY?`At least <b>${f(SUPPLY-nftTok)}</b> always stay as tokens.`:'The whole supply could be in NFT form at once.'}`)
    : high ? `This size is above the maximum of <b>${f(max)}</b> NFTs for this ratio. Lower the size or pick a smaller ratio.`
    : `A collection needs at least <b>${f(MIN)}</b> NFTs. Raise the size to ${f(MIN)} or more.`;
  $('aBar').style.width=(sizeOk?pct:100)+'%'; $('aBar').style.background=sizeOk?'':'#e8b04b';
  $('aRest').style.display=(sizeOk&&pct<100)?'block':'none';
  $('lgN').textContent=sizeOk?f(nftTok):'—'; $('lgNp').textContent=sizeOk?(+pct.toFixed(1))+'%':'';
  $('lgT').textContent=sizeOk?f(SUPPLY-nftTok):'—'; $('lgTp').textContent=sizeOk?(+(100-pct).toFixed(1))+'%':'';
  $('mMax').textContent=`${f(MIN)} to ${f(max)} NFTs`; $('mSize').textContent=f(size||0)+' NFTs';
  $('mTgt').textContent=T>0?`${s1(T)} SOL`:'—'; $('mTgt').style.color=tLow?'#e8b04b':'';
  $('mStatus').className='status '+(ok?'ok':'bad');
  $('mStatusT').textContent= ok ? (P ? `Valid: Plain token, graduates at ${s1(T)} SOL.` : `Valid: ${f(size)} NFTs at ${f(S.ratio)} tokens each.`) : P ? `The graduation target must be at least ${TMIN} SOL.` : !sizeOk ? (high ? `Collection size is above the maximum of ${f(max)} NFTs for this ratio.` : `Collection size is below the minimum of ${f(MIN)} NFTs.`)
    : `The graduation target must be at least ${TMIN} SOL.`;
  $('ckSupply').textContent=ok?'Valid':'Fix'; $('ckSupply').style.color=ok?'var(--up)':'#e8b04b';
  $('eL').textContent=sol(S.ratio*LP)+' SOL'; $('eLu').textContent='≈ $'+(S.ratio*LP*SOLUSD).toFixed(2);
  $('eG').textContent=sol(S.ratio*GP)+' SOL'; $('eGu').textContent='≈ $'+(S.ratio*GP*SOLUSD).toFixed(2);
  $('eF').textContent=`${f(S.ratio)} × token price`;
  // review
  $('rRatio').textContent=`${f(S.ratio)} tokens = 1 NFT`; document.querySelectorAll('.rRatioB').forEach(e=>e.textContent=`${f(S.ratio)} tokens burned = 1 NFT`); $('rSize').textContent=sizeOk?`${f(size)} NFTs (range ${f(MIN)} to ${f(max)})`:'Fix collection size';
  $('rSize').style.color=sizeOk?'':'#e8b04b';
  $('rNft').textContent=sizeOk?`${f(nftTok)} (${+pct.toFixed(2)}%)`:'—'; $('rBurn').textContent=sizeOk?`${f(nftTok)} (${+pct.toFixed(2)}%), supply ≥ ${f(SUPPLY-nftTok)}`:'—';
  $('pTgt').textContent=T>0?`${s1(T)} SOL`:'—'; $('rType').textContent=TN[S.type]; $('mType').textContent=TN[S.type];
  $('rTgt').textContent=`${s1(T)} SOL raised`; $('rTgt').style.color=tLow?'#e8b04b':'';
  $('rrRatio').textContent=f(S.ratio);
  const n=size>=1?size:0; let acc=0; TP.forEach((p,i)=>{const c=i<TP.length-1?Math.floor(n*p/100):n-acc; acc+=c; $('tc'+i).textContent=f(c)}); $('tTot').textContent=f(n);
  const fee=S.ratio<=50000?0.002:S.ratio<=200000?0.005:0.01;
  document.querySelectorAll('.feeX').forEach(e=>e.textContent=fee+' SOL');
  $('mFee').textContent=B?`${fee} SOL + ≈ 0.003–0.004 SOL mint`:`${fee} SOL + 0.0063 SOL deposit (mostly refunded)`;
  $('feeCopy').textContent=B?`Platform fee: ${fee} SOL per burn, set for this collection at launch based on its ratio. %FEEREST%`:`Platform fee: ${fee} SOL per capture (tokens to NFT) or re-roll, set for this collection at launch based on its ratio. Converting back is free. %FEEREST%`;
  const dep=S.type==='hybrid'; $('launchBtn').setAttribute('aria-disabled',ok&&dep?'false':'true'); $('launchBtn').classList.toggle('off',!dep); $('launchBtn').textContent=dep?'Launch on devnet':'Pending deploy: can’t launch yet';
}
document.querySelectorAll('#ratios button').forEach(b=>b.onclick=()=>{S.ratio=+b.dataset.r;render()});
document.querySelectorAll('#types button').forEach(b=>b.onclick=()=>{S.type=b.dataset.type;render()});
$('sizeIn').onchange=e=>{S.size=+e.target.value.replace(/[^0-9]/g,'');render()};
$('tgtIn').onchange=e=>{S.target=+e.target.value.replace(/[^0-9.]/g,'');render()};
render();
</script>"""

GBOX = '''<div class="cos" style="margin-top:{mt}px"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" style="flex:none;margin-top:2px"><path d="M4 17 10 11l4 4 6-7"/><path d="M15 8h5v5"/></svg><span><b>Converting opens when your token graduates.</b> Until then, people trade the token and can preview your traits. %LAZY%<br><b>Captures and re-rolls carry a <span class="feeX">0.01 SOL</span> platform fee.</b> Converting back is free. Tokens are never taken.</span></div>'''.replace("%LAZY%", LAZY_LINE)
GBOX_B = '''<div class="cos" data-t="burn" style="margin-top:{mt}px"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" style="flex:none;margin-top:2px"><path d="M4 17 10 11l4 4 6-7"/><path d="M15 8h5v5"/></svg><span><b>Burning opens when your token graduates.</b> Each burn destroys exactly the ratio and mints the next NFT in collection order. The burner pays the mint cost.<br><b>Each burn carries a <span class="feeX">0.01 SOL</span> platform fee.</b> There's no converting back and no re-roll.</span></div>'''
GBOX_P = '''<div class="cos" data-t="plain" style="margin-top:{mt}px"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" style="flex:none;margin-top:2px"><path d="M4 17 10 11l4 4 6-7"/><path d="M15 8h5v5"/></svg><span><b>When the curve reaches your target, liquidity moves to a DEX.</b> Unsold curve tokens stay locked by the program. A Plain launch has no NFT side and no per-NFT fees.</span></div>'''
def launch(t0="hybrid"):
    ck = icon("check")
    lock = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>'
    warn = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 3 2 20h20L12 3z"/><path d="M12 10v4M12 17.5v.01"/></svg>'
    okic = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="m8 12 3 3 5-6"/></svg>'
    def rbtn(r):
        return (f'<button type="button" data-r="{r}" class="{"on" if r==1_000_000 else ""}"><b>{fmt(r)}</b>'
                f'<span class="mx">max {fmt(min(MAX_SIZE, SUPPLY//r))} NFTs</span><span class="es">≈ {r*LAUNCH_PX:.2g} SOL per NFT (est.)</span><span class="es fe">Fee {fee_for(r):g} SOL</span></button>')
    ratios = "".join(f'<div class="rg"><div class="rgh"><b>{g}</b><span>{d}</span></div><div class="rtiles{" c3" if len(rs) == 3 else ""}">{"".join(rbtn(r) for r in rs)}</div></div>' for g, d, rs in RATIO_GROUPS)
    facts = [("Supply starts at 1,000,000,000", "Mint authority revoked at launch, so nobody can mint more", ""), ("Freeze authority revoked", "Nobody can freeze holders' tokens", ""),
             ("Conversion rate set at launch", "Stored once, with no instruction to change it", "hybrid"), ("Burn rate set at launch", "Supply only goes down, one burn at a time", "burn"),
             ("Launch settings can't be edited", "The program has no instruction to change a Plain launch", "plain"), ("No transfer tax", "A classic SPL token. No tax on any transfer", "")]
    gbox = lambda mt: GBOX.replace("{mt}", str(mt)).replace('<div class="cos"', '<div class="cos" data-t="hybrid"', 1) + GBOX_B.replace("{mt}", str(mt)) + GBOX_P.replace("{mt}", str(mt))
    facts_h = "".join(f'<div class="fact"{f' data-t="{t}"' if t else ''}><div class="ic">{lock}</div><b>{a}</b><span>{b}</span></div>' for a, b, t in facts)
    types_h = "".join(f'<button type="button" data-type="{k}" class="{"on" if k == t0 else ""}">{ticon(k, 24)}<b>{n}</b><em>{sh}</em><span>{one}</span>{status_chip(k, 20)}</button>' for k, n, live, sh, one in TYPES if live)
    soon_h = "".join(f'<div>{ticon(k, 20)}<div><b>{n} <span class="chip soon" style="height:20px;margin-left:6px;vertical-align:1px">Coming soon</span></b><span>{one}</span></div></div>' for k, n, live, sh, one in TYPES if not live)
    D = lambda t: f' data-t="{t}"'
    HB, BU, PL, NB = D("hybrid"), D("burn"), D("plain"), D("hybrid burn")
    body = f"""<div class="wrap">
<div class="lhead"><div><span class="pill"><i></i>Unaudited beta · Devnet · test tokens only</span><h1>Launch a token</h1>
<p>Pick a launch type, then set it up. Plain is just the coin; Hybrid and Burn add an NFT collection. Every launch starts on a bonding curve (a formula that raises the price as people buy) and graduates to a DEX pool once it raises its SOL target. Take your time: the type and anything marked with a lock are permanent once you launch.</p></div>
<div style="display:flex;gap:10px"><a class="btn g" href="#">Save draft</a><a class="btn g" href="#">Launch guide</a></div></div>

<div class="lgrid">
<nav class="rail"><a class="done" href="#type"><span class="n">{ck.replace('18','13')}</span>Launch type</a><a class="done" href="#basics"><span class="n">{ck.replace('18','13')}</span>Basics</a><a class="on" href="#supply"><span class="n">3</span><span{NB}>Supply &amp; conversion</span><span{PL}>Token &amp; curve</span></a>
<a href="#traits"{NB}><span class="n">4</span>Art commitment</a><a href="#review"><span class="n">5</span>Review &amp; launch</a>
<div class="hint">Nothing is created until you press Launch and approve it in your wallet. Drafts are only stored on this device.</div></nav>

<div class="form">
<section class="card sec" id="type"><div class="sh"><div><div class="k">Step 1</div><h2>Launch type</h2><p>The type decides what holders can do with your token. It's fixed on-chain once you launch.</p></div><span class="chip acc">Permanent</span></div>
<div class="types" id="types" role="group" aria-label="Launch type">{types_h}</div>
<div class="types2">{soon_h}</div>
<div class="help" style="margin-top:10px">Tax split and Raffle aren't available yet. They're listed so you know what's planned.</div></section>

<section class="card sec" id="basics"><div class="sh"><div><div class="k">Step 2</div><h2><span{NB}>Collection basics</span><span{PL}>Token basics</span></h2><p>How your <span{NB}>collection</span><span{PL}>token</span> appears across {BRAND} and in wallets.</p></div></div>
<div class="row2"><div class="f"><label>Name <span>9 / 32</span></label><input class="in" value="Low Orbit"></div>
<div class="f"><label>Ticker <span>3 to 6 letters</span></label><div class="pre"><b>$</b><input class="in" value="ORBIT"></div></div></div>
<div class="f" style="margin-bottom:0"><label for="tokUri">Token metadata URI <span>Hosted off-chain</span></label><input class="in" id="tokUri" value="ipfs://bafy…orbit/token.json"><div class="help">The token's image and description live in a metadata file you host yourself (IPFS or Arweave). It's set when the curve pool is created and can't be edited afterwards. {BRAND} has no upload step and doesn't host files. <span class="demo" style="margin-left:4px">Example</span></div></div></section>

<section class="card sec" id="supply"><div class="sh"><div><div class="k">Step 3</div><h2><span{HB}>Supply and conversion</span><span{BU}>Supply and burn rate</span><span{PL}>Token and curve</span></h2><p><span{HB}>Choose how many tokens make one NFT, then how many NFTs the collection has.</span><span{BU}>Choose how many tokens are burned to mint one NFT, then how many NFTs the collection has.</span><span{PL}>A Plain launch only needs a graduation target. Everything else is fixed.</span></p></div><span class="chip acc">Permanent</span></div>
<div class="facts">{facts_h}</div>
{gbox(0)}
<div style="height:22px"></div>
<div class="plainbox"{PL}><b>A Plain launch is just the token:</b> 1,000,000,000 SPL tokens on a bonding curve, mint and freeze authority revoked, no NFT collection, no conversion and no per-NFT fees. It can't add a collection later.</div>
<div{NB}>
<div class="f" style="margin-bottom:0"><label><span{HB} style="color:var(--ink);font-weight:500">Tokens per NFT</span><span{BU} style="color:var(--ink);font-weight:500">Tokens burned per NFT</span> <span>Each option shows its max collection size</span></label><div class="rgroups" id="ratios" role="group" aria-label="Tokens per NFT">{ratios}</div>
<div class="help">Platform fee per <span{HB}>capture or re-roll</span><span{BU}>burn</span> is set by ratio and fixed at launch: 0.002 SOL (50K), 0.005 SOL (100K, 200K), 0.01 SOL (500K and up). Per-NFT prices are estimates: tokens per NFT × the example launch price of {full_price(LAUNCH_PX)} SOL per token. Real prices move with trading.</div></div>
<div class="explain"><div><b>Smaller ratio</b>More, cheaper NFTs. Holders reach a whole NFT sooner.</div><div><b>Larger ratio</b>Fewer, scarcer NFTs. Each one takes more tokens.</div></div>
<div class="sizew"><div class="f" style="margin:0"><label>Collection size <span>Minimum 100 NFTs</span></label><div class="suffix"><input class="in" id="sizeIn" value="500" inputmode="numeric"><em>NFTs</em></div>
<div class="help" id="sizeHelp">1,000 × 1,000,000 = 1,000,000,000 tokens, within the 1,000,000,000 supply.</div></div>
<div class="f" style="margin:0"><label>Allowed size for this ratio</label><div class="maxline"><span>Any size from</span><b id="maxv">100 to 1,000 NFTs</b></div>
<div class="help">At least 100 NFTs, at most 10,000, and size × tokens per NFT can't exceed the supply.</div></div></div>
</div>
<div class="gradw"><div class="f" style="margin:0"><label>Graduation target <span>Default 85 SOL · minimum 10 SOL</span></label><div class="suffix"><input class="in" id="tgtIn" value="85" inputmode="decimal"><em>SOL</em></div>
<div class="help" id="tgtHelp">SOL raised on the bonding curve before your token graduates. Default 85 SOL; the programs accept 10 SOL or more. It must match one of the platform's approved curve settings.</div></div>
<div class="lazybox"{HB}><div class="rl"><span>Mint deposit per capture</span><span class="mono dim">paid by the collector</span></div><b>0.0063 SOL, mostly refunded</b><p>Each capture or re-roll puts up a 0.0063 SOL deposit. About 0.003–0.004 SOL is kept only if the pick is minted for the first time (Solana rent + Metaplex fee); the rest is refunded. Nothing comes out of your graduation proceeds.</p></div>
<div class="lazybox"{BU}><div class="rl"><span>Mint cost per NFT</span><span class="mono dim">paid by the burner</span></div><b>About 0.003–0.004 SOL, every burn</b><p>Every burn mints a new NFT, so the burner pays Solana rent and the Metaplex Core fee directly. No deposit. Nothing comes out of your proceeds.</p></div>
<div class="lazybox"{PL}><div class="rl"><span>After graduation</span><span class="mono dim">automatic</span></div><b>Liquidity moves to a DEX</b><p>The SOL raised and the tokens set aside for the pool go into a public DEX pool. Unsold curve tokens stay locked.</p></div></div>
<div class="stepnav"><span class="dim" style="font-size:12.5px"><span{NB}>Any size from 100 up to the maximum for your ratio works at any graduation target.</span><span{PL}>The graduation target is the only setting.</span></span><a class="btn a" id="nextBtn" href="#traits"><span{NB}>Next: art commitment</span><span{PL}>Next: review</span></a></div>
</section>

<section class="card sec" id="traits"{NB}><div class="sh"><div><div class="k">Step 4</div><h2>Art commitment and randomness</h2><p>Your art is prepared off-chain. Here you enter its addresses and fingerprints, which are locked in for good. Rarity is cosmetic.</p></div><span class="chip acc">Permanent</span></div>
<div class="cos"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" style="flex:none;margin-top:2px"><circle cx="12" cy="12" r="9"/><path d="M12 8v5M12 16.5v.01"/></svg><span><span{HB}>Every NFT in this collection converts back for exactly <b><span id="rrRatio">100,000</span> ORBIT</b>, whatever its traits.</span><span{BU}>Every NFT costs the same to mint, whatever its traits, and none converts back to tokens.</span> Any premium for rare traits is set by buyers and sellers, not guaranteed.</span></div>
<div class="offc"><b>Prepare the art off-chain first. {BRAND} has no upload step.</b><ol><li>Upload each NFT's image and metadata JSON to IPFS or Arweave.</li><li>Build a Merkle tree with one leaf per NFT: its 8 trait values, a salt, the SHA-256 of its image and JSON, and its metadata URI.</li><li>Enter the results below. Keep the full leaf list available: each capture is settled with that NFT's leaf and proof.</li></ol></div>
<div class="row2" style="margin-top:16px"><div class="f"><label for="colName">Collection name <span>Up to 32 characters</span></label><input class="in" id="colName" value="Low Orbit"></div>
<div class="f"><label for="colUri">Collection metadata URI <span>ipfs:// or ar:// only</span></label><input class="in" id="colUri" value="ar://Qm7x…orbit-collection"></div></div>
<div class="row2"><div class="f"><label for="trRoot">Trait root <span>Merkle root, 32 bytes</span></label><input class="in mono" id="trRoot" value="0x7f3a…c91e"></div>
<div class="f"><label for="trSchema">Trait schema hash <span>32 bytes</span></label><input class="in mono" id="trSchema" value="0x1b44…08d2"></div></div>
<div class="help" style="margin:-4px 0 16px">These four values are committed when the collection vault is created, before converting opens. No instruction can change them afterwards. <span class="demo" style="margin-left:4px">Example values</span></div>
<div class="f"><label>How NFTs are assigned <span>Fixed for this type, not a setting</span></label>
<div class="rfx"><div class="fact"><div class="ic">{lock}</div><b>Art and traits committed before converting opens</b><span>A fingerprint (Merkle root) of every NFT's art and traits is published when the collection vault is created. Nobody can swap or edit a piece afterwards.</span></div>
<div class="fact"{HB}><div class="ic">{lock}</div><b>Switchboard VRF picks which NFT you get</b><span>VRF is a verifiable random function: a random number that comes with a proof anyone can check. Used at every capture and re-roll; nobody, including you, can choose a piece or see the next one.</span></div>
<div class="fact"{BU}><div class="ic">{lock}</div><b>NFTs go out in collection order</b><span>#0, #1, #2 and so on, one per burn. No randomness, so the next piece is always public.</span></div></div>
<div class="both"{HB}><b>Hybrid uses both, always.</b> The commitment fixes what's in the collection; VRF decides who gets which piece. There's nothing to choose here.</div>
<div class="both"{BU}><b>The order is public.</b> Anyone can see which NFT comes next, including rare ones, and time a burn for it. If that matters to you, spread rare pieces through your leaf order.</div></div>
<div class="tiers" hidden><div class="th"><span>Tier</span><span>Share</span><span>NFTs</span></div>
<div class="tr"><span>{tier("Common")}</span><span><input class="in sm2" value="60%" readonly aria-label="Common share"></span><b class="num" id="tc0">3,000</b></div><div class="tr"><span>{tier("Uncommon")}</span><span><input class="in sm2" value="25%" readonly aria-label="Uncommon share"></span><b class="num" id="tc1">1,250</b></div><div class="tr"><span>{tier("Rare")}</span><span><input class="in sm2" value="12%" readonly aria-label="Rare share"></span><b class="num" id="tc2">600</b></div><div class="tr"><span>{tier("Legendary")}</span><span><input class="in sm2" value="3%" readonly aria-label="Legendary share"></span><b class="num" id="tc3">150</b></div>
<div class="tf2"><span class="help">Total</span><b>100%</b><b id="tTot">5,000</b></div></div>
<div class="help" style="margin-top:8px">Rarity comes from the trait values in your leaves. Tiers are labels for how often traits appear; they don't change conversion or fees.</div>
<div class="both"{HB} style="margin-top:18px"><b>Re-rolls are built in. There's nothing to set.</b> Any holder can swap an NFT for a different random one. A re-roll costs this collection's platform fee (<span class="feeX">0.01 SOL</span>, the same as a capture) plus the refundable 0.0063 SOL mint deposit, and the result may be more common than the NFT given up.</div>
<div class="both"{BU} style="margin-top:18px"><b>No re-rolls, no converting back.</b> Burn NFTs are final once minted.</div></section>

<section class="card sec" id="review"><div class="sh"><div><div class="k">Step 5</div><h2>Review and launch</h2><p>Check everything once more. Locked settings can't be changed after launch, by you or by us.</p></div><span class="demo">Devnet</span></div>
<div class="review"><div><span>Launch type</span><b id="rType">Hybrid</b></div><div><span>Token</span><b>Low Orbit · $ORBIT</b></div>
<div><span>Supply at launch</span><b>1,000,000,000</b></div><div><span>Graduation target</span><b id="rTgt">85 SOL raised</b></div>
<div><span>Bonding curve</span><b>Platform-approved curve settings</b></div><div><span>Decimals</span><b>6</b></div><div><span>Unsold curve tokens</span><b>Locked by the program</b></div>
<div><span>Mint authority</span><b style="color:var(--up)">Revoked at launch</b></div><div><span>Freeze authority</span><b style="color:var(--up)">Revoked at launch</b></div>
<div><span>Tax</span><b>No transfer tax</b></div><div{PL}><span>NFT collection</span><b>None</b></div>
<div{HB}><span>Conversion rate</span><b id="rRatio">1,000,000 tokens = 1 NFT</b></div><div{BU}><span>Burn rate</span><b class="rRatioB">1,000,000 tokens = 1 NFT</b></div>
<div{NB}><span>Collection size</span><b id="rSize">1,000 NFTs (range 100 to 1,000)</b></div><div{HB}><span>Max in NFT form</span><b id="rNft">1,000,000,000 (100%)</b></div>
<div{BU}><span>Most that can be burned</span><b id="rBurn">—</b></div><div{NB}><span>NFT minting</span><b><span{HB}>On first capture</span><span{BU}>On each burn</span></b></div>
<div{NB}><span>Art commitment</span><b>Trait root 0x7f3a…c91e · ar://… (Example)</b></div><div{HB}><span>Randomness</span><b>Committed art + Switchboard VRF</b></div><div{BU}><span>NFT order</span><b>Collection order, public</b></div>
<div{NB}><span>Platform fee</span><b><span class="feeX">0.01 SOL</span> per <span{HB}>capture or re-roll</span><span{BU}>burn</span></b></div><div{HB}><span>Converting back</span><b>No platform fee</b></div>
<div{BU}><span>Converting back</span><b>Not possible, one-way</b></div><div{NB}><span><span{HB}>Converting</span><span{BU}>Burning</span> opens</span><b>At graduation</b></div>
<div><span>Program upgrades</span><b>{UPG_SHORT}</b></div><div><span>Audit</span><b style="color:#e8b04b">Not yet · unaudited beta</b></div></div>
{gbox(18)}
<div class="fees"><div class="t"><b style="font-weight:600">Fees</b><span class="demo">Curve fee: devnet setting, final pending</span></div>
<div class="r"><span>Launch fee</span><b>None on chain today</b></div><div class="r"><span>Trade fee on the bonding curve</span><b>1%</b></div><div class="r"><span>Tax</span><b>No transfer tax</b></div>
<div class="r"{PL}><span>Per-NFT fees</span><b>None, no NFTs</b></div>
<div class="r"{HB}><span>Platform fee, per capture or re-roll</span><b class="feeX">0.01 SOL</b></div><div class="r"{HB}><span>Mint deposit, per capture or re-roll</span><b>0.0063 SOL, refunded except ≈ 0.003–0.004 SOL if the NFT is minted new</b></div><div class="r"{HB}><span>Converting back (NFT to tokens)</span><b>No platform fee</b></div><div class="r"{HB}><span>Tokens taken on any convert or re-roll</span><b>None</b></div>
<div class="r"{BU}><span>Platform fee, per burn</span><b class="feeX">0.01 SOL</b></div><div class="r"{BU}><span>Mint cost, every burn</span><b>≈ 0.003–0.004 SOL, paid by the burner, no deposit</b></div><div class="r"{BU}><span>Tokens per burn</span><b>The ratio, destroyed</b></div>
<div{NB}><p class="fn" style="margin:10px 0 0" id="feeCopy">{fee_copy("0.01 SOL")}</p><p class="fn" style="margin:6px 0 0">{MINT_DISC}</p><p class="fn" style="margin:6px 0 0">{FEE_ADDR_H}</p></div></div>
<div class="riskin"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#e8b04b" stroke-width="1.8" style="flex:none;margin-top:2px"><path d="M12 3 2 20h20L12 3z"/><path d="M12 10v4M12 17.5v.01"/></svg><span><b>Risk disclosure.</b> {RISK_H}</span></div>
<div style="margin-top:16px">{cap_slot('<span class="dim">Per-wallet limit while the programs are unaudited</span>')}</div>
<div class="ack"><i><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="3"><path d="m5 12 4.5 4.5L19 7"/></svg></i><span>I understand that the launch type<span{NB}>, collection size and committed art</span> can't be changed after launch, that {BRAND} is an unaudited beta, and that the programs can still be upgraded until they're frozen after the audit (see Trust &amp; security).</span></div>
<div class="lbtn"><a class="btn a" id="launchBtn" href="#">Launch on devnet</a><a class="btn g" style="height:48px;border-radius:12px" href="#basics">Back to edit</a><span class="dim" style="font-size:12.5px">Test network only. No real funds.</span></div></section>
</div>

<aside class="aside"><div class="card math"><div style="display:flex;justify-content:space-between;align-items:center"><span class="k">Live math · <span id="mType">Hybrid</span></span><span class="chip">Updates as you edit</span></div>
<div{PL}><div class="eq">1,000,000,000 <span>tokens, fixed</span></div>
<p class="lead2">No NFT side. Tokens trade on the curve until it reaches <b id="pTgt">85 SOL</b>, then move to a DEX pool.</p>
<div class="alloc"><i style="width:55%"></i></div>
<div class="lg"><div><span class="dot" style="background:var(--acc)"></span><span class="mute">Sold on the curve</span><b>550,000,000</b><small>55%</small></div>
<div><span class="dot" style="background:rgba(255,255,255,.25)"></span><span class="mute">Set aside for the DEX pool</span><b>200,000,000</b><small>20%</small></div>
<div><span class="dot" style="background:rgba(255,255,255,.1)"></span><span class="mute">Unsold buffer, locked</span><b>250,000,000</b><small>25%</small></div></div>
<p class="help" style="margin-top:8px">Split from the platform's devnet curve settings; the approved curve settings at launch decide it.</p></div>
<div{NB}><div class="eq" id="mEq">1,000 NFTs <span>× 1,000,000 tokens each</span></div>
<p class="lead2" id="mLead">Up to <b>500,000,000</b> tokens (<b>50%</b> of the fixed supply) can be in NFT form at once. At least <b>500,000,000</b> always stay as tokens.</p>
<div class="alloc"><i id="aBar" style="width:50%"></i><s id="aRest"></s></div>
<div class="lg"><div><span class="dot" style="background:var(--acc)"></span><span class="mute"><span{HB}>Can become NFTs</span><span{BU}>Can be burned</span></span><b id="lgN">500,000,000</b><small id="lgNp">50%</small></div>
<div><span class="dot" style="background:rgba(255,255,255,.25)"></span><span class="mute"><span{HB}>Always tokens</span><span{BU}>Never burned</span></span><b id="lgT">500,000,000</b><small id="lgTp">50%</small></div></div></div>
<div class="mrows"><div><span>Supply at launch</span><b>1,000,000,000</b></div><div{NB}><span>Allowed size at this ratio</span><b id="mMax">100 to 1,000 NFTs</b></div><div{NB}><span>Your collection size</span><b id="mSize">1,000 NFTs</b></div><div><span>Graduation target</span><b id="mTgt">85 SOL</b></div><div{NB}><span>Holder pays per NFT</span><b id="mFee">0.01 SOL</b></div><div{PL}><span>Per-NFT fees</span><b>None</b></div></div>
<div class="status ok" id="mStatus">{okic}<span id="mStatusT">Valid: 1,000 NFTs at 1,000,000 tokens each.</span></div>
<div class="est"{NB}><div class="t"><span><span{HB}>Price of 1 NFT (estimate)</span><span{BU}>Tokens burned per NFT, worth (estimate)</span></span><span class="mono" id="eF" style="color:var(--mute);font-size:11.5px">1,000,000 × token price</span></div>
<div class="p"><div><span>At launch price</span><b id="eL">0.0028 SOL</b><small id="eLu">≈ $0.42</small></div><div><span>At graduation price</span><b id="eG">0.057 SOL</b><small id="eGu">≈ $8.55</small></div></div>
<p>Estimate only: tokens per NFT × current curve price, using example curve prices ({full_price(LAUNCH_PX)} and ≈{full_price(GRAD_PX)} SOL per token) at $150/SOL. Real prices move with trading.</p></div>
<div class="forholders"{NB}><b>What this means for holders:</b> with a smaller ratio, holders reach a whole NFT sooner and there are more NFTs to go around. With a larger ratio, each NFT needs more tokens, so NFTs are fewer and scarcer.</div>
</div>
<div class="card cl"><h4>Launch checklist</h4>
<div><span class="c y">{ck.replace('18','11')}</span>Launch type<b style="color:var(--up)">Chosen</b></div><div><span class="c y">{ck.replace('18','11')}</span>Basics<b style="color:var(--up)">Done</b></div>
<div><span class="c y">{ck.replace('18','11')}</span>Supply and settings<b style="color:var(--up)" id="ckSupply">Valid</b></div>
<div{NB}><span class="c y">{ck.replace('18','11')}</span>Art commitment<b style="color:var(--up)">Set</b></div>
<div><span class="c"></span>Review and launch<b class="dim">Not started</b></div></div>
<div class="card cl eg2"{PL}><h4>Is Plain right for you?</h4><div><b>Choose Plain</b><span>if you want a memecoin and nothing else. Simplest to explain, no NFT costs for holders.</span></div><div><b>Choose Hybrid</b><span>if holders should be able to collect art and change their mind later.</span></div><div><b>Choose Burn</b><span>if collecting should be permanent and shrink the supply.</span></div></div>
<div class="card cl eg2"{NB}><h4>Choosing a ratio</h4>
<div><b>50,000 to 200,000 tokens</b><span>Large collections: up to 10,000 NFTs (5,000 at 200,000). More, cheaper NFTs.</span></div>
<div><b>500,000 or 1,000,000 tokens</b><span>Smaller collections: up to 2,000 or 1,000 NFTs.</span></div>
<div><b>2,500,000 or 5,000,000 tokens</b><span>Small, scarce collections: 100 to 400, or 100 to 200 NFTs. Each NFT takes more tokens.</span></div>
<div><b>Any ratio</b><span>100 to 10,000 NFTs, and never more than the supply allows. NFTs are minted as they're captured, so size doesn't depend on the graduation target.</span></div></div>
</aside>
</div></div>""" + LAUNCH_JS.replace("%MIN%", str(MIN_SIZE)).replace("%CAP%", str(MAX_SIZE)).replace("%LP%", repr(LAUNCH_PX)).replace("%GP%", repr(GRAD_PX)).replace("%FEEREST%", FEE_REST).replace("%T0%", t0)
    return page(f"Launch a token · {BRAND}", LAUNCH_CSS + ".cl h3{margin:0 0 10px;font-size:14px;font-weight:600;letter-spacing:-.015em}", body.replace("<h4>", "<h3>").replace("</h4>", "</h3>"), active="launch", desc="Launch a Solana memecoin as Plain, Hybrid (coin and NFT, both ways) or Burn (burn to mint). See the math for each type before you sign. Unaudited beta on devnet.")

open(os.path.join(OUT, "launch.html"), "w").write(launch())
open(os.path.join(OUT, "launch-plain.html"), "w").write(launch("plain"))
open(os.path.join(OUT, "launch-burn.html"), "w").write(launch("burn"))
print("launch ok")

# ------------------------------------------------------------------ EXPLORE
PAGE_CSS = """
.phd{padding:44px 0 28px;display:flex;justify-content:space-between;align-items:flex-end;gap:32px}
.phd h1{font-family:var(--hd);font-weight:800;font-stretch:82%;font-size:44px;letter-spacing:-.03em;margin:0 0 10px;line-height:1.05}.phd p{margin:0;color:var(--mute);font-size:16px;max-width:620px;line-height:1.6}
.fbar{display:flex;align-items:center;gap:14px;flex-wrap:wrap;padding:14px 0 22px;border-top:1px solid var(--line)}
.fbar .lb{font:500 11px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);margin-right:2px}
.fg{display:flex;gap:4px;padding:3px;border:1px solid var(--line);border-radius:11px;background:var(--s1)}
.fg a{display:inline-flex;align-items:center;gap:6px;padding:6px 11px;border-radius:8px;font-size:13px;font-weight:500;color:var(--mute)}.fg a svg{width:15px;height:15px}
.fg a.on{background:var(--s3);color:var(--ink);box-shadow:0 0 0 1px rgba(255,106,43,.4) inset}.fg a.on svg{color:var(--acc2)}
.fg a.so{color:var(--dim);cursor:default}.fg a.so i{font:500 9.5px/1 var(--mono);text-transform:uppercase;letter-spacing:.05em;border:1px dashed var(--line2);border-radius:5px;padding:3px 5px;font-style:normal}
.srch{display:flex;align-items:center;gap:8px;height:40px;padding:0 14px;border-radius:10px;border:1px solid var(--line2);background:var(--s1);color:var(--dim);font-size:13.5px;min-width:280px}
.kst{display:grid;grid-template-columns:repeat(4,1fr);margin-bottom:24px}.kst div{padding:16px 20px;border-right:1px solid var(--line)}.kst div:last-child{border-right:0}
.kst span{display:block;font-size:12px;color:var(--mute);margin-bottom:4px}.kst b{font-size:20px;font-weight:600;letter-spacing:-.02em}
.soonrow{display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:24px}
.soonc{display:flex;gap:14px;align-items:flex-start;padding:18px 20px;border-radius:16px;border:1px dashed var(--line2);color:var(--dim)}
.soonc b{display:block;color:var(--mute);font-size:15px;font-weight:600;margin-bottom:4px}.soonc p{margin:0;font-size:13px;line-height:1.55}.soonc .chip{margin-left:auto;flex:none}
"""
def explore():
    tabs = '<a class="on">All types</a>' + "".join(f'<a class="{"" if live else "so"}">{ticon(k, 15)}{n}{"" if STATUS[k] == "live" else ("<i>Pending</i>" if STATUS[k] == "pending" else "<i>Soon</i>")}</a>' for k, n, live, _, _ in TYPES)
    cards = "".join(launch_card(L, k) for k, L in enumerate(A_LAUNCHES))
    soon = "".join(f'<div class="soonc">{ticon(k, 22)}<div><b>{n}</b><p>{one} Launches of this type will show here, marked with their type.</p></div><span class="chip soon">Coming soon</span></div>' for k, n, live, _, one in TYPES if not live)
    body = f"""<div class="wrap">
<div class="phd"><div><h1>Explore launches</h1><p>Every token launched on {BRAND}, with its type, phase and market. Plain tokens have no NFTs; Hybrid and Burn tokens open their collection at graduation (Plain and Burn are pending deploy, so their cards are design previews), when the bonding curve (the formula that sets the launch price) reaches its SOL target and trading moves to a DEX.</p></div>
<div class="srch"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>Search by name, ticker or address</div></div>
<div class="card kst"><div><span>Launches</span><b class="num">128</b></div><div><span>On the curve</span><b class="num">97</b></div><div><span>Graduated</span><b class="num">31</b></div><div><span>24h volume</span><b class="num">2,412 SOL</b> <span class="demo" style="display:inline-block;margin-left:6px">Demo data</span></div></div>
<div class="fbar"><span class="lb">Type</span><div class="fg">{tabs}</div></div>
<div class="fbar" style="border-top:0;padding-top:0"><span class="lb">Phase</span><div class="fg"><a class="on">All</a><a>On curve</a><a>Near graduation</a><a>Graduated</a></div>
<span class="lb" style="margin-left:auto">Sort</span><div class="fg"><a class="on">Trending</a><a>Newest</a><a>Market cap</a><a>Volume</a></div></div>
<div class="grid">{cards}</div>
<div class="soonrow">{soon}</div>
<p class="dim" style="font-size:12.5px;margin-top:18px">Figures are demo data. Rankings never imply quality or safety; every token can lose all its value.</p>
</div>"""
    return page(f"Explore launches · {BRAND}", HOME_CSS + PAGE_CSS, body, active="explore", desc=f"Browse Plain, Hybrid and Burn launches on {BRAND}, filtered by type and phase. Unaudited beta on devnet.")

open(os.path.join(OUT, "explore.html"), "w").write(explore())

# ------------------------------------------------------------------ PORTFOLIO (summary, no tile grids)
PORT_CSS = """
.psum{display:grid;grid-template-columns:1.2fr 1fr;gap:0}.psum>div{padding:24px 26px}.psum>div+div{border-left:1px solid var(--line)}
.pv2{font-size:40px;font-weight:600;letter-spacing:-.04em;margin:6px 0 4px}.k2{font:500 11px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim)}
.mix{display:flex;height:8px;border-radius:99px;overflow:hidden;gap:2px;margin:18px 0 12px}.mix i{display:block;height:100%}
.mixl{display:flex;flex-direction:column;gap:8px}.mixl div{display:grid;grid-template-columns:10px 1fr auto auto;gap:10px;align-items:center;font-size:13px}.mixl i{width:8px;height:8px;border-radius:2px}.mixl span{color:var(--mute)}.mixl small{color:var(--dim);font-size:12px;min-width:44px;text-align:right}
.kv2{display:flex;flex-direction:column}.kv2 div{display:flex;justify-content:space-between;padding:11px 0;border-bottom:1px solid var(--line);font-size:13px}.kv2 div:last-child{border-bottom:0}.kv2 span{color:var(--mute)}.kv2 b{font-weight:500}
table.pt{width:100%;border-collapse:collapse;font-size:13px}.pt th{text-align:left;font-weight:500;color:var(--dim);font-size:12px;padding:0 12px 10px;border-bottom:1px solid var(--line)}
.pt td{padding:14px 12px;border-bottom:1px solid var(--line);vertical-align:middle}.pt tr:last-child td{border-bottom:0}.pt .r{text-align:right}
.asst{display:flex;align-items:center;gap:10px}.asst .ic{width:32px;height:32px;border-radius:9px;overflow:hidden;flex:none}.asst .ic svg{width:100%;height:100%;display:block}.asst b{font-weight:600;display:block}.asst small{color:var(--dim);font:500 11.5px var(--mono)}
.acts{display:flex;gap:6px;justify-content:flex-end}
.nl{display:flex;flex-direction:column}.nl>div{display:flex;align-items:center;gap:12px;padding:12px 0;border-bottom:1px solid var(--line);font-size:13px}.nl>div:last-child{border-bottom:0}
.nl .stk{display:flex}.nl .stk span{width:28px;height:28px;border-radius:7px;overflow:hidden;border:2px solid var(--s1);margin-left:-8px}.nl .stk span:first-child{margin-left:0}.nl .stk svg{width:100%;height:100%;display:block}
.nl b{font-weight:600}.nl small{display:block;color:var(--mute);font-size:12px;margin-top:2px}.nl .rt{margin-left:auto;text-align:right}
"""
def portfolio():
    Lk = {L["key"]: L for L in A_LAUNCHES}
    noct, salt, ferro, glass = Lk["noct"], Lk["salt"], Lk["ferro"], Lk["glass"]
    pos = [  # key, tokens, nfts
        ("noct", 5_750_000, 2), ("glass", 600_000, 1), ("salt", 2_400_000, 0), ("ferro", 4_200_000, 1)]
    tok_v = sum(t * Lk[k]["price"] for k, t, n in pos)
    back_v = sum(n * Lk[k]["ratio"] * Lk[k]["price"] for k, t, n in pos if Lk[k]["type"] == "hybrid")
    tot = tok_v + back_v
    rows = ""
    for k, t, n in pos:
        L = Lk[k]; ty = L["type"]; tv = t * L["price"]; nv = n * L["ratio"] * L["price"] if ty == "hybrid" else 0
        ph = '<span class="chip ok2"><i></i>Graduated</span>' if L["prog"] >= 100 else f'<span class="chip acc"><i></i>Curve · {L["prog"]}%</span>'
        nft_c = "—" if ty == "plain" else (f'{n} <span class="dim">· not valued</span>' if ty == "burn" else f'{n} <span class="dim">· {nv:.3f} SOL</span>')
        href = TOK_HREF.get(k, "token.html")
        act = {"plain": f'<a class="btn g sm" href="{href}">Trade</a>', "hybrid": f'<a class="btn g sm" href="{href}">Trade</a><a class="btn g sm" href="token-capture.html">Convert</a>', "burn": f'<a class="btn g sm" href="{href}">Trade</a><a class="btn g sm" href="{href}">Burn</a>'}[ty]
        rows += f'<tr><td><div class="asst"><span class="ic">{art(k, "pp"+k, 7)}</span><div><b>{L["name"]}</b><small>{L["sym"]}</small></div></div></td><td>{tchip(ty)}</td><td>{ph}</td><td class="r num">{fmt(t)}</td><td class="r num">{nft_c}</td><td class="r num"><b style="font-weight:600">{tv+nv:.3f} SOL</b></td><td class="r num {"up" if L["ch"]>=0 else "down"}">{L["ch"]:+.1f}%</td><td><div class="acts">{act}</div></td></tr>'
    mix = [("Tokens", "#ff9a62", tok_v), ("Hybrid NFTs, at their release value", "#a697ff", back_v)]
    mix_bar = "".join(f'<i style="flex:{v:.4f};background:{c}"></i>' for _, c, v in mix)
    mix_l = "".join(f'<div><i style="background:{c}"></i><span>{n}</span><b class="num">{v:.3f} SOL</b><small class="num">{v/tot*100:.0f}%</small></div>' for n, c, v in mix)
    mix_l += '<div><i style="background:rgba(255,255,255,.18)"></i><span>Burn NFTs, no token backing</span><b>Not valued</b><small>1 NFT</small></div>'
    nl = [("noct", [142, 655], "Nocturnes", "2 NFTs · Common, Rare", "Each releases for 1,000,000 NOCT", f"{2*noct['ratio']*noct['price']:.3f} SOL", "token-release.html", "Release"),
          ("glass", [214], "Glasshouse", "1 NFT · Rare", "Releases for 200,000 GLSS", f"{glass['ratio']*glass['price']:.3f} SOL", "token.html", "Release"),
          ("ferro", [201], "Ferro", "1 NFT · Common", "Burned to mint. Can't be converted back", "Not valued", "token-burn.html", "View")]
    nl_h = "".join(f'<div><span class="stk">{"".join(f"<span>{art(k, f'pn{x}', x)}</span>" for x in xs)}</span><div><b>{n}</b><small>{d}</small></div><div class="rt"><b class="num">{v}</b><small>{note}</small></div><a class="btn gh sm" href="{h}">{a}</a></div>' for k, xs, n, d, note, v, h, a in nl)
    hist = [("Captured", "Nocturnes #0655", "0.01 SOL fee · 0.0031 SOL mint kept from the 0.0063 deposit", "Sep 26"), ("Burned to mint", "Ferro #0201 · 1,000,000 FERRO burned", "0.01 SOL fee · 0.0031 SOL mint", "Sep 25"),
            ("Released", "Nocturnes #0588 → 1,000,000 NOCT", "Free", "Sep 25"), ("Re-rolled", "Nocturnes #0021 → #0142", "0.01 SOL fee · deposit refunded in full (already minted)", "Sep 24"), ("Bought", "2,400,000 SALT", "1.236 SOL · 1% trade fee", "Sep 23")]
    hist_h = "".join(f'<tr><td><b style="font-weight:600">{a}</b></td><td>{b}</td><td class="mute">{c}</td><td class="r mute num" style="white-space:nowrap">{d}</td></tr>' for a, b, c, d in hist)
    body = f"""<div class="wrap">
<div class="phd"><div><h1>Portfolio</h1><p>Everything in this wallet across {BRAND}: tokens, NFTs you can release, and what you've paid in fees.</p></div>
<div style="display:flex;gap:10px;align-items:center"><span class="mono mute" style="font-size:13px">7fQa…m2Lx</span><span class="demo">Demo wallet</span></div></div>
<div class="card psum"><div><div class="k2">Total value</div><div class="pv2 num">{tot:.3f} SOL</div><div class="mute num" style="font-size:13px">{usd(tot*SOL_USD,2)} · tokens at current prices, Hybrid NFTs at their release value <span class="demo" style="margin-left:4px">Example</span></div>
<div class="mix">{mix_bar}</div><div class="mixl">{mix_l}</div></div>
<div><div class="k2" style="margin-bottom:6px">Wallet</div><div class="kv2"><div><span>SOL balance</span><b class="num">12.40 SOL</b></div><div><span>Positions</span><b class="num">4 tokens · 4 NFTs</b></div>
<div><span>Fees paid on {BRAND}</span><b class="num">0.03 SOL platform · 0.0062 SOL mint</b></div><div><span>Randomness pending</span><b>None</b></div></div>
<div style="margin-top:14px">{cap_slot('<span class="dim">3.2 SOL used</span>')}</div></div></div>

<div class="card pad" style="margin-top:24px"><div class="ch"><div><h3>Positions</h3><div class="sub2">One row per token. Values move with the market</div></div><span class="demo">Demo data</span></div>
<table class="pt"><thead><tr><th>Token</th><th>Type</th><th>Phase</th><th class="r">Tokens</th><th class="r">NFTs</th><th class="r">Value</th><th class="r">24h</th><th></th></tr></thead><tbody>{rows}</tbody></table></div>

<div class="two" style="margin-top:24px">
<div class="card pad"><div class="ch"><div><h3>NFTs by collection</h3><div class="sub2">What each one is worth to you in tokens</div></div></div><div class="nl">{nl_h}</div>
<p class="fn" style="margin-top:10px">Rarity is cosmetic. Hybrid NFTs release for exactly their ratio in tokens, whatever their traits.</p></div>
<div class="card pad"><div class="ch"><div><h3>History</h3><div class="sub2">Captures, releases, re-rolls, burns and trades</div></div><span class="demo">Demo data</span></div>
<table class="pt"><tbody>{hist_h}</tbody></table></div></div>
</div>"""
    return page(f"Portfolio · {BRAND}", TOKEN_CSS + PAGE_CSS + PORT_CSS + H_UP_CSS, h_up(body), active="portfolio", desc=f"Your {BRAND} portfolio: tokens, NFTs and fees paid, in one summary. Unaudited beta on devnet.")

open(os.path.join(OUT, "portfolio.html"), "w").write(portfolio())


# ------------------------------------------------------------------ STEP 7: trust, bug bounty, FAQ
DOC_CSS = """
.doc{display:grid;grid-template-columns:220px minmax(0,1fr);gap:40px;align-items:start;padding-top:8px}
.toc{position:sticky;top:120px;display:flex;flex-direction:column;gap:2px;font-size:13px}.toc a{padding:7px 10px;border-radius:8px;color:var(--mute)}.toc a:first-child{color:var(--ink);background:rgba(255,255,255,.04)}
.dsec{padding:26px 28px;margin-bottom:20px}.dsec h2{font-size:21px;letter-spacing:-.02em;margin:0 0 6px}.dsec>p{color:var(--mute);font-size:14px;line-height:1.65;margin:0 0 14px;max-width:720px}.dsec>p b{color:var(--ink);font-weight:600}
.sumg{display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin-bottom:24px}.sumg div{padding:16px 18px}.sumg span{display:block;font-size:12px;color:var(--mute);margin-bottom:6px}.sumg b{font-size:16px;font-weight:600;letter-spacing:-.01em}.sumg small{display:block;margin-top:6px;font-size:12px;color:var(--mute);line-height:1.45}
.cols2{display:grid;grid-template-columns:1fr 1fr;gap:14px}.lst2{margin:0;padding:0;list-style:none;display:flex;flex-direction:column;gap:10px}
.lst2 li{display:grid;grid-template-columns:20px 1fr;gap:8px;font-size:13.5px;line-height:1.55;color:var(--mute)}.lst2 li b{color:var(--ink);font-weight:600}.lst2 li:before{content:"";width:7px;height:7px;border-radius:50%;margin-top:7px;background:var(--up)}
.lst2.pl li:before{background:transparent;border:1.5px solid var(--acc2)}.lst2.no li:before{background:var(--down)}
.panel{padding:16px 18px;border-radius:12px;background:var(--s1);border:1px solid var(--line)}.panel h3{font-size:14px;margin:0 0 12px;letter-spacing:-.01em}
table.at{width:100%;border-collapse:collapse;font-size:13px}.at th,.at td{text-align:left;padding:11px 12px;border-bottom:1px solid var(--line);vertical-align:top;line-height:1.5}.at th{color:var(--dim);font-weight:500;font-size:12px}.at td:first-child{color:var(--ink);font-weight:600;width:160px}.at td{color:var(--mute)}
.tl3{display:grid;grid-template-columns:repeat(3,1fr);gap:12px}.tl3 div{padding:16px 18px;border-radius:12px;border:1px solid var(--line);background:var(--s1)}.tl3 div.on{box-shadow:0 0 0 1px rgba(232,176,75,.4) inset}
.tl3 span{font:500 11px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim)}.tl3 b{display:block;font-size:15px;margin:8px 0 6px}.tl3 p{margin:0;font-size:12.5px;color:var(--mute);line-height:1.55}
.rule{padding:22px 24px;border-radius:14px;border:1px solid rgba(255,106,43,.35);background:var(--accbg)}.rule q{display:block;quotes:none;font-family:var(--hd);font-weight:800;font-stretch:82%;font-size:26px;letter-spacing:-.02em;line-height:1.2;color:var(--ink)}
.rule p{margin:10px 0 0;font-size:13px;color:#ffd3bd}
.ph2{padding:14px 16px;border-radius:12px;border:1.5px dashed rgba(160,165,174,.5);font:500 13px var(--mono);color:var(--mute)}
.qa{padding:18px 22px;border-bottom:1px solid var(--line)}.qa:last-child{border-bottom:0}.qa h3{font-size:15px;letter-spacing:-.01em;margin:0 0 6px}.qa p{margin:0;color:var(--mute);font-size:14px;line-height:1.65;max-width:760px}.qa p b{color:var(--ink);font-weight:600}
.fqg{padding:0;margin-bottom:20px}.fqg>h2{font-size:13px;font-family:var(--mono);font-weight:500;text-transform:uppercase;letter-spacing:.06em;color:var(--dim);margin:0;padding:18px 22px 4px}
"""
def _doc(title, lead, chips, toc, secs, active, desc):
    toc_h = "".join(f'<a href="#{a}">{t}</a>' for a, t in toc)
    body = f"""<div class="wrap"><div class="phd"><div><h1>{title}</h1><p>{lead}</p></div><div style="display:flex;gap:8px;flex-wrap:wrap;justify-content:flex-end">{chips}</div></div>
<div class="doc"><nav class="toc" aria-label="On this page">{toc_h}</nav><div>{secs}</div></div></div>"""
    return page(f"{title} · {BRAND}", TOKEN_CSS + PAGE_CSS + DOC_CSS, body, active=active, desc=desc)

def trust():
    kinds = ("plain", "hybrid", "burn")
    rows = {k: {a: (st, c, d) for a, st, c, d in auth_rows(k, "tokens", "the ratio in")} for k in kinds}
    labels = []
    for k in kinds:
        for a in rows[k]:
            if a not in labels: labels.append(a)
    cell = lambda k, a: (f'<span class="st2 {rows[k][a][1]}"><i></i>{rows[k][a][0]}</span>' if a in rows[k] else '<span class="dim">Not applicable</span>')
    at = "".join(f'<tr><td>{a}</td>{"".join(f"<td>{cell(k, a)}</td>" for k in kinds)}</tr>' for a in labels)
    what = [("Mint authority", "The power to create new tokens. Revoked means the supply can never grow."),
            ("Freeze authority", "The power to lock tokens inside someone's wallet. It's never set on Armory tokens."),
            ("Metadata update authority", "The power to change a token's name, ticker or image. Launches must be created without one."),
            ("Upgrade authority", "The power to replace a program's code. This is the one power that still exists during beta. Here's who holds it and how it ends.")]
    what_h = "".join(f'<li><span><b>{a}.</b> {b}</span></li>' for a, b in what)
    secs = f"""
<div class="sumg"><div class="card"><span>Audit</span><b style="color:#e8c27a">Not audited yet</b><small>Internal reviews done. A paid third-party audit comes before mainnet.</small></div>
<div class="card"><span>Network</span><b>Solana devnet only</b><small>Test tokens and demo data. No real funds.</small></div>
<div class="card"><span>Deposit cap</span><b>{DEP_CAP} <span class="demo">Example</span></b><small>The real value isn't set yet.</small></div>
<div class="card"><span>Program upgrades</span><b style="color:#e8c27a">Not locked yet</b><small>{UPG_SHORT}. Frozen after the audit.</small></div></div>

<section class="card dsec" id="audit"><h2>Audit status</h2><p>{BRAND} is an <b>unaudited beta</b>. The programs run on Solana devnet with test tokens. Treat everything here as experimental.</p>
<div class="cols2"><div class="panel"><h3>Done so far</h3><ul class="lst2"><li><span><b>Two internal security reviews</b> of the programs, merged into one list of 41 findings.</span></li>
<li><span><b>Each closed finding has a regression test</b> that fails if the bug comes back.</span></li>
<li><span><b>Still open:</b> moving upgrade keys to a multisig, among others.</span></li></ul></div>
<div class="panel"><h3>Planned before mainnet</h3><ul class="lst2 pl"><li><span><b>A paid, professional third-party audit.</b></span></li><li><span><b>A live bug bounty</b> (<a class="acc" href="bug-bounty.html">details</a>).</span></li>
<li><span><b>The deposit cap switched on.</b></span></li><li><span><b>Upgrade keys held only by humans</b>, in a 3-of-5 multisig.</span></li>
<li><span><b>A verifiable build</b>, with the audited commit and build hash published.</span></li></ul></div></div>
<p style="margin:14px 0 0">The "Unaudited beta" labels come off only after the audit report is published.</p></section>

<section class="card dsec" id="authorities"><h2>Locked authorities, explained</h2><p>An "authority" is a key that's allowed to change something on-chain. Here's what each one means and where it stands for each launch type. Only Hybrid is deployed on devnet today; the Plain and Burn columns describe their code, which is pending deploy.</p>
<ul class="lst2" style="margin-bottom:18px">{what_h}</ul>
<table class="at"><thead><tr><th>Authority</th><th>Plain</th><th>Hybrid</th><th>Burn</th></tr></thead><tbody>{at}</tbody></table></section>

<section class="card dsec" id="upgrades"><h2>Program upgrades</h2><p>Upgrades exist so bugs found before and during the audit can be fixed. They end once the code is stable.</p>
<div class="tl3"><div class="on"><span>Now · devnet</span><b>One development key per program</b><p>Test keys on a test network, not mainnet keys. This is a known open item.</p></div>
<div><span>Mainnet</span><b>3-of-5 multisig, 7-day public delay</b><p>Any upgrade needs 3 of 5 signers and then waits 7 days in public before it can run. Signers are named before mainnet.</p></div>
<div><span>After audit + stabilization</span><b>Upgrades switched off</b><p>The upgrade authority is removed, so the code can never change again. How long stabilization lasts isn't decided yet.</p></div></div></section>

<section class="card dsec" id="keys"><h2>Key policy</h2>
<div class="rule"><q>No AI agent holds mainnet keys. Mainnet keys are held by humans only.</q><p>This applies to every mainnet key: program upgrades, the multisig, and the platform fee address.</p></div>
<ul class="lst2" style="margin-top:16px"><li><span><b>Today:</b> devnet uses throwaway test keys. No mainnet keys exist on our development machines.</span></li>
<li><span><b>At mainnet (planned):</b> the upgrade authority is a 3-of-5 multisig held by people. A single person's key is never enough.</span></li></ul></section>

<section class="card dsec" id="randomness"><h2>Randomness and fairness</h2>
<div class="cols2"><div class="panel"><h3>{tchip("hybrid")}</h3><ul class="lst2"><li><span><b>Art and traits are committed before converting opens</b> as a fingerprint (Merkle root), so nobody can swap a piece later.</span></li>
<li><span><b>Switchboard VRF</b> (a verifiable random function: a random number that comes with a proof) picks which NFT you get at every capture and re-roll.</span></li>
<li><span><b>The randomness provider is pinned</b> to an approved list written into the program.</span></li></ul></div>
<div class="panel"><h3>{tchip("burn")}</h3><ul class="lst2"><li><span><b>Art is committed before burning opens</b>, and each minted NFT is checked against it.</span></li>
<li><span><b>NFTs go out in collection order</b>, with no randomness.</span></li></ul>
<ul class="lst2 no" style="margin-top:10px"><li><span><b>Known limitation:</b> the order is public, so anyone can see which piece is next, rare ones included, and time a burn for it.</span></li></ul></div></div></section>

<section class="card dsec" id="fees"><h2>Fees</h2><p>All {BRAND} fees are flat and paid in SOL. <b>Tokens are never taken as a fee.</b> Each collection's fee is set from its ratio at launch, and the program caps it at 0.01 SOL.</p>
<table class="at"><thead><tr><th>Fee</th><th>Plain</th><th>Hybrid</th><th>Burn</th></tr></thead><tbody>
<tr><td>Platform fee per NFT action</td><td>None (no NFTs)</td><td>0.002 / 0.005 / 0.01 SOL by ratio, per capture or re-roll</td><td>Same tiers, per burn</td></tr>
<tr><td>Mint deposit / cost</td><td>—</td><td>0.0063 SOL deposit per capture or re-roll, refunded except ≈ 0.003–0.004 SOL when the NFT is minted for the first time</td><td>No deposit; ≈ 0.003–0.004 SOL mint cost paid directly on every burn</td></tr>
<tr><td>Randomness</td><td>—</td><td>No separate fee</td><td>Not used</td></tr>
<tr><td>Converting back</td><td>—</td><td>Free (network fee only)</td><td>Not possible</td></tr>
<tr><td>Trading on the curve</td><td colspan="3">1% (devnet curve setting) <span class="demo">Final pending</span></td></tr>
<tr><td>Trading after graduation</td><td colspan="3">Pool fee set by the DEX pool: 0.25% on the devnet setup <span class="demo">Final pending</span></td></tr>
<tr><td>Launch fee</td><td colspan="3">None on chain today</td></tr></tbody></table>
<p style="margin:12px 0 0">Ratio tiers: 50K = 0.002 SOL; 100K and 200K = 0.005 SOL; 500K to 5M = 0.01 SOL. {MINT_DISC} Fees can only go to the platform address written into the program.</p></section>

<section class="card dsec" id="cap"><h2>Deposit cap</h2><p>A per-wallet cap is planned while the programs are unaudited, to limit how much anyone can lose to a bug we haven't found yet. <b>It isn't enforced by the programs yet</b>, so today nothing limits how much one wallet puts in.</p>
{cap_slot('<span class="dim">The real value will be published here before it applies.</span>')}</section>

<section class="card dsec" id="limits"><h2>Known limitations</h2><ul class="lst2 no">
<li><span><b>Upgrades are a trust assumption</b> until they're switched off. During beta, whoever holds the upgrade key could change the code.</span></li>
<li><span><b>Third-party programs</b> (the bonding-curve and DEX pool programs, Switchboard randomness, Metaplex Core NFTs, DEXes and marketplaces) are outside {BRAND}'s control.</span></li>
<li><span><b>Burn order is public</b>, so rare pieces can be targeted.</span></li>
<li><span><b>The kept part of the mint deposit varies.</b> It's Solana rent plus the Metaplex fee, about 0.003–0.004 SOL depending on the length of the NFT's metadata address.</span></li>
<li><span><b>Tax split and Raffle aren't live.</b> They stay off until there's a public way to buy, the stuck-funds fixes are done, and Raffle has a legal check.</span></li></ul></section>

<section class="card dsec" id="addresses"><h2>Program addresses</h2><p>The mainnet program addresses and the verifiable build hash will be published here at launch.</p>
<div class="ph2">Placeholder: program addresses and build hash, published at mainnet launch</div></section>

<section class="card dsec"><h2>Found a problem?</h2><p>Please report security issues privately rather than posting them publicly.</p><a class="btn a" href="bug-bounty.html">Report a vulnerability</a></section>"""
    toc = [("audit", "Audit status"), ("authorities", "Locked authorities"), ("upgrades", "Program upgrades"), ("keys", "Key policy"), ("randomness", "Randomness"), ("fees", "Fees"), ("cap", "Deposit cap"), ("limits", "Known limitations"), ("addresses", "Program addresses")]
    chips = '<span class="chip ua">Unaudited beta</span><span class="chip">Devnet</span>'
    return _doc("Trust &amp; security", f"What's locked, what isn't yet, and what happens before {BRAND} goes to mainnet. Plain answers, checked against our own engineering records.", chips, toc, secs, "trust",
                f"{BRAND} trust and security: audit status, locked authorities, program upgrades, key policy, fees and the deposit cap. Unaudited beta on devnet.")

def bounty():
    secs = f"""
<section class="card dsec" id="status"><h2>Status</h2><p>The bug bounty <b>isn't live yet</b>. It opens before mainnet, as part of the launch checklist alongside the third-party audit. Reports about the devnet beta are welcome now.</p>
<div style="display:flex;gap:8px;flex-wrap:wrap"><span class="chip">Not live yet</span><span class="chip ua">Unaudited beta</span></div></section>
<section class="card dsec" id="scope"><h2>Scope <span class="demo" style="vertical-align:3px">Proposed</span></h2><p>This is a proposed scope, drawn from what {BRAND} runs. The final scope is published when the bounty opens.</p>
<div class="cols2"><div class="panel"><h3>In scope</h3><ul class="lst2"><li><span><b>{BRAND}'s on-chain programs:</b> the launch program and the vault program, covering Plain, Hybrid and Burn.</span></li>
<li><span><b>The {BRAND} website</b> and the transactions it asks you to sign.</span></li></ul></div>
<div class="panel"><h3>Out of scope</h3><ul class="lst2 no"><li><span><b>Third-party programs:</b> the bonding-curve program, Switchboard, Metaplex, DEXes and marketplaces. Please report to them directly.</span></li>
<li><span><b>Social engineering</b>, phishing or physical attacks.</span></li><li><span><b>Denial of service</b> and spam.</span></li>
<li><span><b>Issues already listed</b> under <a class="acc" href="trust.html#limits">Known limitations</a>.</span></li></ul></div></div></section>
<section class="card dsec" id="report"><h2>How to report</h2><p>Send reports privately. Please don't open a public issue or post details until a fix is out.</p>
<div class="ph2">Placeholder: security contact address, to be published</div>
<ul class="lst2" style="margin-top:16px"><li><span><b>What's affected:</b> the program or page, plus the launch type.</span></li><li><span><b>Steps to reproduce</b> on devnet, with transaction signatures if you have them.</span></li>
<li><span><b>The impact</b> you think it has.</span></li></ul></section>
<section class="card dsec" id="rules"><h2>Ground rules</h2><ul class="lst2"><li><span><b>Test on devnet only</b>, with test tokens.</span></li><li><span><b>Don't touch other people's funds or data.</b></span></li>
<li><span><b>Give us reasonable time to fix</b> before you publish.</span></li></ul><p style="margin:14px 0 0">Safe-harbor terms are published with the bounty.</p></section>
<section class="card dsec" id="rewards"><h2>Rewards</h2><p><b>Rewards to be announced.</b> Reward tiers and amounts haven't been decided. They'll be published here when the bounty goes live.</p></section>"""
    toc = [("status", "Status"), ("scope", "Scope"), ("report", "How to report"), ("rules", "Ground rules"), ("rewards", "Rewards")]
    return _doc("Bug bounty", f"Help keep {BRAND} safe. Here's what's in scope, how to report, and where rewards stand.", '<span class="chip">Not live yet</span><span class="chip soon">Rewards to be announced</span>', toc, secs, "trust",
                f"{BRAND} bug bounty: proposed scope, how to report, ground rules. Rewards to be announced.")

def faq():
    G = [("basics", "Basics", [
        ("What is Armory?", f"A place to launch Solana memecoins. A launch can be <b>Plain</b> (just the coin), <b>Hybrid</b> (the coin is also an NFT collection, both ways) or <b>Burn</b> (burn coins to mint an NFT). It's an unaudited beta on devnet with test tokens."),
        ("What's a bonding curve?", "A formula that sets the price: it goes up as people buy and down as they sell. Every launch starts on a bonding curve."),
        ("What does \"graduation\" mean?", "When the curve has raised its target (85 SOL by default, and never less than 10 SOL), the token graduates. The SOL raised and the tokens set aside go into a public DEX pool, and trading continues there. Unsold curve tokens stay locked by the program."),
        ("Which launch types are live?", "Hybrid is live on devnet. Plain and Burn are built and tested, but not deployed yet: they're pending deploy. Tax split and Raffle are coming later.")]),
      ("hybrid", "Hybrid", [
        ("What is SPL-404?", "A standard Solana token (SPL) paired with an NFT collection, so a fixed number of tokens and one NFT can be swapped for each other. Hybrid launches work this way."),
        ("How do I get an NFT?", "After graduation, lock the collection's ratio of tokens (for example 1,000,000) and you get one NFT. You pay a flat SOL platform fee set by the ratio and put up a 0.0063 SOL mint deposit. The deposit is refunded except about 0.003–0.004 SOL if your NFT is minted for the first time (Solana rent + Metaplex fee). There's no separate randomness fee. Your tokens are held, not spent."),
        ("Which NFT do I get?", "A random one. Switchboard VRF, a random number that comes with a proof, picks it from the art committed before converting opened. Nobody can choose a piece or see the next one."),
        ("Can I get my tokens back?", "Yes. Release an NFT and you get exactly the ratio back, whatever its traits. Release has no platform fee; you only pay the Solana network fee."),
        ("What's a re-roll?", "Swapping your NFT for a different random one. It costs the same as a capture, and the result may be more common than what you had."),
        ("Does rarity change what I get back?", "No. Rarity is cosmetic. Every NFT releases for the same number of tokens. Any marketplace price is set by buyers and sellers, not by Armory.")]),
      ("burn", "Burn and Plain", [
        ("How does a Burn launch work?", "After graduation, burn the ratio of tokens to mint the next NFT in the collection. The tokens are destroyed, so the supply goes down. You pay the platform fee and about 0.003–0.004 SOL mint cost, directly, with no deposit."),
        ("Can I undo a burn?", "No. Burn NFTs can't be converted back to tokens or re-rolled. They have no token backing; their price is whatever buyers pay."),
        ("Which NFT do I get from a burn?", "The next one in collection order. The order is public, so you can see what's next."),
        ("What's a Plain launch?", "Just the coin: 1,000,000,000 tokens on the curve, then a DEX. No NFTs and no per-NFT fees.")]),
      ("fees", "Fees", [
        ("What does it cost?", "Platform fees are flat SOL by ratio: 0.002 SOL (50K), 0.005 SOL (100K and 200K) or 0.01 SOL (500K and up), per capture, re-roll or burn. Release is free. Hybrid captures and re-rolls also put up a 0.0063 SOL mint deposit, refunded except about 0.003–0.004 SOL when an NFT is minted for the first time; Burn pays that mint cost directly. That's Solana rent plus the Metaplex fee, not an Armory fee. There's no randomness fee and no launch fee on chain today. The 1% curve trade fee is the devnet curve setting."),
        ("Do you take tokens as a fee?", "No. Fees are paid in SOL only, and go to a platform address fixed in the program.")]),
      ("safety", "Safety", [
        ("Is Armory audited?", "Not yet. There have been internal security reviews, and a paid third-party audit is required before mainnet. Until then, everything is labeled \"Unaudited beta\"."),
        ("What's the deposit cap?", f"A planned per-wallet limit while the programs are unaudited. The programs don't enforce it yet and the value isn't set; \"{DEP_CAP}\" on the site is an example."),
        ("Can anyone freeze my tokens or mint more?", "No. Freeze authority is never set and mint authority is revoked at launch. Token names and images can't be edited either."),
        ("Can anyone pause the app?", "No key or multisig can pause the vault, so captures, releases, re-rolls and burns can't be halted by us."),
        ("Who can change the programs?", "Today, on devnet, each program has one development key. The plan for mainnet: upgrades need a 3-of-5 multisig and a public 7-day delay, and they're switched off after the audit and a stabilization period."),
        ("Does an AI hold any keys?", "No AI agent holds mainnet keys. Mainnet keys are held by humans only."),
        ("How do I report a bug?", 'Privately, through the <a class="acc" href="bug-bounty.html">bug bounty page</a>. Rewards are to be announced.')]),
      ("soon", "Coming soon", [
        ("Why aren't Tax split and Raffle live?", "They're on hold until there's a public way to buy those tokens and two fixes for stuck funds are done. Raffle also needs a legal check first.")])]
    secs = "".join(f'<section class="card fqg" id="{gid}"><h2>{gt}</h2>' + "".join(f'<div class="qa" id="{gid}-{i}"><h3>{q}</h3><p>{a}</p></div>' for i, (q, a) in enumerate(qs)) + "</section>" for gid, gt, qs in G)
    secs = secs.replace('id="basics-1"', 'id="curve"')
    toc = [(g, t) for g, t, _ in G]
    return _doc("FAQ", f"Plain-language answers about {BRAND}. Every answer here matches how the programs actually work today.", '<span class="chip ua">Unaudited beta</span>', toc, secs, "faq",
                f"{BRAND} FAQ: launch types, bonding curves, graduation, fees, randomness and safety, in plain language.")

open(os.path.join(OUT, "trust.html"), "w").write(trust())
open(os.path.join(OUT, "bug-bounty.html"), "w").write(bounty())
open(os.path.join(OUT, "faq.html"), "w").write(faq())
print("trust/bounty/faq ok")

# ------------------------------------------------------------------ DESIGN SYSTEM: components.html (design/system/)
def components():
    lc = LAUNCH_CSS.split("\n")
    form_css = "\n".join(l for l in lc if l.startswith((".f{", ".f label", ".in{", ".in.warn", ".help", ".suffix", ".pre{", ".seg{", ".seg button")))
    css = TOKEN_CSS + form_css + """
.sys{padding:40px 0 80px}.sys h1{font-family:var(--hd);font-weight:800;font-stretch:82%;font-size:44px;letter-spacing:-.03em;margin:0}
.sysh{display:flex;justify-content:space-between;align-items:flex-end;padding-bottom:28px;border-bottom:1px solid var(--line);margin-bottom:28px}.sysh p{color:var(--mute);margin:10px 0 0;font-size:15px}
.sg{display:grid;grid-template-columns:1fr 1fr;gap:20px}.sg .full{grid-column:1/-1}
.sc{padding:22px 24px}.sc h2{font-size:15px;letter-spacing:-.01em;margin:0 0 4px}.sc .d{font-size:12.5px;color:var(--mute);margin:0 0 18px}
.rowx{display:flex;flex-wrap:wrap;gap:10px;align-items:center}.lab{font:500 10.5px var(--mono);color:var(--dim);text-transform:uppercase;letter-spacing:.06em;margin:16px 0 8px}
.sw2{display:grid;grid-template-columns:repeat(6,1fr);gap:10px}.sw2 div{border-radius:10px;border:1px solid var(--line);overflow:hidden;font-size:11.5px}.sw2 i{display:block;height:44px}.sw2 span{display:block;padding:7px 9px;color:var(--mute);font-family:var(--mono)}
.icx{display:grid;grid-template-columns:repeat(5,1fr);gap:10px}.icx div{display:flex;flex-direction:column;align-items:center;gap:8px;padding:16px 8px;border-radius:12px;border:1px solid var(--line);background:var(--s1);font-size:12.5px}.icx svg{color:var(--acc2)}.icx .so svg{color:var(--dim)}
.cdemo{display:grid;grid-template-columns:1fr 1fr;gap:12px}
.tt{position:absolute;padding:8px 10px;border-radius:9px;background:#14161a;border:1px solid var(--line2);box-shadow:0 8px 24px -8px rgba(0,0,0,.6);font-size:12px;line-height:1.5;pointer-events:none}.tt b{font-weight:600}
.spk{display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:14px}.spk div{padding:12px 14px;border-radius:12px;border:1px solid var(--line);background:var(--s1);font-size:12.5px;color:var(--mute)}
.bpx{display:grid;grid-template-columns:repeat(5,1fr);gap:8px}.bpx div{padding:10px 12px;border-radius:10px;border:1px solid var(--line);background:var(--s1);font-size:12px}.bpx b{display:block;font:500 13px var(--mono);color:var(--ink)}.bpx span{color:var(--mute)}
"""
    sw = [("--bg", "#08090a"), ("--s1 surface", "#0e0f11"), ("--ink", "#eceef1"), ("--acc Ember", "#ff6a2b"), ("--accp pressed", "#e85a1e"), ("--acc2 hover", "#ff9a62"), ("--up", "#3ecf8e"), ("--down", "#f25c7a"), ("warning", "#e8b04b"), ("--mute", "#a0a5ae"), ("--dim", "#858a94"), ("--line2", "rgba(255,255,255,.12)")]
    sw_h = "".join(f'<div><i style="background:{c}"></i><span>{n}<br>{c}</span></div>' for n, c in sw)
    icons_h = "".join(f'<div class="{"" if live else "so"}">{ticon(k, 28)}<b style="font-weight:600">{n}</b>{status_chip(k)}<span class="mono dim" style="font-size:10.5px">icons/{k if k != "tax" else "tax-split"}.svg</span></div>' for k, n, live, _, _ in TYPES)
    # chart: area+line with tooltip
    vals = price_series(n=48, seed=5, end=420)
    W, H = 560, 170
    d, pts, lo, hi = line_path(vals, W - 46, H, 12)
    grid = "".join(f'<line x1="0" x2="{W-40}" y1="{12+k*(H-24)/3:.0f}" y2="{12+k*(H-24)/3:.0f}" stroke="rgba(255,255,255,.05)"/><text x="{W}" y="{16+k*(H-24)/3:.0f}" text-anchor="end" fill="#858a94" font-size="11" font-family="Geist Mono, monospace">{(hi-k*(hi-lo)/3):.0f}</text>' for k in range(4))
    tx, ty_ = pts[30]
    chart_h = f'''<div style="position:relative"><svg viewBox="0 0 {W} {H+22}" width="100%"><defs><linearGradient id="cs" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff6a2b" stop-opacity=".28"/><stop offset="1" stop-color="#ff6a2b" stop-opacity="0"/></linearGradient></defs>{grid}
<path d="{d} L{W-46} {H} L0 {H} Z" fill="url(#cs)"/><path d="{d}" fill="none" stroke="#ff9a62" stroke-width="1.8" stroke-linejoin="round"/>
<line x1="{tx:.0f}" x2="{tx:.0f}" y1="0" y2="{H}" stroke="rgba(255,255,255,.18)" stroke-dasharray="3 3"/><circle cx="{tx:.1f}" cy="{ty_:.1f}" r="4.5" fill="#fff" stroke="#ff6a2b" stroke-width="3"/>
<text x="0" y="{H+18}" fill="#858a94" font-size="11" font-family="Geist Mono, monospace">Sep 22</text><text x="{(W-46)/2}" y="{H+18}" fill="#858a94" font-size="11" font-family="Geist Mono, monospace" text-anchor="middle">Sep 23</text><text x="{W-46}" y="{H+18}" fill="#858a94" font-size="11" font-family="Geist Mono, monospace" text-anchor="end">now</text></svg>
<div class="tt" style="left:{tx/W*100+2:.1f}%;top:{max(0, ty_-60):.0f}px"><span class="mute">Sep 23 · 14:00</span><br><b class="num">0.000000412 SOL</b> <span class="up num">+3.2%</span></div></div>'''
    def spark(up):
        v = price_series(n=24, seed=2 if up else 9, end=300)
        if not up: v = v[::-1]
        dd, *_ = line_path(v, 220, 44, 4)
        return f'<svg viewBox="0 0 220 44" width="100%" height="44"><path d="{dd}" fill="none" stroke="{"#3ecf8e" if up else "#f25c7a"}" stroke-width="1.6"/></svg>'
    bps = [("sm", "640px", "Phones: single column, nav collapses"), ("md", "900px", "Tablets: trade panel moves under content"), ("lg", "1200px", "Laptops: 2-column token layout"), ("xl", "1440px", "Desktop design width"), ("container", "1312px", "Content max width (.wrap)")]
    body = f"""<div class="wrap sys">
<div class="sysh"><div><span class="brand" style="font-size:20px">{BRAND}</span><h1 style="margin-top:14px">Components</h1><p>Design system reference for direction A (Obsidian, Ember). Tokens live in tokens.json; no motion anywhere.</p></div><span class="chip ua">Unaudited beta</span></div>
<div class="sg">
<div class="card sc full"><h2>Color</h2><p class="d">One accent. Ember fills primary buttons with dark text (#120805); pressed is #e85a1e. Up and down colors are only for price changes and buy/sell.</p><div class="sw2">{sw_h}</div></div>

<div class="card sc"><h2>Buttons</h2><p class="d">40px default, 36px small, 48px large. 10px radius. No hover motion.</p>
<div class="lab">Variants</div><div class="rowx"><a class="btn a">Primary</a><a class="btn p">Primary, neutral</a><a class="btn g">Secondary</a><a class="btn gh">Ghost</a><a class="btn d">Danger</a></div>
<div class="lab">States</div><div class="rowx"><a class="btn a" style="background:var(--accp)">Pressed</a><a class="btn a off">Disabled</a><a class="btn g off">Disabled</a><a class="btn a sm">Small</a><a class="btn a lg">Large</a></div></div>

<div class="card sc"><h2>Badges and chips</h2><p class="d">Phase, launch type and status labels. Example tags any figure that isn't real.</p>
<div class="lab">Phase</div><div class="rowx"><span class="chip acc"><i></i>On curve · 64%</span><span class="chip ok2"><i></i>Graduated</span><span class="chip">Converting opens at graduation</span></div>
<div class="lab">Launch type</div><div class="rowx">{"".join(tchip(k) for k in ("plain", "hybrid", "burn", "tax", "raffle"))}</div>
<div class="lab">Status</div><div class="rowx"><span class="tagwn beta">Beta</span><span class="chip bt">Beta</span><span class="chip ua">Unaudited beta</span><span class="demo">Example</span><span class="demo">Demo data</span><span class="chip soon">Coming soon</span></div></div>

<div class="card sc full"><h2>Launch type icons</h2><p class="d">24×24 grid, 1.75 stroke, round caps, currentColor. Ember for types in the product (live or pending deploy), dim when coming soon. Status chips: Live on devnet, Pending deploy, Coming soon. Abstract shapes only, no mascot motifs.</p><div class="icx">{icons_h}</div></div>

<div class="card sc"><h2>Cards</h2><p class="d">16px radius, 1px hairline, faint top-lit gradient. Header row: title, subline, one chip.</p>
<div class="card pad" style="background:var(--s1)"><div class="ch"><div><h3>Card title</h3><div class="sub2">Subline that explains the card</div></div><span class="chip ok2"><i></i>Open</span></div><p class="note" style="margin:0">Body copy is 13–14px, muted, with <b>bold for the one thing to remember</b>.</p></div>
<div class="lab">Beta banner (site-wide slot)</div><div style="border-radius:10px;overflow:hidden;border:1px solid var(--line)">{BETA_BAR.replace('<div class="wrap">', '<div style="display:flex;align-items:center;gap:12px;height:38px;padding:0 12px">').replace(f'<a href="trust.html">How {BRAND} is secured →</a>', '<a href="#">Trust →</a>')}</div>
<div class="lab">Deposit cap slot (wizard and trade panels)</div>{cap_slot()}
<div class="lab">Placeholder mascot art (every mascot spot, temporary until the Higgsfield mascot)</div><div class="rowx">{MPH.format(cls="").replace('class="mph "', 'class="mph" style="width:160px;height:160px"').replace("../../placeholder/", "../placeholder/")}</div></div>

<div class="card sc"><h2>Inputs</h2><p class="d">44px fields, 11px radius. Warnings in amber, never red, unless funds are at risk.</p>
<div class="f"><label>Name <span>9 / 32</span></label><input class="in" value="Low Orbit"></div>
<div class="f"><label>Graduation target <span>Default 85 SOL · minimum 10 SOL</span></label><div class="suffix"><input class="in" value="85"><em>SOL</em></div><div class="help">SOL raised on the curve before graduation.</div></div>
<div class="f"><label>Collection size</label><div class="suffix"><input class="in warn" value="40"><em>NFTs</em></div><div class="help warn">Collections need at least 100 NFTs.</div></div>
<div class="f" style="margin:0"><label>Segmented control</label><div class="seg"><button class="on">Tokens → NFT</button><button>NFT → tokens</button></div></div></div>

<div class="card sc"><h2>Stat and fact rows</h2><p class="d">Label left in muted, value right in ink. Mono numerals.</p>
<div class="card facts" style="background:var(--s1)"><div><span>Launch type</span><b>Hybrid · SPL-404 (token + NFTs)</b></div><div><span>Supply</span><b class="num">1,000,000,000 · fixed</b></div><div><span>Mint authority</span><b class="ok">{icon("check").replace("18","13")}Revoked</b></div><div><span>Audit</span><b style="color:#e8b04b">Not yet · unaudited beta</b></div></div>
<div class="rows"><div><span>Platform fee</span><b class="num">0.01 SOL</b></div><div><span>Mint deposit <span class="dim">(refundable)</span></span><b class="num">{MINT_DEP}</b></div><div><span>Paid now</span><b class="num">{CAP_NOW}</b></div></div></div>

<div class="card sc"><h2>Progress bar</h2><p class="d">4px track, Ember fill, a still dot at the tip. No stripes, no shimmer, no animation.</p>
<div class="bar" role="img" aria-label="64% filled"><i style="width:64%"></i></div><div class="rows" style="margin-top:10px"><div><span><b class="num">54.4</b> of 85 SOL raised</span><b class="num">64%</b></div></div>
<div class="lab">Complete</div><div class="bar done"><i style="width:100%"></i></div>
<div class="lab">Breakpoints (tokens.json)</div><div class="bpx">{"".join(f'<div><b>{n} {v}</b><span>{d}</span></div>' for n, v, d in bps)}</div></div>

<div class="card sc full"><h2>Charts</h2><p class="d">Price line in Ember light (#ff9a62, 1.8px) over an Ember area fading 28% to 0. Hairline grid at 5% white, axis labels in Geist Mono 11px #858a94 on the right and bottom. Tooltip: dark surface, date muted, value in ink, change in up or down color.</p>
<div class="cdemo"><div>{chart_h}</div><div><div class="spk"><div>Up · #3ecf8e{spark(True)}<span class="up num">+12.4%</span></div><div>Down · #f25c7a{spark(False)}<span class="down num">−3.2%</span></div></div>
<div class="rows" style="margin-top:14px"><div><span>Buy rows, positive change</span><b class="up">#3ecf8e</b></div><div><span>Sell rows, negative change</span><b class="down">#f25c7a</b></div><div><span>Price line</span><b class="acc">#ff9a62</b></div><div><span>Current point</span><b>white, 3px Ember ring</b></div></div></div></div></div>
</div></div>"""
    return a11y(HEAD.replace("%TITLE%", f"Components · {BRAND} design system").replace("%DESC%", f"{BRAND} design system components").replace("%CSS%", css + RARITY_CSS + RESP_CSS).replace("<body>", '<body><a class="skip" href="#main">Skip to content</a>', 1) + '<main id="main">' + body + "</main></body></html>")

open(os.path.join(os.path.dirname(__file__), "..", "..", "system", "components.html"), "w").write(components())
print("explore/portfolio/components ok")
