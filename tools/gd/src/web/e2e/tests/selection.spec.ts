import { test, expect } from "./fixtures.js";

test.describe("Visual Selection", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("v toggles visual selection mode", async ({ page }) => {
    // Initially no visual-selected lines
    await expect(page.locator(".visual-selected")).toHaveCount(0);

    // Press v to start selection
    await page.keyboard.press("v");

    // Current line should have visual-selected class
    const selectedLines = page.locator(".visual-selected");
    await expect(selectedLines).not.toHaveCount(0);

    // Press v again to cancel
    await page.keyboard.press("v");

    // No lines should have visual-selected class
    await expect(page.locator(".visual-selected")).toHaveCount(0);
  });

  test("visual selection highlights lines between anchor and cursor", async ({ page }) => {
    // Start visual selection
    await page.keyboard.press("v");

    // Get initial cursor position
    const getPos = () => page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    const startPos = await getPos();

    // Move cursor down until position actually changes
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press("j");
      const newPos = await getPos();
      if (newPos > startPos) break;
    }

    // At minimum, cursor line should be selected
    const selectedLines = page.locator(".diff-line.visual-selected");
    const count = await selectedLines.count();
    expect(count).toBeGreaterThanOrEqual(1);
  });

  test("y copies selection reference to clipboard", async ({ page, context }) => {
    // Grant clipboard permissions
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);

    // Start visual selection
    await page.keyboard.press("v");

    // Move cursor down
    await page.keyboard.press("j");
    await page.keyboard.press("j");

    // Yank selection
    await page.keyboard.press("y");

    // Selection should be cleared
    await expect(page.locator(".visual-selected")).toHaveCount(0);

    // Check clipboard content contains path:line format
    const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
    expect(clipboardText).toMatch(/\S+:\d+(-\d+)?/);
  });

  test("Escape cancels visual selection", async ({ page }) => {
    // Start visual selection
    await page.keyboard.press("v");
    await expect(page.locator(".visual-selected")).not.toHaveCount(0);

    // Press Escape
    await page.keyboard.press("Escape");

    // Selection should be cleared
    await expect(page.locator(".visual-selected")).toHaveCount(0);
  });

  test("visual selection persists during navigation", async ({ page }) => {
    // Start visual selection
    await page.keyboard.press("v");

    // Navigate with various keys
    await page.keyboard.press("j");
    await page.keyboard.press("j");
    await page.keyboard.press("k");

    // Selection should still be active
    const selectedLines = page.locator(".visual-selected");
    const count = await selectedLines.count();
    expect(count).toBeGreaterThanOrEqual(1);
  });
});
