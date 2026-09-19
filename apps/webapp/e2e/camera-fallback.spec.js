// "Take a photo" where there is no webcam in the page (APP-97): a computer
// with no camera, and a phone, which keeps the system camera. The webcam
// case is camera.spec.js; the two need different browser launch flags, and
// Playwright takes those per file.
import { test, expect } from "@playwright/test";

const home = async (page) => {
  await page.goto("/");
  await page.locator("#app").getByText(/documents from/).waitFor();
};

test.describe("on a computer whose camera will not start", () => {
  // No fake device: headless Chromium refuses getUserMedia outright
  // (NotSupportedError), which is the generic "could not be started" path.
  test("says so, and offers the file picker instead", async ({ page }) => {
    await home(page);
    await page.getByRole("button", { name: "Take a photo" }).click();
    const dialog = page.getByRole("dialog", { name: "Take a photo" });
    await expect(dialog.getByRole("alert")).toBeVisible({ timeout: 15_000 });
    const chooser = page.waitForEvent("filechooser");
    await dialog.getByRole("button", { name: "Choose a photo instead" }).click();
    await chooser;
    await expect(dialog).toBeHidden();
  });
});

// What a real computer raises, and what the person is told. Headless
// Chromium cannot produce these on its own, so getUserMedia is made to.
for (const [name, message] of [
  ["NotAllowedError", /not allowed to use the camera/],
  ["NotFoundError", /No camera was found/],
  ["NotReadableError", /in use by another app/],
]) {
  test(`${name} is explained in words, with the file picker beside it`, async ({ page }) => {
    await page.addInitScript((n) => {
      navigator.mediaDevices.getUserMedia = async () => {
        throw new DOMException("refused", n);
      };
    }, name);
    await home(page);
    await page.getByRole("button", { name: "Take a photo" }).click();
    const dialog = page.getByRole("dialog", { name: "Take a photo" });
    await expect(dialog.getByRole("alert")).toContainText(message);
    await expect(dialog.getByRole("button", { name: "Choose a photo instead" })).toBeVisible();
  });
}

test.describe("on a phone", () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });

  test("Take a photo still hands over to the system camera", async ({ page }) => {
    await home(page);
    const chooser = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Take a photo" }).click();
    const fc = await chooser;
    expect(await fc.element().getAttribute("capture")).toBe("user");
    await expect(page.getByRole("dialog")).toHaveCount(0);
  });
});
