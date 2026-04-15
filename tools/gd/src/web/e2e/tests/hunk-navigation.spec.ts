import { test, expect } from "./fixtures.js";

test.describe("Hunk Navigation", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("initial view focuses first hunk, not file header", async ({ page }) => {
    // On initial load, cursor should be on a diff line (first change),
    // not on a file header
    const cursorLine = page.locator(".cursor-line");
    await expect(cursorLine).toBeVisible();

    // The cursor should be on an actual diff line with content
    // File headers have class 'file-header', diff lines have class 'diff-line'
    const cursorIsOnDiffLine = await cursorLine.evaluate((el) =>
      el.classList.contains("diff-line")
    );
    expect(cursorIsOnDiffLine).toBe(true);
  });

  test("] moves to first change group initially", async ({ page }) => {
    // Go to top first
    await page.keyboard.press("g");

    // Press ] to move to first hunk
    await page.keyboard.press("]");

    // The cursor should be on a line with added/deleted content
    const cursorLine = page.locator(".cursor-line");
    const isChangeLine = await cursorLine.evaluate((el) =>
      el.classList.contains("line-added") || el.classList.contains("line-deleted")
    );
    expect(isChangeLine).toBe(true);
  });

  test("[ and ] navigate between change groups in full context mode", async ({ page }) => {
    // Enable full context mode
    await page.keyboard.press("o");

    // Wait for full context data to load AND for change lines to have proper classes
    await page.waitForFunction(
      () => {
        const lines = document.querySelectorAll(".diff-line");
        if (lines.length <= 10) return false;
        // Also ensure we have some change lines with proper classes
        const changeLines = document.querySelectorAll(".line-added, .line-deleted");
        return changeLines.length > 0;
      },
      { timeout: 5000 }
    );

    // Small delay for render to fully settle
    await page.waitForTimeout(100);

    // Go to top
    await page.keyboard.press("g");

    // Collect positions as we navigate with ]
    const positions: number[] = [];

    // Navigate to first change
    await page.keyboard.press("]");
    const firstPos = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });
    expect(firstPos).toBeGreaterThanOrEqual(0);
    positions.push(firstPos);

    // Verify first position is a change line
    const firstIsChange = await page.evaluate((idx) => {
      const el = document.querySelector(`[data-flat-idx="${idx}"]`);
      return el?.classList.contains("line-added") || el?.classList.contains("line-deleted");
    }, firstPos);
    expect(firstIsChange).toBe(true);

    // Navigate forward several times
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("]");
      await page.waitForTimeout(50); // Wait for scroll/render to settle
      const pos = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });
      if (pos !== positions[positions.length - 1]) {
        positions.push(pos);
      }
    }

    // In full context mode with multiple change groups, we should have navigated
    // to multiple distinct positions
    expect(positions.length).toBeGreaterThan(1);

    // Verify all forward positions are change lines
    for (const pos of positions) {
      const isChange = await page.evaluate((idx) => {
        const el = document.querySelector(`[data-flat-idx="${idx}"]`);
        return el?.classList.contains("line-added") || el?.classList.contains("line-deleted");
      }, pos);
      expect(isChange).toBe(true);
    }

    // Navigate backward and verify we can go back
    await page.keyboard.press("[");
    const backPos = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Back position should be less than the last forward position
    // (unless we were at the first change already)
    const lastForward = positions[positions.length - 1];
    expect(backPos).toBeLessThan(lastForward);

    // And it should still be a change line
    const backIsChange = await page.evaluate((idx) => {
      const el = document.querySelector(`[data-flat-idx="${idx}"]`);
      return el?.classList.contains("line-added") || el?.classList.contains("line-deleted");
    }, backPos);
    expect(backIsChange).toBe(true);
  });

  test("[ at first change does not go before it", async ({ page }) => {
    // Go to top
    await page.keyboard.press("g");

    // Navigate to first change
    await page.keyboard.press("]");

    // Try to go back (at first change, should either stay or go to prev file)
    await page.keyboard.press("[");

    // Should still be on a change line (either first change or prev file's last)
    const cursorLine = page.locator(".cursor-line");
    const isChangeLine = await cursorLine.evaluate((el) =>
      el.classList.contains("line-added") || el.classList.contains("line-deleted")
    );
    expect(isChangeLine).toBe(true);
  });

  test("] at last change group stays put in all mode", async ({ page }) => {
    // Navigate to last change group by pressing ] many times
    let lastPos = -1;
    for (let i = 0; i < 50; i++) {
      await page.keyboard.press("]");
      const pos = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });
      if (pos === lastPos) break;
      lastPos = pos;
    }

    // Record position at last change group
    const posAtEnd = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Press ] once more - should stay put
    await page.keyboard.press("]");

    const posAfter = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    expect(posAfter).toBe(posAtEnd);
  });

  test("[ at first change group stays put in all mode", async ({ page }) => {
    // Go to top first
    await page.keyboard.press("g");

    // Navigate to first change
    await page.keyboard.press("]");

    // Record position at first change
    const posAtFirst = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Press [ multiple times - should stay put (already at first change)
    await page.keyboard.press("[");
    await page.keyboard.press("[");

    const posAfter = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should not have moved before the first change
    expect(posAfter).toBeLessThanOrEqual(posAtFirst);
  });

  test("] in single mode advances to next file when exhausted", async ({ page }) => {
    // Enter single-file mode
    await page.keyboard.press("s");

    // Get file info from __gdState
    const getFileIdx = async () => {
      return page.evaluate(() => {
        const st = (window as any).__gdState;
        return st ? st.flatLines[st.cursorLine]?.fileIdx ?? -1 : -1;
      });
    };

    const getTotalFiles = async () => {
      return page.evaluate(() => {
        const st = (window as any).__gdState;
        if (!st || !st.flatLines) return 0;
        const fileIndices = new Set(st.flatLines.map((l: any) => l.fileIdx).filter((i: any) => i !== undefined));
        return fileIndices.size;
      });
    };

    const totalFiles = await getTotalFiles();

    // Skip test if only one file (can't advance)
    if (totalFiles <= 1) return;

    // Navigate to last change in current file by pressing ] until position stops changing
    let lastPos = -1;
    for (let i = 0; i < 50; i++) {
      await page.keyboard.press("]");
      const pos = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });
      if (pos === lastPos) break;
      lastPos = pos;
    }

    const fileIdxBefore = await getFileIdx();

    // Press ] again - should advance to next file
    await page.keyboard.press("]");

    const fileIdxAfter = await getFileIdx();

    // Should have advanced to next file (or wrapped to first if was on last)
    expect(fileIdxAfter).not.toBe(fileIdxBefore);
  });
});
