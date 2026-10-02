"""Fuze mascot (working name): cartoon black bomb with a lit fuse.
Hand-built comic-style SVG (thick ink line, flat cel shading). One shared rig, six poses + front.
Run: python3 mascot.py  -> writes ../svg/*.svg and ../_build/sheet.html, palette.html
"""
import math, os, random
HERE = os.path.dirname(os.path.abspath(__file__))
SVG_DIR = os.path.join(HERE, "..", "svg")

# ---- palette (only the spark is saturated) ----
INK = "#050507"; RIM = "#3b3b49"
BODY = "#2a2a35"; SHADE = "#16161d"; HI = "#4a4a5e"
CREAM = "#f3efe6"; CREAM_SH = "#cdc7ba"
METAL = "#6a6a7a"; METAL_SH = "#454553"; METAL_HI = "#9a9aab"
ROPE = "#8d8678"; ROPE_D = "#5f5a50"
SPARK = "#FF6A2B"; SPARK2 = "#FFC24D"; SPARK_CORE = "#FFF6DF"
MOUTH = "#0c0c10"; TONGUE = "#7a4b4f"
SMOKE = "#6c6c7a"; DROP = "#c9d3de"

CX, CY, R = 200, 228, 88

def P(x, y): return f"{x:.1f},{y:.1f}"

def star(cx, cy, ro, ri, n, rot=0):
    pts = []
    for i in range(n * 2):
        a = math.radians(rot + i * 180 / n - 90)
        r = ro if i % 2 == 0 else ri
        pts.append(P(cx + r * math.cos(a), cy + r * math.sin(a)))
    return " ".join(pts)

def noodle(d, w=8):
    return (f'<path d="{d}" fill="none" stroke="{RIM}" stroke-width="{w+11}" stroke-linecap="round" stroke-linejoin="round"/>'
            f'<path d="{d}" fill="none" stroke="{INK}" stroke-width="{w+7}" stroke-linecap="round" stroke-linejoin="round"/>'
            f'<path d="{d}" fill="none" stroke="{BODY}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>')

def arm(s, c, h):
    return noodle(f"M{P(*s)} Q{P(*c)} {P(*h)}")

def glove(h, c, flip=False):
    """white cartoon glove at hand point h; c = control point (arm comes from there)"""
    ang = math.degrees(math.atan2(h[1] - c[1], h[0] - c[0]))
    th = -1 if flip else 1
    return (f'<g transform="translate({P(*h)}) rotate({ang:.1f})">'
            f'<rect x="-21" y="-11" width="12" height="22" rx="4" fill="{CREAM_SH}" stroke="{INK}" stroke-width="4.5"/>'
            f'<circle cx="2" cy="0" r="15" fill="{CREAM}" stroke="{INK}" stroke-width="4.5"/>'
            f'<ellipse cx="-2" cy="{-13*th}" rx="7" ry="6" fill="{CREAM}" stroke="{INK}" stroke-width="4"/>'
            f'<path d="M6 {-6*th} q6 6 0 12" fill="none" stroke="{CREAM_SH}" stroke-width="3" stroke-linecap="round"/></g>')

def leg(hip, foot, bend=0):
    mx, my = (hip[0] + foot[0]) / 2 + bend, (hip[1] + foot[1]) / 2
    return noodle(f"M{P(*hip)} Q{P(mx, my)} {P(*foot)}", 9)

def shoe(f, d=1, rot=0):
    x, y = f
    return (f'<g transform="translate({P(x, y)}) rotate({rot}) scale({d},1)">'
            f'<path d="M-16 6 C-18 -10 2 -14 10 -6 C18 -4 28 0 27 8 C27 14 20 15 6 15 L-12 15 C-17 15 -17 10 -16 6 Z" fill="{CREAM}" stroke="{INK}" stroke-width="4.5" stroke-linejoin="round"/>'
            f'<path d="M-15 10 L25 10" stroke="{CREAM_SH}" stroke-width="4" stroke-linecap="round"/>'
            f'<path d="M-2 -5 l6 4 M3 -8 l6 4" stroke="{INK}" stroke-width="2.5" stroke-linecap="round"/></g>')

