import { describe, it, expect } from "vitest";
import { mapKey } from "../utils/keyboard";

function key(k: string, opts?: { metaKey?: boolean; ctrlKey?: boolean }) {
  return { key: k, metaKey: opts?.metaKey ?? false, ctrlKey: opts?.ctrlKey ?? false };
}

describe("mapKey", () => {
  it("maps vim navigation keys", () => {
    expect(mapKey(key("j"))).toBe("cursor-down");
    expect(mapKey(key("k"))).toBe("cursor-up");
    expect(mapKey(key("g"))).toBe("top");
    expect(mapKey(key("G"))).toBe("bottom");
    expect(mapKey(key("d"))).toBe("half-page-down");
    expect(mapKey(key("u"))).toBe("half-page-up");
    expect(mapKey(key("z"))).toBe("center-cursor");
  });

  it("maps standard navigation keys", () => {
    expect(mapKey(key("ArrowDown"))).toBe("cursor-down");
    expect(mapKey(key("ArrowUp"))).toBe("cursor-up");
    expect(mapKey(key("Home"))).toBe("top");
    expect(mapKey(key("End"))).toBe("bottom");
    expect(mapKey(key("Enter"))).toBe("cursor-down");
  });

  it("maps hunk/file navigation", () => {
    expect(mapKey(key("]"))).toBe("next-hunk");
    expect(mapKey(key("["))).toBe("prev-hunk");
    expect(mapKey(key("}"))).toBe("next-file");
    expect(mapKey(key("{"))).toBe("prev-file");
  });

  it("maps view mode toggles", () => {
    expect(mapKey(key("s"))).toBe("toggle-single-file");
    expect(mapKey(key("o"))).toBe("toggle-full-context");
    expect(mapKey(key("l"))).toBe("toggle-tree");
    expect(mapKey(key("t"))).toBe("toggle-tree-focus");
  });

  it("maps selection and copy", () => {
    expect(mapKey(key("v"))).toBe("visual-select");
    expect(mapKey(key("y"))).toBe("yank");
    expect(mapKey(key("c"))).toBe("copy-path");
    expect(mapKey(key("C"))).toBe("copy-abs-path");
  });

  it("maps search", () => {
    expect(mapKey(key("/"))).toBe("open-search");
    expect(mapKey(key("k", { metaKey: true }))).toBe("open-search");
    expect(mapKey(key("n"))).toBe("next-match");
    expect(mapKey(key("N"))).toBe("prev-match");
  });

  it("maps misc keys", () => {
    expect(mapKey(key("?"))).toBe("toggle-help");
    expect(mapKey(key("T"))).toBe("cycle-theme");
    expect(mapKey(key("q"))).toBe("quit");
    expect(mapKey(key("c", { ctrlKey: true }))).toBe("quit");
  });

  it("returns null for unmapped keys", () => {
    expect(mapKey(key("x"))).toBeNull();
    expect(mapKey(key("1"))).toBeNull();
    expect(mapKey(key("Escape"))).toBe("cancel-selection");
  });
});
