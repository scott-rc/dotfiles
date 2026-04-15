import { describe, it, expect } from "vitest";
import {
  isChangeLine,
  findContentLine,
  filePathAt,
  findNextHunk,
  findPrevHunk,
  findNextFile,
  findPrevFile,
} from "../utils/navigation";
import type { DisplayItem } from "../utils/display";

// --- Test helpers ---

function line(kind: "context" | "added" | "deleted", fileIdx = 0, hunkIdx = 0): DisplayItem {
  return {
    type: "line",
    fileIdx,
    hunkIdx,
    line: {
      kind,
      content_html: "",
      raw_content: "",
      old_lineno: kind === "added" ? null : 1,
      new_lineno: kind === "deleted" ? null : 1,
      line_idx: 0,
    },
  };
}

function fileHeader(fileIdx: number): DisplayItem {
  return {
    type: "file-header",
    fileIdx,
    file: { path: `file${fileIdx}.rs`, old_path: null, status: "modified", hunks: [] },
  };
}

function hunkSep(fileIdx: number, hunkIdx: number): DisplayItem {
  return { type: "hunk-sep", fileIdx, hunkIdx };
}

function collapsedContext(fileIdx: number, hunkIdx: number): DisplayItem {
  return {
    type: "collapsed-context",
    fileIdx,
    hunkIdx,
    groupKey: "0-0-0",
    lines: [],
    count: 5,
  };
}

// --- isChangeLine ---

describe("isChangeLine", () => {
  it("returns true for added lines", () => {
    expect(isChangeLine([line("added")], 0)).toBe(true);
  });

  it("returns true for deleted lines", () => {
    expect(isChangeLine([line("deleted")], 0)).toBe(true);
  });

  it("returns false for context lines", () => {
    expect(isChangeLine([line("context")], 0)).toBe(false);
  });

  it("returns false for non-line items", () => {
    expect(isChangeLine([fileHeader(0)], 0)).toBe(false);
    expect(isChangeLine([hunkSep(0, 0)], 0)).toBe(false);
    expect(isChangeLine([collapsedContext(0, 0)], 0)).toBe(false);
  });

  it("returns false for out-of-bounds index", () => {
    expect(isChangeLine([], 0)).toBe(false);
    expect(isChangeLine([line("added")], -1)).toBe(false);
    expect(isChangeLine([line("added")], 5)).toBe(false);
  });
});

// --- findContentLine ---

describe("findContentLine", () => {
  const items: DisplayItem[] = [
    fileHeader(0),     // 0
    hunkSep(0, 0),     // 1
    line("added", 0),  // 2
    line("context", 0),// 3
  ];

  it("returns the index if it is already a line", () => {
    expect(findContentLine(items, 2, 1, 99)).toBe(2);
  });

  it("skips non-line items going forward", () => {
    expect(findContentLine(items, 0, 1, 99)).toBe(2);
  });

  it("skips non-line items going backward", () => {
    expect(findContentLine(items, 3, -1, 99)).toBe(3);
    expect(findContentLine(items, 1, -1, 99)).toBe(99); // no line before index 1
  });

  it("returns fallback when no line found forward", () => {
    expect(findContentLine([fileHeader(0), hunkSep(0, 0)], 0, 1, 42)).toBe(42);
  });

  it("returns fallback when no line found backward", () => {
    expect(findContentLine([fileHeader(0), hunkSep(0, 0)], 1, -1, 42)).toBe(42);
  });

  it("returns fallback for empty items", () => {
    expect(findContentLine([], 0, 1, 7)).toBe(7);
  });

  it("returns fallback for negative start", () => {
    expect(findContentLine(items, -1, -1, 7)).toBe(7);
  });

  it("returns fallback for start beyond length", () => {
    expect(findContentLine(items, 100, 1, 7)).toBe(7);
  });
});

// --- filePathAt ---