def collar(angle=18):
    return (f'<g transform="rotate({angle} {CX} {CY})">'
            f'<rect x="{CX-24}" y="{CY-R-24}" width="48" height="34" rx="7" fill="{RIM}"/>'
            f'<rect x="{CX-22}" y="{CY-R-22}" width="44" height="30" rx="6" fill="{METAL}" stroke="{INK}" stroke-width="5"/>'
            f'<rect x="{CX+4}" y="{CY-R-19.5}" width="15" height="25" rx="3" fill="{METAL_SH}"/>'
            f'<rect x="{CX-16}" y="{CY-R-18}" width="6" height="22" rx="3" fill="{METAL_HI}"/>'
            f'<ellipse cx="{CX}" cy="{CY-R-22}" rx="13" ry="4" fill="{INK}"/></g>')

def fuse_start(angle=18):
    a = math.radians(angle); d = R + 23
    return (CX + d * math.sin(a), CY - d * math.cos(a))

def fuse(d):
    return (f'<path d="{d}" fill="none" stroke="{INK}" stroke-width="14" stroke-linecap="round"/>'
            f'<path d="{d}" fill="none" stroke="{ROPE}" stroke-width="7.5" stroke-linecap="round"/>'
            f'<path d="{d}" fill="none" stroke="{ROPE_D}" stroke-width="7.5" stroke-dasharray="3 6" stroke-linecap="butt"/>')

def spark(x, y, k=1.0, pid="p", particles=6, seed=1):
    rnd = random.Random(seed)
    parts = []
    for i in range(particles):
        a = rnd.uniform(0, 2 * math.pi); dist = rnd.uniform(34, 52) * k
        px, py = x + dist * math.cos(a), y + dist * math.sin(a)
        s = rnd.uniform(4, 7) * k
        col = SPARK2 if i % 2 else SPARK
        parts.append(f'<polygon points="{star(px, py, s, s*0.35, 4, rnd.uniform(0,45))}" fill="{col}"/>')
    return (f'<circle cx="{x}" cy="{y}" r="{48*k:.1f}" fill="url(#glow{pid})"/>'
            + "".join(parts) +
            f'<polygon points="{star(x, y, 27*k, 11*k, 9, 8)}" fill="{SPARK}" stroke="{INK}" stroke-width="3.5" stroke-linejoin="round"/>'
            f'<polygon points="{star(x, y, 16*k, 7*k, 7, 20)}" fill="{SPARK2}"/>'
            f'<circle cx="{x}" cy="{y}" r="{5.5*k:.1f}" fill="{SPARK_CORE}"/>')

def body(pid):
    return (f'<clipPath id="bc{pid}"><circle cx="{CX}" cy="{CY}" r="{R}"/></clipPath>'
            f'<circle cx="{CX}" cy="{CY}" r="{R+6}" fill="{RIM}"/>'
            f'<g clip-path="url(#bc{pid})"><rect x="{CX-R}" y="{CY-R}" width="{2*R}" height="{2*R}" fill="{SHADE}"/>'
            f'<circle cx="{CX-13}" cy="{CY-16}" r="{R}" fill="{BODY}"/>'
            f'<ellipse cx="{CX-44}" cy="{CY-46}" rx="26" ry="12" transform="rotate(-40 {CX-44} {CY-46})" fill="{HI}"/>'
            f'<ellipse cx="{CX-58}" cy="{CY-26}" rx="5" ry="9" transform="rotate(-20 {CX-58} {CY-26})" fill="{HI}"/></g>'
            f'<ellipse cx="{CX-40}" cy="{CY-52}" rx="9" ry="5" transform="rotate(-40 {CX-40} {CY-52})" fill="{CREAM}" opacity=".9"/>'
            f'<circle cx="{CX}" cy="{CY}" r="{R}" fill="none" stroke="{INK}" stroke-width="6.5"/>')

