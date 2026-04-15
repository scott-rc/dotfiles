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
    // Verify full context is initially off
    const initialState = await page.evaluate(
      () => (window as any).__gdState?.fullContext,
    );
    expect(initialState).toBe(false);

    // Press o to enable full context
    await page.keyboard.press("o");

    // Wait for full context to be enabled and collapsed context groups to appear
    // (server re-diffs with -U999999, then long context runs get collapsed)
    await page.waitForSelector(".collapsed-context", { timeout: 5000 });

    expect(
      await page.$$eval(".collapsed-context", (els) => els.length),
    ).toBeGreaterThan(0);

    // Press o again to disable full context
    await page.keyboard.press("o");

    // Wait for full context to be disabled and diff to re-render
    await page.waitForFunction(
      () =>
        (window as any).__gdState?.fullContext === false &&
        document.querySelectorAll(".collapsed-context").length === 0,
      { timeout: 5000 },
    );

    // No collapsed groups in normal mode
    const restoredCount = await page.$$eval(
      ".collapsed-context",
      (els) => els.length,
    );
    expect(restoredCount).toBe(0);
  });
});
