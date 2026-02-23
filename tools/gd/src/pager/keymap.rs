#![allow(private_interfaces)]

use tui::pager::Key;

use crate::pager::types::{ActionId, HelpGroup, KeyContext};

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
    use HelpGroup::{DiffNav, Navigation, Other, Selection};
    use KeyContext::Normal;
    static ENTRIES: &[KeymapEntry] = &[
        KeymapEntry {
            action: ScrollDown,
            context: Normal,
            keys: &[Key::Char('j'), Key::Down, Key::Enter],
            group: Navigation,
            key_display: "j/Down/Enter",
            label: "Scroll down",
        },
        KeymapEntry {
            action: ScrollUp,
            context: Normal,
            keys: &[Key::Char('k'), Key::Up],
            group: Navigation,
            key_display: "k/Up",
            label: "Scroll up",
        },
        KeymapEntry {
            action: HalfPageDown,
            context: Normal,
            keys: &[Key::Char('d')],
            group: Navigation,
            key_display: "d",
            label: "Half page down",
        },
        KeymapEntry {
            action: HalfPageUp,
            context: Normal,
            keys: &[Key::Char('u')],
            group: Navigation,
            key_display: "u",
            label: "Half page up",
        },
        KeymapEntry {
            action: Top,
            context: Normal,
            keys: &[Key::Char('g'), Key::Home],
            group: Navigation,
            key_display: "g/Home",
            label: "Top",
        },
        KeymapEntry {
            action: Bottom,
            context: Normal,
            keys: &[Key::Char('G'), Key::End],
            group: Navigation,
            key_display: "G/End",
            label: "Bottom",
        },
        KeymapEntry {
            action: CenterViewport,
            context: Normal,
            keys: &[Key::Char('z')],
            group: Navigation,
            key_display: "z",
            label: "Center viewport",
        },
        KeymapEntry {
            action: NextHunk,
            context: Normal,
            keys: &[Key::Char(']')],
            group: DiffNav,
            key_display: "]",
            label: "Next hunk",
        },
        KeymapEntry {
            action: PrevHunk,
            context: Normal,
            keys: &[Key::Char('[')],
            group: DiffNav,
            key_display: "[",
            label: "Previous hunk",
        },
        KeymapEntry {
            action: NextFile,
            context: Normal,
            keys: &[Key::Char('}')],
            group: DiffNav,
            key_display: "}",
            label: "Next file",
        },
        KeymapEntry {
            action: PrevFile,
            context: Normal,
            keys: &[Key::Char('{')],
            group: DiffNav,
            key_display: "{",
            label: "Previous file",
        },
        KeymapEntry {
            action: ToggleSingleFile,
            context: Normal,
            keys: &[Key::Char('s')],
            group: DiffNav,
            key_display: "s",
            label: "Toggle single file",
        },
        KeymapEntry {
            action: ToggleFullContext,
            context: Normal,
            keys: &[Key::Char('o')],
            group: DiffNav,
            key_display: "o",
            label: "Toggle full file context",
        },
        KeymapEntry {
            action: ActionId::Search,
            context: Normal,
            keys: &[Key::Char('/')],
            group: HelpGroup::Search,
            key_display: "/",
            label: "Search",
        },
        KeymapEntry {
            action: SearchSubmit,
            context: KeyContext::Search,
            keys: &[Key::Enter],
            group: HelpGroup::Search,
            key_display: "Enter",
            label: "Apply search",
        },
        KeymapEntry {
            action: SearchCancel,
            context: KeyContext::Search,
            keys: &[Key::Escape, Key::CtrlC],
            group: HelpGroup::Search,
            key_display: "Esc",
            label: "Cancel search",
        },
        KeymapEntry {
            action: NextMatch,
            context: Normal,
            keys: &[Key::Char('n')],
            group: HelpGroup::Search,
            key_display: "n",
            label: "Next match",
        },
        KeymapEntry {
            action: PrevMatch,
            context: Normal,
            keys: &[Key::Char('N')],
            group: HelpGroup::Search,
            key_display: "N",
            label: "Previous match",
        },
        KeymapEntry {
            action: ToggleTree,
            context: Normal,
            keys: &[Key::Char('l')],
            group: Other,
            key_display: "l",
            label: "Toggle tree panel",
        },
        KeymapEntry {
            action: SetMark,
            context: Normal,
            keys: &[Key::Char('m')],
            group: Selection,
            key_display: "m",
            label: "Set mark",
        },
        KeymapEntry {
            action: YankToMark,
            context: Normal,
            keys: &[Key::Char('y')],
            group: Selection,
            key_display: "y",
            label: "Copy mark..cursor",
        },
        KeymapEntry {
            action: OpenEditor,
            context: Normal,
            keys: &[Key::Char('e')],
            group: Other,
            key_display: "e",
            label: "Open in editor",
        },
        KeymapEntry {
            action: Quit,
            context: Normal,
            keys: &[Key::Char('q'), Key::CtrlC],
            group: Other,
            key_display: "q",
            label: "Quit",
        },
        KeymapEntry {
            action: ToggleTooltip,
            context: Normal,
            keys: &[Key::Char('?')],
            group: Other,
            key_display: "?",
            label: "Toggle key hints",
        },
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

pub(crate) fn keymap_help_lines() -> Vec<String> {
    use HelpGroup::{DiffNav, Navigation, Other, Search, Selection};
    use std::collections::HashSet;
    let order = [Navigation, DiffNav, Search, Selection, Other];
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
                Selection => "Selection",
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
