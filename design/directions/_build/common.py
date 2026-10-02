"""Shared sample data, number helpers and generative SVG art for the three direction mockups.
All data is DEMO data. Supply is always 1,000,000,000; SOL assumed at $150."""
import random, math

SOL_USD = 150
SUPPLY = 1_000_000_000
BRAND = "Mintmark"  # working name, see NAMES.md

def fmt(n, d=0):
    return f"{n:,.{d}f}"

def usd(n, d=0):
    return "$" + fmt(n, d)

def sub_price(p):
    """0.000000384 -> 0.0<sub>6</sub>384 (DexScreener-style zero compression)."""
    s = f"{p:.12f}".rstrip("0")
    dec = s.split(".")[1]
    z = len(dec) - len(dec.lstrip("0"))
    sig = dec.lstrip("0")[:4]
    return f"0.0<sub>{z}</sub>{sig}"

def full_price(p):
    return f"{p:.12f}".rstrip("0")

# ratio = tokens per NFT; pieces = SUPPLY / ratio (must divide cleanly)
LAUNCHES = [
    dict(key="noct",  name="Nocturnes",    sym="NOCT",  price=0.000000384, prog=64, ratio=1_000_000, tax=2.0, holders=1286, ch=+12.4, vol=212.4, age="2d"),
    dict(key="salt",  name="Salt Flats",   sym="SALT",  price=0.000000512, prog=88, ratio=100_000,   tax=1.0, holders=2904, ch=+4.1,  vol=388.0, age="5d"),
    dict(key="kite",  name="Paper Kites",  sym="KITE",  price=0.000000147, prog=21, ratio=500_000,   tax=1.5, holders=311,  ch=+31.8, vol=41.7,  age="9h"),
    dict(key="glass", name="Glasshouse",   sym="GLSS",  price=0.00000231,  prog=100,ratio=250_000,   tax=0.0, holders=6120, ch=-3.2,  vol=1204.5,age="18d"),
    dict(key="low",   name="Lowlight",     sym="LOW",   price=0.000000268, prog=43, ratio=1_000_000, tax=2.0, holders=804,  ch=-1.9,  vol=96.3,  age="3d"),
    dict(key="note",  name="Field Notes",  sym="NOTE",  price=0.000000421, prog=71, ratio=2_000_000, tax=0.0, holders=1540, ch=+7.7,  vol=174.9, age="4d"),
    dict(key="quiet", name="Quiet Engine", sym="QUIET", price=0.000000093, prog=9,  ratio=100_000,   tax=2.5, holders=97,   ch=+58.0, vol=12.2,  age="2h"),
    dict(key="ferro", name="Ferro",        sym="FERRO", price=0.000000334, prog=56, ratio=1_000_000, tax=1.0, holders=1102, ch=+2.6,  vol=133.0, age="6d"),
]
for L in LAUNCHES:
    assert SUPPLY % L["ratio"] == 0
    L["pieces"] = SUPPLY // L["ratio"]
    L["mc_sol"] = round(L["price"] * SUPPLY, 4)
    L["mc_usd"] = L["mc_sol"] * SOL_USD
    L["nft_sol"] = L["price"] * L["ratio"]

def ratio_short(r):
    return f"{r//1_000_000}M" if r >= 1_000_000 and r % 1_000_000 == 0 else f"{r//1000}K"

T = LAUNCHES[0]  # featured token: Nocturnes
# Nocturnes detail (demo)
TOK = dict(
    raised=54.4, grad=85.0, curve_sold=512_000_000, curve_total=800_000_000,
    top10=18.4, draw_no=7, draw_utc="Sep 27, 20:00 UTC", draw_in="2d 23h 48m",
    hold_h=72, hold_left="41h 12m", tickets_total=612, vault_sol=1.18,
    wrap_fee_sol=0.01, trade_fee=1.0,
    mint="NoCT…7xQ4", hybrid="MPLh…9rT2", vault="PRZv…3kLm",
)
assert abs(TOK["raised"]/TOK["grad"]*100 - 64) < 0.1
assert TOK["curve_sold"]/TOK["curve_total"] == 0.64
PRIZES = [("#0412", 0.39), ("#0871", 0.40), ("#0093", 0.39)]
BUY_SOL = 1.00
BUY_FEE = BUY_SOL * TOK["trade_fee"]/100
BUY_GROSS = round((BUY_SOL - BUY_FEE) / T["price"])          # 2,578,125
BUY_TAX = round(BUY_GROSS * T["tax"]/100)                     # 51,563 (rounded)
BUY_NET = BUY_GROSS - BUY_TAX                                 # 2,526,562

