import { describe, it, expect } from "vitest";
import { flattenFiles, ITEM_HEIGHTS } from "../utils/display";
import type { WebDiffFile } from "../utils/types";

function makeFile(
  path: string,
  hunkLineCounts: number[],
  kind: "context" | "added" | "deleted" = "context",
): WebDiffFile {
  return {
    path,
    old_path: null,
    status: "modified",
    hunks: hunkLineCounts.map((count) => ({
      old_start: 1,
      new_start: 1,
      lines: Array.from({ length: count }, (_, i) => ({
        kind,
        content_html: `line ${i}`,
        raw_content: `line ${i}`,
        old_lineno: i + 1,
        new_lineno: i + 1,
        line_idx: i,
      })),
    })),
  };
}

describe("flattenFiles", () => {
  it("returns empty array for no files", () => {
    expect(flattenFiles([])).toEqual([]);
  });

  it("flattens single file with one hunk (short context kept)", () => {
    const items = flattenFiles([makeFile("a.rs", [3])]);
    // 3 context lines <= threshold, so kept individual: 1 header + 3 lines
    expect(items.length).toBe(4);
    expect(items[0].type).toBe("file-header");
    expect(items[1].type).toBe("line");
    expect(items[3].type).toBe("line");
  });

  it("collapses long context runs", () => {
    const items = flattenFiles([makeFile("a.rs", [6])]);
    // 6 context lines > threshold: 1 header + 1 collapsed-context
    expect(items.length).toBe(2);
    expect(items[0].type).toBe("file-header");
    expect(items[1].type).toBe("collapsed-context");
    if (items[1].type === "collapsed-context") {
      expect(items[1].count).toBe(6);
      expect(items[1].groupKey).toBe("0-0-0");
    }
  });

  it("expands collapsed groups when key is in expandedGroups", () => {
    const expanded = new Set(["0-0-0"]);
    const items = flattenFiles([makeFile("a.rs", [6])], expanded);
    // 6 context lines expanded: 1 header + 6 lines
    expect(items.length).toBe(7);
    expect(items.every((i) => i.type !== "collapsed-context")).toBe(true);
  });

  it("inserts hunk separators between hunks", () => {
    // Use "added" lines so they don't get grouped as context
    const items = flattenFiles([makeFile("a.rs", [2, 2], "added")]);
    // 1 header + 2 lines + 1 sep + 2 lines = 6
    expect(items.length).toBe(6);
    expect(items[0].type).toBe("file-header");
    expect(items[3].type).toBe("hunk-sep");
  });

  it("flattens multiple files", () => {
    const items = flattenFiles([
      makeFile("a.rs", [1], "added"),
      makeFile("b.rs", [1], "added"),
    ]);
    expect(items.length).toBe(4);
    expect(items[0].type).toBe("file-header");
    expect(items[2].type).toBe("file-header");
  });

  it("assigns unique groupKeys per hunk", () => {
    // Two hunks each with >3 context lines
    const file: WebDiffFile = {
      path: "a.rs",
      old_path: null,
      status: "modified",
      hunks: [
        { old_start: 1, new_start: 1, lines: Array.from({ length: 5 }, (_, i) => ({
          kind: "context" as const, content_html: "", raw_content: "",
          old_lineno: i, new_lineno: i, line_idx: i,
        })) },
        { old_start: 10, new_start: 10, lines: Array.from({ length: 5 }, (_, i) => ({
          kind: "context" as const, content_html: "", raw_content: "",
          old_lineno: i + 10, new_lineno: i + 10, line_idx: i,
        })) },
      ],
    };
    const items = flattenFiles([file]);
    const collapsed = items.filter((i) => i.type === "collapsed-context");
    expect(collapsed.length).toBe(2);
    if (collapsed[0].type === "collapsed-context" && collapsed[1].type === "collapsed-context") {
      expect(collapsed[0].groupKey).toBe("0-0-0");
      expect(collapsed[1].groupKey).toBe("0-1-0");
    }
  });
});

describe("ITEM_HEIGHTS", () => {
  it("has expected values", () => {
    expect(ITEM_HEIGHTS["file-header"]).toBe(35);
    expect(ITEM_HEIGHTS["hunk-sep"]).toBe(20);
    expect(ITEM_HEIGHTS.line).toBe(20);
    expect(ITEM_HEIGHTS["collapsed-context"]).toBe(28);
  });
});
