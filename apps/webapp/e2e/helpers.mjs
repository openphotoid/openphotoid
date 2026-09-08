import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const here = dirname(fileURLToPath(import.meta.url));
// The reference portrait already lives in the repo's testdata/; the suite
// reads it from there rather than carrying a second 1.2 MB copy.
export const fixture = (name) =>
  name === "portrait-obama.jpg" ? join(here, "..", "..", "..", "testdata", name) : join(here, "fixtures", name);

export const FIXTURES = {
  obama: "portrait-obama.jpg",
  nasa: [
    "nasa-official-portrait-of-nasa-astronaut-jack.jpg",
    "nasa-official-portrait-of-nasa-astronaut-jess.jpg",
    "nasa-official-portrait-of-esa-european-space-.jpg",
  ],
  noFace: "no-face-landscape.png",
};

/** Home → choose a photo → the picker → one document → the studio. */
export async function openPhoto(page, file, specId = "us-passport") {
  await page.goto("/");
  await page.getByText(/documents from/).waitFor();
  await page.locator('input[type="file"]').first().setInputFiles(fixture(file));
  await page.waitForURL(/#\/pick/);
  await page.getByPlaceholder(/Search country or document/i).fill(specId);
  const row = page.locator("ul.list button").first();
  await row.waitFor();
  await row.click();
  await page.waitForURL(new RegExp(`#/studio/${specId}`));
}

/** Wait for the checklist, then return report state, timings and provider. */
export async function waitForResult(page, timeout = 150_000) {
  await page.locator("ul.checks li").first().waitFor({ timeout });
  await page.locator('img[alt="Result"]').waitFor({ timeout });
  return page.evaluate(() => {
    const checks = [...document.querySelectorAll("ul.checks li")].map((li) => ({
      name: li.querySelector(".cname").textContent.trim(),
      status: li.dataset.status,
      detail: li.querySelector(".cdetail").textContent.trim(),
    }));
    const overall = document.querySelector(".checkhead span").textContent.trim();
    const timings = Object.fromEntries(
      performance
        .getEntriesByType("measure")
        .filter((m) => m.name.startsWith("openphotoid:"))
        .map((m) => [m.name.slice("openphotoid:".length), Math.round(m.duration)]),
    );
    return { checks, overall, timings, provider: globalThis.__openphotoid?.provider };
  });
}

/** Width and height from a JPEG's SOF marker — enough to check an export. */
export function jpegSize(buf) {
  let i = 2;
  while (i < buf.length) {
    if (buf[i] !== 0xff) {
      i++;
      continue;
    }
    const marker = buf[i + 1];
    if (marker === 0xc0 || marker === 0xc1 || marker === 0xc2) {
      return { height: buf.readUInt16BE(i + 5), width: buf.readUInt16BE(i + 7) };
    }
    i += 2 + buf.readUInt16BE(i + 2);
  }
  return null;
}
