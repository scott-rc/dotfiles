import { test, expect } from "./fixtures.js";

test.describe("Collapsible Context", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
    // Enable full context mode to trigger collapsible groups
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => (window as any).__gdState?.fullContext === true,
      { timeout: 5000 },
    );
    // Wait for collapsed-context elements to appear
    await page.waitForSelector(".collapsed-context", { timeout: 5000 });
  });

  test("displays collapsed context groups with line count", async ({
    page,
  }) => {
    const groups = await page.$$eval(".collapsed-context", (els) =>
      els.map((el) => el.textContent?.trim()),
    );
    expect(groups.length).toBeGreaterThan(0);
    for (const text of groups) {
      expect(text).toMatch(/^\d+ unmodified lines$/);
    }
  });

  test("click expands a collapsed group", async ({ page }) => {
    const countBefore = await page.$$eval(
      ".collapsed-context",
      (els) => els.length,
    );
    expect(countBefore).toBeGreaterThan(0);

    // Click the first collapsed group
    await page.click(".collapsed-context");

    // Should have one fewer collapsed group
    const countAfter = await page.$$eval(
      ".collapsed-context",
      (els) => els.length,
    );
    expect(countAfter).toBe(countBefore - 1);
  });

  test("expanded group shows individual context lines", async ({ page }) => {
    // Get the line count from the first group
    const groupText = await page.$eval(
      ".collapsed-context",
      (el) => el.textContent?.trim() ?? "",
    );
    const lineCount = parseInt(groupText.match(/^(\d+)/)?.[1] ?? "0", 10);
    expect(lineCount).toBeGreaterThan(3);

    // Count total lines before
    const linesBefore = await page.$$eval(".diff-line", (els) => els.length);

    // Click to expand
    await page.click(".collapsed-context");

    // Count total lines after
    const linesAfter = await page.$$eval(".diff-line", (els) => els.length);

    // Should have gained the collapsed line count
    expect(linesAfter).toBe(linesBefore + lineCount);
  });

  test("collapsed context has correct ARIA label", async ({ page }) => {
    const label = await page.$eval(".collapsed-context", (el) =>
      el.getAttribute("aria-label"),
    );
    expect(label).toMatch(/^\d+ unmodified lines — click to expand$/);
  });

  test("collapsed-context items appear in __gdState", async ({ page }) => {
    const types = await page.evaluate(() => {
      const st = (window as any).__gdState;
      return st.flatLines.map((i: any) => i.type);
    });
    expect(types).toContain("collapsed-context");
  });

  test("no collapsed groups in normal context mode", async ({ page }) => {
    // Toggle full context off
    await page.keyboard.press("o");
    // Wait for full context to be disabled AND the diff to re-render
    await page.waitForFunction(
      () =>
        (window as any).__gdState?.fullContext === false &&
        document.querySelectorAll(".collapsed-context").length === 0,
      { timeout: 5000 },
    );
    // Standard 3-line context should not trigger grouping (threshold > 3)
    const count = await page.$$eval(
      ".collapsed-context",
      (els) => els.length,
    );
    expect(count).toBe(0);
  });
});
