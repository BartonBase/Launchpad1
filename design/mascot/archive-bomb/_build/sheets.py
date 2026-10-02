"""Builds sheet.html (mascot-sheet.png) and palette.html (palette.png). Run after mascot.py."""
import os
from mascot import POSES, SVG_DIR
HERE = os.path.dirname(os.path.abspath(__file__))
GF = '<link href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wdth,wght@12..96,75..100,600..800&family=Archivo:wdth,wght@62..125,800&family=Geist:wght@400;500;600&family=Geist+Mono:wght@400;500&display=swap" rel="stylesheet">'
svg = lambda k: open(os.path.join(SVG_DIR, f"mascot-{k}.svg")).read()
BASE = """*{box-sizing:border-box}body{margin:0;background:#050506;font-family:Geist,sans-serif;color:#eceef1}
.hd{font-family:"Bricolage Grotesque";font-weight:800;font-stretch:82%;letter-spacing:-.02em;font-variation-settings:"opsz" 96}
.mono{font-family:"Geist Mono",monospace}"""

panels = "".join(f"""<div class="p"><div class="art">{svg(k)}</div><div class="lb"><b class="hd">{n}</b><span class="mono">{d}</span></div><span class="fn mono">mascot-{k}.png</span></div>""" for k, n, d, _ in POSES[1:])
sheet = f"""<!doctype html><html><head><meta charset="utf-8">{GF}<style>{BASE}
#card{{width:1800px;padding:44px 48px 36px;background:#0B0B0F}}
.top{{display:flex;align-items:flex-end;justify-content:space-between;margin-bottom:28px}}
.top h1{{margin:0;font-size:54px;line-height:1}}.top h1 em{{font-style:normal;color:#FF6A2B}}
.top .m{{text-align:right;font-size:13px;color:#8b909a;line-height:1.7}}
.g{{display:grid;grid-template-columns:560px 1fr;gap:20px}}
.p{{position:relative;background:#111116;border:1px solid rgba(255,255,255,.07);border-radius:18px;padding:10px 10px 16px}}
.p .art svg{{width:100%;height:auto;display:block}}
.lb{{display:flex;align-items:baseline;gap:12px;padding:0 10px}}.lb b{{font-size:26px}}.lb span{{font-size:12.5px;color:#8b909a}}
.fn{{position:absolute;bottom:18px;right:16px;font-size:11px;color:#5d626c}}
.big{{display:flex;flex-direction:column;justify-content:space-between;background:radial-gradient(420px 360px at 50% 42%,rgba(255,106,43,.10),transparent 70%),#111116}}
.big .lb b{{font-size:34px}}
.grid6{{display:grid;grid-template-columns:repeat(3,1fr);gap:20px}}
.foot{{display:flex;gap:28px;margin-top:22px;font-size:12.5px;color:#8b909a}}.foot i{{display:inline-block;width:10px;height:10px;border-radius:50%;background:#FF6A2B;margin-right:7px;box-shadow:0 0 10px #FF6A2B}}
</style></head><body><div id="card">
<div class="top"><h1 class="hd">Fuze<em>.</em> mascot sheet <span style="color:#5d626c">v1</span></h1>
<div class="m mono">Working name · comic-ink bomb character · 7 poses<br>Spark #FF6A2B is the only saturated colour</div></div>
<div class="g"><div class="p big"><span class="fn mono">mascot-front.png</span><div class="art" style="margin-top:40px">{svg("front")}</div>
<div class="lb"><b class="hd">Front</b><span class="mono">Hero · default pose</span></div></div>
<div class="grid6">{panels}</div></div>
<div class="foot mono"><span><i></i>Spark = accent</span><span>Ink #050507 · body #2A2A35 · face/gloves #F3EFE6</span><span>BOOM is a celebration burst, never damage</span><span>Never used to imply returns</span></div>
</div></body></html>"""
open(os.path.join(HERE, "sheet.html"), "w").write(sheet)

SW = [("Background", "#08090A", "Page"), ("Surface 1", "#0E0F11", "Inputs, footer"), ("Surface 2", "#131417", "Panels"), ("Surface 3", "#1A1B1F", "Chips, raised"),
      ("Border", "rgba(255,255,255,.07)", "Hairline"), ("Border strong", "rgba(255,255,255,.12)", "Inputs, hover"),
      ("Text", "#ECEEF1", "Primary"), ("Text muted", "#8B909A", "Secondary"), ("Text dim", "#5D626C", "Tertiary, labels")]
ST = [("Success", "#3ECF8E", "Revoked, graduated, up"), ("Warn", "#E8B04B", "Demo / Example tags, pending"), ("Error", "#F25C7A", "Errors, sells, down")]
def chip(n, h, r, dark=False):
    bg = h if not h.startswith("rgba") else f"linear-gradient({h},{h}),#08090A"
    return f'<div class="sw"><div class="c" style="background:{bg}"></div><b>{n}</b><span class="mono">{h}</span><small>{r}</small></div>'
