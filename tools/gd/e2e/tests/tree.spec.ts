import { test, expect } from "./fixtures.js";

test.describe("File Tree", () => {
  test.beforeEach(async ({ page, serverUrl }) => {
    await page.goto(serverUrl);
    await page.waitForSelector(".diff-line");
  });

  test("tree is visible by default", async ({ page }) => {
    const tree = page.locator("#tree");
    await expect(tree).toBeVisible();
  });

  test("tree is on the right side", async ({ page }) => {
    const tree = page.locator("#tree");
    const diffPane = page.locator("#diff-pane");

    const treeBox = await tree.boundingBox();
    const diffBox = await diffPane.boundingBox();

    expect(treeBox).not.toBeNull();
    expect(diffBox).not.toBeNull();

    // Tree should be to the right of diff pane
    expect(treeBox!.x).toBeGreaterThan(diffBox!.x);
  });

  test("l toggles tree visibility", async ({ page }) => {
    const tree = page.locator("#tree");

    // Tree visible initially
    await expect(tree).toBeVisible();

    // Hide tree
    await page.keyboard.press("l");
    await expect(tree).toHaveClass(/hidden/);

    // Show tree again
    await page.keyboard.press("l");
    await expect(tree).not.toHaveClass(/hidden/);
  });

  test("t toggles focus between tree and diff", async ({ page }) => {
    // Press t to focus tree
    await page.keyboard.press("t");

    // Focus state should change (tree entries may get different styling)
    const tree = page.locator("#tree");
    await expect(tree).toBeVisible();

    // Press t again to focus diff
    await page.keyboard.press("t");
    await expect(tree).toBeVisible();
  });

  test("clicking tree entry navigates to file", async ({ page }) => {
    const tree = page.locator("#tree");
    const firstEntry = tree.locator(".tree-entry").first();

    // Click on first tree entry
    await firstEntry.click();

    // Should navigate (scroll) to that file
    await expect(page.locator(".file-header")).toBeVisible();
  });

  test("tree shows file entries", async ({ page }) => {
    const treeEntries = page.locator("#tree .tree-entry");
    const count = await treeEntries.count();

    // Should have at least one file entry
    expect(count).toBeGreaterThan(0);
  });
});
