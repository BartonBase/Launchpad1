// usage: node crop.js <html> <selector> <out.png>   (element screenshot, 1440x3000 viewport so sticky nav never overlaps)
const { chromium } = require('playwright-core');
(async () => {
  const [f, sel, out] = process.argv.slice(2);
  const b = await chromium.launch({ executablePath: '/usr/bin/google-chrome', args:['--no-sandbox'] });
  const p = await b.newPage({ viewport: { width: 1440, height: 3000 }, deviceScaleFactor: 1 });
  await p.goto('file://' + f, { waitUntil: 'networkidle' });
  await p.evaluate(() => document.fonts.ready); await p.waitForTimeout(300);
  const el = await p.$(sel); await el.screenshot({ path: out });
  console.log(out, JSON.stringify(await el.boundingBox()));
  await b.close();
})();