pal = f"""<!doctype html><html><head><meta charset="utf-8">{GF}<style>{BASE}
#card{{width:1440px;padding:44px 48px;background:#0B0B0F;display:grid;grid-template-columns:1fr 1fr;gap:36px}}
h2{{margin:0 0 18px;font-size:13px;font-family:"Geist Mono";font-weight:500;text-transform:uppercase;letter-spacing:.08em;color:#8b909a}}
.acc{{position:relative;height:220px;border-radius:20px;background:#FF6A2B;padding:24px;color:#120805;box-shadow:0 0 0 1px rgba(255,255,255,.08) inset,0 30px 80px -20px rgba(255,106,43,.55);overflow:hidden}}
.acc b{{font-size:44px}}.acc .mono{{font-size:14px;display:block;margin-top:6px}}
.acc img{{position:absolute;right:-6px;bottom:-24px;width:210px}}
.accrow{{display:grid;grid-template-columns:repeat(4,1fr);gap:10px;margin-top:12px}}
.mini{{border-radius:12px;padding:12px;border:1px solid rgba(255,255,255,.07);font-size:12px;color:#8b909a}}.mini b{{display:block;color:#eceef1;font-weight:500;margin-bottom:3px}}
.grid{{display:grid;grid-template-columns:repeat(3,1fr);gap:10px}}
.sw{{background:#111116;border:1px solid rgba(255,255,255,.07);border-radius:12px;padding:8px 8px 10px}}.sw .c{{height:46px;border-radius:7px;border:1px solid rgba(255,255,255,.06);margin-bottom:8px}}
.sw b{{display:block;font-size:13px;font-weight:500}}.sw span{{display:block;font-size:11.5px;color:#8b909a;margin-top:2px}}.sw small{{font-size:11px;color:#5d626c}}
.spec{{background:#111116;border:1px solid rgba(255,255,255,.07);border-radius:18px;padding:26px 28px}}
.spec .h1{{font-size:68px;line-height:.98;margin:8px 0 14px}}.spec .h1 span{{color:#a9adb6}}
.tag{{font-family:"Geist Mono";font-size:11.5px;color:#5d626c;text-transform:uppercase;letter-spacing:.06em}}
.body{{font-size:16px;line-height:1.6;color:#8b909a;max-width:560px}}
.alt{{font-family:Archivo;font-weight:800;font-stretch:72%;font-size:44px;letter-spacing:-.01em;line-height:1;margin:8px 0 0}}
.btns{{display:flex;gap:10px;margin-top:18px}}.btn{{height:40px;padding:0 16px;border-radius:10px;display:inline-flex;align-items:center;font:600 14px Geist}}
.ba{{background:#FF6A2B;color:#120805;box-shadow:0 8px 24px -8px rgba(255,106,43,.6)}}.bg{{border:1px solid rgba(255,255,255,.12);color:#eceef1}}
.chipa{{height:26px;padding:0 10px;border-radius:7px;display:inline-flex;align-items:center;font-size:12px;background:rgba(255,106,43,.12);border:1px solid rgba(255,106,43,.3);color:#FF9A62}}
.fusebar{{position:relative;height:6px;border-radius:99px;background:rgba(255,255,255,.07);margin:26px 0 8px}}
.fusebar i{{position:absolute;left:0;top:0;bottom:0;width:64%;border-radius:99px;background:repeating-linear-gradient(90deg,#8d8678 0 4px,#5f5a50 4px 8px)}}
.fusebar i:after{{content:"";position:absolute;right:-9px;top:50%;width:18px;height:18px;margin-top:-9px;border-radius:50%;background:radial-gradient(circle,#FFF6DF 0 18%,#FFC24D 30%,#FF6A2B 55%,rgba(255,106,43,0) 72%)}}
</style></head><body><div id="card">
<div><h2>Accent · from the fuse spark</h2>
<div class="acc"><b class="hd">Ember</b><span class="mono">#FF6A2B · the only saturated colour</span><img src="../mascot-loading.png"></div>
<div class="accrow"><div class="mini" style="background:#FF9A62;color:#120805;border:0"><b style="color:#120805">Accent text</b><span class="mono">#FF9A62</span></div>
<div class="mini" style="background:#E85A1E;color:#120805;border:0"><b style="color:#120805">Pressed</b><span class="mono">#E85A1E</span></div>
<div class="mini" style="background:rgba(255,106,43,.12)"><b>Tint bg</b><span class="mono">255,106,43 / .12</span></div>
<div class="mini" style="box-shadow:0 0 28px rgba(255,106,43,.45) inset"><b>Glow</b><span class="mono">255,106,43 / .45</span></div></div>
<h2 style="margin-top:28px">Neutrals (Obsidian, unchanged)</h2><div class="grid">{"".join(chip(*x) for x in SW)}</div>
<h2 style="margin-top:28px">Status</h2><div class="grid">{"".join(chip(*x) for x in ST)}</div></div>
<div><h2>Type</h2><div class="spec"><div class="tag">Headline · Bricolage Grotesque ExtraBold · width 82 · opsz 96</div>
<div class="h1 hd">Trade the meme.<br><span>Collect the art.</span></div>
<div class="tag" style="margin-top:20px">UI and body · Geist 400/500/600</div>
<p class="body">On Fuze, a meme token and its NFT collection are one SPL-404 asset. Hold it as tokens or as NFTs, and switch at a fixed rate once the token graduates.</p>
<div class="tag" style="margin-top:6px">Labels and numbers · Geist Mono</div><p class="mono" style="margin:6px 0 0;font-size:14px;color:#eceef1">54.4 of 85 SOL raised · 1,000,000 NOCT = 1 NFT</p>
<div class="fusebar"><i></i></div><div class="mono" style="font-size:11.5px;color:#5d626c">Curve progress as a burning fuse, spark at the tip</div>
<div class="btns"><span class="btn ba">Buy NOCT</span><span class="btn bg">Launch a collection</span><span class="chipa">On curve · 64%</span></div>
<div class="tag" style="margin-top:26px">Alternate headline · Archivo ExtraBold, condensed (width ~72)</div>
<div class="alt">Trade the meme. Collect the art.</div></div></div>
</div></body></html>"""
open(os.path.join(HERE, "palette.html"), "w").write(pal)
print("sheets ok")
