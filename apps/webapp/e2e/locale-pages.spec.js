// A translated page has to stay translated, online and off (APP-115 §5).
//
// The site publishes one page per language -- /de.html, /ja.html -- each
// declaring its own `<html lang>` and the hreflang ring. Two things used to
// undo that once the app ran:
//
//   - the app chose its language from storage and the browser, never from
//     the page, then wrote the result back into `<html lang>`: an English
//     browser opening /de.html got an English app on a page a crawler had
//     indexed as German;
//   - the service worker stored every navigation as ./index.html, so after
//     visiting /de.html the offline English home page was German.
//
// The locale pages are made here from the built shell, because the real
// ones live in the private site repo. What matters is only what they
// declare: a `lang` and the ring. URLs are `.html`, as openphotoid.com
// serves them -- it has no clean-URL fallback, /privacy is a 404.
//
// `SITE_ROOT` points the suite at another build, which is how the old one
// is shown failing: SITE_ROOT=/tmp/old-dist npx playwright test locale-pages
import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { cpSync, mkdtempSync, readFileSync, writeFileSync, statSync, createReadStream } from "node:fs";
import { tmpdir } from "node:os";
import { join, extname, normalize } from "node:path";
import { here } from "./helpers.mjs";

const DIST = process.env.SITE_ROOT ?? join(here, "..", "dist");
const PAGES = { en: "index.html", de: "de.html", ja: "ja.html" };

let server;
let base;

test.beforeAll(async () => {
  const root = mkdtempSync(join(tmpdir(), "opid-locales-"));
  cpSync(DIST, root, { recursive: true, filter: (src) => !src.includes("/models") });
  const shell = readFileSync(join(DIST, "index.html"), "utf8");
  const port = 5199 + 20;
  const origin = `http://localhost:${port}`;
  const ring =
    Object.entries(PAGES)
      .map(([code, file]) => `<link rel="alternate" hreflang="${code}" href="${origin}/${file === "index.html" ? "" : file}" />`)
      .join("\n") + `\n<link rel="alternate" hreflang="x-default" href="${origin}/" />\n`;
  writeFileSync(join(root, "bare.html"), shell);
  for (const [code, file] of Object.entries(PAGES)) {
    const html = shell
      .replace(/<html lang="[^"]*"/, `<html lang="${code}"`)
      .replace("</head>", `${ring}<meta name="locale-page" content="${code}" />\n</head>`);
    writeFileSync(join(root, file), html);
  }
  const types = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm", ".webmanifest": "application/manifest+json", ".svg": "image/svg+xml", ".png": "image/png" };
  server = createServer((req, res) => {
    let path = decodeURIComponent(new URL(req.url, origin).pathname);
    if (path.endsWith("/")) path += "index.html";
    const file = normalize(join(root, path));
    try {
      if (!file.startsWith(root) || !statSync(file).isFile()) throw 0;
    } catch {
      res.writeHead(404).end("not found");
      return;
    }
    res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream", "cache-control": "no-store" });
    createReadStream(file).pipe(res);
  });
  await new Promise((r) => server.listen(port, r));
  base = origin;
});

test.afterAll(() => server?.close());

const promise = (page) => page.locator("#app").getByText(/Free, all of it|Kostenlos, alles|すべて無料/).first();

test("an English browser on /de.html gets a German app, and lang stays de", async ({ browser }) => {
  const ctx = await browser.newContext({ locale: "en-US", serviceWorkers: "block" });
  const page = await ctx.newPage();
  // A stored English choice from an earlier visit must not win either.
  await page.addInitScript(() => localStorage.setItem("openphotoid.lang", "en"));
  await page.goto(`${base}/de.html`);
  await expect(promise(page)).toHaveText("Kostenlos, alles");
  expect(await page.evaluate(() => document.documentElement.lang)).toBe("de");
  await ctx.close();
});

test("a page that names no translations still follows the browser", async ({ browser }) => {
  const ctx = await browser.newContext({ locale: "ja-JP", serviceWorkers: "block" });
  const page = await ctx.newPage();
  // bare.html is the shell as built: lang="en", no ring. That "en" is a
  // default, not a choice, so a Japanese browser still gets Japanese.
  await page.goto(`${base}/bare.html`);
  await expect(promise(page)).toHaveText("すべて無料");
  await ctx.close();
});

test("the picker goes to the chosen language's page instead of relabelling this one", async ({ browser }) => {
  const ctx = await browser.newContext({ locale: "en-US", serviceWorkers: "block" });
  const page = await ctx.newPage();
  await page.goto(`${base}/`);
  await expect(promise(page)).toHaveText("Free, all of it");
  await page.locator("label.lang select").first().selectOption("ja");
  await page.waitForURL(`${base}/ja.html`, { timeout: 10_000 });
  await expect(promise(page)).toHaveText("すべて無料");
  expect(await page.evaluate(() => document.documentElement.lang)).toBe("ja");
  await ctx.close();
});

test("offline, / is still the English page after /de.html was visited", async ({ browser }) => {
  const ctx = await browser.newContext({ locale: "en-US" });
  const page = await ctx.newPage();
  await page.goto(`${base}/`);
  await page.evaluate(() => navigator.serviceWorker.ready);
  // Twice, so both pages go through a controlled navigation.
  await page.reload();
  await page.evaluate(() => navigator.serviceWorker.ready);
  // Which document came back is read from a marker in its <head>, not from
  // the app's text: this test is about the cache, and must not fail on the
  // language choice the test above covers.
  const marker = () => page.evaluate(() => document.querySelector('meta[name="locale-page"]')?.content);
  await page.goto(`${base}/de.html`);
  expect(await marker()).toBe("de");
  await ctx.setOffline(true);
  await page.goto(`${base}/`);
  const offline = await page.evaluate(() => ({
    lang: document.documentElement.lang,
    marker: document.querySelector('meta[name="locale-page"]')?.content,
  }));
  expect(offline).toEqual({ lang: "en", marker: "en" });
  // And the German page is still German offline, from its own entry.
  await page.goto(`${base}/de.html`);
  expect(await marker()).toBe("de");
  await ctx.close();
});
