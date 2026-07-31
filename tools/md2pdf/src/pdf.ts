import puppeteer, { type Browser, type Page } from "puppeteer";

export interface PdfOptions {
  /** Emit one continuous page sized to the content instead of A4 pagination. */
  singlePage?: boolean;
}

// A4 at Chromium's 96 CSS px per inch: 210mm wide, 20mm/18mm print margins.
const a4WidthPx = Math.round((210 / 25.4) * 96);
const pageMargin = { top: "20mm", right: "18mm", bottom: "20mm", left: "18mm" };

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

async function printSinglePage(page: Page): Promise<Uint8Array> {
  // Print with zero margins and the margins applied as body padding instead,
  // at a px-denominated paper width equal to the viewport width -- the layout
  // used for measuring scrollHeight is then exactly the layout that prints,
  // so the page height fits the content with no reflow drift.
  await page.setViewport({ width: a4WidthPx, height: 600 });
  const heightPx = await page.evaluate(
    ({ top, right, bottom, left }) => {
      document.body.style.padding = `${top} ${right} ${bottom} ${left}`;
      return document.documentElement.scrollHeight;
    },
    pageMargin,
  );
  return await page.pdf({
    width: `${a4WidthPx}px`,
    height: `${Math.ceil(heightPx)}px`,
    printBackground: true,
  });
}

export async function htmlToPdf(html: string, options: PdfOptions = {}): Promise<Uint8Array> {
  const browser = await getBrowser();
  const page = await browser.newPage();
  try {
    await page.setContent(html, { waitUntil: "load" });
    await page.evaluate(() => document.fonts.ready);
    if (options.singlePage) return await printSinglePage(page);
    return await page.pdf({
      format: "A4",
      margin: pageMargin,
      printBackground: true,
    });
  } finally {
    await page.close();
  }
}
