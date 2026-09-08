// One module, one mark.
//
// The app's icons and the website's icons come from the same drawing, so the
// browser tab, the home-screen icon and the link-preview card cannot become
// three different pictures of the same product.
//
//   node scripts/make-assets.mjs
//
// Writes into apps/webapp/public/icons/ and ../../../openphotoid-website-deploy/.
//
// The SVG hardcodes its colours on purpose: an <img>-loaded SVG can read
// neither a webfont nor a custom property, so an export context carries
// literals. They are the literals the tokens resolve to — the tile ground is
// the family's, and the frame is --violet (accent slot 3, --app-frame), which
// is what the app's wordmark dot and the site's use.

import { chromium } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const WEB_DIR = dirname(dirname(fileURLToPath(import.meta.url)));
const APP_ICONS = join(WEB_DIR, "public", "icons");
const SITE = join(WEB_DIR, "..", "..", "..", "openphotoid-website-deploy");

const GROUND = "#111111"; // the family tile ground
const PAPER = "#f8f8f7"; // --gray-050
const FACE = "#c1c0b8"; // --gray-300: the silhouette, not a skin tone
const VIOLET = "#8c6cff"; // --violet, accent slot 3

// An ID-photo: a portrait card with the silhouette every form shows, inside
// the crop frame the product draws. Two ideas, because a third dissolves at 16px.
const glyph = `
  <rect x="136" y="96" width="240" height="320" rx="20" fill="${PAPER}"/>
  <circle cx="256" cy="222" r="58" fill="${FACE}"/>
  <path d="M164 400c14-58 46-90 92-90s78 32 92 90z" fill="${FACE}"/>
  <g stroke="${VIOLET}" stroke-width="18" stroke-linecap="round" fill="none">
    <path d="M104 156 L104 108 L152 108"/>
    <path d="M360 108 L408 108 L408 156"/>
    <path d="M408 356 L408 404 L360 404"/>
    <path d="M152 404 L104 404 L104 356"/>
  </g>`;

const iconSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" role="img" aria-label="OpenPhotoId">
  <rect width="512" height="512" rx="112" fill="${GROUND}"/>
${glyph}
</svg>
`;
// Launchers crop a maskable icon to any shape: everything that must survive
// sits inside the middle 80% and the ground bleeds to the edge.
const maskableSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" role="img" aria-label="OpenPhotoId">
  <rect width="512" height="512" fill="${GROUND}"/>
  <g transform="translate(256 256) scale(0.68) translate(-256 -256)">
${glyph}
  </g>
</svg>
`;

async function png(page, svg, size) {
  await page.setViewportSize({ width: size, height: size });
  await page.setContent(
    `<style>html,body{margin:0;padding:0;background:transparent}svg{display:block;width:${size}px;height:${size}px}</style>${svg}`,
  );
  return page.screenshot({ omitBackground: true });
}

// An ICO is a header, one directory entry per image, then the images. Modern
// decoders accept PNG payloads verbatim. A 256px entry is written as 0.
function ico(images) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(images.length, 4);
  const entries = [];
  let offset = 6 + images.length * 16;
  for (const { size, data } of images) {
    const e = Buffer.alloc(16);
    e.writeUInt8(size >= 256 ? 0 : size, 0);
    e.writeUInt8(size >= 256 ? 0 : size, 1);
    e.writeUInt16LE(1, 4);
    e.writeUInt16LE(32, 6);
    e.writeUInt32LE(data.length, 8);
    e.writeUInt32LE(offset, 12);
    entries.push(e);
    offset += data.length;
  }
  return Buffer.concat([header, ...entries, ...images.map((i) => i.data)]);
}