def _buy(sol):  # tokens received after 1% trade fee and 2% transfer tax (spot price, demo)
    return round(sol * 0.99 / T["price"] * 0.98)
def _sell(tk):  # SOL received for tokens after 2% transfer tax and 1% trade fee
    return round(tk * 0.98 * T["price"] * 0.99, 3)
ACTIVITY = [  # (kind, wallet, tokens, sol, ago, extra)
    ("Buy",    "7fQa…m2Lx", _buy(2.5), 2.50, "12s", ""),
    ("Wrap",   "Hk3P…9wQe", 1_000_000, None, "48s", "Nocturnes #0588"),
    ("Sell",   "C2vN…t8Rb", 4_000_000, _sell(4_000_000), "1m", ""),
    ("Tax",    "Prize vault", 80_000, None, "1m", "2% of a 4,000,000 transfer"),
    ("Buy",    "9mTe…Ka71", _buy(0.5), 0.50, "2m", ""),
    ("Unwrap", "Bq8s…Zp4D", 1_000_000, None, "4m", "Nocturnes #0231"),
    ("Buy",    "4LxW…eR0c", _buy(1.2), 1.20, "5m", ""),
    ("Sell",   "Fz1k…uY6n", 750_000, _sell(750_000), "7m", ""),
]

LOCKED = [
    ("Supply is fixed at 1,000,000,000", "Mint authority is revoked. Nobody can ever create more NOCT."),
    ("Nobody can freeze your tokens", "Freeze authority is revoked at launch."),
    ("1 NFT is always 1,000,000 NOCT", "The conversion rate is fixed. Converting back always returns exactly 1,000,000 NOCT."),
    ("The transfer tax stays at 2%", "The fee authority is revoked. It can't be raised, lowered, or redirected."),
    ("Tax can only buy prize NFTs", "It sits in a program-controlled vault. No person or wallet can withdraw it."),
    ("Ticket rules are fixed", "Tickets = floor(balance ÷ 1,000,000), counted only after a 72-hour hold."),
    ("Winners come from verifiable randomness", "Every draw uses VRF. Anyone can check the proof on-chain."),
    ("Liquidity is burned at graduation", "When the curve completes, pool liquidity is locked for good."),
]

RISK = ("Memecoins and NFTs are highly volatile. You can lose everything you put in. "
        "Nothing here is financial advice. Lottery prizes are not guaranteed and depend on tax collected. "
        "Contracts are unaudited testnet software (audit: pending). Availability may be restricted in your country.")

# ---------------- generative art ----------------
PALETTES = {
    "noct":  ["#0b1026", "#1b2a6b", "#6a7cff", "#e8e4ff"],
    "salt":  ["#efe9df", "#d9cbb4", "#9fb3c8", "#3d4a5c"],
    "kite":  ["#f4efe6", "#e2573b", "#1f3a5f", "#f2b33d"],
    "glass": ["#0e2a24", "#1f6f5c", "#8fe3c4", "#f1fff9"],
    "low":   ["#1a0f14", "#5e1f33", "#e0795f", "#ffd9b8"],
    "note":  ["#f3f1ea", "#2b2b2b", "#c9c3b3", "#b5452d"],
    "quiet": ["#16181d", "#2c313c", "#9aa4b5", "#e7ecf3"],
    "ferro": ["#1b1b1f", "#4a4d57", "#c0c4cc", "#ff6a3d"],
    "mono":  ["#121110", "#2e2923", "#c9a86a", "#f4e6c8"],
}