EL, ER, EY = (CX - 24, CY - 10), (CX + 24, CY - 10), 0
def eye_open(c, look=(0, 0), squint=0):
    x, y = c; lx, ly = look
    s = (f'<ellipse cx="{x}" cy="{y}" rx="15" ry="{19-squint}" fill="{CREAM}" stroke="{INK}" stroke-width="4"/>'
         f'<circle cx="{x+lx}" cy="{y+ly+2}" r="8" fill="{INK}"/>'
         f'<circle cx="{x+lx+3}" cy="{y+ly-1}" r="2.8" fill="#fff"/>')
    return s

def eye_happy(c):
    x, y = c
    return f'<path d="M{x-12} {y+4} Q{x} {y-12} {x+12} {y+4}" fill="none" stroke="{CREAM}" stroke-width="5.5" stroke-linecap="round"/>'

def eye_half(c, look=(0, 2)):
    x, y = c
    return (eye_open(c, look) +
            f'<path d="M{x-17} {y-2} Q{x} {y-26} {x+17} {y-2} Z" fill="{BODY}" stroke="{INK}" stroke-width="4" stroke-linejoin="round"/>')

def brow(c, tilt=0, dy=0):
    x, y = c; y = y - 29 + dy
    t = math.radians(tilt); dx, ddy = 11 * math.cos(t), 11 * math.sin(t)
    return f'<path d="M{x-dx:.1f} {y-ddy:.1f} L{x+dx:.1f} {y+ddy:.1f}" stroke="{CREAM}" stroke-width="5" stroke-linecap="round"/>'

def cheeks(o=.32):
    return (f'<ellipse cx="{CX-46}" cy="{CY+18}" rx="11" ry="6" fill="{SPARK}" opacity="{o}"/>'
            f'<ellipse cx="{CX+46}" cy="{CY+18}" rx="11" ry="6" fill="{SPARK}" opacity="{o}"/>')

def mouth(kind):
    m = CY + 26
    if kind == "smile":
        return f'<path d="M{CX-15} {m} Q{CX} {m+14} {CX+15} {m}" fill="none" stroke="{CREAM}" stroke-width="5" stroke-linecap="round"/>'
    if kind == "grin":
        return (f'<path d="M{CX-20} {m-2} Q{CX} {m+2} {CX+20} {m-2} Q{CX+16} {m+22} {CX} {m+22} Q{CX-16} {m+22} {CX-20} {m-2} Z" fill="{MOUTH}" stroke="{CREAM}" stroke-width="4.5" stroke-linejoin="round"/>'
                f'<path d="M{CX-9} {m+18} Q{CX} {m+10} {CX+9} {m+18}" fill="{TONGUE}"/>')
    if kind == "big":
        return (f'<path d="M{CX-27} {m-6} Q{CX} {m} {CX+27} {m-6} Q{CX+22} {m+30} {CX} {m+30} Q{CX-22} {m+30} {CX-27} {m-6} Z" fill="{MOUTH}" stroke="{CREAM}" stroke-width="4.5" stroke-linejoin="round"/>'
                f'<path d="M{CX-13} {m+25} Q{CX} {m+13} {CX+13} {m+25}" fill="{TONGUE}"/>')
    if kind == "o":
        return f'<ellipse cx="{CX}" cy="{m+6}" rx="8" ry="10" fill="{MOUTH}" stroke="{CREAM}" stroke-width="4.5"/>'
    if kind == "wobbly":
        return f'<path d="M{CX-17} {m+8} q4.25 -6 8.5 0 t8.5 0 t8.5 0 t8.5 0" fill="none" stroke="{CREAM}" stroke-width="4.5" stroke-linecap="round" stroke-linejoin="round"/>'
    return ""

def defs(pid):
    return (f'<defs><radialGradient id="glow{pid}"><stop offset="0" stop-color="{SPARK}" stop-opacity=".7"/>'
            f'<stop offset=".45" stop-color="{SPARK}" stop-opacity=".22"/><stop offset="1" stop-color="{SPARK}" stop-opacity="0"/></radialGradient></defs>')

# ------------------------------------------------------------------ poses
SH_L, SH_R = (CX - 80, CY + 12), (CX + 80, CY + 12)
HIP_L, HIP_R = (CX - 26, CY + 70), (CX + 26, CY + 70)

