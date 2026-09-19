// "Take a photo" on a computer (APP-97).
//
// `capture="user"` asks a phone for its camera and is ignored by every
// desktop browser, so on a computer the button opened the same file picker
// as "Choose a photo" and looked broken. On a computer it now opens the
// webcam in the page; on a phone it still hands over to the system camera,
// which is the better camera app there.
//
// The webcam is Chromium's fake capture device fed a real portrait, so the
// frame taken goes through the real pipeline rather than a test pattern.
import { test, expect } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fixture, FIXTURES, waitForResult } from "./helpers.mjs";

// A 1280×720 webcam frame with the reference portrait in it, built here
// rather than committed: as Y4M it is 1.4 MB of raw pixels.
const FEED = join(tmpdir(), "openphotoid-webcam-720p.y4m");
let feedReady = existsSync(FEED);
if (!feedReady) {
  try {
    execFileSync("ffmpeg", [
      "-y", "-loglevel", "error", "-i", fixture(FIXTURES.obama),
      "-vf", "scale=-2:720,pad=1280:720:(ow-iw)/2:0:color=0x9a9a9a",
      "-frames:v", "1", "-pix_fmt", "yuv420p", FEED,
    ]);
    feedReady = true;
  } catch {
    feedReady = false;
  }
}

test.use({
  launchOptions: {
    args: [
      "--use-fake-ui-for-media-stream",
      "--use-fake-device-for-media-stream",
      ...(feedReady ? [`--use-file-for-fake-video-capture=${FEED}`] : []),
    ],
  },
});

const home = async (page) => {
  await page.goto("/");
  await page.locator("#app").getByText(/documents from/).waitFor();
};

test.describe("on a computer with a webcam", () => {
  test("Take a photo opens the camera, not the file picker", async ({ page }) => {
    // Every track the page is handed, so the test can ask afterwards whether
    // any of them is still live — the camera light, in effect.
    await page.addInitScript(() => {
      const get = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
      globalThis.__tracks = [];
      navigator.mediaDevices.getUserMedia = async (c) => {
        const s = await get(c);
        globalThis.__tracks.push(...s.getTracks());
        return s;
      };
    });
    await home(page);
    let picker = false;
    page.on("filechooser", () => (picker = true));
    await page.getByRole("button", { name: "Take a photo" }).click();

    const dialog = page.getByRole("dialog", { name: "Take a photo" });
    await expect(dialog).toBeVisible();
    // Live frames, not a black box: the video has a size once it plays.
    await expect
      .poll(() => dialog.locator("video").evaluate((v) => v.videoWidth), { timeout: 15_000 })
      .toBeGreaterThan(0);
    expect(picker).toBe(false);

    // Close stops the camera: no live track left behind in the tab.
    await dialog.getByRole("button", { name: "Close" }).click();
    await expect(dialog).toBeHidden();
    const states = () => page.evaluate(() => globalThis.__tracks.map((t) => t.readyState));
    expect((await states()).length).toBeGreaterThan(0);
    await expect.poll(states).toEqual(expect.not.arrayContaining(["live"]));

    // Esc too, which only the browser sees.
    await page.getByRole("button", { name: "Take a photo" }).click();
    await expect
      .poll(() => dialog.locator("video").evaluate((v) => v.videoWidth), { timeout: 15_000 })
      .toBeGreaterThan(0);
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
    await expect.poll(states).toEqual(expect.not.arrayContaining(["live"]));
  });

  test("capture, retake, then use the photo all the way to a checked result", async ({ page }) => {
    test.skip(!feedReady, "ffmpeg is needed to build the webcam feed from the reference portrait");
    await home(page);
    await page.getByRole("button", { name: "Take a photo" }).click();
    const dialog = page.getByRole("dialog", { name: "Take a photo" });
    await expect
      .poll(() => dialog.locator("video").evaluate((v) => v.videoWidth), { timeout: 15_000 })
      .toBe(1280);

    await dialog.getByRole("button", { name: "Capture" }).click();
    await expect(dialog.getByRole("img", { name: "The photo you took" })).toBeVisible();
    await dialog.getByRole("button", { name: "Retake" }).click();
    await expect(dialog.locator("video")).toBeVisible();
    await dialog.getByRole("button", { name: "Capture" }).click();
    await dialog.getByRole("button", { name: "Use this photo" }).click();

    await page.waitForURL(/#\/pick/);
    await page.getByPlaceholder(/Search country or document/i).fill("us-passport");
    await page.locator("ul.list button").first().click();
    await page.waitForURL(/#\/studio\/us-passport/);
    const r = await waitForResult(page);
    const face = r.checks.find((c) => c.name.toLowerCase() === "face count");
    expect(face?.status, JSON.stringify(r.checks)).toBe("pass");
  });
});