def art(key, uid, seed=0, rounded=0):
    """Return a self-contained inline SVG (square). uid keeps gradient ids unique per page."""
    r = random.Random(hash((key, seed)) & 0xffffffff)
    r.seed(f"{key}-{seed}")
    c = PALETTES[key]
    if key == "noct":
        c = [["#0b1026", "#1b2a6b", "#6a7cff", "#e8e4ff"], ["#0d0a1f", "#3b1d5e", "#b58cff", "#f3e9ff"],
             ["#061a22", "#0f4a5c", "#5fd0e6", "#e6fbff"], ["#140d0b", "#4a2618", "#ff9a6a", "#ffe8da"],
             ["#0a0f14", "#23313f", "#9fb4c8", "#f2f6fa"]][seed % 5]
    i = f"{key}{uid}"
    defs = (f'<defs><linearGradient id="g{i}" x1="0" y1="0" x2="1" y2="1">'
            f'<stop offset="0" stop-color="{c[0]}"/><stop offset="1" stop-color="{c[1]}"/></linearGradient>'
            f'<radialGradient id="r{i}" cx=".5" cy=".5" r=".5"><stop offset="0" stop-color="{c[3]}"/>'
            f'<stop offset="1" stop-color="{c[2]}"/></radialGradient>'
            f'<filter id="n{i}"><feTurbulence type="fractalNoise" baseFrequency=".9" numOctaves="2" stitchTiles="stitch"/>'
            f'<feColorMatrix values="0 0 0 0 0.5 0 0 0 0 0.5 0 0 0 0 0.5 0 0 0 .09 0"/></filter></defs>')
    body = f'<rect width="400" height="400" fill="url(#g{i})"/>'
    if key == "noct":
        cx, cy = 150 + r.randint(0, 100), 140 + r.randint(0, 60)
        for k in range(7, 0, -1):
            body += f'<circle cx="{cx}" cy="{cy}" r="{k*34}" fill="none" stroke="{c[2]}" stroke-opacity="{0.05+0.03*(7-k)}" stroke-width="1"/>'
        body += f'<circle cx="{cx}" cy="{cy}" r="62" fill="url(#r{i})"/>'
        ox, oy = r.choice([(22, -10), (-24, -6), (14, 18), (-12, -20), (30, 4)])
        body += f'<circle cx="{cx+ox}" cy="{cy+oy}" r="{r.choice([50, 56, 60])}" fill="{c[0]}" fill-opacity=".92"/>'
        for _ in range(40):
            body += f'<circle cx="{r.randint(0,400)}" cy="{r.randint(0,400)}" r="{r.choice([.6,.8,1.2])}" fill="{c[3]}" fill-opacity="{r.uniform(.3,.9):.2f}"/>'
        body += f'<path d="M0 {300+r.randint(0,30)} Q 200 {260+r.randint(0,30)} 400 {310+r.randint(0,20)} V400 H0Z" fill="{c[0]}" fill-opacity=".85"/>'
    elif key == "salt":
        for k in range(14):
            y = 200 + k*k*1.1
            body += f'<line x1="0" x2="400" y1="{y:.1f}" y2="{y:.1f}" stroke="{c[3]}" stroke-opacity="{0.08+k*0.02:.2f}"/>'
        body += f'<circle cx="{120+r.randint(0,160)}" cy="{120+r.randint(0,30)}" r="{40+r.randint(0,20)}" fill="{c[2]}" fill-opacity=".8"/>'
        body += f'<rect x="0" y="198" width="400" height="2" fill="{c[3]}" fill-opacity=".5"/>'
    elif key == "kite":
        for _ in range(3):
            x, y, s = r.randint(60, 300), r.randint(60, 260), r.randint(50, 110)
            col = r.choice(c[1:])
            body += f'<path d="M{x} {y-s} L{x+s*.7:.0f} {y} L{x} {y+s*1.2:.0f} L{x-s*.7:.0f} {y}Z" fill="{col}" fill-opacity=".92"/>'
            body += f'<path d="M{x} {y+s*1.2:.0f} q 20 40 -10 80 t 10 80" fill="none" stroke="{c[2]}" stroke-width="1.2"/>'
    elif key == "glass":
        for _ in range(6):
            x, y = r.randint(20, 240), r.randint(20, 240)
            w, h = r.randint(90, 180), r.randint(90, 180)
            body += f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{c[2]}" fill-opacity=".16" stroke="{c[3]}" stroke-opacity=".35"/>'
        body += f'<circle cx="{r.randint(150,250)}" cy="{r.randint(150,250)}" r="46" fill="url(#r{i})" fill-opacity=".9"/>'
    elif key == "low":
        for _ in range(4):
            body += f'<circle cx="{r.randint(60,340)}" cy="{r.randint(60,340)}" r="{r.randint(60,130)}" fill="{r.choice(c[1:])}" fill-opacity=".45" filter="url(#b{i})"/>'
        defs = defs.replace("</defs>", f'<filter id="b{i}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="28"/></filter></defs>')
    elif key == "note":
        for gx in range(20, 400, 20):
            for gy in range(20, 400, 20):
                body += f'<circle cx="{gx}" cy="{gy}" r="1" fill="{c[2]}"/>'
        pts = " ".join(f"{x},{200+60*math.sin(x/40+r.random()*2)+r.randint(-20,20):.0f}" for x in range(20, 400, 40))
        body += f'<polyline points="{pts}" fill="none" stroke="{c[1]}" stroke-width="2"/>'
        body += f'<circle cx="{r.randint(80,320)}" cy="{r.randint(80,160)}" r="18" fill="{c[3]}"/>'
    elif key == "quiet":
        s = r.randint(0, 20)
        for k in range(9):
            m = 30 + k*19 + s
            body += f'<rect x="{m}" y="{m}" width="{400-2*m}" height="{400-2*m}" fill="none" stroke="{c[2]}" stroke-opacity="{0.15+k*0.08:.2f}"/>'
        body += f'<rect x="190" y="190" width="20" height="20" fill="{c[3]}"/>'
    elif key == "ferro":
        for k in range(0, 400, 10):
            body += f'<rect x="{k}" y="0" width="5" height="400" fill="{c[2]}" fill-opacity="{0.05+0.25*abs(math.sin(k/60+seed)):.2f}"/>'
        body += f'<circle cx="{r.randint(120,280)}" cy="{r.randint(120,280)}" r="60" fill="none" stroke="{c[3]}" stroke-width="10"/>'
    elif key == "mono":
        hz = 250 + r.randint(0, 30)
        body += f'<circle cx="{r.choice([r.randint(60,90), r.randint(310,340)])}" cy="{r.randint(80,130)}" r="{r.randint(22,32)}" fill="{c[3]}" fill-opacity=".85"/>'
        body += f'<rect x="0" y="{hz}" width="400" height="{400-hz}" fill="{c[0]}" fill-opacity=".7"/>'
        w, x = r.randint(58, 80), r.randint(120, 200)
        body += f'<rect x="{x}" y="{hz-190}" width="{w}" height="190" fill="#0c0b0a"/><rect x="{x}" y="{hz-190}" width="2" height="190" fill="{c[2]}" fill-opacity=".8"/>'
        body += f'<rect x="{x}" y="{hz}" width="{w}" height="{120}" fill="{c[2]}" fill-opacity=".07"/>'
        body += f'<line x1="0" x2="400" y1="{hz}" y2="{hz}" stroke="{c[2]}" stroke-opacity=".45"/>'
    body += f'<rect width="400" height="400" filter="url(#n{i})"/>'
    rx = f' rx="{rounded}"' if rounded else ""
    return (f'<svg viewBox="0 0 400 400" preserveAspectRatio="xMidYMid slice" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Generative placeholder art">'
            f'{defs}<clipPath id="c{i}"><rect width="400" height="400"{rx}/></clipPath><g clip-path="url(#c{i})">{body}</g></svg>')