def rig(pid, *, tilt=0, arms, feet=((CX-34, CY+118), (CX+34, CY+118)), legbend=(-6, 6), face, fuse_d, spark_at=None,
        spark_k=1.0, back="", front="", top="", char_tf="", fuse_extra="", parts=6, shoe_rot=(0, 0)):
    (sl, cl, hl, fl), (sr, cr, hr, fr) = arms
    head = (collar() + body(pid) + face + fuse(fuse_d) + fuse_extra +
            (spark(*spark_at, k=spark_k, pid=pid, particles=parts, seed=len(pid)) if spark_at else ""))
    g = (leg(HIP_L, feet[0], legbend[0]) + leg(HIP_R, feet[1], legbend[1]) +
         shoe(feet[0], -1, shoe_rot[0]) + shoe(feet[1], 1, shoe_rot[1]) +
         arm(sl, cl, hl) + arm(sr, cr, hr) +
         f'<g transform="rotate({tilt} {CX} {CY+60})">{head}</g>' +
         glove(hl, cl, fl) + glove(hr, cr, fr) + front)
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" width="400" height="400">{defs(pid)}'
            f'{back}<g transform="{char_tf}">{g}</g>{top}</svg>')

FS = fuse_start()
FUSE_STD = f"M{P(*FS)} C{P(FS[0]+12, FS[1]-30)} {P(FS[0]+44, FS[1]-24)} {P(FS[0]+36, FS[1]-50)} S{P(FS[0]+14, FS[1]-66)} {P(FS[0]+22, FS[1]-78)}"
TIP_STD = (FS[0] + 22, FS[1] - 80)

def face_std(look=(0, 0), mouth_k="grin", brows=(-8, 8), bdy=0, ch=.32, eyes=None):
    e = eyes if eyes is not None else eye_open(EL, look) + eye_open(ER, look)
    b = (brow(EL, brows[0], bdy) + brow(ER, brows[1], bdy)) if brows else ""
    return cheeks(ch) + e + b + mouth(mouth_k)

def pose_front():
    return rig("front", tilt=-3,
               arms=((SH_L, (CX - 118, CY + 40), (CX - 108, CY + 78), False),
                     (SH_R, (CX + 128, CY - 2), (CX + 132, CY - 72), True)),
               face=face_std((1, 0), "grin", (-6, 10)), fuse_d=FUSE_STD, spark_at=TIP_STD)

def pose_idle():
    return rig("idle", tilt=4,
               arms=((SH_L, (CX - 112, CY + 42), (CX - 100, CY + 82), False),
                     (SH_R, (CX + 112, CY + 42), (CX + 100, CY + 82), True)),
               face=face_std(eyes=eye_half(EL) + eye_half(ER), mouth_k="smile", brows=None, ch=.26),
               fuse_d=FUSE_STD, spark_at=TIP_STD, spark_k=.62, parts=2)

def pose_loading():
    d = f"M{P(*FS)} C{P(FS[0]+14, FS[1]-34)} {P(FS[0]+48, FS[1]-28)} {P(FS[0]+40, FS[1]-56)} S{P(FS[0]+16, FS[1]-70)} {P(FS[0]+24, FS[1]-82)}"
    lines = "".join(f'<path d="M{P(TIP_STD[0]+18*math.cos(a), TIP_STD[1]+2+18*math.sin(a))} L{P(TIP_STD[0]+70*math.cos(a), TIP_STD[1]+2+70*math.sin(a))}" stroke="{SPARK2}" stroke-width="3" stroke-linecap="round" opacity=".8"/>'
                    for a in [math.radians(x) for x in (-150, -110, -60, -20, 25)])
    return rig("loading", tilt=-2,
               arms=((SH_L, (CX - 126, CY + 6), (CX - 116, CY - 36), False),
                     (SH_R, (CX + 124, CY + 30), (CX + 118, CY - 10), True)),
               feet=((CX - 40, CY + 118), (CX + 40, CY + 118)),
               face=face_std((4, -7), "o", (-14, 14), bdy=-4), fuse_d=d, spark_at=(TIP_STD[0]+2, TIP_STD[1]-2), spark_k=1.3,
               parts=10, top="", fuse_extra=lines)

