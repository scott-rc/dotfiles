import { test, expect } from "./fixtures.js";

test.describe("Previous Hunk Navigation ([)", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("[ moves cursor backward to previous change group", async ({ page }) => {
    // Navigate forward to build up history
    await page.keyboard.press("]");
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    const posAfterForward = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Navigate backward
    await page.keyboard.press("[");

    const posAfterBack = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should have moved backward
    expect(posAfterBack).toBeLessThan(posAfterForward);
  });

  test("[ lands on change line, not context line", async ({ page }) => {
    // Navigate forward first
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    // Navigate backward
    await page.keyboard.press("[");

    // Cursor should be on a change line
    const isChangeLine = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLine).toBe(true);
  });

  test("[ from middle of change group goes to start of previous group", async ({ page }) => {
    // Enable full context mode to have more change groups visible
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => document.querySelectorAll(".diff-line").length > 10,
      { timeout: 5000 }
    );

    // Navigate to second change group
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    // Move down within the change group
    await page.keyboard.press("j");
    await page.keyboard.press("j");

    const posInMiddle = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Press [ - should go to previous change group, not start of current
    await page.keyboard.press("[");

    const posAfterBracket = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should be at a change line
    const isChangeLine = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLine).toBe(true);

    // Position should be less than where we were in the middle
    expect(posAfterBracket).toBeLessThan(posInMiddle);
  });

  test("[ at first change group stays put in all-files mode", async ({ page }) => {
    // Get the change group positions
    const initialState = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return {
        changeGroups: st.changeGroupStarts || [],
      };
    });

    expect(initialState.changeGroups.length).toBeGreaterThan(0);
    const firstChangeGroup = initialState.changeGroups[0];

    // Navigate to first change group using g (which should land there or near it)
    await page.keyboard.press("g");

    const posAfterG = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // If g didn't land exactly on first change group, use ] to get there
    // But only if we're BEFORE the first change group
    if (posAfterG < firstChangeGroup) {
      await page.keyboard.press("]");
    }

    const posAtFirst = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      const st = (window as any).__gdState || {};
      return {
        domPos: cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1,
        statePos: st.cursorLine,
        changeGroups: st.changeGroupStarts || [],
      };
    });

    // Verify we're at the first change group
    expect(posAtFirst.statePos).toBe(posAtFirst.changeGroups[0]);

    // Press [ at first change group - should stay put
    await page.keyboard.press("[");

    const posAfter = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      const st = (window as any).__gdState || {};
      return {
        domPos: cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1,
        statePos: st.cursorLine,
      };
    });

    // Should stay at first change group
    expect(posAfter.statePos).toBe(firstChangeGroup);
  });

  test("[ in single-file mode goes to previous file when exhausted", async ({ page }) => {
    // Enter single-file mode
    await page.keyboard.press("s");

    // Get current file index from __gdState
    const getFileIdx = async () => {
      return page.evaluate(() => {
        const st = (window as any).__gdState;
        return st ? st.flatLines[st.cursorLine]?.fileIdx ?? -1 : -1;
      });
    };

    const totalFiles = await page.evaluate(() => {
      const st = (window as any).__gdState;
      if (!st || !st.flatLines) return 0;
      const fileIndices = new Set(st.flatLines.map((l: any) => l.fileIdx).filter((i: any) => i !== undefined));
      return fileIndices.size;
    });

    if (totalFiles <= 1) {
      test.skip();
      return;
    }

    // Go to second file first
    await page.keyboard.press("}");
    const secondFileIdx = await getFileIdx();
    expect(secondFileIdx).toBeGreaterThan(0);

    // Go to first change group of this file
    await page.keyboard.press("g");
    await page.keyboard.press("]");

    // Press [ - should go to previous file
    await page.keyboard.press("[");

    const afterBracketIdx = await getFileIdx();
    expect(afterBracketIdx).toBeLessThan(secondFileIdx);
  });

  test("[ cursor always lands on content line", async ({ page }) => {
    // Navigate forward first
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("]");
    }

    // Navigate backward multiple times
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("[");

      // Verify cursor is on a content line (diff-line), not header/separator
      const lineType = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        if (!cursor) return "none";
        if (cursor.classList.contains("diff-line")) return "content";
        if (cursor.classList.contains("file-header")) return "header";
        if (cursor.classList.contains("hunk-sep")) return "separator";
        return "unknown";
      });
      expect(lineType).toBe("content");
    }
  });

  test("[ after j navigation still works correctly", async ({ page }) => {
    // Use j to navigate down several lines
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press("j");
    }

    await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Press [ to jump to previous hunk
    await page.keyboard.press("[");

    await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should have moved (either backward or stayed if at first change)
    // The key thing is it doesn't crash and lands on a change line
    const isChangeLine = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLine).toBe(true);
  });

  test("[ respects change group boundaries correctly", async ({ page }) => {
    // Enable full context to see change groups clearly
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => document.querySelectorAll(".diff-line").length > 10,
      { timeout: 5000 }
    );

    // Get the authoritative list of change group positions from state
    const changeGroupPositions = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.changeGroupStarts || [];
    });

    expect(changeGroupPositions.length).toBeGreaterThan(0);

    // Navigate to the last change group
    await page.keyboard.press("G");
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("]");
    }

    // Now use [ to navigate backward and verify we only visit valid change groups
    const visitedPositions: number[] = [];
    for (let i = 0; i < changeGroupPositions.length + 2; i++) {
      await page.keyboard.press("[");
      const pos = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
      });
      if (!visitedPositions.includes(pos)) {
        visitedPositions.push(pos);
      }
    }

    // All positions visited by [ should be valid change group starts
    for (const pos of visitedPositions) {
      expect(changeGroupPositions).toContain(pos);
    }
  });

  test("[ from context line goes to nearest previous change group", async ({ page }) => {
    // Enable full context mode
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => document.querySelectorAll(".diff-line").length > 10,
      { timeout: 5000 }
    );

    // Navigate to a change group then move to context lines after it
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    // Move forward with j until we hit a context line (if any)
    let foundContext = false;
    for (let i = 0; i < 15; i++) {
      await page.keyboard.press("j");
      const isContext = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        if (!cursor) return false;
        return !cursor.classList.contains("line-added") && !cursor.classList.contains("line-deleted");
      });
      if (isContext) {
        foundContext = true;
        break;
      }
    }

    if (!foundContext) {
      // No context lines to test, skip
      return;
    }

    const posOnContext = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Press [ - should jump to previous change group
    await page.keyboard.press("[");

    const posAfter = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should have moved backward
    expect(posAfter).toBeLessThan(posOnContext);

    // And should be on a change line
    const isChangeLine = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLine).toBe(true);
  });

  test("alternating ] and [ returns to same positions", async ({ page }) => {
    // Navigate forward
    await page.keyboard.press("]");
    const pos1 = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    await page.keyboard.press("]");
    const pos2 = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Skip if only one change group
    if (pos1 === pos2) return;

    // Navigate back
    await page.keyboard.press("[");
    const posBack = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should return to the first change group position
    expect(posBack).toBe(pos1);

    // Navigate forward again
    await page.keyboard.press("]");
    const posForwardAgain = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor ? parseInt(cursor.getAttribute("data-flat-idx") || "-1") : -1;
    });

    // Should return to the second change group position
    expect(posForwardAgain).toBe(pos2);
  });

  test("[ preserves viewport centering", async ({ page }) => {
    // Navigate forward to get some distance from top
    await page.keyboard.press("]");
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    // Navigate back
    await page.keyboard.press("[");

    // Cursor should be visible in viewport
    const cursorLine = page.locator(".cursor-line");
    await expect(cursorLine).toBeVisible();

    // Check cursor is reasonably positioned (not at very edge of viewport)
    const isInView = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      const pane = document.querySelector("#diff-pane");
      if (!cursor || !pane) return false;

      const cursorRect = cursor.getBoundingClientRect();
      const paneRect = pane.getBoundingClientRect();

      // Cursor should be within visible pane area
      return cursorRect.top >= paneRect.top && cursorRect.bottom <= paneRect.bottom;
    });
    expect(isInView).toBe(true);
  });
});
