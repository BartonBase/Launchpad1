// node render.js  -> renders palette.html to ../palette.png
const { chromium } = require('/usr/local/lib/pnpm/5/.pnpm/playwright-core@1.59.1/node_modules/playwright-core');
const fs = require('fs'), path = require('path');
const ROOT = path.resolve(__dirname, '..');
const FONTS = `<style>@font-face{font-family:"Bricolage Grotesque";src:local("Bricolage Grotesque")}</style>`;
(async () => {
  const b = await chromium.launch({ executablePath: '/usr/bin/google-chrome', args: ['--no-sandbox'] });
  const only = process.argv[2];
  for (const [html, out] of [['palette.html', 'palette.png']]) {
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
