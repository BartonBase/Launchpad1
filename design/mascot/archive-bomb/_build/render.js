// node render.js  -> renders svg/*.svg to transparent PNGs, plus sheet.html / palette.html if present
const { chromium } = require('/usr/local/lib/pnpm/5/.pnpm/playwright-core@1.59.1/node_modules/playwright-core');
const fs = require('fs'), path = require('path');
const ROOT = path.resolve(__dirname, '..');
const FONTS = `<style>@font-face{font-family:"Bricolage Grotesque";src:local("Bricolage Grotesque")}</style>`;
(async () => {
  const b = await chromium.launch({ executablePath: '/usr/bin/google-chrome', args: ['--no-sandbox'] });
  const only = process.argv[2];
  const p = await b.newPage({ viewport: { width: 400, height: 400 }, deviceScaleFactor: 2 });
  if (!only || only === 'poses') {
    for (const f of fs.readdirSync(path.join(ROOT, 'svg')).filter(f => f.endsWith('.svg'))) {
      const svg = fs.readFileSync(path.join(ROOT, 'svg', f), 'utf8');
      await p.setContent(`<html><head>${FONTS}</head><body style="margin:0;background:transparent">${svg}</body></html>`);
      await p.evaluate(() => document.fonts.ready);
      await p.locator('svg').first().screenshot({ path: path.join(ROOT, f.replace('.svg', '.png')), omitBackground: true });
      console.log('png', f);
    }
  }
  for (const [html, out] of [['sheet.html', 'mascot-sheet.png'], ['palette.html', 'palette.png']]) {
    if (only && only !== html) continue;
    const fp = path.join(__dirname, html); if (!fs.existsSync(fp)) continue;
    const q = await b.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
    await q.goto('file://' + fp, { waitUntil: 'networkidle' });
    await q.evaluate(() => document.fonts.ready); await q.waitForTimeout(200);
    await q.locator('#card').screenshot({ path: path.join(ROOT, out) });
    console.log('png', out);
  }
  await b.close();
})();
