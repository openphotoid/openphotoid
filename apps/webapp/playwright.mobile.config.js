import { defineConfig, devices } from "@playwright/test";

// The same suite, on phone and tablet engines: WebKit with iPhone and iPad
// emulation (viewport, touch, user agent), and Chromium as a Pixel. It is
// not iOS Safari — that is what #/diagnostics/run on a simulator is for —
// but it is the layout and the touch flow the phone will get.
export default defineConfig({
  testDir: "./e2e",
  testMatch: /.*\.spec\.js/,
  timeout: 240_000,
  fullyParallel: false,
  workers: 1,
  reporter: [["list"]],
  use: { baseURL: "http://localhost:5199", trace: "retain-on-failure" },
  webServer: { command: "node e2e/server.mjs", url: "http://localhost:5199/", reuseExistingServer: !process.env.CI },
  projects: [
    { name: "iphone", use: { ...devices["iPhone 15"], browserName: "webkit" } },
    { name: "ipad", use: { ...devices["iPad Pro 11"], browserName: "webkit" } },
    { name: "pixel", use: { ...devices["Pixel 7"], browserName: "chromium" } },
  ],
});
