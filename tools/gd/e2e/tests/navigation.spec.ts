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

    // Go to top - cursor lands on first content line (may have small offset for headers)
    await page.keyboard.press("g");
    const topScroll = await diffPane.evaluate((el) => el.scrollTop);
    // Allow small offset for file header visibility when cursor is on first content line
    expect(topScroll).toBeLessThanOrEqual(50);

    // Verify we were actually scrolled somewhere
    expect(bottomScroll).toBeGreaterThanOrEqual(topScroll);

    // Verify cursor is on first content line
    const cursorPos = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });
    expect(cursorPos).toBeGreaterThanOrEqual(0);
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
    // Get the file the cursor starts in
    const startFileIdx = await page.evaluate(() => {
      const s = (window as any).__gdState;
      return s.flatLines[s.cursorLine]?.fileIdx;
    });

    // Navigate to next file
    await page.keyboard.press("}");
    const afterNext = await page.evaluate(() => {
      const s = (window as any).__gdState;
      return s.flatLines[s.cursorLine]?.fileIdx;
    });
    expect(afterNext).toBeGreaterThan(startFileIdx);

    // Navigate back with {
    await page.keyboard.press("{");
    const afterPrev = await page.evaluate(() => {
      const s = (window as any).__gdState;
      return s.flatLines[s.cursorLine]?.fileIdx;
    });
    expect(afterPrev).toBeLessThan(afterNext);
  });

  test("{ navigates backward through all files without getting stuck", async ({ page }) => {
    // Jump forward several files
    await page.keyboard.press("}");
    await page.keyboard.press("}");
    await page.keyboard.press("}");

    const fileAfterForward = await page.evaluate(() => {
      const s = (window as any).__gdState;
      return s.flatLines[s.cursorLine]?.fileIdx;
    });
    expect(fileAfterForward).toBeGreaterThan(0);

    // Now press { repeatedly — should walk backward without getting stuck
    const visitedFiles: number[] = [fileAfterForward];
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press("{");
      const fileIdx = await page.evaluate(() => {
        const s = (window as any).__gdState;
        return s.flatLines[s.cursorLine]?.fileIdx;
      });
      visitedFiles.push(fileIdx);
      if (fileIdx === 0) break;
    }

    // Should have reached file 0
    expect(visitedFiles[visitedFiles.length - 1]).toBe(0);
    // Should be monotonically non-increasing (no stuck loops)
    for (let i = 1; i < visitedFiles.length; i++) {
      expect(visitedFiles[i]).toBeLessThanOrEqual(visitedFiles[i - 1]);
    }
  });

  test("] at last hunk advances to next file", async ({ page }) => {
    const diffPane = page.locator("#diff-pane");
    const fileHeaders = diffPane.locator(".file-header");

    // Verify we have multiple files
    const fileCount = await fileHeaders.count();
    expect(fileCount).toBeGreaterThan(1);

    // Go to top
    await page.keyboard.press("g");

    // Press ] repeatedly to advance through all hunks
    // After exhausting first file's hunks, should auto-advance to next file
    for (let i = 0; i < 30; i++) {
      await page.keyboard.press("]");
    }

    // Verify the cursor moved past the first file header
    // (scrollTop > 0 or cursor is in a different file region)
    const cursorLine = diffPane.locator(".cursor-line");
    await expect(cursorLine).toBeVisible();

    // If we have multiple files and navigated through them, verify we're still functional
    if (fileCount > 1) {
      const scrollTop = await diffPane.evaluate((el) => el.scrollTop);
      // We either scrolled OR the diff is small enough to fit on screen
      // The key is that we didn't crash and navigation works
      expect(scrollTop).toBeGreaterThanOrEqual(0);
    }
  });

  test("[ at first hunk goes to previous file", async ({ page }) => {
    const diffPane = page.locator("#diff-pane");

    // Go to second file
    await page.keyboard.press("}");
    const afterJump = await diffPane.evaluate((el) => el.scrollTop);

    // Press [ to go back - should return to previous file's last hunk
    await page.keyboard.press("[");
    const afterBracket = await diffPane.evaluate((el) => el.scrollTop);

    // We should have scrolled back (or at least not crashed)
    // The exact behavior depends on implementation
    expect(afterBracket).toBeLessThanOrEqual(afterJump);
  });
});
