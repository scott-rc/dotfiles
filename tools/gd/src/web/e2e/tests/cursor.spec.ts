import { test, expect } from "./fixtures.js";

test.describe("Cursor Behavior", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test.describe("Cursor Visibility", () => {
    test("cursor is visible on context lines", async ({ page }) => {
      // Enable full context mode to get context lines
      await page.keyboard.press("o");
      await page.waitForFunction(
        () => document.querySelectorAll(".diff-line").length > 10,
        { timeout: 5000 }
      );

      // Navigate to find a context line (not added/deleted)
      let foundContext = false;
      for (let i = 0; i < 30; i++) {
        const cursorInfo = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          if (!cursor) return null;
          return {
            isAdded: cursor.classList.contains("line-added"),
            isDeleted: cursor.classList.contains("line-deleted"),
            bgColor: window.getComputedStyle(cursor).backgroundColor,
          };
        });

        if (cursorInfo && !cursorInfo.isAdded && !cursorInfo.isDeleted) {
          // Found a context line, verify cursor styling
          expect(cursorInfo.bgColor).toBe("rgb(36, 46, 62)");
          foundContext = true;
          break;
        }
        await page.keyboard.press("j");
      }
      expect(foundContext).toBe(true);
    });

    test("cursor is visible on added lines", async ({ page }) => {
      // Navigate to find an added line
      let foundAdded = false;
      for (let i = 0; i < 30; i++) {
        const cursorInfo = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          if (!cursor) return null;
          return {
            isAdded: cursor.classList.contains("line-added"),
            bgColor: window.getComputedStyle(cursor).backgroundColor,
          };
        });

        if (cursorInfo?.isAdded) {
          // Found an added line, verify cursor styling overrides the added-line styling
          // --bg-cursor is rgb(36, 46, 62), NOT --bg-added which is rgb(22, 39, 27)
          expect(cursorInfo.bgColor).toBe("rgb(36, 46, 62)");
          foundAdded = true;
          break;
        }
        await page.keyboard.press("j");
      }
      expect(foundAdded).toBe(true);
    });

    test("cursor is visible on deleted lines", async ({ page }) => {
      // Check if fixture has any deleted lines
      const hasDeletedLines = await page.evaluate(() =>
        document.querySelectorAll(".diff-line.line-deleted").length > 0
      );

      // Skip if no deleted lines in fixture (working tree diff may only have additions)
      if (!hasDeletedLines) {
        test.skip();
        return;
      }

      // Navigate to find a deleted line
      let foundDeleted = false;
      for (let i = 0; i < 50; i++) {
        const cursorInfo = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          if (!cursor) return null;
          return {
            isDeleted: cursor.classList.contains("line-deleted"),
            bgColor: window.getComputedStyle(cursor).backgroundColor,
          };
        });

        if (cursorInfo?.isDeleted) {
          // Found a deleted line, verify cursor styling overrides the deleted-line styling
          // --bg-cursor is rgb(36, 46, 62), NOT --bg-deleted which is rgb(50, 24, 24)
          expect(cursorInfo.bgColor).toBe("rgb(36, 46, 62)");
          foundDeleted = true;
          break;
        }
        await page.keyboard.press("j");
      }
      expect(foundDeleted).toBe(true);
    });

    test("cursor styling has higher specificity than line-type styling", async ({ page }) => {
      // Navigate through the diff and verify cursor is always visible
      const positions = new Set<number>();

      for (let i = 0; i < 15; i++) {
        await page.keyboard.press("j");

        const cursorInfo = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          if (!cursor) return null;
          const style = window.getComputedStyle(cursor);
          return {
            pos: cursor.getAttribute("data-flat-idx"),
            bgColor: style.backgroundColor,
            isAdded: cursor.classList.contains("line-added"),
            isDeleted: cursor.classList.contains("line-deleted"),
          };
        });

        if (cursorInfo && cursorInfo.pos) {
          positions.add(parseInt(cursorInfo.pos));
          // Regardless of line type, cursor should have cursor background color
          expect(cursorInfo.bgColor).toBe("rgb(36, 46, 62)");
        }
      }

      // Verify we actually visited multiple positions
      expect(positions.size).toBeGreaterThan(1);
    });
  });

  test.describe("Directional Navigation", () => {
    test("k navigates backward when landing on header", async ({ page }) => {
      // This tests the fix for the bug where k would incorrectly search forward
      // Go to top
      await page.keyboard.press("g");

      // Move forward past first file header to get some content lines
      await page.keyboard.press("]");
      await page.keyboard.press("j");
      await page.keyboard.press("j");
      await page.keyboard.press("j");

      // Record position
      const posBeforeK = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      // Navigate up - cursor should move to a LOWER index (backward)
      await page.keyboard.press("k");

      const posAfterK = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      // Position should be less (we moved backward)
      expect(posAfterK).toBeLessThan(posBeforeK);

      // And cursor should still be on a content line
      const isContentLine = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor?.classList.contains("diff-line");
      });
      expect(isContentLine).toBe(true);
    });

    test("k skips headers in backward direction", async ({ page }) => {
      // Navigate to second file
      await page.keyboard.press("}");

      // Get current position (should be first content line of second file)
      const posAtSecondFile = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      // Press k to go back - should skip file header and land on content line
      await page.keyboard.press("k");

      const posAfterK = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      // Should have moved backward
      expect(posAfterK).toBeLessThan(posAtSecondFile);

      // Verify it's a content line, not a header
      const lineType = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        if (cursor?.classList.contains("file-header")) return "header";
        if (cursor?.classList.contains("hunk-sep")) return "separator";
        if (cursor?.classList.contains("diff-line")) return "content";
        return "unknown";
      });
      expect(lineType).toBe("content");
    });

    test("j skips headers in forward direction", async ({ page }) => {
      // Go to top
      await page.keyboard.press("g");

      // Navigate with j multiple times and verify we never land on non-content
      for (let i = 0; i < 20; i++) {
        await page.keyboard.press("j");

        const lineType = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          if (cursor?.classList.contains("file-header")) return "header";
          if (cursor?.classList.contains("hunk-sep")) return "separator";
          if (cursor?.classList.contains("diff-line")) return "content";
          return "unknown";
        });
        expect(lineType).toBe("content");
      }
    });

    test("cursor never lands on file header during j/k navigation", async ({ page }) => {
      // Navigate around the diff extensively
      const keys = ["j", "j", "j", "k", "j", "j", "k", "k", "j", "d", "u", "j", "k"];

      for (const key of keys) {
        await page.keyboard.press(key);

        // After each key, verify cursor is NOT on a header
        const isOnHeader = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          return cursor?.classList.contains("file-header") || false;
        });
        expect(isOnHeader).toBe(false);
      }
    });

    test("cursor never lands on hunk separator during j/k navigation", async ({ page }) => {
      // Navigate through the diff
      for (let i = 0; i < 30; i++) {
        await page.keyboard.press("j");

        const isOnSeparator = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          // Hunk separators have class .hunk-sep, not .diff-line
          return cursor?.classList.contains("hunk-sep") || false;
        });
        expect(isOnSeparator).toBe(false);
      }
    });

    test("cursor always on content line after random navigation", async ({ page }) => {
      // Generate a sequence of navigation keys
      const navKeys = ["j", "k", "d", "u", "g", "G", "]", "[", "}", "{"];

      for (let i = 0; i < 50; i++) {
        const key = navKeys[Math.floor(Math.random() * navKeys.length)];
        await page.keyboard.press(key);

        const cursorType = await page.evaluate(() => {
          const cursor = document.querySelector(".cursor-line");
          if (!cursor) return "none";
          if (cursor.classList.contains("diff-line")) return "content";
          if (cursor.classList.contains("file-header")) return "header";
          if (cursor.classList.contains("hunk-sep")) return "separator";
          return "unknown";
        });

        // Cursor should always be on content or not exist (if we're at boundary)
        expect(["content", "none"]).toContain(cursorType);
      }
    });
  });

  test.describe("Edge Cases", () => {
    test("k at top of diff stays on first content line", async ({ page }) => {
      // Go to top
      await page.keyboard.press("g");

      // Get position
      const posAtTop = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      // Press k multiple times - should stay put at top
      await page.keyboard.press("k");
      await page.keyboard.press("k");
      await page.keyboard.press("k");

      const posAfterK = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      expect(posAfterK).toBeLessThanOrEqual(posAtTop);
    });

    test("j at bottom of diff stays on last content line", async ({ page }) => {
      // Go to bottom
      await page.keyboard.press("G");

      const posAtBottom = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      // Press j multiple times - should stay at bottom
      await page.keyboard.press("j");
      await page.keyboard.press("j");
      await page.keyboard.press("j");

      const posAfterJ = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });

      expect(posAfterJ).toBeGreaterThanOrEqual(posAtBottom);
    });

    test("cursor position persists through view mode toggles", async ({ page }) => {
      // Navigate to a specific position
      await page.keyboard.press("]");
      await page.keyboard.press("j");
      await page.keyboard.press("j");

      // Toggle single-file view and back
      await page.keyboard.press("s");
      await page.keyboard.press("s");

      // Cursor should still be on a valid content line
      const isContentLine = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor?.classList.contains("diff-line") || false;
      });
      expect(isContentLine).toBe(true);
    });
  });
});