def coin(x, y, r=22):
    return (f'<g><circle cx="{x}" cy="{y}" r="{r+4}" fill="{RIM}"/><circle cx="{x}" cy="{y}" r="{r}" fill="{METAL_HI}" stroke="{INK}" stroke-width="4.5"/>'
            f'<path d="M{x+r*.15} {y-r+3} A{r-3} {r-3} 0 0 1 {x+r*.15} {y+r-3}" fill="none" stroke="{METAL}" stroke-width="5"/>'
            f'<circle cx="{x}" cy="{y}" r="{r*.55:.1f}" fill="none" stroke="{INK}" stroke-width="3"/>'
            f'<path d="M{x-5} {y-7} h10 M{x} {y-7} v14" stroke="{INK}" stroke-width="3.2" stroke-linecap="round"/></g>')

def card(x, y, rot=10, w=36, h=48):
    return (f'<g transform="translate({x} {y}) rotate({rot})"><rect x="{-w/2-3}" y="{-h/2-3}" width="{w+6}" height="{h+6}" rx="8" fill="{RIM}"/>'
            f'<rect x="{-w/2}" y="{-h/2}" width="{w}" height="{h}" rx="6" fill="{CREAM}" stroke="{INK}" stroke-width="4.5"/>'
            f'<rect x="{-w/2+5}" y="{-h/2+5}" width="{w-10}" height="{h-20}" rx="3" fill="{HI}"/>'
            f'<circle cx="{w/2-12}" cy="{-h/2+12}" r="4" fill="{CREAM}"/>'
            f'<path d="M{-w/2+5} {h/2-17} l9 -10 l7 6 l6 -5 l{w/2-17+5} 9 z" fill="{BODY}"/>'
            f'<rect x="{-w/2+6}" y="{h/2-10}" width="{w-20}" height="4" rx="2" fill="{CREAM_SH}"/></g>')

def swap_arrows(x, y):
    a = f'stroke="{CREAM}" stroke-width="5" fill="none" stroke-linecap="round" stroke-linejoin="round"'
    o = f'stroke="{INK}" stroke-width="10" fill="none" stroke-linecap="round" stroke-linejoin="round"'
    p1 = f"M{x-26} {y-8} Q{x} {y-26} {x+26} {y-8}"; h1 = f"M{x+16} {y-16} L{x+27} {y-7} L{x+14} {y-2}"
    p2 = f"M{x+26} {y+10} Q{x} {y+28} {x-26} {y+10}"; h2 = f"M{x-16} {y+18} L{x-27} {y+9} L{x-14} {y+4}"
    return "".join(f'<path d="{d}" {o}/>' for d in (p1, h1, p2, h2)) + "".join(f'<path d="{d}" {a}/>' for d in (p1, h1, p2, h2))

def pose_convert():
    hl, hr = (CX - 128, CY + 26), (CX + 128, CY + 26)
    front = coin(hl[0] - 4, hl[1] - 26) + card(hr[0] + 4, hr[1] - 30, 12)
    return rig("convert", tilt=0,
               arms=((SH_L, (CX - 116, CY + 46), hl, True), (SH_R, (CX + 116, CY + 46), hr, False)),
               face=face_std((5, 0), "smile", (-4, 12)), fuse_d=FUSE_STD, spark_at=TIP_STD, spark_k=.85,
               front=front, top=swap_arrows(92, 92))

def die(x, y, rot, pips):
    s = 30
    pos = {1: [(0, 0)], 2: [(-7, -7), (7, 7)], 3: [(-8, -8), (0, 0), (8, 8)], 4: [(-7, -7), (7, -7), (-7, 7), (7, 7)],
           5: [(-8, -8), (8, -8), (0, 0), (-8, 8), (8, 8)], 6: [(-7, -9), (7, -9), (-7, 0), (7, 0), (-7, 9), (7, 9)]}[pips]
    return (f'<g transform="translate({x} {y}) rotate({rot})"><rect x="{-s/2-3}" y="{-s/2-3}" width="{s+6}" height="{s+6}" rx="9" fill="{RIM}"/>'
            f'<rect x="{-s/2}" y="{-s/2}" width="{s}" height="{s}" rx="7" fill="{CREAM}" stroke="{INK}" stroke-width="4.5"/>'
            f'<path d="M{-s/2+4} {s/2-3} H{s/2-4}" stroke="{CREAM_SH}" stroke-width="4" stroke-linecap="round"/>'
            + "".join(f'<circle cx="{a}" cy="{b}" r="3.2" fill="{INK}"/>' for a, b in pos) + '</g>')

