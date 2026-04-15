import type { WebDiffFile } from "../utils/types";
import { DiffLine } from "./DiffLine";

interface DiffFileProps {
  file: WebDiffFile;
}

export function DiffFile({ file }: DiffFileProps) {
  const additions = file.hunks.reduce(
    (sum, h) => sum + h.lines.filter((l) => l.kind === "added").length,
    0,
  );
  const deletions = file.hunks.reduce(
    (sum, h) => sum + h.lines.filter((l) => l.kind === "deleted").length,
    0,
  );

  return (
    <div class="mb-4">
      <div class="sticky top-0 z-10 flex items-center gap-2 px-3 py-1.5 bg-[var(--bg-secondary)] border-b border-[var(--fg-sep)] rounded-t-md">
        <span class="font-semibold text-sm text-[var(--fg-file-header)] truncate">
          {file.path}
        </span>
        <span class="ml-auto flex gap-2 text-xs shrink-0">
          {additions > 0 && (
            <span class="text-[var(--fg-added-marker)]">+{additions}</span>
          )}
          {deletions > 0 && (
            <span class="text-[var(--fg-deleted-marker)]">-{deletions}</span>
          )}
        </span>
      </div>
      <div class="border-x border-b border-[var(--fg-sep)] rounded-b-md overflow-hidden">
        {file.hunks.map((hunk, hi) => (
          <div key={hi}>
            {hi > 0 && (
              <div class="h-5 flex items-center px-3 text-xs text-[var(--fg-sep)] select-none">
                <span class="w-full border-t border-dashed border-[var(--fg-sep)]" />
              </div>
            )}
            {hunk.lines.map((line) => (
              <DiffLine key={`${hi}-${line.line_idx}`} line={line} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
