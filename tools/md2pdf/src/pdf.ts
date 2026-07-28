import puppeteer, { type Browser } from "puppeteer";

let browserPromise: Promise<Browser> | undefined;

function getBrowser(): Promise<Browser> {
  browserPromise ??= puppeteer.launch();
  return browserPromise;
}

export async function closeBrowser(): Promise<void> {
  if (!browserPromise) return;
  const browser = await browserPromise;
  browserPromise = undefined;
  await browser.close();
}

export async function htmlToPdf(html: string): Promise<Uint8Array> {
  const browser = await getBrowser();
  const page = await browser.newPage();
  try {
    await page.setContent(html, { waitUntil: "load" });
    return await page.pdf({
      format: "A4",
      margin: { top: "20mm", right: "18mm", bottom: "20mm", left: "18mm" },
      printBackground: true,
    });
  } finally {
    await page.close();
  }
}
