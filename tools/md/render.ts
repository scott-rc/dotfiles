import type { Token, Tokens } from "marked";
import * as style from "./style.ts";
import { highlightCode } from "./highlight.ts";
import { visibleLength, wordWrap } from "./wrap.ts";

export interface RenderOptions {
  width: number;
}

/** Render an array of marked tokens into a styled terminal string. */
export async function renderTokens(
  tokens: Token[],
  options: RenderOptions,
): Promise<string> {
  const parts: string[] = [];

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];
    const rendered = await renderToken(token, options);
    if (rendered !== null) {
      parts.push(rendered);
    }
  }

  return parts.join("\n\n");
}

async function renderToken(
  token: Token,
  options: RenderOptions,
): Promise<string | null> {
  switch (token.type) {
    case "heading":
      return renderHeading(token as Tokens.Heading, options);
    case "paragraph":
      return renderParagraph(token as Tokens.Paragraph, options);
    case "code":
      return await renderCodeBlock(token as Tokens.Code, options);
    case "blockquote":
      return await renderBlockquote(token as Tokens.Blockquote, options);
    case "list":
      return renderList(token as Tokens.List, options, 0);
    case "hr":
      return renderHr(options);
    case "html":
      return renderParagraph(token as unknown as Tokens.Paragraph, options);
    case "space":
      return null;
    default:
      // Unknown token type — render raw text if available
      if ("text" in token && typeof token.text === "string") {
        return wordWrap(token.text, options.width);
      }
      return null;
  }
}

function renderHeading(
  token: Tokens.Heading,
  _options: RenderOptions,
): string {
  const text = renderInline(token.tokens);
  const prefix = style.marker("#".repeat(token.depth)) + " ";
  const styleFn = [style.h1, style.h2, style.h3, style.h4, style.h5, style.h6][
    token.depth - 1
  ] ?? style.h6;

  return prefix + styleFn(text);
}

function renderParagraph(
  token: Tokens.Paragraph | { text: string; tokens?: Token[] },
  options: RenderOptions,
): string {
  const text = token.tokens ? renderInline(token.tokens) : token.text;
  return wordWrap(text, options.width);
}

async function renderCodeBlock(
  token: Tokens.Code,
  _options: RenderOptions,
): Promise<string> {
  const parts: string[] = [];

  const opening = token.lang ? "```" + token.lang : "```";
  parts.push(style.marker(opening));
  parts.push(await highlightCode(token.text, token.lang));
  parts.push(style.marker("```"));

  return parts.join("\n");
}

async function renderBlockquote(
  token: Tokens.Blockquote,
  options: RenderOptions,
): Promise<string> {
  const prefix = style.marker(">") + " ";
  const innerWidth = options.width - 3;

  const inner = await renderTokens(token.tokens, {
    ...options,
    width: innerWidth,
  });
  const lines = inner.split("\n");

  return lines
    .map((line) => prefix + style.blockquoteText(line))
    .join("\n");
}

function renderList(
  token: Tokens.List,
  options: RenderOptions,
  depth: number,
): string {
  const indent = "  ".repeat(depth);
  const parts: string[] = [];

  for (let i = 0; i < token.items.length; i++) {
    const item = token.items[i];
    const marker = token.ordered
      ? style.listMarker(`${Number(token.start ?? 1) + i}.`) + " "
      : style.listMarker("-") + " ";

    const markerWidth = visibleLength(marker);
    const contentIndent = indent + " ".repeat(markerWidth);
    const contentWidth = options.width - visibleLength(contentIndent);

    const inlineParts: string[] = [];

    for (const child of item.tokens) {
      if (child.type === "list") {
        const nested = renderList(
          child as Tokens.List,
          options,
          depth + 1,
        );
        inlineParts.push(nested);
      } else if (child.type === "text" && "tokens" in child && child.tokens) {
        const text = renderInline(child.tokens as Token[]);
        inlineParts.push(text);
      } else if ("text" in child) {
        inlineParts.push(child.text as string);
      }
    }

    const content = inlineParts.join("\n");
    const wrapped = wordWrap(content, contentWidth);
    const lines = wrapped.split("\n");

    const first = indent + marker + lines[0];
    const rest = lines.slice(1).map((l) => contentIndent + l);

    parts.push([first, ...rest].join("\n"));
  }

  return parts.join("\n");
}

function renderHr(_options: RenderOptions): string {
  return style.hrStyle("---");
}

/** Render frontmatter attributes as a styled key-value block. */
export function renderFrontmatter(
  attrs: Record<string, unknown>,
  width?: number,
): string {
  const entries = Object.entries(attrs);
  if (entries.length === 0) return "";

  const maxKeyLen = Math.max(...entries.map(([k]) => k.length));
  const indent = " ".repeat(maxKeyLen + 2);

  return entries.map(([key, value]) => {
    const paddedKey = key.padEnd(maxKeyLen);
    const formatted = formatValue(value);
    if (width) {
      const firstLineWidth = width - maxKeyLen - 2;
      const wrapped = wordWrap(formatted, firstLineWidth);
      const lines = wrapped.split("\n");
      const styledLines = lines.map((line, i) =>
        i === 0
          ? style.frontmatterKey(paddedKey) + "  " + style.frontmatterValue(line)
          : indent + style.frontmatterValue(line)
      );
      return styledLines.join("\n");
    }
    return style.frontmatterKey(paddedKey) + "  " +
      style.frontmatterValue(formatted);
  }).join("\n");
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (value instanceof Date) return value.toISOString().split("T")[0];
  if (Array.isArray(value)) return value.map(String).join(", ");
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

/** Render inline tokens (bold, italic, code, links, text) into a string. */
export function renderInline(tokens: Token[]): string {
  return tokens.map(renderInlineToken).join("");
}

function renderInlineToken(token: Token): string {
  switch (token.type) {
    case "text":
      if ("tokens" in token && token.tokens) {
        return renderInline(token.tokens as Token[]);
      }
      return token.text as string;
    case "strong":
      return style.marker("**") + style.strongStyle(renderInline((token as Tokens.Strong).tokens)) + style.marker("**");
    case "em":
      return style.marker("*") + style.emStyle(renderInline((token as Tokens.Em).tokens)) + style.marker("*");
    case "codespan":
      return style.codeSpan((token as Tokens.Codespan).text);
    case "link": {
      const link = token as Tokens.Link;
      const text = renderInline(link.tokens);
      return style.marker("[") + style.linkText(text) + style.marker("](") + style.linkUrl(link.href) + style.marker(")");
    }
    case "br":
      return "\n";
    case "escape":
      return (token as Tokens.Escape).text;
    case "html":
      return (token as Tokens.HTML).text;
    default:
      return "text" in token ? (token.text as string) : "";
  }
}
