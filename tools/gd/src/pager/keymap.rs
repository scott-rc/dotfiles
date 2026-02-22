#![allow(dead_code)]
#![allow(private_interfaces)]

use tui::pager::Key;

use crate::pager::types::{ActionId, HelpGroup, KeyContext};

/// Keymap entry: keys that trigger an action in a context, plus help display.
#[allow(dead_code)]
struct KeymapEntry {
    action: ActionId,
    context: KeyContext,
    keys: &'static [Key],
    group: HelpGroup,
    key_display: &'static str,
    label: &'static str,
}

fn keymap_entries() -> &'static [KeymapEntry] {
    use ActionId::*;
    use HelpGroup::*;
    use KeyContext::*;
    static ENTRIES: &[KeymapEntry] = &[
        KeymapEntry { action: ScrollDown, context: Normal, keys: &[Key::Char('j'), Key::Down, Key::Enter], group: Navigation, key_display: "j/\u{2193}/Enter", label: "Scroll down" },
        KeymapEntry { action: ScrollUp, context: Normal, keys: &[Key::Char('k'), Key::Up], group: Navigation, key_display: "k/\u{2191}", label: "Scroll up" },
        KeymapEntry { action: HalfPageDown, context: Normal, keys: &[Key::CtrlD, Key::PageDown], group: Navigation, key_display: "Ctrl-D", label: "Half page down" },
        KeymapEntry { action: HalfPageUp, context: Normal, keys: &[Key::CtrlU, Key::PageUp], group: Navigation, key_display: "Ctrl-U", label: "Half page up" },
        KeymapEntry { action: Top, context: Normal, keys: &[Key::Char('g'), Key::Home], group: Navigation, key_display: "g/Home", label: "Top" },
        KeymapEntry { action: Bottom, context: Normal, keys: &[Key::Char('G'), Key::End], group: Navigation, key_display: "G/End", label: "Bottom" },
        KeymapEntry { action: NextHunk, context: Normal, keys: &[Key::Char('d')], group: DiffNav, key_display: "d", label: "Next hunk" },
        KeymapEntry { action: PrevHunk, context: Normal, keys: &[Key::Char('u')], group: DiffNav, key_display: "u", label: "Previous hunk" },
        KeymapEntry { action: NextFile, context: Normal, keys: &[Key::Char('D')], group: DiffNav, key_display: "D", label: "Next file" },
        KeymapEntry { action: PrevFile, context: Normal, keys: &[Key::Char('U')], group: DiffNav, key_display: "U", label: "Previous file" },
        KeymapEntry { action: ToggleSingleFile, context: Normal, keys: &[Key::Char('a')], group: DiffNav, key_display: "a", label: "Toggle single file" },
        KeymapEntry { action: ToggleFullContext, context: Normal, keys: &[Key::Char('z')], group: DiffNav, key_display: "z", label: "Toggle full file context" },
        KeymapEntry { action: ActionId::Search, context: Normal, keys: &[Key::Char('/')], group: HelpGroup::Search, key_display: "/", label: "Search" },
        KeymapEntry { action: SearchSubmit, context: KeyContext::Search, keys: &[Key::Enter], group: HelpGroup::Search, key_display: "Enter", label: "Apply search" },
        KeymapEntry { action: SearchCancel, context: KeyContext::Search, keys: &[Key::Escape, Key::CtrlC], group: HelpGroup::Search, key_display: "Esc", label: "Cancel search" },
        KeymapEntry { action: NextMatch, context: Normal, keys: &[Key::Char('n')], group: HelpGroup::Search, key_display: "n", label: "Next match" },
        KeymapEntry { action: PrevMatch, context: Normal, keys: &[Key::Char('N')], group: HelpGroup::Search, key_display: "N", label: "Previous match" },
        KeymapEntry { action: ToggleTree, context: Normal, keys: &[Key::Char('e')], group: FileTree, key_display: "e", label: "Toggle tree panel" },
        KeymapEntry { action: FocusTree, context: Normal, keys: &[Key::Tab], group: FileTree, key_display: "Tab", label: "Focus panel" },
        KeymapEntry { action: FocusTreeOrShow, context: Normal, keys: &[Key::Char('1')], group: FileTree, key_display: "1", label: "Toggle tree focus" },
        KeymapEntry { action: FocusTreeOrShow, context: Normal, keys: &[Key::CtrlL, Key::Char('l')], group: FileTree, key_display: "l/Ctrl-L", label: "Show + focus tree" },
        KeymapEntry { action: ReturnToDiff, context: Tree, keys: &[Key::CtrlH, Key::Escape, Key::Tab, Key::Char('1'), Key::Char('h')], group: FileTree, key_display: "h/Ctrl-H", label: "Return to diff" },
        KeymapEntry { action: TreeClose, context: Tree, keys: &[Key::Char('e')], group: FileTree, key_display: "e", label: "Close tree" },
        KeymapEntry { action: TreeFirst, context: Tree, keys: &[Key::Char('g'), Key::Home], group: FileTree, key_display: "g/Home", label: "First file" },
        KeymapEntry { action: TreeLast, context: Tree, keys: &[Key::Char('G'), Key::End], group: FileTree, key_display: "G/End", label: "Last file" },
        KeymapEntry { action: TreeNavDown, context: Tree, keys: &[Key::Char('j'), Key::Down], group: FileTree, key_display: "j/k", label: "(tree) Navigate" },
        KeymapEntry { action: TreeNavUp, context: Tree, keys: &[Key::Char('k'), Key::Up], group: FileTree, key_display: "j/k", label: "(tree) Navigate" },
        KeymapEntry { action: TreeSelect, context: Tree, keys: &[Key::Enter], group: FileTree, key_display: "Enter", label: "Select / toggle folder" },
        KeymapEntry { action: ToggleSingleFile, context: Tree, keys: &[Key::Char('a')], group: FileTree, key_display: "a", label: "Toggle single file" },
        KeymapEntry { action: NextHunk, context: Tree, keys: &[Key::Char('d')], group: FileTree, key_display: "d", label: "Next hunk" },
        KeymapEntry { action: PrevHunk, context: Tree, keys: &[Key::Char('u')], group: FileTree, key_display: "u", label: "Previous hunk" },
        KeymapEntry { action: Quit, context: Tree, keys: &[Key::Char('q'), Key::CtrlC], group: FileTree, key_display: "q", label: "Quit" },
        KeymapEntry { action: EnterVisual, context: Normal, keys: &[Key::Char('v')], group: VisualMode, key_display: "v", label: "Enter visual mode" },
        KeymapEntry { action: VisualExtendDown, context: Visual, keys: &[Key::Char('j'), Key::Down], group: VisualMode, key_display: "j/k", label: "Extend selection" },
        KeymapEntry { action: VisualExtendUp, context: Visual, keys: &[Key::Char('k'), Key::Up], group: VisualMode, key_display: "j/k", label: "Extend selection" },
        KeymapEntry { action: VisualCopy, context: Visual, keys: &[Key::Char('y')], group: VisualMode, key_display: "y", label: "Copy path:lines" },
        KeymapEntry { action: VisualCancel, context: Visual, keys: &[Key::Escape, Key::CtrlC], group: VisualMode, key_display: "Esc", label: "Cancel" },
        KeymapEntry { action: Quit, context: Visual, keys: &[Key::Char('q')], group: VisualMode, key_display: "q", label: "Quit" },
        KeymapEntry { action: OpenEditor, context: Normal, keys: &[Key::Char('E')], group: Other, key_display: "E", label: "Open in editor" },
        KeymapEntry { action: Quit, context: Normal, keys: &[Key::Char('q'), Key::CtrlC], group: Other, key_display: "q", label: "Quit" },
        KeymapEntry { action: Help, context: Normal, keys: &[Key::Char('?')], group: Other, key_display: "? / Esc", label: "Close help" },
    ];
    ENTRIES
}