def price_series(n=96, seed=3, end=None):
    """Upward-ish random walk ending at the current Nocturnes price (units: 1e-9 SOL)."""
    r = random.Random(seed)
    v = [120.0]
    for k in range(n-1):
        v.append(max(40, v[-1] * (1 + r.gauss(0.012, 0.045))))
    end = end or T["price"] * 1e9
    f = end / v[-1]
    return [x * f for x in v]

def line_path(vals, w, h, pad=6):
    lo, hi = min(vals), max(vals)
    pts = []
    for k, v in enumerate(vals):
        x = k / (len(vals)-1) * w
        y = pad + (1 - (v - lo) / (hi - lo)) * (h - 2*pad)
        pts.append((x, y))
    d = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts)
    return d, pts, lo, hi

def spark(vals, w=90, h=28, stroke="#fff", sw=1.5):
    d, *_ = line_path(vals, w, h, 3)
    return f'<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}"><path d="{d}" fill="none" stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round"/></svg>'

def spark_vals(L, n=24):
    r = random.Random(L["sym"])
    v = [1.0]
    drift = L["ch"] / 100 / n
    for _ in range(n-1):
        v.append(v[-1] * (1 + drift + r.gauss(0, 0.02)))
    return v

# Demo wallet for holdings / convert panels (Nocturnes)
WALLET = dict(tokens=3_750_000, nfts=["#0142", "#0655"])
W_TOK_SOL = WALLET["tokens"] * T["price"]                 # 1.44 SOL
W_NFT_SOL = len(WALLET["nfts"]) * T["ratio"] * T["price"] # 0.768 SOL (valued at token equivalent)
W_TOTAL_SOL = W_TOK_SOL + W_NFT_SOL                       # 2.208 SOL
W_WRAPPABLE = WALLET["tokens"] // T["ratio"]              # 3 NFTs
W_WRAPPABLE_SOL = W_WRAPPABLE * T["ratio"] * T["price"]   # 1.152 SOL
W_TICKETS = WALLET["tokens"] // 1_000_000                 # 3
W_REMAINDER = WALLET["tokens"] % 1_000_000                # 750,000
