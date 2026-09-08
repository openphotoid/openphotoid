import { test, expect } from "@playwright/test";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { FIXTURES, fixture, here, openPhoto, waitForResult, jpegSize } from "./helpers.mjs";

const SPEC_COUNT = 21; // data/specs/*.json
const CHECKS = [
  "face count", "head height", "eye line", "centering", "roll", "inter eye distance",
  "blur", "lighting symmetry", "background uniformity", "resolution", "file size",
];

test.describe("home", () => {
  test("opens on the promise and the coverage count", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("Passport and ID photos that pass");
    await expect(page.getByText(`${SPEC_COUNT} documents from`)).toBeVisible();
    await expect(page.locator("header .brand")).toHaveText(/OpenPhotoId/);
    for (const promise of ["Free, all of it", "Your photo stays here", "Works with no signal"]) {
      await expect(page.getByText(promise, { exact: true })).toBeVisible();
    }
    // Nothing on the product's pages names the platform.
    await expect(page.locator("body")).not.toContainText("OpenApps");
  });

  test("Simplified Chinese switches every string and survives a reload", async ({ page }) => {
    await page.goto("/");
    await page.locator("header select").selectOption("zh-CN");
    await expect(page.locator("h1")).toContainText(/[一-鿿]/);
    await expect(page.locator("h1")).not.toContainText("Passport");
    await page.reload();
    await expect(page.locator("h1")).toContainText(/[一-鿿]/);
    expect(await page.evaluate(() => document.documentElement.lang)).toBe("zh-CN");
  });
});

test.describe("picker and coverage", () => {
  test("search finds a document by country name, code, id and Chinese name", async ({ page }) => {
    await page.goto("/#/pick");
    // Located by type, not placeholder: the placeholder is translated when
    // the language switches below.
    const search = page.locator('input[type="search"]');
    const rows = page.locator("ul.list button");
    await expect(rows).toHaveCount(SPEC_COUNT);
    await search.fill("united states");
    await expect(rows.first()).toContainText("Passport");
    const us = await rows.count();
    expect(us).toBeGreaterThanOrEqual(2);
    await search.fill("us-passport");
    await expect(rows).toHaveCount(1);
    await search.fill("zzzz");
    await expect(page.getByText(/Nothing matches/)).toBeVisible();
    await page.locator("header select").selectOption("zh-CN");
    await search.fill("美国");
    await expect(rows).toHaveCount(us);
  });

  test("the coverage page lists every spec with its source and check date", async ({ page }) => {
    await page.goto("/#/coverage");
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(SPEC_COUNT);
    await expect(page.locator('a:has-text("Official source")')).toHaveCount(SPEC_COUNT);
    await expect(rows.filter({ hasText: /600\s*×\s*600/ }).first()).toBeVisible();
    // Every source link opens elsewhere without handing the page a referrer.
    const rels = await page.locator("tbody a").evaluateAll((as) => as.map((a) => a.rel));
    expect(rels.every((r) => r.includes("noreferrer"))).toBe(true);
  });
});

