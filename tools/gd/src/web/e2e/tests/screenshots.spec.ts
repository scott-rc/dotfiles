import { test, expect } from "./fixtures.js";

test.describe("Screenshots", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
    // Set consistent viewport for screenshot comparisons
    await page.setViewportSize({ width: 1280, height: 720 });
  });

  test("initial load with tree visible", async ({ page }) => {
    await expect(page).toHaveScreenshot("initial-load.png");
  });

  test("tree hidden", async ({ page }) => {
    await page.keyboard.press("l");
    // Tree is removed from DOM when hidden
    await expect(page.locator("#tree")).toHaveCount(0);
    await expect(page).toHaveScreenshot("tree-hidden.png");
  });

  test("tree focused", async ({ page }) => {
    await page.keyboard.press("t");
    // When tree is focused, active entry should NOT have unfocused class
    await expect(page.locator(".tree-entry.active")).toBeVisible();
    await expect(page.locator(".tree-entry.active.unfocused")).not.toBeVisible();
    await expect(page).toHaveScreenshot("tree-focused.png");
  });

  test("help overlay visible", async ({ page }) => {
    await page.keyboard.press("?");
    await expect(page.locator("#help-overlay")).toBeVisible();
    await expect(page).toHaveScreenshot("help-overlay.png");
  });

  test("single-file view", async ({ page }) => {
    await page.keyboard.press("s");
    await expect(page).toHaveScreenshot("single-file-view.png");
  });

  test("search overlay open", async ({ page }) => {
    await page.keyboard.press("/");
    await expect(page.locator("#search-input")).toBeVisible();
    await expect(page).toHaveScreenshot("search-overlay.png");
  });

  test("visual selection at cursor", async ({ page }) => {
    // Navigate to first hunk
    await page.keyboard.press("]");
    // Move to a content line
    await page.keyboard.press("j");
    // Start visual selection
    await page.keyboard.press("v");
    await expect(page.locator(".visual-selected").first()).toBeVisible();
    await expect(page).toHaveScreenshot("visual-selection-single.png");
  });

  test("visual selection extended", async ({ page }) => {
    // Navigate to first hunk
    await page.keyboard.press("]");
    // Start visual selection
    await page.keyboard.press("v");
    // Navigate down (selection range expands)
    await page.keyboard.press("j");
    await page.keyboard.press("j");
    await page.keyboard.press("j");
    // All 4 lines (anchor + 3 j presses) should be selected
    await expect(page.locator(".visual-selected")).toHaveCount(4);
    await expect(page).toHaveScreenshot("visual-selection-extended.png");
  });
});
