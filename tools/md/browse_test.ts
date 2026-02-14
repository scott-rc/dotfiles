import { assertEquals } from "@std/assert";
import {
  buildBrowseCmd,
  buildFindCmd,
  buildPickCmd,
  parseSelection,
  shellQuote,
  shouldPage,
} from "./browse.ts";

// shellQuote

Deno.test("shellQuote wraps simple string", () => {
  assertEquals(shellQuote("hello"), "'hello'");
});

Deno.test("shellQuote handles spaces", () => {
  assertEquals(shellQuote("hello world"), "'hello world'");
});

Deno.test("shellQuote escapes single quotes", () => {
  assertEquals(shellQuote("it's"), "'it'\\''s'");
});

Deno.test("shellQuote handles empty string", () => {
  assertEquals(shellQuote(""), "''");
});

Deno.test("shellQuote handles multiple single quotes", () => {
  assertEquals(shellQuote("a'b'c"), "'a'\\''b'\\''c'");
});

// buildFindCmd

Deno.test("buildFindCmd uses default template", () => {
  const cmd = buildFindCmd("/some/dir");
  assertEquals(
    cmd,
    "find '/some/dir' -type f \\( -name '*.md' -o -name '*.mdx' \\)",
  );
});

Deno.test("buildFindCmd uses custom template", () => {
  const cmd = buildFindCmd("/docs", "find {dir} -name '*.md'");
  assertEquals(cmd, "find '/docs' -name '*.md'");
});

Deno.test("buildFindCmd handles dir with spaces", () => {
  const cmd = buildFindCmd("/my docs/notes");
  assertEquals(
    cmd,
    "find '/my docs/notes' -type f \\( -name '*.md' -o -name '*.mdx' \\)",
  );
});

Deno.test("buildFindCmd appends dir when no {dir} placeholder", () => {
  const cmd = buildFindCmd("/docs", "fd -e md");
  assertEquals(cmd, "fd -e md '/docs'");
});

// buildPickCmd

Deno.test("buildPickCmd uses default template", () => {
  const cmd = buildPickCmd();
  assertEquals(cmd, "fzf");
});

Deno.test("buildPickCmd uses custom template", () => {
  const cmd = buildPickCmd("fzf --exact");
  assertEquals(cmd, "fzf --exact");
});

// buildBrowseCmd

Deno.test("buildBrowseCmd creates default pipeline", () => {
  const cmd = buildBrowseCmd("/docs");
  assertEquals(
    cmd,
    "find '/docs' -type f \\( -name '*.md' -o -name '*.mdx' \\) | fzf",
  );
});

Deno.test("buildBrowseCmd uses custom commands", () => {
  const cmd = buildBrowseCmd(
    "/docs",
    "find {dir} -name '*.md'",
    "fzf --exact",
  );
  assertEquals(cmd, "find '/docs' -name '*.md' | fzf --exact");
});

// parseSelection

Deno.test("parseSelection trims and returns path", () => {
  assertEquals(parseSelection("  docs/README.md  \n"), "docs/README.md");
});

Deno.test("parseSelection returns null for empty string", () => {
  assertEquals(parseSelection(""), null);
});

Deno.test("parseSelection returns null for whitespace-only", () => {
  assertEquals(parseSelection("   \n  "), null);
});

Deno.test("parseSelection returns normal path as-is", () => {
  assertEquals(parseSelection("src/main.ts"), "src/main.ts");
});

// shouldPage

Deno.test("shouldPage returns false when --no-pager", () => {
  assertEquals(
    shouldPage({
      noPager: true,
      isTTY: true,
      contentLines: 100,
      terminalRows: 24,
      browsing: false,
    }),
    false,
  );
});

Deno.test("shouldPage returns false when not a TTY", () => {
  assertEquals(
    shouldPage({
      noPager: false,
      isTTY: false,
      contentLines: 100,
      terminalRows: 24,
      browsing: false,
    }),
    false,
  );
});

Deno.test("shouldPage returns true when content exceeds terminal height", () => {
  assertEquals(
    shouldPage({
      noPager: false,
      isTTY: true,
      contentLines: 100,
      terminalRows: 24,
      browsing: false,
    }),
    true,
  );
});

Deno.test("shouldPage returns false when content fits in terminal", () => {
  assertEquals(
    shouldPage({
      noPager: false,
      isTTY: true,
      contentLines: 10,
      terminalRows: 24,
      browsing: false,
    }),
    false,
  );
});

Deno.test("shouldPage returns true when browsing even if content fits", () => {
  assertEquals(
    shouldPage({
      noPager: false,
      isTTY: true,
      contentLines: 5,
      terminalRows: 24,
      browsing: true,
    }),
    true,
  );
});

Deno.test("shouldPage respects --no-pager even when browsing", () => {
  assertEquals(
    shouldPage({
      noPager: true,
      isTTY: true,
      contentLines: 5,
      terminalRows: 24,
      browsing: true,
    }),
    false,
  );
});
