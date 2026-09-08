// Phone and tablet screenshots from WebKit with device emulation. Not iOS
// Safari — #/diagnostics/run on a simulator is — but the layout and the
// touch flow a phone gets.  node e2e/capture-mobile.mjs <outdir>
import { webkit, devices } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { FIXTURES, fixture } from "./helpers.mjs";

const out = process.argv[2] ?? "screenshots-mobile";
const base = process.env.BASE_URL ?? "http://localhost:5199";
mkdirSync(out, { recursive: true });
const browser = await webkit.launch();

for (const [tag, dev] of [["iphone", devices["iPhone 15"]], ["ipad", devices["iPad Pro 11"]]]) {
  const ctx = await browser.newContext({ ...dev, colorScheme: "light" });
  const page = await ctx.newPage();
  const shoot = async (name, opts = {}) => {
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(150);
    await page.screenshot({ path: join(out, `${tag}-${name}.png`), ...opts });
    console.log("  ", tag, name);
  };
  await page.goto(`${base}/`);
  await page.getByText(/documents from/).waitFor();
  await shoot("01-home");
  await page.locator('input[type="file"]').first().setInputFiles(fixture(FIXTURES.nasa[1]));
  await page.waitForURL(/#\/pick/);
  await page.locator("ul.list button").nth(5).waitFor();
  await shoot("02-picker");
  await page.locator('input[type="search"]').fill("uk-passport");
  await page.locator("ul.list button").first().click();
  await page.locator("ul.checks li").first().waitFor({ timeout: 150_000 });
  await page.locator('img[alt="Result"]').waitFor();
  await page.waitForFunction(() => !document.querySelector(".veil"));
  await shoot("03-studio");
  await page.locator('button.head:has-text("Position")').click();
  await page.evaluate(() => window.scrollTo(0, 0));
  await shoot("04-studio-full", { fullPage: true });
  await page.goto(`${base}/#/batch`);
  await page.locator("main select").waitFor();
  await shoot("05-batch");
  await page.goto(`${base}/#/account`);
  await page.getByTestId("account-panel").waitFor({ timeout: 30_000 });
  await shoot("06-account");
  await page.goto(`${base}/#/diagnostics/run`);
  await page.getByTestId("diag-result").waitFor({ timeout: 150_000 });
  await shoot("07-diagnostics", { fullPage: true });
  await ctx.close();
}
await browser.close();