describe("filePathAt", () => {
  const items: DisplayItem[] = [
    fileHeader(0),
    line("added", 0),
    fileHeader(1),
    line("deleted", 1),
  ];
  const paths = ["src/main.rs", "src/lib.rs"];

  it("returns path for a line item", () => {
    expect(filePathAt(items, paths, 1)).toBe("src/main.rs");
    expect(filePathAt(items, paths, 3)).toBe("src/lib.rs");
  });

  it("returns path for a file header", () => {
    expect(filePathAt(items, paths, 0)).toBe("src/main.rs");
  });

  it("returns null for out-of-bounds index", () => {
    expect(filePathAt(items, paths, -1)).toBeNull();
    expect(filePathAt(items, paths, 99)).toBeNull();
  });

  it("returns null when fileIdx exceeds paths array", () => {
    const items: DisplayItem[] = [line("added", 5)];
    expect(filePathAt(items, paths, 0)).toBeNull();
  });

  it("returns null for empty items", () => {
    expect(filePathAt([], paths, 0)).toBeNull();
  });
});

// --- findNextHunk ---

describe("findNextHunk", () => {
  // Layout: header, context, added, added, context, deleted, context
  const items: DisplayItem[] = [
    fileHeader(0),       // 0
    line("context", 0),  // 1
    line("added", 0),    // 2  <- change group 1
    line("added", 0),    // 3
    line("context", 0),  // 4
    line("deleted", 0),  // 5  <- change group 2
    line("context", 0),  // 6
  ];

  it("jumps from context before first group to first group", () => {
    expect(findNextHunk(items, 1)).toBe(2);
  });

  it("jumps from inside first group to second group", () => {
    expect(findNextHunk(items, 2)).toBe(5);
    expect(findNextHunk(items, 3)).toBe(5);
  });

  it("jumps from context between groups to second group", () => {
    expect(findNextHunk(items, 4)).toBe(5);
  });

  it("returns null from last group", () => {
    expect(findNextHunk(items, 5)).toBeNull();
  });

  it("returns null from context after last group", () => {
    expect(findNextHunk(items, 6)).toBeNull();
  });

  it("returns null for empty items", () => {
    expect(findNextHunk([], 0)).toBeNull();
  });

  it("handles items with no change lines", () => {
    const noChanges = [fileHeader(0), line("context", 0), line("context", 0)];
    expect(findNextHunk(noChanges, 0)).toBeNull();
  });

  it("handles single change line", () => {
    const single = [line("added", 0)];
    // From the only change line, no next group
    expect(findNextHunk(single, 0)).toBeNull();
  });

  it("skips non-line items between groups", () => {
    const withSeps: DisplayItem[] = [
      line("added", 0),    // 0
      hunkSep(0, 1),       // 1
      line("deleted", 0),  // 2
    ];
    expect(findNextHunk(withSeps, 0)).toBe(2);
  });
});

// --- findPrevHunk ---

describe("findPrevHunk", () => {
  const items: DisplayItem[] = [
    fileHeader(0),       // 0
    line("context", 0),  // 1
    line("added", 0),    // 2  <- change group 1
    line("added", 0),    // 3
    line("context", 0),  // 4
    line("deleted", 0),  // 5  <- change group 2
    line("context", 0),  // 6
  ];

  it("returns null from first change group", () => {
    expect(findPrevHunk(items, 2)).toBeNull();
    expect(findPrevHunk(items, 3)).toBeNull();
  });

  it("jumps from second group to start of first group", () => {
    expect(findPrevHunk(items, 5)).toBe(2);
  });

  it("from context after group, returns adjacent group start", () => {
    expect(findPrevHunk(items, 6)).toBe(5);
  });

  it("from context between groups, returns previous group start", () => {
    expect(findPrevHunk(items, 4)).toBe(2);
  });

  it("returns null from context before any groups", () => {
    expect(findPrevHunk(items, 1)).toBeNull();
  });

  it("returns null for empty items", () => {
    expect(findPrevHunk([], 0)).toBeNull();
  });

  it("handles items with no change lines", () => {
    const noChanges = [fileHeader(0), line("context", 0)];
    expect(findPrevHunk(noChanges, 1)).toBeNull();
  });

  it("navigates three groups correctly", () => {
    const three: DisplayItem[] = [
      line("added", 0),    // 0  <- group 1
      line("context", 0),  // 1
      line("deleted", 0),  // 2  <- group 2
      line("context", 0),  // 3
      line("added", 0),    // 4  <- group 3
    ];
    expect(findPrevHunk(three, 4)).toBe(2);
    expect(findPrevHunk(three, 2)).toBe(0);
    expect(findPrevHunk(three, 0)).toBeNull();
  });
});