def pose_reroll():
    hl = (CX - 108, CY - 98)
    motion = (f'<path d="M{hl[0]+6} {hl[1]-28} q-24 -30 4 -62" fill="none" stroke="{CREAM}" stroke-width="3.5" stroke-linecap="round" stroke-dasharray="2 9" opacity=".7"/>'
              f'<path d="M{hl[0]+30} {hl[1]-18} q10 -40 46 -50" fill="none" stroke="{CREAM}" stroke-width="3.5" stroke-linecap="round" stroke-dasharray="2 9" opacity=".7"/>')
    top = motion + die(hl[0] - 6, hl[1] - 62, -18, 5) + die(hl[0] + 64, hl[1] - 58, 22, 3)
    eyes = eye_open(EL, (-4, -6)) + eye_happy(ER)
    return rig("reroll", tilt=-6,
               arms=((SH_L, (CX - 132, CY - 30), hl, False), (SH_R, (CX + 126, CY + 54), (CX + 92, CY + 64), True)),
               feet=((CX - 38, CY + 118), (CX + 36, CY + 114)), shoe_rot=(0, -8),
               face=face_std(mouth_k="grin", brows=(-14, 6), eyes=eyes), fuse_d=FUSE_STD, spark_at=TIP_STD, spark_k=.9,
               top=top)

def confetti(seed, n, box, cols):
    rnd = random.Random(seed); out = []
    x0, y0, x1, y1 = box
    for i in range(n):
        x, y = rnd.uniform(x0, x1), rnd.uniform(y0, y1); r = rnd.uniform(0, 360); c = cols[i % len(cols)]
        k = i % 3
        if k == 0: out.append(f'<rect x="{x-5:.1f}" y="{y-2.5:.1f}" width="10" height="5" rx="1" fill="{c}" stroke="{INK}" stroke-width="1.5" transform="rotate({r:.0f} {x:.1f} {y:.1f})"/>')
        elif k == 1: out.append(f'<polygon points="{star(x, y, 6, 2.4, 4, r)}" fill="{c}"/>')
        else: out.append(f'<path d="M{x-7:.1f} {y:.1f} q3.5 -6 7 0 t7 0" fill="none" stroke="{c}" stroke-width="3" stroke-linecap="round" transform="rotate({r:.0f} {x:.1f} {y:.1f})"/>')
    return "".join(out)

def pose_boom():
    burst = (f'<polygon points="{star(206, 214, 196, 150, 16, 4)}" fill="{SPARK}" stroke="{INK}" stroke-width="6" stroke-linejoin="round"/>'
             f'<polygon points="{star(206, 214, 158, 120, 14, 12)}" fill="{SPARK2}"/>'
             f'<polygon points="{star(206, 214, 112, 92, 12, 0)}" fill="{SPARK_CORE}" opacity=".55"/>')
    conf = confetti(7, 34, (8, 8, 392, 392), [CREAM, SPARK, CREAM_SH, SPARK2, METAL_HI, CREAM])
    word = ('<g transform="translate(118 74) rotate(-9)"><text x="0" y="0" text-anchor="middle" '
            'font-family="Bricolage Grotesque, Impact, sans-serif" font-weight="800" font-size="70" font-stretch="75%" '
            f'letter-spacing="1" fill="{CREAM}" stroke="{INK}" stroke-width="10" stroke-linejoin="round" paint-order="stroke">BOOM!</text></g>')
    eyes = eye_happy(EL) + eye_happy(ER)
    d = f"M{P(*FS)} C{P(FS[0]+16, FS[1]-30)} {P(FS[0]+48, FS[1]-18)} {P(FS[0]+44, FS[1]-46)} S{P(FS[0]+26, FS[1]-70)} {P(FS[0]+36, FS[1]-80)}"
    return rig("boom", tilt=0, char_tf="translate(52 82) scale(.74)",
               arms=((SH_L, (CX - 132, CY - 20), (CX - 118, CY - 96), False), (SH_R, (CX + 132, CY - 20), (CX + 126, CY - 92), True)),
               feet=((CX - 48, CY + 112), (CX + 48, CY + 112)), legbend=(-16, 16), shoe_rot=(18, -18),
               face=face_std(mouth_k="big", brows=(-10, 10), bdy=-6, ch=.45, eyes=eyes), fuse_d=d,
               spark_at=(FS[0]+36, FS[1]-82), spark_k=1.15, parts=8,
               back=burst + conf, top=word)

