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

  test("different file types have different icons", async ({ page }) => {
    const tree = page.locator("#tree");

    // Get icons for different file types by finding entries containing the filenames
    const rsIcon = await tree
      .locator(".tree-entry", { hasText: ".rs" })
      .first()
      .locator(".tree-icon")
      .textContent();
    const jsIcon = await tree
      .locator(".tree-entry", { hasText: ".js" })
      .first()
      .locator(".tree-icon")
      .textContent();
    const mdIcon = await tree
      .locator(".tree-entry", { hasText: ".md" })
      .first()
      .locator(".tree-icon")
      .textContent();

    // All three file types should have distinct icons
    expect(rsIcon).not.toEqual(jsIcon);
    expect(rsIcon).not.toEqual(mdIcon);
    expect(jsIcon).not.toEqual(mdIcon);
  });

  test("file icons have type-specific colors", async ({ page }) => {
    const tree = page.locator("#tree");

    // Find a .rs file entry and check its icon has a color style
    const rsEntry = tree.locator(".tree-entry", { hasText: ".rs" }).first();
    const rsIconElement = rsEntry.locator(".tree-icon");

    const style = await rsIconElement.getAttribute("style");
    expect(style).not.toBeNull();
    expect(style).toContain("color:");
  });

  test("g goes to first tree entry when tree is focused", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry");

    // Focus tree
    await page.keyboard.press("t");

    // Move down a few times
    await page.keyboard.press("j");
    await page.keyboard.press("j");
    await page.keyboard.press("j");

    // Press g to go to top
    await page.keyboard.press("g");

    // First entry should be active
    const firstEntry = entries.first();
    await expect(firstEntry).toHaveClass(/active/);
  });

  test("G goes to last tree entry when tree is focused", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry");

    // Focus tree
    await page.keyboard.press("t");

    // Press G to go to bottom
    await page.keyboard.press("G");

    // Last entry should be active
    const lastEntry = entries.last();
    await expect(lastEntry).toHaveClass(/active/);
  });

  test("Home goes to first tree entry", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry");

    // Focus tree and move down
    await page.keyboard.press("t");
    await page.keyboard.press("j");

    // Press Home
    await page.keyboard.press("Home");

    // First entry should be active
    const firstEntry = entries.first();
    await expect(firstEntry).toHaveClass(/active/);
  });

  test("End goes to last tree entry", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry");

    // Focus tree
    await page.keyboard.press("t");

    // Press End
    await page.keyboard.press("End");

    // Last entry should be active
    const lastEntry = entries.last();
    await expect(lastEntry).toHaveClass(/active/);
  });

  test("za toggles directory collapse", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry");

    // Focus tree
    await page.keyboard.press("t");

    // Find the directory entry (has .dir class)
    const dirEntries = tree.locator(".tree-entry.dir");
    const dirCount = await dirEntries.count();

    // Skip if no directories
    if (dirCount === 0) {
      return;
    }

    // Navigate to first directory
    await page.keyboard.press("g");
    let foundDir = false;
    for (let i = 0; i < 20; i++) {
      const active = tree.locator(".tree-entry.active");
      const isDir = await active.evaluate((el) => el.classList.contains("dir"));
      if (isDir) {
        foundDir = true;
        break;
      }
      await page.keyboard.press("j");
    }

    if (!foundDir) return;

    // Count entries before collapse
    const countBefore = await entries.count();

    // Press z then a
    await page.keyboard.press("z");
    await page.keyboard.press("a");

    // Count should have changed (collapsed or expanded)
    const countAfter = await entries.count();
    expect(countAfter).not.toEqual(countBefore);
  });

  test("zA toggles recursive collapse", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry");

    // Focus tree
    await page.keyboard.press("t");

    // Navigate to first directory
    await page.keyboard.press("g");
    let foundDir = false;
    for (let i = 0; i < 20; i++) {
      const active = tree.locator(".tree-entry.active");
      const isDir = await active.evaluate((el) => el.classList.contains("dir"));
      if (isDir) {
        foundDir = true;
        break;
      }
      await page.keyboard.press("j");
    }

    if (!foundDir) return;

    // Count entries before
    const countBefore = await entries.count();

    // Press z then A (shift+a)
    await page.keyboard.press("z");
    await page.keyboard.press("A");

    // Count should have changed
    const countAfter = await entries.count();
    expect(countAfter).not.toEqual(countBefore);
  });

  test("z followed by other key does nothing special", async ({ page }) => {
    const tree = page.locator("#tree");

    // Focus tree
    await page.keyboard.press("t");

    // Get initial active entry
    await page.keyboard.press("g");
    const initialActive = await tree.locator(".tree-entry.active").getAttribute("data-tree-idx");

    // Press z then j (should just do normal j navigation)
    await page.keyboard.press("z");
    await page.keyboard.press("j");

    // Should have moved down
    const newActive = await tree.locator(".tree-entry.active").getAttribute("data-tree-idx");
    expect(Number(newActive)).toBeGreaterThan(Number(initialActive));
  });
});
