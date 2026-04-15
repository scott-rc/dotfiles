import { helpOpen } from "../state/store";

const KEYBINDINGS = [
  ["Navigation", [
    ["j / ↓", "Scroll down"],
    ["k / ↑", "Scroll up"],
    ["d", "Half page down"],
    ["u", "Half page up"],
    ["g / Home", "Top"],
    ["G / End", "Bottom"],
    ["z", "Center cursor"],
  ]],
  ["Diff Navigation", [
    ["] / [", "Next / prev hunk"],
    ["} / {", "Next / prev file"],
    ["s", "Toggle single-file view"],
    ["o", "Toggle full context"],
  ]],
  ["Tree", [
    ["l", "Toggle tree"],
    ["t", "Toggle tree focus"],
  ]],
  ["Search", [
    ["/ or ⌘K", "Open search"],
    ["n / N", "Next / prev match"],
  ]],
  ["Selection", [
    ["v", "Visual select"],
    ["y", "Yank path:lines"],
    ["c / C", "Copy path"],
  ]],
  ["Other", [
    ["za", "Toggle file collapse"],
    ["?", "Toggle help"],
    ["T", "Cycle theme"],
    ["q", "Quit"],
  ]],
] as const;

export function HelpOverlay() {
  if (!helpOpen.value) return null;

  return (
    <div id="help-overlay" class="fixed inset-0 z-50 flex items-center justify-center" role="dialog" aria-label="Keyboard shortcuts" aria-modal="true">
      <div
        class="absolute inset-0 bg-black/60"
        onClick={() => { helpOpen.value = false; }}
      />
      <div id="help-content" class="relative bg-[var(--bg-secondary)] border border-[var(--fg-sep)] rounded-lg shadow-xl p-6 max-w-lg max-h-[80vh] overflow-y-auto animate-in">
        <h2 class="text-sm font-bold mb-4">Keybindings</h2>
        {KEYBINDINGS.map(([section, bindings]) => (
          <div key={section} class="mb-3">
            <h3 class="text-xs text-[var(--fg-gutter)] font-semibold mb-1">{section}</h3>
            <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-0.5">
              {bindings.map(([key, desc]) => (
                <>
                  <kbd class="text-xs text-[var(--fg-file-header)] font-mono">{key}</kbd>
                  <span class="text-xs text-[var(--fg)]">{desc}</span>
                </>
              ))}
            </div>
          </div>
        ))}
        <p class="text-xs text-[var(--fg-gutter)] mt-3">Press ? to close</p>
      </div>
    </div>
  );
}
