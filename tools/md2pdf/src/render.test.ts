import { readFile } from "node:fs/promises";
import { expect, test } from "vitest";
import { renderMarkdown } from "./render.js";

const fixture = await readFile(new URL("testdata/fixture.md", import.meta.url), "utf8");

test("document embeds typography rules, page-break CSS, and Inter font faces", async () => {
  const { document } = await renderMarkdown(fixture);

  // Typography plugin actually compiled -- not a silently empty Tailwind build.
  // Tailwind v4 emits nested selectors: `.prose { :where(h2) ... }`.
  expect(document).toContain("var(--tw-prose-body)");
  expect(document).toMatch(/:where\(h2\)/);

  // Headings stay attached to the content that follows them.
  expect(document).toMatch(/break-after:\s*avoid/);

  // Inter faces embedded as data URIs, covering every weight/style pair the
  // compiled prose CSS can request.
  const faces: Array<[number, string]> = [
    [400, "normal"],
    [500, "normal"],
    [600, "normal"],
    [700, "normal"],
    [800, "normal"],
    [900, "normal"],
    [400, "italic"],
    [500, "italic"],
    [600, "italic"],
    [700, "italic"],
  ];
  for (const [weight, style] of faces) {
    expect(document).toMatch(
      new RegExp(`font-style:\\s*${style};[^}]*font-weight:\\s*${weight};[^}]*data:font/woff2;base64,`),
    );
  }
});

test("body renders soft line breaks as <br>", async () => {
  const { body } = await renderMarkdown(fixture);
  expect(body).toContain("<br");
});

test("fixture body markup matches snapshot", async () => {
  const { body } = await renderMarkdown(fixture);
  expect(body).toMatchSnapshot();
});

test("body markup is deterministic across renders", async () => {
  const [a, b] = await Promise.all([renderMarkdown(fixture), renderMarkdown(fixture)]);
  expect(a.body).toBe(b.body);
});
