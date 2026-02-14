const DEFAULT_FIND =
  "find {dir} -type f \\( -name '*.md' -o -name '*.mdx' \\)";
const DEFAULT_PICK = "fzf";

export function shellQuote(s: string): string {
  return "'" + s.replace(/'/g, "'\\''") + "'";
}

export function buildFindCmd(dir: string, template?: string): string {
  const t = template ?? DEFAULT_FIND;
  const quoted = shellQuote(dir);
  if (t.includes("{dir}")) {
    return t.replace("{dir}", quoted);
  }
  return t + " " + quoted;
}

export function buildPickCmd(template?: string): string {
  return template ?? DEFAULT_PICK;
}

export function buildBrowseCmd(
  dir: string,
  findCmd?: string,
  pickCmd?: string,
): string {
  return buildFindCmd(dir, findCmd) + " | " + buildPickCmd(pickCmd);
}

export function shouldPage(opts: {
  noPager: boolean;
  isTTY: boolean;
  contentLines: number;
  terminalRows: number;
  browsing: boolean;
}): boolean {
  if (opts.noPager || !opts.isTTY) return false;
  if (opts.browsing) return true;
  return opts.contentLines > opts.terminalRows;
}

export function parseSelection(raw: string): string | null {
  const trimmed = raw.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export async function browseDirectory(
  dir: string,
  viewFile: (path: string) => Promise<void>,
  opts?: { findCmd?: string; pickCmd?: string },
): Promise<void> {
  const cmd = buildBrowseCmd(dir, opts?.findCmd, opts?.pickCmd);

  while (true) {
    const proc = new Deno.Command(Deno.env.get("SHELL") || "sh", {
      args: ["-c", cmd],
      stdin: "inherit",
      stdout: "piped",
      stderr: "inherit",
    }).spawn();

    const { code, stdout } = await proc.output();

    if (code !== 0) break;

    const selection = parseSelection(new TextDecoder().decode(stdout));
    if (!selection) break;

    await viewFile(selection);
  }
}