def pose_error():
    d = f"M{P(*FS)} C{P(FS[0]+22, FS[1]-14)} {P(FS[0]+56, FS[1]-2)} {P(FS[0]+60, FS[1]+30)} S{P(FS[0]+66, FS[1]+62)} {P(FS[0]+74, FS[1]+64)}"
    tip = (FS[0] + 76, FS[1] + 64)
    smoke = (f'<g fill="{SMOKE}" stroke="{INK}" stroke-width="3.5" opacity=".95">'
             f'<circle cx="{tip[0]+6}" cy="{tip[1]-14}" r="10"/><circle cx="{tip[0]+18}" cy="{tip[1]-30}" r="8"/><circle cx="{tip[0]+12}" cy="{tip[1]-48}" r="6"/></g>'
             f'<circle cx="{tip[0]}" cy="{tip[1]}" r="4.5" fill="{SMOKE}" stroke="{INK}" stroke-width="3"/>'
             f'<path d="M{tip[0]+30} {tip[1]-62} q6 -6 0 -12 q-6 -6 0 -12" fill="none" stroke="{SMOKE}" stroke-width="3.5" stroke-linecap="round"/>')
    drop = (f'<path d="M{CX+58} {CY-46} q-11 16 0 22 q11 -6 0 -22 z" fill="{DROP}" stroke="{INK}" stroke-width="3.5" stroke-linejoin="round"/>')
    eyes = eye_open(EL, (-5, 4), squint=2) + eye_open(ER, (-5, 4), squint=2)
    face = face_std(mouth_k="wobbly", brows=(-20, 20), bdy=2, ch=.18, eyes=eyes) + drop
    hl = (CX - 106, CY - 96)
    scratch = (f'<path d="M{hl[0]-20} {hl[1]-26} l-8 -6 M{hl[0]-4} {hl[1]-34} l-2 -10 M{hl[0]+14} {hl[1]-30} l5 -8" stroke="{CREAM}" stroke-width="3.5" stroke-linecap="round"/>')
    return rig("error", tilt=7,
               arms=((SH_L, (CX - 136, CY - 22), hl, True), (SH_R, (CX + 112, CY + 50), (CX + 96, CY + 84), True)),
               feet=((CX - 30, CY + 118), (CX + 36, CY + 118)), legbend=(4, 2), shoe_rot=(0, 6),
               face=face, fuse_d=d, fuse_extra=smoke, top=scratch)

POSES = [("front", "Front", "Hero · default", pose_front), ("idle", "Idle", "Empty states · resting", pose_idle),
         ("loading", "Loading", "Fuse sparking · pending tx", pose_loading),
         ("convert", "Convert", "Wrap / unwrap · token ⇄ card", pose_convert),
         ("reroll", "Re-roll", "Dice toss · new pick", pose_reroll),
         ("boom", "Graduation", "BOOM · celebration", pose_boom), ("error", "Error", "Fuse fizzled · sheepish", pose_error)]

if __name__ == "__main__":
    os.makedirs(SVG_DIR, exist_ok=True)
    for key, _, _, fn in POSES:
        open(os.path.join(SVG_DIR, f"mascot-{key}.svg"), "w").write(fn())
    print("svgs ok", [p[0] for p in POSES])
