// Store-listing screenshots: the app driven as a person would, at the
// pixel sizes each store asks for, once per language.
//   node e2e/capture-store.mjs <outdir>       (server: node e2e/server.mjs)
//
// They are captures of the app's own UI, which the phone apps share with
// the browser build (apps/mobile packages this same front end), so the
// browser at the device's CSS size and scale factor produces the same
// pixels the app draws — minus the status bar, which stores prefer absent.
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { FIXTURES, fixture } from "./helpers.mjs";

const out = process.argv[2] ?? "../../store/screenshots";
const base = process.env.BASE_URL ?? "http://localhost:5199";
const only = process.env.LANGS?.split(",");

// App Store Connect: 6.9" iPhone 1320×2868 and 13" iPad 2064×2752 are the
// sizes that satisfy every current requirement. Google Play: a phone
// 9:16 at 1080×2400 and a 10" tablet at 1600×2560 (both within its
// 320–3840 px and 2:1 aspect limits).
const DEVICES = {
  "appstore-iphone-6.9": { viewport: { width: 440, height: 956 }, scale: 3, mobile: true },
  "appstore-ipad-13": { viewport: { width: 1032, height: 1376 }, scale: 2, mobile: true },
  "play-phone": { viewport: { width: 360, height: 800 }, scale: 3, mobile: true },
  "play-tablet-10": { viewport: { width: 800, height: 1280 }, scale: 2, mobile: true },
};
const LANGS = ["en", "zh-Hans", "zh-Hant", "ja", "ko", "de", "es", "pt"].filter((l) => !only || only.includes(l));

const browser = await chromium.launch();
async function shoot(page, dir, name) {
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(200);
  await page.screenshot({ path: join(dir, `${name}.png`) });
}

for (const [device, d] of Object.entries(DEVICES)) {
  for (const lang of LANGS) {
    const dir = join(out, device, lang);
    mkdirSync(dir, { recursive: true });
    const context = await browser.newContext({
      viewport: d.viewport,
      deviceScaleFactor: d.scale,
      isMobile: d.mobile,
      hasTouch: d.mobile,
      locale: lang,
    });
    const page = await context.newPage();
    // The phone apps set this attribute (platform.js) and the site copy
    // sharing the document is hidden; the store screenshots show the app.
    await page.addInitScript(() => {
      // Before any module script runs: <html> exists once the parser has
      // created it, which is before scripts, so an observer on the document
      // marks it in time for platform.js to read the attribute at load.
      const mark = () => document.documentElement && (document.documentElement.dataset.native = "");
      if (!mark()) new MutationObserver((_, o) => { if (mark() !== undefined) o.disconnect(); }).observe(document, { childList: true });
    });
    await page.goto(`${base}/?store=1`);
    await page.locator("header select").waitFor();
    await page.locator("header select").selectOption(lang);
    await page.waitForTimeout(300);
    await shoot(page, dir, "01-home");

    await page.goto(`${base}/?store=1#/pick`);
    await page.locator("ul.list button").nth(10).waitFor();
    await shoot(page, dir, "02-documents");

    // The studio, on the reference portrait as a US passport photo.
    await page.goto(`${base}/?store=1`);
    await page.locator("header select").waitFor();
    await page.locator('input[type="file"]').first().setInputFiles(fixture(FIXTURES.obama));
    await page.waitForURL(/#\/pick/);
    await page.locator('input[type="search"]').fill("us-passport");
    await page.locator("ul.list button").first().click();
    await page.locator("ul.checks li").first().waitFor({ timeout: 180_000 });
    await page.waitForFunction(() => !document.querySelector(".veil"));
    await page.evaluate(() => window.scrollTo(0, 0));
    await shoot(page, dir, "03-result");

    // The checklist from its heading down.
    await page.evaluate(() => document.querySelector("ul.checks")?.parentElement?.scrollIntoView({ block: "start" }));
    await shoot(page, dir, "04-checks");

    // The save section: the three ways out, sizes stated.
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await page.waitForTimeout(300);
    await shoot(page, dir, "05-save");

    await context.close();
    console.log("  ", device, lang);
  }
}
await browser.close();
