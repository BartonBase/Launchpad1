"""Armory knight, PLACEHOLDER mascot (code-drawn SVG). Temporary stand-in until Barton's Higgsfield art arrives.
Style follows the archived bomb sheet: thick ink line, flat cel shading, cream highlights, Ember as the only accent.
Run: python3 knight.py  -> ../knight-placeholder.svg"""
import os
INK, CREAM, CREAM2 = "#050507", "#f3efe6", "#cdc7ba"
ST, ST2, ST3, STH = "#c3c7d2", "#8f94a5", "#5f6475", "#eef0f4"   # steel light / shade / deep / highlight
DK, DK2 = "#3b3b49", "#2a2a35"                                   # dark leather / visor
EM, EM2, EM3 = "#FF6A2B", "#E85A1E", "#b8461a"                   # Ember + its own shades only
W = 5  # ink line width (viewBox 400)

def p(d, fill, sw=W, extra=""):
    return f'<path d="{d}" fill="{fill}" stroke="{INK}" stroke-width="{sw}" stroke-linejoin="round" stroke-linecap="round"{extra}/>'
def line(d, col=INK, sw=3):
    return f'<path d="{d}" fill="none" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round"/>'
def c(x, y, r, fill, sw=W):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="{fill}" stroke="{INK}" stroke-width="{sw}"/>'
def e(x, y, rx, ry, fill, sw=W, extra=""):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="{fill}" stroke="{INK}" stroke-width="{sw}"{extra}/>'

parts = []
A = parts.append
# --- plume (Ember), sweeping back to the right
A(p("M196,92 C188,52 214,22 252,20 C276,19 292,32 296,46 C282,40 268,42 260,50 C272,52 280,60 282,70 C266,64 250,66 240,76 C230,84 222,90 214,96 Z", EM))
A(line("M206,84 C214,62 236,44 262,36", EM3, 3.5)); A(line("M220,88 C232,72 248,62 268,58", EM3, 3))
A('<path d="M204,70 C210,50 228,34 250,28" fill="none" stroke="#ffb184" stroke-width="4" stroke-linecap="round" opacity=".9"/>')
# --- legs + boots
for x in (176, 210):
    A(p(f"M{x},300 L{x},340 Q{x},346 {x+7},346 L{x+7+0},346 Q{x+14},346 {x+14},340 L{x+14},300 Z", ST2))
A(p("M160,342 Q160,330 174,330 L190,330 Q194,330 194,336 L194,352 Q194,358 188,358 L160,358 Q154,358 156,350 Z", ST))
A(p("M206,336 Q206,330 210,330 L226,330 Q240,330 240,342 L244,350 Q246,358 240,358 L212,358 Q206,358 206,352 Z", ST))
A(line("M164,350 L190,350", ST2, 4)); A(line("M210,350 L238,350", ST2, 4))
# --- waving arm (viewer's right), behind torso
A(f'<path d="M244,258 Q284,246 300,196" fill="none" stroke="{INK}" stroke-width="30" stroke-linecap="round"/>')
A(f'<path d="M244,258 Q284,246 300,196" fill="none" stroke="{ST2}" stroke-width="20" stroke-linecap="round"/>')
A(line("M276,240 L286,228", INK, 3)); A(line("M290,222 L296,210", INK, 3))
# gauntlet: open waving hand
A(p("M288,196 Q284,170 292,160 Q298,154 302,162 L304,150 Q308,142 314,148 L314,158 Q320,150 325,156 L322,176 Q330,170 334,176 Q330,192 316,202 Q300,208 288,196 Z", ST))
A(line("M300,182 Q306,186 314,184", ST2, 3))
# --- torso (breastplate), cel-shaded right half
A('<clipPath id="torso"><path d="M156,236 Q148,296 160,314 L240,314 Q252,296 244,236 Z"/></clipPath>')
A(p("M156,236 Q148,296 160,314 L240,314 Q252,296 244,236 Z", ST))
A(f'<g clip-path="url(#torso)"><path d="M206,230 Q222,280 210,320 L260,320 L260,230 Z" fill="{ST2}"/><path d="M168,246 Q166,268 172,282" fill="none" stroke="{STH}" stroke-width="6" stroke-linecap="round"/></g>')
A(p("M156,236 Q148,296 160,314 L240,314 Q252,296 244,236 Z", "none"))
A(line("M200,250 L200,290", ST3, 3))
# belt
A(p("M156,292 L244,292 L242,308 L158,308 Z", DK))
A(p("M190,290 L210,290 L210,310 L190,310 Z", CREAM, 4)); A(p("M195,295 L205,295 L205,305 L195,305 Z", DK, 3))
# pauldrons
A(p("M132,262 Q130,234 158,230 Q176,230 178,248 Q160,252 132,262 Z", ST)); A(line("M140,252 Q152,244 166,242", STH, 4))
A(p("M268,262 Q270,234 242,230 Q224,230 222,248 Q240,252 268,262 Z", ST2))
# gorget
A(p("M168,224 Q200,240 232,224 L236,240 Q200,256 164,240 Z", DK))
# --- helmet (big round chibi head)
A('<clipPath id="helm"><circle cx="200" cy="160" r="78"/></clipPath>')
A(c(200, 160, 78, ST))
A(f'<g clip-path="url(#helm)"><path d="M232,70 Q300,150 236,250 L300,250 L300,70 Z" fill="{ST2}"/></g>')
A(c(200, 160, 78, "none"))
A(e(158, 112, 16, 9, STH, 0, ' transform="rotate(-35 158 112)"'))
A(e(150, 130, 4.5, 4.5, STH, 0))
# crest ridge
A(p("M192,84 Q200,80 208,84 L208,134 L192,134 Z", ST2, 4)); A(line("M196,92 L196,128", STH, 3))
# visor opening (open face) + lifted visor brim
A(p("M146,148 Q146,136 160,136 L240,136 Q254,136 254,148 L254,192 Q254,212 230,214 L170,214 Q146,212 146,192 Z", DK2))
A(p("M138,140 Q140,122 162,120 L238,120 Q260,122 262,140 Q262,146 256,146 L144,146 Q138,146 138,140 Z", ST))
A(line("M150,138 L250,138", ST2, 3))
for x in (152, 174, 226, 248):
    A(c(x, 130, 2.6, ST3, 0))
