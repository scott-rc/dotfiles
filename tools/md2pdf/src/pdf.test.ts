import { afterAll, expect, test } from "vitest";
import { closeBrowser, htmlToPdf } from "./pdf.js";

afterAll(async () => {
  await closeBrowser();
});

test("produces non-empty PDF bytes with %PDF magic", async () => {
  const pdf = await htmlToPdf("<h1>Hello</h1><p>Smoke test.</p>");
  expect(pdf.length).toBeGreaterThan(0);
  expect(Buffer.from(pdf.subarray(0, 5)).toString("ascii")).toBe("%PDF-");
});

test("single-page mode emits one continuous page for content spanning multiple A4 pages", async () => {
  const longHtml = `<div>${'<p>Filler paragraph to force pagination.</p>'.repeat(200)}</div>`;
  const { getDocument } = await import("pdfjs-dist/legacy/build/pdf.mjs");

  const paged = await getDocument({ data: new Uint8Array(await htmlToPdf(longHtml)) }).promise;
  expect(paged.numPages).toBeGreaterThan(1);

  const single = await getDocument({
    data: new Uint8Array(await htmlToPdf(longHtml, { singlePage: true })),
  }).promise;
  expect(single.numPages).toBe(1);

  const page = await single.getPage(1);
  const { width, height } = page.getViewport({ scale: 1 });
  // A4 width preserved (~595pt), height grows with content.
  expect(width).toBeGreaterThan(590);
  expect(width).toBeLessThan(600);
  expect(height).toBeGreaterThan(842);
});

test("preserves hyperlinks as link annotations", async () => {
  const pdf = await htmlToPdf('<p><a href="https://example.com/target">a link</a></p>');
  const { getDocument } = await import("pdfjs-dist/legacy/build/pdf.mjs");
  const doc = await getDocument({ data: new Uint8Array(pdf) }).promise;
  const page = await doc.getPage(1);
  const annotations = await page.getAnnotations();
  const urls = annotations.map((a) => a.url).filter(Boolean);
  expect(urls).toContain("https://example.com/target");
});
