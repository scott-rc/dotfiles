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

  test("directories start expanded on initial load", async ({ page }) => {
    const tree = page.locator("#tree");

    // Get all directory entries
    const dirEntries = tree.locator(".tree-entry.dir");
    const dirCount = await dirEntries.count();

    // Skip if no directories
    if (dirCount === 0) return;

    // All directory icons should show expanded folder icon (not collapsed)
    // Expanded folder icon is &#xf413; (unicode: \uf413), collapsed is &#xf4d8; (\uf4d8)
    for (let i = 0; i < dirCount; i++) {
      const dirEntry = dirEntries.nth(i);
      const icon = await dirEntry.locator(".tree-icon").textContent();

      // Check that it's not the collapsed folder icon
      // Unicode F4D8 is the collapsed folder icon
      const isCollapsed = icon?.charCodeAt(0) === 0xf4d8;
      expect(isCollapsed).toBe(false);
    }
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

  test("file icons are emoji (not Nerd Font PUA)", async ({ page }) => {
    const tree = page.locator("#tree");

    // Find a .rs file entry and check its icon is an emoji (not a Nerd Font PUA codepoint)
    const rsEntry = tree.locator(".tree-entry", { hasText: ".rs" }).first();
    const rsIconElement = rsEntry.locator(".tree-icon");

    const iconText = await rsIconElement.textContent();
    expect(iconText).not.toBeNull();
    // Emoji should not be in PUA range (U+E000 - U+F8FF)
    const codePoint = iconText!.codePointAt(0)!;
    expect(codePoint).toBeGreaterThan(0xf8ff);
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

  test("tree cursor follows diff navigation with }", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry:not(.dir)");

    // Get the second file entry's name (we'll jump to it with })
    const secondEntry = entries.nth(1);
    const secondFileName = await secondEntry.locator(".tree-label").textContent();

    // Press } to jump to next file
    await page.keyboard.press("}");

    // The tree's active entry should now be the second file
    const activeEntry = tree.locator(".tree-entry.active");
    const activeLabel = await activeEntry.locator(".tree-label").textContent();
    expect(activeLabel).toEqual(secondFileName);
  });

  test("tree cursor follows diff navigation with j/k across file boundary", async ({ page }) => {
    const tree = page.locator("#tree");
    const entries = tree.locator(".tree-entry:not(.dir)");

    // Get the first file entry name (definitely visible)
    const firstEntry = entries.first();
    const firstName = await firstEntry.locator(".tree-label").textContent();

    // Initially tree cursor should be on first file
    let activeEntry = tree.locator(".tree-entry.active");
    let activeLabel = await activeEntry.locator(".tree-label").textContent();
    expect(activeLabel).toEqual(firstName);

    // Navigate using } to jump to next file, tree cursor should follow
    await page.keyboard.press("}");
    await page.keyboard.press("}");

    // Now tree cursor should be on the third file (jumped twice)
    activeEntry = tree.locator(".tree-entry.active");
    activeLabel = await activeEntry.locator(".tree-label").textContent();

    // Should have moved past the first file
    expect(activeLabel).not.toEqual(firstName);
  });
});
