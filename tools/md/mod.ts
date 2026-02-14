import { Lexer } from "marked";
import { renderTokens, type RenderOptions } from "./render.ts";

/** Render a markdown string into styled terminal output. */
export function renderMarkdown(
  markdown: string,
  options: RenderOptions,
): string {
  const tokens = new Lexer().lex(markdown);
  return renderTokens(tokens, options);
}

export type { RenderOptions };
