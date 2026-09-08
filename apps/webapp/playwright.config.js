import { defineConfig } from "@playwright/test";

// Against dist/, never the dev server: the guide documents what ships.
// Port 5199 — the suite's siblings hold 5183–5194.
export default defineConfig({
  testDir: "./e2e",
  testMatch: /.*\.spec\.js/,
  timeout: 180_000,
  fullyParallel: false,
  workers: 1,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:5199",
    viewport: { width: 1440, height: 900 },
    trace: "retain-on-failure",
  },
  webServer: {
    command: "node e2e/server.mjs",
    url: "http://localhost:5199/",
    reuseExistingServer: !process.env.CI,
  },
});
