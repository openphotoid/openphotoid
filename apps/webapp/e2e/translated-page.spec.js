// The app inside a translated page (APP-177).
//
// openphotoid.com publishes the same page in eight languages: /de.html is a
// German document, one of an hreflang ring, and the page's own globe switches
// between them by navigating. Two things follow for the app inside it, and
// both were fixed in source and then sat unshipped while the language pages
// went live:
//
//   5.1  the interface follows the page, not the browser or a stored choice
//   5.2  each page is cached under its own URL, so offline serves the page
//        that was asked for rather than whichever was visited last
//
// These run only against the composed site; the app's bare shell publishes no
// translations and keeps its own control, which app.spec.js covers.
import { test, expect } from "@playwright/test";

const GERMAN = "Pass- und Ausweisfotos";
const ENGLISH = "Passport and ID photos";

async function translatedSiteOrSkip(page) {
  const res = await page.goto("/de.html").catch(() => null);
  test.skip(!res || !res.ok(), "no /de.html here — this root is the app's bare shell");
}

test.describe("the app inside a translated page", () => {
  test("German page, English browser: the interface is German", async ({ page }) => {
    await translatedSiteOrSkip(page);
    expect(await page.evaluate(() => document.documentElement.lang)).toBe("de");
    await expect(page.locator("#app")).toContainText("Foto aufnehmen");
    await expect(page.locator("#app")).not.toContainText("Take a photo");
  });

  // The page writes its language into the app's storage as a stopgap, so the
  // interface follows even with the old bundle — until storage is refused,
  // which is a private window, blocked site data, or a browser with cookies
  // off. Then only reading `<html lang>` gets it right.
  test("still German when the page cannot write to storage", async ({ page }) => {
    await translatedSiteOrSkip(page);
    await page.addInitScript(() => {
      const refuse = () => {
        throw new DOMException("blocked", "SecurityError");
      };
      Object.defineProperty(window, "localStorage", {
        configurable: true,
        get: () => ({ getItem: refuse, setItem: refuse, removeItem: refuse }),
      });
    });
    await page.goto("/de.html");
    await expect(page.locator("#app")).toContainText("Foto aufnehmen");
    await expect(page.locator("#app")).not.toContainText("Take a photo");
  });

  // A stored choice from an earlier visit must not put an English app inside
  // a German document: the page the reader asked for outranks it.
  test("a stored English choice does not override the German page", async ({ page }) => {
    await translatedSiteOrSkip(page);
    await page.evaluate(() => localStorage.setItem("openphotoid.lang", "en"));
    await page.goto("/de.html");
    await expect(page.locator("#app")).toContainText("Foto aufnehmen");
  });

  test("the page's globe is the only language control, and it changes the URL", async ({ page }) => {
    await translatedSiteOrSkip(page);
    await page.goto("/");
    await expect(page.locator("#app-controls-slot select, header .lang select")).toHaveCount(0);
    // The globe is a <details>: open it, then take the German link inside.
    await page.locator("header details.lang-switch summary").click();
    await page.locator('header details.lang-switch a[hreflang="de"]').click();
    await page.waitForURL(/\/de\.html$/);
    await expect(page.locator("#app")).toContainText("Foto aufnehmen");
  });

  // 5.2. Every navigation used to be cached as "./index.html", so the offline
  // copy of any page was whichever language was loaded last. Visiting German
  // and then English is what makes that visible.
  test("offline, each language page is the one that comes back", async ({ page, context }) => {
    await translatedSiteOrSkip(page);
    await page.goto("/de.html");
    await page.waitForFunction(() => navigator.serviceWorker.controller !== null, null, { timeout: 30_000 });
    await page.goto("/de.html"); // once more, now that the worker is in charge
    await page.goto("/"); // the English page is cached second
    await page.waitForTimeout(500); // the cache write is not awaited by the response

    await context.setOffline(true);
    await page.goto("/de.html");
    await expect(page.locator("h1")).toContainText(GERMAN);
    await page.goto("/");
    await expect(page.locator("h1")).toContainText(ENGLISH);
    await context.setOffline(false);
  });
});
