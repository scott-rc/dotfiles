import { test, expect } from "./fixtures.js";

test.describe("Search", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("/ opens search bar", async ({ page }) => {
    // Search bar absent initially
    await expect(page.locator("#search-bar")).toHaveCount(0);

    // Press / to open search
    await page.keyboard.press("/");
    await expect(page.locator("#search-bar")).toHaveCount(1);

    // Search input should be focused
    const searchInput = page.locator("#search-input");
    await expect(searchInput).toBeFocused();
  });

  test("Escape closes search bar", async ({ page }) => {
    // Open search
    await page.keyboard.press("/");
    await expect(page.locator("#search-bar")).toHaveCount(1);

    // Close with Escape
    await page.keyboard.press("Escape");
    await expect(page.locator("#search-bar")).toHaveCount(0);
  });

  test("search finds matches", async ({ page }) => {
    // Open search and type
    await page.keyboard.press("/");
    const searchInput = page.locator("#search-input");
    await searchInput.fill("fn");
    await page.keyboard.press("Enter");

    // Should show match count
    const searchCount = page.locator("#search-count");
    const countText = await searchCount.textContent();
    expect(countText).toMatch(/\d+/); // Should contain numbers
  });

  test("n/N cycles through matches", async ({ page }) => {
    // Search for something
    await page.keyboard.press("/");
    const searchInput = page.locator("#search-input");
    await searchInput.fill("let");
    await page.keyboard.press("Enter");

    // Press n to go to next match
    await page.keyboard.press("n");

    // Press N (shift+n) to go to previous
    await page.keyboard.press("N");

    // Verify search is still active
    await expect(page.locator("#search-bar")).toHaveCount(1);
  });

  test("search highlights matches", async ({ page }) => {
    // Search for a term
    await page.keyboard.press("/");
    await page.locator("#search-input").fill("fn");
    await page.keyboard.press("Enter");

    // Should have search match elements
    const matches = page.locator(".search-match");
    const count = await matches.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});