// The card is type. Authored against the real vendored stylesheets and
// photographed; goto() rather than setContent(), because about:blank cannot
// load a file:// stylesheet and the card would come out unstyled.
const CARD = `<!doctype html>
<html class="oa-dark">
<head>
<meta charset="utf-8" />
<link rel="stylesheet" href="tokens/css/colors.css" />
<link rel="stylesheet" href="tokens/css/typography.css" />
<link rel="stylesheet" href="tokens/css/spacing.css" />
<link rel="stylesheet" href="tokens/css/radius.css" />
<link rel="stylesheet" href="tokens/css/fonts.css" />
<style>
  :root { --accent: var(--app-frame); --accent-lift: color-mix(in oklab, var(--accent) 80%, var(--white)); }
  * { box-sizing: border-box; }
  html, body { margin: 0; padding: 0; }
  body { width: 1200px; height: 630px; background: var(--bg-page); color: var(--text-body);
    font-family: var(--font-sans); display: flex; flex-direction: column; justify-content: space-between;
    padding: 72px 80px; -webkit-font-smoothing: antialiased; }
  .brand { display: flex; align-items: center; gap: 18px; font: var(--weight-medium) 34px/1 var(--font-display);
    letter-spacing: var(--logo-tracking); color: var(--text-strong); }
  .brand img { width: 52px; height: 52px; border-radius: 12px; display: block; }
  .brand .prefix { color: var(--text-muted); }
  .brand .dot { color: var(--accent-lift); }
  h1 { font: var(--weight-medium) 74px/1.04 var(--font-display); letter-spacing: var(--tracking-display);
       color: var(--text-strong); margin: 0 0 22px; max-width: 18ch; }
  p { font-size: 27px; line-height: 1.45; color: var(--text-muted); margin: 0; max-width: 44ch; }
  .foot { display: flex; gap: 40px; font-family: var(--font-mono); font-size: 20px;
          letter-spacing: var(--tracking-caps); text-transform: uppercase; color: var(--text-faint); }
  .foot b { color: var(--accent-lift); font-weight: var(--weight-medium); }
</style>
</head>
<body>
  <div class="brand"><img src="favicon.svg" alt="" /><span><span class="prefix">Open</span>PhotoId<span class="dot">.</span></span></div>
  <div>
    <h1>Passport and ID photos that pass.</h1>
    <p>Pick the country, choose a photo, get a file that meets the official rules — made in your browser, never uploaded.</p>
  </div>
  <div class="foot"><span><b>21</b> documents</span><span><b>11</b> official checks</span><span><b>0</b> uploads</span></div>
</body>
</html>
`;

mkdirSync(APP_ICONS, { recursive: true });
mkdirSync(SITE, { recursive: true });
writeFileSync(join(APP_ICONS, "favicon.svg"), iconSvg);
writeFileSync(join(SITE, "favicon.svg"), iconSvg);

const browser = await chromium.launch();
const page = await browser.newPage({ deviceScaleFactor: 1 });
const written = [];
const w = (dir, name, data) => {
  writeFileSync(join(dir, name), data);
  written.push(`${dir === SITE ? "site" : "app "}  ${name.padEnd(22)} ${String(data.length).padStart(7)} bytes`);
};
w(APP_ICONS, "icon-192.png", await png(page, iconSvg, 192));
w(APP_ICONS, "icon-512.png", await png(page, iconSvg, 512));
w(APP_ICONS, "apple-touch-icon.png", await png(page, iconSvg, 180));
w(APP_ICONS, "icon-maskable-512.png", await png(page, maskableSvg, 512));
for (const size of [16, 32, 48, 180, 192, 512]) w(SITE, `icon-${size}.png`, await png(page, iconSvg, size));
w(SITE, "favicon.ico", ico([
  { size: 16, data: await png(page, iconSvg, 16) },
  { size: 32, data: await png(page, iconSvg, 32) },
  { size: 48, data: await png(page, iconSvg, 48) },
]));
const cardPath = join(SITE, ".og-card.html");
writeFileSync(cardPath, CARD);
await page.setViewportSize({ width: 1200, height: 630 });
await page.goto(`file://${cardPath}`);
await page.evaluate(() => document.fonts.ready);
w(SITE, "og-image.png", await page.screenshot({ clip: { x: 0, y: 0, width: 1200, height: 630 } }));
await browser.close();
console.log(written.join("\n"));
