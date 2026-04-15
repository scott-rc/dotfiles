import { describe, it, expect } from "vitest";
import { isDiffData } from "../utils/types";

describe("isDiffData", () => {
  it("returns true for valid DiffData message", () => {
    const msg = {
      type: "DiffData",
      files: [],
      tree: [],
      branch: "main",
      source_label: "working tree",
    };
    expect(isDiffData(msg)).toBe(true);
  });

  it("returns false for null", () => {
    expect(isDiffData(null)).toBe(false);
  });

  it("returns false for non-object", () => {
    expect(isDiffData("string")).toBe(false);
    expect(isDiffData(42)).toBe(false);
  });

  it("returns false for wrong type field", () => {
    expect(isDiffData({ type: "Other" })).toBe(false);
  });

  it("returns false for missing type field", () => {
    expect(isDiffData({ files: [], tree: [] })).toBe(false);
  });

  it("returns false when files is missing", () => {
    expect(isDiffData({ type: "DiffData", tree: [] })).toBe(false);
  });

  it("returns false when tree is missing", () => {
    expect(isDiffData({ type: "DiffData", files: [] })).toBe(false);
  });

  it("returns false when files is not an array", () => {
    expect(isDiffData({ type: "DiffData", files: "not-array", tree: [] })).toBe(false);
  });

  it("returns false when tree is not an array", () => {
    expect(isDiffData({ type: "DiffData", files: [], tree: {} })).toBe(false);
  });

  it("returns false for undefined", () => {
    expect(isDiffData(undefined)).toBe(false);
  });
});