test.describe("the pipeline on real photos", () => {
  test("a real portrait becomes a US passport photo that passes, and nothing is sent anywhere", async ({ page, baseURL }) => {
    const external = [];
    page.on("request", (r) => {
      const u = r.url();
      if (!u.startsWith(baseURL) && !u.startsWith("data:") && !u.startsWith("blob:")) external.push(u);
    });

    await openPhoto(page, FIXTURES.obama, "us-passport");
    const t0 = Date.now();
    const result = await waitForResult(page);
    const wall = Date.now() - t0;

    console.log(`provider=${result.provider} threads=${await page.evaluate(() => globalThis.__openphotoid?.threads)} wall=${wall}ms stages=${JSON.stringify(result.timings)}`);
    console.log("checks: " + result.checks.map((c) => `${c.name}=${c.status} (${c.detail})`).join("; "));
    expect(result.checks.map((c) => c.name)).toEqual(CHECKS);
    expect(["Passes every check", "Passes, with cautions"]).toContain(result.overall);
    for (const c of result.checks) expect(c.detail.length, c.name).toBeGreaterThan(0);

    // The export is the size the document demands, under its KB ceiling.
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("button", { name: "JPEG" }).click(),
    ]);
    expect(download.suggestedFilename()).toBe("us-passport.jpg");
    const bytes = readFileSync(await download.path());
    expect(jpegSize(bytes)).toEqual({ width: 600, height: 600 });
    expect(bytes.length).toBeLessThanOrEqual(240 * 1024);
    expect(bytes.length).toBeGreaterThan(20 * 1024);

    // A slider drag re-renders without a model run and changes the output.
    await page.locator('button.head:has-text("Position")').click();
    const before = await page.locator('img[alt="Result"]').getAttribute("src");
    const head = page.locator('input[type="range"]').first();
    await head.evaluate((el) => {
      el.value = el.max;
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await expect(page.locator('img[alt="Result"]')).not.toHaveAttribute("src", before);
    const after = await waitForResult(page);
    console.log(`slider re-render: render=${after.timings.render}ms validate=${after.timings.validate}ms`);
    // A slider move must feel live: the cheap half stays well under a second.
    expect(after.timings.render).toBeLessThan(600);

    // Formal clothes: choosing a jacket must reach the pipeline. It did
    // not, once — the pill lit up and nothing else happened — which the
    // screenshot pass caught and this now guards.
    await page.locator('button.head:has-text("Formal clothes")').click();
    const plain = await page.locator('img[alt="Result"]').getAttribute("src");
    await page.getByRole("radio", { name: "Jacket and tie" }).click();
    await expect(page.locator(".colors input[type=color]")).toHaveCount(3);
    await expect(page.locator('img[alt="Result"]')).not.toHaveAttribute("src", plain);
    await page.getByRole("radio", { name: "Keep my clothes" }).click();
    await expect(page.locator(".colors input[type=color]")).toHaveCount(0);

    // The usable-time claim: the whole first run, model sessions included.
    expect(wall).toBeLessThan(120_000);
    expect(external).toEqual([]);
  });

  test("three NASA portraits: studio side-lighting is what the checklist objects to", async ({ page }) => {
    // Official astronaut portraits are lit from one side, as portraits
    // are. Two of the three fail only "lighting symmetry" (25% left/right
    // luminance difference; the fail line is 25.5%), the third warns at
    // 21%. Everything else passes or warns. Encoded here so a threshold
    // change is a deliberate one.
    const expected = {
      [FIXTURES.nasa[0]]: { overall: "Does not pass yet", failing: ["lighting symmetry"] },
      [FIXTURES.nasa[1]]: { overall: "Passes, with cautions", failing: [] },
      [FIXTURES.nasa[2]]: { overall: "Does not pass yet", failing: ["lighting symmetry"] },
    };
    const verdicts = [];
    for (const file of FIXTURES.nasa) {
      await openPhoto(page, file, "us-passport");
      const r = await waitForResult(page);
      const failing = r.checks.filter((c) => c.status === "fail").map((c) => c.name);
      verdicts.push({ file, overall: r.overall, failing, notPassing: r.checks.filter((c) => c.status !== "pass"), timings: r.timings });
      expect(r.checks).toHaveLength(CHECKS.length);
      expect(r.checks.find((c) => c.name === "face count").status, file).toBe("pass");
      expect({ overall: r.overall, failing }, file).toEqual(expected[file]);
    }
    console.log(JSON.stringify(verdicts, null, 1));
  });

  test("a photo with no face says so and offers another try", async ({ page }) => {
    await openPhoto(page, FIXTURES.noFace, "us-passport");
    await expect(page.getByText(/No face found/)).toBeVisible({ timeout: 150_000 });
    await expect(page.getByRole("button", { name: "Use a different photo" })).toBeVisible();
    await expect(page.locator("ul.checks")).toHaveCount(0);
  });

  test("batch processes several photos and writes the per-photo CSV report", async ({ page }) => {
    await page.goto("/#/batch");
    await page.locator("main select").selectOption("us-passport");
    await page.locator('input[type="file"][multiple]').setInputFiles(FIXTURES.nasa.map(fixture));
    await page.getByRole("button", { name: /Process 3/ }).click();
    const items = page.locator("ul.results li");
    await expect(items).toHaveCount(3, { timeout: 170_000 });
    await expect(page.getByRole("button", { name: /Process 3/ })).toBeVisible();
    await expect(page.locator("ul.results li.failed")).toHaveCount(0);
    await expect(items.locator("img")).toHaveCount(3);

    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("button", { name: /check report/ }).click(),
    ]);
    const csv = readFileSync(await download.path(), "utf8").trim().split("\n");
    expect(csv).toHaveLength(4);
    const header = csv[0].split(",").map((s) => s.replaceAll('"', ""));
    expect(header.slice(0, 3)).toEqual(["file", "overall", "detail"]);
    expect(header.slice(3)).toEqual(CHECKS.map((c) => c.replaceAll(" ", "_")));
    // Same photos as the studio test, same verdicts, one row each.
    expect(csv.slice(1).map((l) => l.split(",")[1])).toEqual(['"fail"', '"warn"', '"fail"']);
    for (const line of csv.slice(1)) expect(line).toMatch(/^"nasa-.*","(pass|warn|fail)","",("(pass|warn|fail)",?){11}$/);
  });
});