// --- findNextFile ---

describe("findNextFile", () => {
  const items: DisplayItem[] = [
    fileHeader(0),       // 0
    line("added", 0),   // 1
    fileHeader(1),       // 2
    line("deleted", 1),  // 3
    fileHeader(2),       // 4
    line("context", 2),  // 5
  ];

  it("jumps from first file to second file's first line", () => {
    const result = findNextFile(items, 1, 1);
    expect(result).toEqual({ cursor: 3, headerIdx: 2 });
  });

  it("jumps from second file to third file", () => {
    const result = findNextFile(items, 3, 3);
    expect(result).toEqual({ cursor: 5, headerIdx: 4 });
  });

  it("returns null from last file", () => {
    expect(findNextFile(items, 5, 5)).toBeNull();
  });

  it("returns null for empty items", () => {
    expect(findNextFile([], 0, 0)).toBeNull();
  });

  it("uses fallback when no line follows file header", () => {
    const noLines: DisplayItem[] = [
      fileHeader(0),
      line("added", 0),
      fileHeader(1),
      // no lines after second header
    ];
    const result = findNextFile(noLines, 0, 42);
    expect(result).toEqual({ cursor: 42, headerIdx: 2 });
  });

  it("skips non-line items after header to find first line", () => {
    const withSep: DisplayItem[] = [
      fileHeader(0),
      line("added", 0),
      fileHeader(1),
      hunkSep(1, 0),
      line("context", 1),
    ];
    const result = findNextFile(withSep, 1, 1);
    expect(result).toEqual({ cursor: 4, headerIdx: 2 });
  });
});

// --- findPrevFile ---

describe("findPrevFile", () => {
  const items: DisplayItem[] = [
    fileHeader(0),       // 0
    line("added", 0),   // 1
    fileHeader(1),       // 2
    line("deleted", 1),  // 3
    fileHeader(2),       // 4
    line("context", 2),  // 5
  ];

  it("jumps from third file to second file's first line", () => {
    const result = findPrevFile(items, 5, 5);
    expect(result).toEqual({ cursor: 3, headerIdx: 2 });
  });

  it("jumps from second file to first file's first line", () => {
    const result = findPrevFile(items, 3, 3);
    expect(result).toEqual({ cursor: 1, headerIdx: 0 });
  });

  it("returns null from first file", () => {
    expect(findPrevFile(items, 1, 1)).toBeNull();
  });

  it("returns null for empty items", () => {
    expect(findPrevFile([], 0, 0)).toBeNull();
  });

  it("returns null when cursor is before any file header", () => {
    const noHeader: DisplayItem[] = [line("added", 0)];
    expect(findPrevFile(noHeader, 0, 0)).toBeNull();
  });

  it("uses fallback when no line follows previous header", () => {
    const sparse: DisplayItem[] = [
      fileHeader(0),       // 0 — no lines under this file
      fileHeader(1),       // 1
      line("added", 1),   // 2
    ];
    // From file 1 content, prev file is header at 0.
    // findContentLine(items, 1, 1, fallback) finds index 2 (line from file 1).
    // This means the cursor lands on a line from the wrong file — a quirk
    // when a file has no content lines of its own.
    const result = findPrevFile(sparse, 2, 42);
    expect(result).toEqual({ cursor: 2, headerIdx: 0 });
  });

  it("truly uses fallback when no lines exist after previous header", () => {
    const sparse: DisplayItem[] = [
      fileHeader(0),       // 0 — no lines at all
    ];
    // cursor at 0, no previous file header → null
    expect(findPrevFile(sparse, 0, 42)).toBeNull();
  });
});