# rivets on cheeks
A(c(134, 176, 4, ST2, 3)); A(c(266, 176, 4, ST2, 3))
# face: eyes, blush, smile
for x in (178, 222):
    A(e(x, 172, 14, 17, CREAM))
    A(c(x + 2, 175, 7.5, INK, 0)); A(c(x + 5, 171, 2.6, "#ffffff", 0))
A(e(160, 196, 9, 5, "#7a4b4f", 0, ' opacity=".95"')); A(e(240, 196, 9, 5, "#7a4b4f", 0, ' opacity=".95"'))
A(p("M188,196 Q200,208 212,196 Q200,200 188,196 Z", CREAM, 3.5))
# --- shield (viewer's left), in front of body, Ember emblem
A('<clipPath id="sh"><path d="M104,246 L184,246 L184,286 Q184,326 144,346 Q104,326 104,286 Z"/></clipPath>')
A(p("M104,246 L184,246 L184,286 Q184,326 144,346 Q104,326 104,286 Z", ST))
A(f'<g clip-path="url(#sh)"><path d="M144,240 L190,240 L190,350 L144,350 Z" fill="{ST2}"/></g>')
A(p("M104,246 L184,246 L184,286 Q184,326 144,346 Q104,326 104,286 Z", "none"))
A(p("M114,256 L174,256 L174,286 Q174,318 144,334 Q114,318 114,286 Z", DK2, 3.5))
# emblem: Ember chevron with a spark
A(p("M126,302 L144,276 L162,302 L152,302 L144,290 L136,302 Z", EM, 3.5))
A(p("M144,262 L147,269 L154,270 L148,274 L150,281 L144,277 L138,281 L140,274 L134,270 L141,269 Z", EM, 2.5))
# hand gripping shield edge
A(p("M176,276 Q190,270 196,280 Q198,292 186,296 Q174,296 172,286 Z", ST)); A(line("M182,282 L190,280", ST2, 3))

svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" width="400" height="400" role="img" aria-labelledby="t d">'
       f'<title id="t">Armory knight (placeholder art)</title><desc id="d">Temporary placeholder mascot, code-drawn, until the Higgsfield mascot art arrives.</desc>'
       + '<g transform="translate(200 200) scale(1.08) translate(-219 -189)">' + "".join(parts) + "</g></svg>\n")
out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "knight-placeholder.svg")
open(out, "w").write(svg); print(os.path.abspath(out), len(svg))