test.describe("offline and privacy", () => {
  test("after one visit the app opens with no network and says it is ready", async ({ page, context }) => {
    await openPhoto(page, FIXTURES.obama, "us-passport");
    await waitForResult(page);
    // The worker that installed on the first load controls the page only
    // after a reload; that is the moment the badge is allowed to appear.
    await page.goto("/");
    await page.waitForFunction(() => navigator.serviceWorker?.controller);
    await page.reload();
    await expect(page.getByText("Ready to work offline")).toBeVisible();

    await context.setOffline(true);
    await page.reload();
    await expect(page.locator("h1")).toContainText("Passport and ID photos that pass");
    await expect(page.getByText(`${SPEC_COUNT} documents from`)).toBeVisible();
    await page.goto("/#/coverage");
    await expect(page.locator("tbody tr")).toHaveCount(SPEC_COUNT);
    await context.setOffline(false);
  });

  test("from the second visit the CPU path runs multi-threaded", async ({ page }) => {
    // The first load is not controlled by the service worker, so it is
    // single-threaded; the isolation headers the worker adds take effect
    // on the next navigation. Both runs are measured on the same photo.
    await openPhoto(page, FIXTURES.nasa[1], "us-passport");
    const first = await waitForResult(page);
    const before = await page.evaluate(() => ({ isolated: globalThis.crossOriginIsolated, threads: globalThis.__openphotoid?.threads }));
    await page.goto("/");
    await page.waitForFunction(() => navigator.serviceWorker?.controller);
    await page.reload();
    const isolated = await page.evaluate(() => globalThis.crossOriginIsolated);
    await openPhoto(page, FIXTURES.nasa[1], "us-passport");
    const second = await waitForResult(page);
    const after = await page.evaluate(() => ({ isolated: globalThis.crossOriginIsolated, threads: globalThis.__openphotoid?.threads }));
    console.log(`first visit: ${JSON.stringify(before)} matting=${first.timings.matting}ms detail=${first.timings.detail}ms`);
    console.log(`second visit: ${JSON.stringify(after)} matting=${second.timings.matting}ms detail=${second.timings.detail}ms`);
    expect(before.isolated).toBe(false);
    expect(isolated).toBe(true);
    expect(after.threads).toBeGreaterThan(1);
    expect(second.provider).toBe("cpu");
    expect(second.timings.matting).toBeLessThan(first.timings.matting);
  });

  test("the shipped bundle names no third-party endpoint", () => {
    const assets = join(here, "..", "dist", "assets");
    const files = readdirSync(assets).filter((f) => /\.(js|css)$/.test(f));
    expect(files.length).toBeGreaterThan(3);
    // svelte.dev: the Svelte runtime's error messages end with a docs
    // link; it is text in a thrown Error, never a request.
    // gist.github.com: a help link inside the vendored account bundle's
    // Nostr sign-in text. Text in a component, never a request.
    // The product's own hostnames are not third parties.
    const allowed = /^(localhost|www\.w3\.org|schema\.org|svelte\.dev|gist\.github\.com|[a-z]+\.openphotoid\.com)$/;
    // onnxruntime-web's own chunks carry the CDN it would fetch its wasm
    // from by default. src/lib/ort.js points it at ./ort/ instead and the
    // "nothing is sent anywhere" test proves the CDN is never contacted,
    // so this is allowed *only* in those two chunks.
    const ortOnly = /^(cdn\.jsdelivr\.net|web\.dev)$/;
    for (const f of files) {
      const text = readFileSync(join(assets, f), "utf8");
      const hosts = new Set([...text.matchAll(/https?:\/\/([a-z0-9.-]+)/gi)].map((m) => m[1].toLowerCase()));
      const bad = [...hosts].filter((h) => !allowed.test(h) && !(f.startsWith("ort.") && ortOnly.test(h)));
      expect(bad, `${f} references ${bad.join(", ")}`).toEqual([]);
    }
  });
});

test.describe("diagnostics", () => {
  test("the on-device test page runs the pipeline on the shipped sample and reports what ran", async ({ page }) => {
    await page.goto("/#/diagnostics/run");
    await expect(page.getByTestId("diag-result")).toBeVisible({ timeout: 150_000 });
    const text = await page.getByTestId("diag-result").innerText();
    expect(text).toMatch(/Ran on\s+(cpu|webgpu)/);
    expect(text).toMatch(/matting\s+\d+ ms/);
    await expect(page.locator("[data-testid=diag-result] ul.checks li")).toHaveCount(11);
    await expect(page.locator("[data-testid=diag-result] img")).toBeVisible();
  });
});

// ---------------------------------------------------------------- account
//
// Every failure here is silent: a missing CORS origin, a configure() that
// never ran, a refactor that reintroduces the shared backend's hostname —
// none of them throw, and all of them leave a page that renders perfectly
// and never signs anyone in. No real Google or wallet sign-in is driven;
// the assertions sit on everything either side of the identity provider.
const AUTH_HOST = "https://auth.openphotoid.com";
const GATEWAY_HOST = "https://gateway.openphotoid.com";

