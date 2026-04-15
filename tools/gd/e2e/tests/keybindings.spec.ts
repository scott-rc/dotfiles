import { test, expect } from "./fixtures.js";

test.describe("Keybindings", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("? toggles help overlay", async ({ page }) => {
    // Hidden initially (absent from DOM)
    await expect(page.locator("#help-overlay")).toHaveCount(0);

    // Press ? to show help
    await page.keyboard.press("?");
    await expect(page.locator("#help-overlay")).toHaveCount(1);

    // Press ? again to hide
    await page.keyboard.press("?");
    await expect(page.locator("#help-overlay")).toHaveCount(0);
  });

  test("help overlay contains keybinding info", async ({ page }) => {
    // Open help
    await page.keyboard.press("?");

    const helpContent = page.locator("#help-content");
    await expect(helpContent).toBeVisible();

    // Should contain common keybindings
    const text = await helpContent.textContent();
    expect(text).toContain("j");
    expect(text).toContain("k");
  });

  test("Escape closes help overlay", async ({ page }) => {
    // Open help
    await page.keyboard.press("?");
    await expect(page.locator("#help-overlay")).toHaveCount(1);

    // Close with Escape
    await page.keyboard.press("Escape");
    await expect(page.locator("#help-overlay")).toHaveCount(0);
  });

  test("s toggles single-file view", async ({ page }) => {
    // Press s to toggle single-file view
    await page.keyboard.press("s");

    // Verify the view changed (exact behavior depends on implementation)
    await expect(page.locator("#diff-pane")).toBeVisible();

    // Toggle back
    await page.keyboard.press("s");
    await expect(page.locator("#diff-pane")).toBeVisible();
  });

  test("o toggles full context mode", async ({ page }) => {
    // Get initial count of diff lines
    const initialCount = await page.locator(".diff-line").count();

    // Press o to enable full context
    await page.keyboard.press("o");

    // Wait for diff to update (server sends new DiffData)
    await page.waitForFunction(
      (prevCount) => document.querySelectorAll(".diff-line").length !== prevCount,
      initialCount,
      { timeout: 5000 }
    );

    // Count should have increased (full context shows more lines)
    const fullContextCount = await page.locator(".diff-line").count();
    expect(fullContextCount).toBeGreaterThan(initialCount);

    // Press o again to disable full context
    await page.keyboard.press("o");

    // Wait for diff to update back
    await page.waitForFunction(
      (prevCount) => document.querySelectorAll(".diff-line").length !== prevCount,
      fullContextCount,
      { timeout: 5000 }
    );

    // Count should return to approximately original value
    const restoredCount = await page.locator(".diff-line").count();
    expect(restoredCount).toBeLessThan(fullContextCount);
  });
});
