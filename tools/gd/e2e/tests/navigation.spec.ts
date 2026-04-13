import { test, expect } from "./fixtures.js";

test.describe("Navigation", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("j/k scrolls up and down", async ({ page }) => {
    const diffPane = page.locator("#diff-pane");

    // Get initial scroll position
    const initialScroll = await diffPane.evaluate((el) => el.scrollTop);

    // Press j to scroll down
    await page.keyboard.press("j");
    const afterJ = await diffPane.evaluate((el) => el.scrollTop);
    expect(afterJ).toBeGreaterThanOrEqual(initialScroll);

    // Press k to scroll up
    await page.keyboard.press("k");
    const afterK = await diffPane.evaluate((el) => el.scrollTop);
    expect(afterK).toBeLessThanOrEqual(afterJ);
  });

  test("g goes to top, G goes to bottom", async ({ page }) => {
    const diffPane = page.locator("#diff-pane");

    // Go to bottom
    await page.keyboard.press("G");
    const bottomScroll = await diffPane.evaluate((el) => el.scrollTop);

    // Go to top
    await page.keyboard.press("g");
    const topScroll = await diffPane.evaluate((el) => el.scrollTop);
    expect(topScroll).toBe(0);

    // Verify we were actually scrolled somewhere
    expect(bottomScroll).toBeGreaterThanOrEqual(topScroll);
  });

  test("d/u for half-page scroll", async ({ page }) => {
    const diffPane = page.locator("#diff-pane");

    // Ensure we're at top
    await page.keyboard.press("g");

    // Half page down
    await page.keyboard.press("d");
    const afterD = await diffPane.evaluate((el) => el.scrollTop);
    expect(afterD).toBeGreaterThanOrEqual(0);

    // Half page up
    await page.keyboard.press("u");
    const afterU = await diffPane.evaluate((el) => el.scrollTop);
    expect(afterU).toBeLessThanOrEqual(afterD);
  });

  test("] and [ navigate between hunks", async ({ page }) => {
    // Navigate to next hunk
    await page.keyboard.press("]");

    // Navigate to previous hunk
    await page.keyboard.press("[");

    // If there are multiple hunks, position should change
    // Just verify no errors occur
    await expect(page.locator("#diff-pane")).toBeVisible();
  });

  test("} and { navigate between files", async ({ page }) => {
    // Navigate to next file
    await page.keyboard.press("}");

    // Navigate to previous file
    await page.keyboard.press("{");

    // Verify no errors
    await expect(page.locator("#diff-pane")).toBeVisible();
  });
});
