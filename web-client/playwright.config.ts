import { defineConfig } from "@playwright/test";

const tutorialTimeoutMs = 30 * 60 * 1000;

export default defineConfig({
  testDir: "./tests",
  timeout: tutorialTimeoutMs,
  retries: Number(process.env.TUTORIAL_RETRIES ?? 1),
  fullyParallel: false,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: "http://localhost:3000",
    headless: true,
  },
  webServer: {
    command: "yarn dev",
    url: "http://localhost:3000",
    reuseExistingServer: false,
    timeout: 120 * 1000,
  },
});
