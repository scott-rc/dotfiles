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

  // Inter faces embedded as data URIs: normal 400/500/600/700/800 + italic 400.
  for (const weight of [400, 500, 600, 700, 800]) {
    expect(document).toMatch(
      new RegExp(`font-style:\\s*normal;[^}]*font-weight:\\s*${weight};[^}]*data:font/woff2;base64,`),
    );
  }
  expect(document).toMatch(
    /font-style:\s*italic;[^}]*font-weight:\s*400;[^}]*data:font\/woff2;base64,/,
  );
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
