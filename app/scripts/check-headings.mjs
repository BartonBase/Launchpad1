#!/usr/bin/env node
/**
 * a11y check (design/system/a11y.md §5): every route renders exactly one <h1> and never skips a
 * heading level. Runs against a running server: node scripts/check-headings.mjs [baseUrl]
 */
const base = process.argv[2] ?? "http://localhost:3000";
const M = "Dhv3VkqeTYJiahzcVJLmJ1snApdYtQxsAKeUnv5GTNos"; // devnet test launch "Armory Test" (ARMT)
const routes = ["/", "/explore", "/launch", "/launch?type=plain", "/launch?type=burn", "/portfolio", "/trust", "/faq", "/bug-bounty",
  `/t/${M}`, `/t/${M}?panel=capture`, "/t/example-plain", "/t/example-burn", "/t/notamint"];
let bad = 0;
for (const r of routes) {
  const html = await (await fetch(base + r)).text();
  const body = html.replace(/<script[\s\S]*?<\/script>/g, "").replace(/<template[\s\S]*?<\/template>/g, "");
  const levels = [...body.matchAll(/<h([1-6])[\s>]/g)].map((m) => Number(m[1]));
  const h1 = levels.filter((l) => l === 1).length;
  const skips = levels.flatMap((l, i) => (i > 0 && l > levels[i - 1] + 1 ? [`h${levels[i - 1]}→h${l}`] : []));
  const ok = h1 === 1 && skips.length === 0 && levels[0] === 1;
  if (!ok) bad++;
  console.log(`${ok ? "ok  " : "FAIL"} ${r}  h1=${h1}${skips.length ? ` skips=${[...new Set(skips)].join(",")}` : ""}${levels[0] !== 1 ? ` first=h${levels[0]}` : ""}`);
}
process.exit(bad ? 1 : 0);
