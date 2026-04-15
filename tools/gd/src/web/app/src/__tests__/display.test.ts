import { describe, it, expect } from "vitest";
import { flattenFiles, ITEM_HEIGHTS } from "../utils/display";
import type { WebDiffFile } from "../utils/types";

function makeFile(path: string, hunkLineCounts: number[]): WebDiffFile {
  return {
    path,
    old_path: null,
    status: "modified",
    hunks: hunkLineCounts.map((count) => ({
      old_start: 1,
      new_start: 1,
      lines: Array.from({ length: count }, (_, i) => ({
        kind: "context" as const,
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

  it("flattens single file with one hunk", () => {
    const items = flattenFiles([makeFile("a.rs", [3])]);
    expect(items.length).toBe(4); // 1 header + 3 lines
    expect(items[0].type).toBe("file-header");
    expect(items[1].type).toBe("line");
    expect(items[3].type).toBe("line");
  });

  it("inserts hunk separators between hunks", () => {
    const items = flattenFiles([makeFile("a.rs", [2, 2])]);
    // 1 header + 2 lines + 1 sep + 2 lines = 6
    expect(items.length).toBe(6);
    expect(items[0].type).toBe("file-header");
    expect(items[3].type).toBe("hunk-sep");
  });

  it("flattens multiple files", () => {
    const items = flattenFiles([makeFile("a.rs", [1]), makeFile("b.rs", [1])]);
    // file1: 1 header + 1 line = 2
    // file2: 1 header + 1 line = 2
    expect(items.length).toBe(4);
    expect(items[0].type).toBe("file-header");
    expect(items[2].type).toBe("file-header");
  });
});

describe("ITEM_HEIGHTS", () => {
  it("has expected values", () => {
    expect(ITEM_HEIGHTS["file-header"]).toBe(35);
    expect(ITEM_HEIGHTS["hunk-sep"]).toBe(20);
    expect(ITEM_HEIGHTS.line).toBe(20);
  });
});
