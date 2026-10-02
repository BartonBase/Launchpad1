"""Static a11y checks for generated pages: heading order, one h1, labelled form fields, svg semantics, focusable anchors, landmarks."""
import sys, re
from html.parser import HTMLParser
class P(HTMLParser):
    def __init__(s):
        super().__init__(); s.h=[]; s.inputs=[]; s.labels=set(); s.svg_bad=0; s.svg=0; s.a_nohref=0; s.main=0; s.imgnoalt=0; s.mph=0; s.mph_ok=0; s.btn_noname=[]; s._btn=None
    def handle_starttag(s,t,a):
        a=dict(a)
        if re.fullmatch(r'h[1-6]',t): s.h.append(int(t[1]))
        if t in('input','textarea','select') and a.get('type')!='hidden': s.inputs.append(a)
        if t=='label' and a.get('for'): s.labels.add(a['for'])
        if t=='svg':
            s.svg+=1
            if not(a.get('aria-hidden')=='true' or a.get('role')=='img' and a.get('aria-label')): s.svg_bad+=1
        if t=='a' and 'href' not in a and a.get('tabindex')!='-1': s.a_nohref+=1
        if t=='main': s.main+=1
        if t=='img' and 'alt' not in a: s.imgnoalt+=1
        if 'mph' in (a.get('class') or '').split(): s.mph+=1; s.mph_ok+= ((a.get('role')=='img' and bool(a.get('aria-label'))) or t=='figure')
        if t=='button': s._btn=[a,'']
    def handle_data(s,d):
        if s._btn is not None: s._btn[1]+=d.strip()
    def handle_endtag(s,t):
        if t=='button' and s._btn is not None:
            a,txt=s._btn
            if not txt and not a.get('aria-label'): s.btn_noname.append(a.get('class'))
            s._btn=None
for f in sys.argv[1:]:
    p=P(); p.feed(open(f).read())
    skips=[(p.h[i-1],p.h[i]) for i in range(1,len(p.h)) if p.h[i]-p.h[i-1]>1]
    unl=[i.get('id') or i.get('value') for i in p.inputs if not(i.get('id') in p.labels or i.get('aria-label'))]
    print(f"{f.split('/')[-1]:24s} h1={p.h.count(1)} skips={skips[:4]} unlabeled={unl} svg_unlabeled={p.svg_bad}/{p.svg} a_nohref={p.a_nohref} main={p.main} img_noalt={p.imgnoalt} mascot_ph={p.mph_ok}/{p.mph} btn_noname={p.btn_noname}")
