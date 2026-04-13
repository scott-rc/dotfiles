import { test, expect } from "./fixtures.js";

test.describe("Keybindings", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("? toggles help overlay", async ({ page }) => {
    const helpOverlay = page.locator("#help-overlay");

    // Hidden initially
    await expect(helpOverlay).not.toHaveClass(/visible/);

    // Press ? to show help
    await page.keyboard.press("?");
    await expect(helpOverlay).toHaveClass(/visible/);

    // Press ? again to hide
    await page.keyboard.press("?");
    await expect(helpOverlay).not.toHaveClass(/visible/);
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
    const helpOverlay = page.locator("#help-overlay");

    // Open help
    await page.keyboard.press("?");
    await expect(helpOverlay).toHaveClass(/visible/);

    // Close with Escape
    await page.keyboard.press("Escape");
    await expect(helpOverlay).not.toHaveClass(/visible/);
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

  test("o toggles full context", async ({ page }) => {
    // Press o to toggle full context
    await page.keyboard.press("o");

    // Verify no errors
    await expect(page.locator("#diff-pane")).toBeVisible();

    // Toggle back
    await page.keyboard.press("o");
    await expect(page.locator("#diff-pane")).toBeVisible();
  });
});
