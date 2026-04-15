import { test, expect } from "./fixtures.js";

test.describe("Full Context Mode Navigation", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
    // Enable full context mode for all tests
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => document.querySelectorAll(".diff-line").length > 10,
      { timeout: 5000 }
    );
  });

  test("] skips context lines and lands on change group starts", async ({ page }) => {
    // Get change group positions from state
    const changeGroups = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.changeGroupStarts || [];
    });
    expect(changeGroups.length).toBeGreaterThan(0);

    // Go to top
    await page.keyboard.press("g");

    // Navigate forward and verify we land exactly on change group starts
    const visitedPositions: number[] = [];
    for (let i = 0; i < Math.min(changeGroups.length + 2, 10); i++) {
      await page.keyboard.press("]");
      const pos = await page.evaluate(() => {
        const st = (window as any).__gdState || {};
        return st.cursorLine;
      });
      if (!visitedPositions.includes(pos)) {
        visitedPositions.push(pos);
      }
    }

    // Every position visited by ] should be a valid change group start
    for (const pos of visitedPositions) {
      expect(changeGroups).toContain(pos);
    }

    // Verify we visited change groups in order
    for (let i = 1; i < visitedPositions.length; i++) {
      expect(visitedPositions[i]).toBeGreaterThan(visitedPositions[i - 1]);
    }
  });

  test("] does not stop on context lines between change groups", async ({ page }) => {
    // Go to first change group
    await page.keyboard.press("g");
    await page.keyboard.press("]");

    const pos1 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Press ] again to go to next change group
    await page.keyboard.press("]");

    const pos2 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // If positions are different, there's a gap - check no context lines between are in changeGroupStarts
    if (pos1 !== pos2) {
      const changeGroups = await page.evaluate(() => {
        const st = (window as any).__gdState || {};
        return st.changeGroupStarts || [];
      });

      // Verify pos2 is a change group start, not just any line
      expect(changeGroups).toContain(pos2);

      // Verify cursor is on a change line (not context)
      const isChangeLine = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
      });
      expect(isChangeLine).toBe(true);
    }
  });

  test("[ skips context lines when navigating backward", async ({ page }) => {
    // Navigate forward to build up some distance
    await page.keyboard.press("g");
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press("]");
    }

    const posAfterForward = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Navigate backward
    await page.keyboard.press("[");

    const posAfterBack = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Should have moved backward
    expect(posAfterBack).toBeLessThan(posAfterForward);

    // Position should be a valid change group start
    const changeGroups = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.changeGroupStarts || [];
    });
    expect(changeGroups).toContain(posAfterBack);
  });

  test("alternating ] and [ is symmetric in full context mode", async ({ page }) => {
    // Go to top first
    await page.keyboard.press("g");

    // Navigate to first change group
    await page.keyboard.press("]");
    const pos1 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Navigate to second change group
    await page.keyboard.press("]");
    const pos2 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Skip test if only one change group
    if (pos1 === pos2) {
      return;
    }

    // Navigate to third change group (if exists)
    await page.keyboard.press("]");
    const pos3 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Navigate back and verify symmetry
    await page.keyboard.press("[");
    const backPos1 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // If we advanced to pos3, going back should return to pos2
    if (pos3 !== pos2) {
      expect(backPos1).toBe(pos2);
    } else {
      // We were at pos2, going back should return to pos1
      expect(backPos1).toBe(pos1);
    }

    // Navigate back again
    await page.keyboard.press("[");
    const backPos2 = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Should have moved further back
    expect(backPos2).toBeLessThanOrEqual(backPos1);

    // Navigate forward again to verify round trip
    await page.keyboard.press("]");
    const forwardAgain = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Should be at the same position we were before the last [
    expect(forwardAgain).toBe(backPos1);
  });

  test("multiple round trips maintain consistent positions", async ({ page }) => {
    // Go to a middle change group
    await page.keyboard.press("g");
    await page.keyboard.press("]");
    await page.keyboard.press("]");
    await page.keyboard.press("]");

    const startPos = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Do multiple round trips: [ then ]
    for (let i = 0; i < 3; i++) {
      await page.keyboard.press("[");
      await page.keyboard.press("]");
    }

    const endPos = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Should end up at same position
    expect(endPos).toBe(startPos);
  });

  test("] from context line jumps to next change group", async ({ page }) => {
    // Navigate to a change group
    await page.keyboard.press("g");
    await page.keyboard.press("]");

    // Move forward with j until we find a context line (or give up after 20 tries)
    let foundContext = false;
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("j");
      const isContext = await page.evaluate(() => {
        const cursor = document.querySelector(".cursor-line");
        if (!cursor) return false;
        return !cursor.classList.contains("line-added") &&
               !cursor.classList.contains("line-deleted");
      });
      if (isContext) {
        foundContext = true;
        break;
      }
    }

    if (!foundContext) {
      // No context lines in this file, skip test
      return;
    }

    const contextPos = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Press ] from context line - should jump to next change group
    await page.keyboard.press("]");

    const newPos = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Should have moved forward
    expect(newPos).toBeGreaterThan(contextPos);

    // Should be on a change line
    const isChangeLine = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLine).toBe(true);

    // Should be a valid change group start
    const changeGroups = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.changeGroupStarts || [];
    });
    expect(changeGroups).toContain(newPos);
  });

  test("navigation works correctly after toggling full context off and on", async ({ page }) => {
    // Get positions in full context mode
    await page.keyboard.press("g");
    await page.keyboard.press("]");
    const posInFull = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Toggle full context off
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => {
        const st = (window as any).__gdState || {};
        return st.fullContext === false;
      },
      { timeout: 2000 }
    );

    // Navigate in non-full context mode
    await page.keyboard.press("g");
    await page.keyboard.press("]");
    const posInNormal = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Cursor should still be on a change line
    const isChangeLineNormal = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLineNormal).toBe(true);

    // Toggle full context back on
    await page.keyboard.press("o");
    await page.waitForFunction(
      () => {
        const st = (window as any).__gdState || {};
        return st.fullContext === true;
      },
      { timeout: 2000 }
    );

    // Navigate again
    await page.keyboard.press("g");
    await page.keyboard.press("]");
    const posInFullAgain = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.cursorLine;
    });

    // Should still be on a change line
    const isChangeLineFull = await page.evaluate(() => {
      const cursor = document.querySelector(".cursor-line");
      return cursor?.classList.contains("line-added") || cursor?.classList.contains("line-deleted");
    });
    expect(isChangeLineFull).toBe(true);
  });

  test("change group computation is correct in full context mode", async ({ page }) => {
    // Get all change group starts
    const changeGroups = await page.evaluate(() => {
      const st = (window as any).__gdState || {};
      return st.changeGroupStarts || [];
    });

    expect(changeGroups.length).toBeGreaterThan(0);

    // Verify each change group start is actually the start of a contiguous block
    for (const startIdx of changeGroups) {
      // The line at startIdx should be a change line
      const isStartChange = await page.evaluate((idx) => {
        const st = (window as any).__gdState || {};
        const line = st.flatLines[idx];
        if (!line || line.type !== 'line') return false;
        return line.data.kind === 'added' || line.data.kind === 'deleted';
      }, startIdx);
      expect(isStartChange).toBe(true);

      // The line before startIdx (if exists) should NOT be a change line
      if (startIdx > 0) {
        const isPrevChange = await page.evaluate((idx) => {
          const st = (window as any).__gdState || {};
          const line = st.flatLines[idx - 1];
          if (!line || line.type !== 'line') return false;
          return line.data.kind === 'added' || line.data.kind === 'deleted';
        }, startIdx);
        expect(isPrevChange).toBe(false);
      }
    }
  });
});
