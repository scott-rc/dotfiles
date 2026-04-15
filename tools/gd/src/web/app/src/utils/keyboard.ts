export type Action =
  | "cursor-down"
  | "cursor-up"
  | "half-page-down"
  | "half-page-up"
  | "top"
  | "bottom"
  | "center-cursor"
  | "next-hunk"
  | "prev-hunk"
  | "next-file"
  | "prev-file"
  | "toggle-single-file"
  | "toggle-full-context"
  | "toggle-tree"
  | "toggle-tree-focus"
  | "toggle-collapse"
  | "toggle-collapse-recursive"
  | "visual-select"
  | "yank"
  | "copy-path"
  | "copy-abs-path"
  | "open-search"
  | "next-match"
  | "prev-match"
  | "toggle-help"
  | "cycle-theme"
  | "cancel-selection"
  | "quit";

interface KeyEvent {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
}

/** Map a keyboard event to an action. Returns null if unmapped. */
export function mapKey(e: KeyEvent): Action | null {
  const k = e.key;

  // Meta/Cmd combos
  if (e.metaKey && k === "k") return "open-search";

  // Ctrl combos
  if (e.ctrlKey && k === "c") return "quit";

  // Single keys
  switch (k) {
    case "j":
    case "ArrowDown":
    case "Enter":
      return "cursor-down";
    case "k":
    case "ArrowUp":
      return "cursor-up";
    case "d":
      return "half-page-down";
    case "u":
      return "half-page-up";
    case "g":
    case "Home":
      return "top";
    case "G":
    case "End":
      return "bottom";
    case "z":
      return "center-cursor";
    case "]":
      return "next-hunk";
    case "[":
      return "prev-hunk";
    case "}":
      return "next-file";
    case "{":
      return "prev-file";
    case "s":
      return "toggle-single-file";
    case "o":
      return "toggle-full-context";
    case "l":
      return "toggle-tree";
    case "t":
      return "toggle-tree-focus";
    case "v":
      return "visual-select";
    case "y":
      return "yank";
    case "c":
      return "copy-path";
    case "C":
      return "copy-abs-path";
    case "/":
      return "open-search";
    case "n":
      return "next-match";
    case "N":
      return "prev-match";
    case "?":
      return "toggle-help";
    case "T":
      return "cycle-theme";
    case "q":
      return "quit";
    case "Escape":
      return "cancel-selection";
    default:
      return null;
  }
}
