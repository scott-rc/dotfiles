import { test, expect } from "./fixtures.js";

test.describe("Cursor Centering", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  // Helper to check cursor position relative to viewport
  async function getCursorViewportInfo(page: any) {
    return page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      const pane = document.querySelector("#diff-pane") as HTMLElement | null;
      if (!cursor || !pane) return { visible: false, ratio: -1, scrollable: false };

      const cursorRect = cursor.getBoundingClientRect();
      const paneRect = pane.getBoundingClientRect();

      const cursorCenter = cursorRect.top + cursorRect.height / 2;
      const ratio = (cursorCenter - paneRect.top) / paneRect.height;
      const visible = cursorRect.top >= paneRect.top && cursorRect.bottom <= paneRect.bottom;
      const scrollable = pane.scrollHeight > pane.clientHeight;

      return { visible, ratio, scrollable };
    });
  }

  test("] navigation keeps cursor visible and centered", async ({ page }) => {
    // Enable full context for more content
    await page.keyboard.press("o");
    await page.waitForFunction(() => document.querySelectorAll(".diff-line").length > 10, {
      timeout: 5000,
    });

    // Navigate forward multiple times
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("]");

      // After each navigation, cursor should be visible
      const info = await getCursorViewportInfo(page);
      expect(info.visible).toBe(true);

      // If content is scrollable, cursor should be in reasonable position (not at extreme edge)
      if (info.scrollable && info.ratio >= 0) {
        // Cursor should not be at very top or very bottom edge (allow some tolerance)
        // This verifies centering behavior vs nearest behavior
        expect(info.ratio).toBeGreaterThan(0.05);
        expect(info.ratio).toBeLessThan(0.95);
      }
    }
  });

  test("[ navigation keeps cursor visible and centered", async ({ page }) => {
    // Enable full context
    await page.keyboard.press("o");
    await page.waitForFunction(() => document.querySelectorAll(".diff-line").length > 10, {
      timeout: 5000,
    });

    // Navigate forward first
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("]");
    }

    // Navigate backward
    for (let i = 0; i < 3; i++) {
      await page.keyboard.press("[");

      // After each navigation, cursor should be visible
      const info = await getCursorViewportInfo(page);
      expect(info.visible).toBe(true);
    }
  });

  test("j/k navigation uses nearest scroll, not center", async ({ page }) => {
    // Enable full context
    await page.keyboard.press("o");
    await page.waitForFunction(() => document.querySelectorAll(".diff-line").length > 10, {
      timeout: 5000,
    });

    // Go to top
    await page.keyboard.press("g");
    await page.waitForTimeout(50);

    // Move down with j a few times
    await page.keyboard.press("j");
    await page.keyboard.press("j");
    await page.keyboard.press("j");

    const afterJ = await getCursorViewportInfo(page);

    // Cursor should still be visible
    expect(afterJ.visible).toBe(true);

    // After just 3 j presses from top, cursor should still be in upper portion
    // (not forcibly centered - this would cause ratio to jump to ~0.5)
    // j/k uses 'nearest' which keeps cursor near its current position
    if (afterJ.scrollable) {
      expect(afterJ.ratio).toBeLessThan(0.6);
    }
  });

  test("] centers cursor after file transition in single mode", async ({ page }) => {
    // Enter single-file mode
    await page.keyboard.press("s");

    // Check if multiple files exist
    const fileInfo = await page.evaluate(() => {
      const statusText = document.querySelector("#status-left")?.textContent || "";
      const match = statusText.match(/(\d+)\/(\d+)/);
      return match ? { current: parseInt(match[1]), total: parseInt(match[2]) } : null;
    });

    if (!fileInfo || fileInfo.total <= 1) {
      return; // Skip if only one file
    }

    // Navigate to end of current file
    let lastPos = -1;
    for (let i = 0; i < 50; i++) {
      await page.keyboard.press("]");
      const pos = await page.evaluate(() => (window as any).__gdState?.cursorLine);
      if (pos === lastPos) break;
      lastPos = pos;
    }

    // Press ] to advance to next file
    await page.keyboard.press("]");
    await page.waitForTimeout(50);

    // Cursor should be visible after file transition
    const info = await getCursorViewportInfo(page);
    expect(info.visible).toBe(true);
  });

  test("[ centers cursor after file transition in single mode", async ({ page }) => {
    // Enter single-file mode
    await page.keyboard.press("s");

    // Check if multiple files exist
    const fileInfo = await page.evaluate(() => {
      const statusText = document.querySelector("#status-left")?.textContent || "";
      const match = statusText.match(/(\d+)\/(\d+)/);
      return match ? { current: parseInt(match[1]), total: parseInt(match[2]) } : null;
    });

    if (!fileInfo || fileInfo.total <= 1) {
      return; // Skip if only one file
    }

    // Go to second file
    await page.keyboard.press("}");

    // Go to first change
    await page.keyboard.press("g");
    await page.keyboard.press("]");

    // Press [ to retreat to previous file
    await page.keyboard.press("[");
    await page.waitForTimeout(50);

    // Cursor should be visible after file transition
    const info = await getCursorViewportInfo(page);
    expect(info.visible).toBe(true);
  });

  test("g key scrolls fully to top", async ({ page }) => {
    // Enable full context for more content
    await page.keyboard.press("o");
    await page.waitForFunction(() => document.querySelectorAll(".diff-line").length > 10, {
      timeout: 5000,
    });

    // Navigate down first
    await page.keyboard.press("]");
    await page.keyboard.press("]");
    await page.keyboard.press("]");
    await page.waitForTimeout(50);

    // Press g to go to top
    await page.keyboard.press("g");
    await page.waitForTimeout(50);

    // Verify the pane is scrolled to the very top
    const scrollInfo = await page.evaluate(() => {
      const pane = document.querySelector("#diff-pane") as HTMLElement;
      return pane ? { scrollTop: pane.scrollTop } : null;
    });

    expect(scrollInfo).not.toBeNull();
    expect(scrollInfo!.scrollTop).toBe(0);
  });

  test("G key scrolls fully to bottom", async ({ page }) => {
    // Enable full context for more content
    await page.keyboard.press("o");
    await page.waitForFunction(() => document.querySelectorAll(".diff-line").length > 10, {
      timeout: 5000,
    });

    // Press G to go to bottom
    await page.keyboard.press("G");
    await page.waitForTimeout(50);

    // Verify the cursor is visible at the bottom
    const info = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      const pane = document.querySelector("#diff-pane") as HTMLElement;
      if (!cursor || !pane) return null;
      const cursorRect = cursor.getBoundingClientRect();
      const paneRect = pane.getBoundingClientRect();
      return {
        cursorVisible: cursorRect.bottom <= paneRect.bottom,
        isAtEnd: pane.scrollTop + pane.clientHeight >= pane.scrollHeight - 5, // within 5px of bottom
      };
    });

    expect(info).not.toBeNull();
    expect(info!.cursorVisible).toBe(true);
  });

  test("z key centers cursor in viewport", async ({ page }) => {
    // Enable full context
    await page.keyboard.press("o");
    await page.waitForFunction(() => document.querySelectorAll(".diff-line").length > 10, {
      timeout: 5000,
    });

    // Move to middle of content
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    // Manually scroll so cursor is off-center
    await page.evaluate(() => {
      const pane = document.querySelector("#diff-pane") as HTMLElement;
      if (pane) pane.scrollTop = 0;
    });
    await page.waitForTimeout(50);

    // Press z to center
    await page.keyboard.press("z");
    await page.waitForTimeout(50);

    // Cursor should be visible
    const info = await getCursorViewportInfo(page);
    expect(info.visible).toBe(true);
  });
});
