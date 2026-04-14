import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  workers: 1,
  reporter: "html",
  use: {
    trace: "on-first-retry",
    colorScheme: "dark",
  },
});
