import { expandedContextGroups } from "../state/store";
import { ITEM_HEIGHTS } from "../utils/display";

export function CollapsedContext({
  groupKey,
  count,
}: {
  groupKey: string;
  count: number;
}) {
  function expand() {
    const next = new Set(expandedContextGroups.value);
    next.add(groupKey);
    expandedContextGroups.value = next;
  }

  return (
    <div
      class="collapsed-context flex items-center justify-center text-xs text-[var(--fg-gutter)] cursor-pointer select-none hover:bg-[var(--bg-secondary)] transition-colors duration-100 rounded-md mx-2"
      style={{ height: `${ITEM_HEIGHTS["collapsed-context"]}px` }}
      role="button"
      aria-label={`${count} unmodified lines — click to expand`}
      onClick={expand}
    >
      <span class="border border-dashed border-[var(--fg-sep)] rounded-md px-3 py-0.5">
        {count} unmodified lines
      </span>
    </div>
  );
}
