// Screenshots of the built app, driven exactly as a person would drive it.
//   node e2e/capture.mjs <outdir>       (server: node e2e/server.mjs)
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { FIXTURES, fixture } from "./helpers.mjs";

const out = process.argv[2] ?? "screenshots";
const base = process.env.BASE_URL ?? "http://localhost:5199";
mkdirSync(out, { recursive: true });

const browser = await chromium.launch();
const shoot = async (page, name, opts = {}) => {
  await page.evaluate(() => document.fonts.ready);
  // A full-page capture stitches from the current scroll position, and
  // the sticky top bar lands wherever the page was scrolled to.
  if (opts.fullPage) await page.evaluate(() => window.scrollTo(0, 0));
  await page.waitForTimeout(150); // let the last transition settle
  await page.screenshot({ path: join(out, `${name}.png`), fullPage: false, ...opts });
  console.log("  ", name);
};

async function toStudio(page, file, specId) {
  await page.goto(`${base}/`);
  await page.getByText(/documents from/).waitFor();
  await page.locator('input[type="file"]').first().setInputFiles(fixture(file));
  await page.waitForURL(/#\/pick/);
  await page.locator('input[type="search"]').fill(specId);
  await page.locator("ul.list button").first().click();
  await page.locator("ul.checks li").first().waitFor({ timeout: 150_000 });
  await page.locator('img[alt="Result"]').waitFor();
  await page.waitForFunction(() => !document.querySelector(".veil"));
}

for (const [scheme, tag] of [["light", ""], ["dark", "-dark"]]) {
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
    colorScheme: scheme,
  });
  const page = await context.newPage();

  if (scheme === "light") {
    await page.goto(`${base}/`);
    await page.getByText(/documents from/).waitFor();
    await shoot(page, "01-home");

    await page.goto(`${base}/#/pick`);
    await page.locator("ul.list button").nth(20).waitFor();
    await page.locator('input[type="search"]').fill("passport");
    await shoot(page, "02-picker");
  }

  await toStudio(page, FIXTURES.obama, "us-passport");
  await shoot(page, `03-studio${tag}`);

  if (scheme === "light") {
    await page.locator('button.head:has-text("Position")').click();
    await page.locator('button.head:has-text("Background")').click();
    await shoot(page, "04-checklist-panels", { fullPage: true });

    // The drawn jacket on someone already in a suit is invisible; on a
    // spacesuit it is the whole point.
    await toStudio(page, FIXTURES.nasa[1], "uk-passport");
    await page.locator('button.head:has-text("Formal clothes")').click();
    await page.getByRole("radio", { name: "Jacket and tie" }).click();
    await page.locator('input[type="color"]').nth(1).waitFor();
    await page.waitForFunction(() => !document.querySelector(".veil"));
    await page.locator(".preview").scrollIntoViewIfNeeded();
    await shoot(page, "05-garment");

    await page.goto(`${base}/#/batch`);
    await page.locator("main select").selectOption("us-visa-dv");
    await page.locator('input[type="file"][multiple]').setInputFiles(FIXTURES.nasa.map(fixture));
    await page.getByRole("button", { name: /Process 3/ }).click();
    await page.locator("ul.results li img").nth(2).waitFor({ timeout: 170_000 });
    await shoot(page, "06-batch");

    await page.goto(`${base}/#/coverage`);
    await page.locator("tbody tr").nth(20).waitFor();
    await shoot(page, "07-coverage", { fullPage: true });

    // One home screen per language the picker offers, so the guide shows
    // each catalogue on the page rather than in a file.
    for (const [tag, probe] of [
      ["zh-Hans", /个/], ["zh-Hant", /種/], ["ja", /種類/], ["ko", /종/],
      ["de", /Dokumente/], ["es", /documentos/], ["pt", /documentos/],
    ]) {
      await page.goto(`${base}/`);
      await page.locator("header select").selectOption(tag);
      await page.getByText(probe).first().waitFor();
      await shoot(page, tag === "zh-Hans" ? "08-home-zh" : `08-home-${tag}`);
    }
    await page.locator("header select").selectOption("en");

    await page.goto(`${base}/`);
    await page.getByText(/documents from/).waitFor();
    await page.locator('input[type="file"]').first().setInputFiles(fixture(FIXTURES.noFace));
    await page.waitForURL(/#\/pick/);
    await page.locator('input[type="search"]').fill("us-passport");
    await page.locator("ul.list button").first().click();
    await page.getByText(/No face found/).waitFor({ timeout: 150_000 });
    await shoot(page, "09-no-face");
  }
  await context.close();
}

// The form factor the research says this category is won on.
const phone = await browser.newContext({
  viewport: { width: 390, height: 844 },
  deviceScaleFactor: 3,
  isMobile: true,
  hasTouch: true,
  colorScheme: "light",
});
const page = await phone.newPage();
await toStudio(page, FIXTURES.nasa[1], "china-visa-cova");
await shoot(page, "10-phone-studio");
await phone.close();

await browser.close();
