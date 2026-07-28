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