test.describe("the account", () => {
  test("the masking is a static property of the source", () => {
    const src = join(here, "..", "src");
    const walk = (d) => readdirSync(d, { withFileTypes: true }).flatMap((e) =>
      e.isDirectory() ? (e.name === "vendor" || e.name === "wasm" ? [] : walk(join(d, e.name))) : [join(d, e.name)]);
    const offenders = walk(src)
      .filter((f) => /\.(js|svelte|html|css)$/.test(f) && !f.endsWith(join("lib", "openapps.js")))
      .filter((f) => /openapps\.network/.test(readFileSync(f, "utf8")));
    expect(offenders).toEqual([]);
    const config = readFileSync(join(src, "lib", "openapps.js"), "utf8");
    expect(config.match(/OPENAPPS_BASE_URL\s*=\s*"([^"]+)"/)?.[1]).toBe(AUTH_HOST);
    expect(config.match(/OPENAPPS_GATEWAY_URL\s*=\s*"([^"]+)"/)?.[1]).toBe(GATEWAY_HOST);
  });

  test("both hosts answer, the app origin is allowed, and the auth host accepts its own /signin", async ({ baseURL }) => {
    for (const host of [AUTH_HOST, GATEWAY_HOST]) {
      const res = await fetch(`${host}/healthz`);
      expect(res.ok, host).toBe(true);
    }
    const cors = await fetch(`${AUTH_HOST}/v1/payments/packages`, { headers: { Origin: baseURL } });
    expect(cors.headers.get("access-control-allow-origin")).toBe(baseURL);
    const rt = await fetch(`${AUTH_HOST}/v1/auth/oidc/google/start?return_to=${encodeURIComponent(AUTH_HOST + "/signin")}`, { redirect: "manual" });
    expect(rt.status).toBe(307);
    const methods = await (await fetch(`${AUTH_HOST}/v1/auth/methods`)).json();
    expect(Object.values(methods).some(Boolean)).toBe(true);
  });

  test("the account page reaches the product's host and never names the platform", async ({ page }) => {
    const requests = [];
    page.on("request", (r) => requests.push(r.url()));
    await page.goto("/");
    // The control: round, icon-only, top right, on every route.
    const control = page.getByRole("button", { name: "Account" });
    await expect(control).toBeVisible();
    const box = await control.boundingBox();
    const vw = page.viewportSize().width;
    expect(box.x).toBeGreaterThan(vw / 2);
    expect(box.y).toBeLessThan(80);
    await control.click();
    await page.waitForURL(/#\/account/);
    await expect(page.getByTestId("account-panel")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(/Could not reach/)).toHaveCount(0);
    await expect(page.getByText(/unlocks nothing/)).toBeVisible();
    expect(requests.some((u) => u.startsWith(AUTH_HOST))).toBe(true);
    expect(requests.filter((u) => /openapps\.network/.test(u))).toEqual([]);
    // Visible text, shadow roots included: the default heading lives inside
    // the login element's own shadow DOM.
    const visible = await page.evaluate(() => [document.body.innerText,
      ...[...document.querySelectorAll("*")].filter((e) => e.shadowRoot).map((e) => e.shadowRoot.textContent)].join(" "));
    expect(visible).not.toMatch(/openapps/i);
    expect(visible).toMatch(/Sign in to OpenPhotoId/);
    const mark = await page.evaluate(() => document.querySelector("openapps-login")?.shadowRoot?.querySelector(".mark")?.textContent?.trim());
    expect(mark).toBe("▣");
    await expect(page.locator("footer a, nav.footer a").filter({ hasText: /account/i })).toHaveCount(0);
  });

  test("a sign-in return in the fragment reaches the account view and is exchanged", async ({ page }) => {
    // A fresh load, as a real cross-origin return is: the component mounts
    // and completeRedirect() runs from its connectedCallback.
    const exchanges = [];
    page.on("request", (r) => { if (/\/v1\/auth\/oidc\/exchange/.test(r.url())) exchanges.push(r.url()); });
    await page.goto("/#code=not-a-real-code&state=probe");
    await expect(page.getByTestId("account-panel")).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(1500);
    expect(exchanges.length, "the junk code must at least be attempted").toBeGreaterThan(0);
    expect(exchanges.every((u) => u.startsWith(AUTH_HOST))).toBe(true);
    // Locators pierce shadow DOM, and the login panel has a heading of its
    // own, so ask for the page's exactly.
    await expect(page.getByRole("heading", { name: "Account", exact: true })).toBeVisible();
  });
});
