import type { WebDiffLine } from "../utils/types";

interface DiffLineProps {
  line: WebDiffLine;
}

export function DiffLine({ line }: DiffLineProps) {
  const lineno =
    line.kind === "deleted" ? line.old_lineno : line.new_lineno;

  const bgClass =
    line.kind === "added"
      ? "bg-[var(--bg-added)]"
      : line.kind === "deleted"
        ? "bg-[var(--bg-deleted)]"
        : "";

  const borderClass =
    line.kind === "added"
      ? "border-l-[3px] border-l-[var(--fg-added-marker)]"
      : line.kind === "deleted"
        ? "border-l-[3px] border-l-[var(--fg-deleted-marker)]"
        : "border-l-[3px] border-l-transparent";

  return (
    <div class={`flex ${bgClass} hover:brightness-110`}>
      <span
        class={`shrink-0 w-12 text-right pr-3 select-none text-[var(--fg-gutter)] text-xs leading-5 ${borderClass}`}
      >
        {lineno ?? ""}
      </span>
      <span
        class="flex-1 pl-2 pr-4 leading-5 whitespace-pre overflow-x-hidden text-sm diff-content"
        dangerouslySetInnerHTML={{ __html: line.content_html }}
      />
    </div>
  );
}