pub(crate) fn keymap_lookup(key: Key, context: KeyContext) -> Option<ActionId> {
    for e in keymap_entries() {
        if e.context == context && e.keys.contains(&key) {
            return Some(e.action);
        }
    }
    None
}

/// Build help lines from keymap. Preserves grouping (Navigation, Diff Nav, Search, File Tree, Visual, Other).
/// Returns raw lines for format_help_lines to pad/center.
pub(crate) fn keymap_help_lines() -> Vec<String> {
    use HelpGroup::*;
    use std::collections::HashSet;
    let order = [Navigation, DiffNav, Search, FileTree, VisualMode, Other];
    let mut lines: Vec<String> = Vec::new();
    for group in order {
        let mut seen: HashSet<(&'static str, &'static str)> = HashSet::new();
        let mut group_lines: Vec<(&'static str, &'static str)> = Vec::new();
        for e in keymap_entries() {
            if e.group == group && !e.keys.is_empty() && !e.key_display.is_empty() {
                let k = (e.key_display, e.label);
                if !seen.contains(&k) {
                    seen.insert(k);
                    group_lines.push((e.key_display, e.label));
                }
            }
        }
        if !group_lines.is_empty() {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            let group_name = match group {
                Navigation => "Navigation",
                DiffNav => "Diff Navigation",
                Search => "Search",
                FileTree => "File Tree",
                VisualMode => "Visual Mode",
                Other => "Other",
            };
            lines.push(group_name.to_string());
            for (k, l) in group_lines {
                let pad = 12usize.saturating_sub(k.chars().count());
                lines.push(format!("{}{}  {}", k, " ".repeat(pad), l));
            }
        }
    }
    lines
}
