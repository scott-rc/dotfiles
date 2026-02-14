// deno-lint-ignore-file no-control-regex

const ANSI_RE = /\x1b\[[0-9;]*m/g;

/** Remove ANSI escape codes from a string. */
export function stripAnsi(text: string): string {
  return text.replace(ANSI_RE, "");
}

/** Get the visible length of a string, ignoring ANSI escape codes. */
export function visibleLength(text: string): number {
  return stripAnsi(text).length;
}

/**
 * Word-wrap text to a given width, optionally prepending an indent to each line.
 * ANSI escape codes are not counted toward visible width.
 */
export function wordWrap(
  text: string,
  width: number,
  indent = "",
): string {
  const indentWidth = visibleLength(indent);
  const available = width - indentWidth;
  if (available <= 0) return text;

  const lines = text.split("\n");
  const result: string[] = [];

  for (const line of lines) {
    if (visibleLength(line) === 0) {
      result.push(indent);
      continue;
    }

    const wrapped = wrapLine(line, available);
    for (const w of wrapped) {
      result.push(indent + w);
    }
  }

  return result.join("\n");
}

/**
 * Wrap a single line (no embedded newlines) to fit within `width` visible characters.
 * Preserves ANSI codes by splitting on word boundaries in the visible text
 * and then reconstructing with the original codes.
 */
function wrapLine(line: string, width: number): string[] {
  if (visibleLength(line) <= width) return [line];

  // Split into segments: alternating text and ANSI codes
  const segments = splitAnsi(line);
  const results: string[] = [];
  let currentLine = "";
  let currentWidth = 0;

  for (const seg of segments) {
    if (seg.match(ANSI_RE)) {
      // ANSI code: append without counting width
      currentLine += seg;
      continue;
    }

    // Plain text: split into words
    const words = seg.split(/( +)/);
    for (const word of words) {
      if (word === "") continue;

      const wordLen = word.length;

      // If this word alone exceeds width, force-break it
      if (wordLen > width && currentWidth === 0) {
        for (let i = 0; i < word.length; i += width) {
          if (i > 0) {
            results.push(currentLine);
            currentLine = "";
            currentWidth = 0;
          }
          const chunk = word.slice(i, i + width);
          currentLine += chunk;
          currentWidth += chunk.length;
        }
        continue;
      }

      // If adding this word would exceed width, wrap
      if (currentWidth + wordLen > width && currentWidth > 0) {
        // Trim trailing spaces from current line
        results.push(currentLine.replace(/ +$/, ""));
        currentLine = "";
        currentWidth = 0;

        // Skip leading spaces at the start of a new line
        if (word.match(/^ +$/)) continue;
      }

      currentLine += word;
      currentWidth += wordLen;
    }
  }

  if (currentLine.length > 0) {
    results.push(currentLine);
  }

  return results;
}

/** Split a string into alternating plain-text and ANSI-code segments. */
export function splitAnsi(text: string): string[] {
  const parts: string[] = [];
  let lastIndex = 0;

  for (const match of text.matchAll(new RegExp(ANSI_RE, "g"))) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    parts.push(match[0]);
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts;
}
