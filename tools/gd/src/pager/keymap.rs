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
    use HelpGroup::{DiffNav, Navigation, Other, Selection};
    use KeyContext::Normal;
    static ENTRIES: &[KeymapEntry] = &[
        KeymapEntry {
            action: ActionId::ScrollDown,
            context: Normal,
            keys: &[Key::Char('j'), Key::Down, Key::Enter],
            group: Navigation,
            key_display: "j/Down/Enter",
            label: "Scroll down",
        },
        KeymapEntry {
            action: ActionId::ScrollUp,
            context: Normal,
            keys: &[Key::Char('k'), Key::Up],
            group: Navigation,
            key_display: "k/Up",
            label: "Scroll up",
        },
        KeymapEntry {
            action: ActionId::HalfPageDown,
            context: Normal,
            keys: &[Key::Char('d')],
            group: Navigation,
            key_display: "d",
            label: "Half page down",
        },
        KeymapEntry {
            action: ActionId::HalfPageUp,
            context: Normal,
            keys: &[Key::Char('u')],
            group: Navigation,
            key_display: "u",
            label: "Half page up",
        },
        KeymapEntry {
            action: ActionId::Top,
            context: Normal,
            keys: &[Key::Char('g'), Key::Home],
            group: Navigation,
            key_display: "g/Home",
            label: "Top",
        },
        KeymapEntry {
            action: ActionId::Bottom,
            context: Normal,
            keys: &[Key::Char('G'), Key::End],
            group: Navigation,
            key_display: "G/End",
            label: "Bottom",
        },
        KeymapEntry {
            action: ActionId::CenterViewport,
            context: Normal,
            keys: &[Key::Char('z')],
            group: Navigation,
            key_display: "z",
            label: "Center viewport",
        },
        KeymapEntry {
            action: ActionId::NextHunk,
            context: Normal,
            keys: &[Key::Char(']')],
            group: DiffNav,
            key_display: "]",
            label: "Next hunk",
        },
        KeymapEntry {
            action: ActionId::PrevHunk,
            context: Normal,
            keys: &[Key::Char('[')],
            group: DiffNav,
            key_display: "[",
            label: "Previous hunk",
        },
        KeymapEntry {
            action: ActionId::NextFile,
            context: Normal,
            keys: &[Key::Char('}')],
            group: DiffNav,
            key_display: "}",
            label: "Next file",
        },
        KeymapEntry {
            action: ActionId::PrevFile,
            context: Normal,
            keys: &[Key::Char('{')],
            group: DiffNav,
            key_display: "{",
            label: "Previous file",
        },
        KeymapEntry {
            action: ActionId::ToggleSingleFile,
            context: Normal,
            keys: &[Key::Char('s')],
            group: DiffNav,
            key_display: "s",
            label: "Toggle single file",
        },
        KeymapEntry {
            action: ActionId::ToggleFullContext,
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
            action: ActionId::SearchSubmit,
            context: KeyContext::Search,
            keys: &[Key::Enter],
            group: HelpGroup::Search,
            key_display: "Enter",
            label: "Apply search",
        },
        KeymapEntry {
            action: ActionId::SearchCancel,
            context: KeyContext::Search,
            keys: &[Key::Escape, Key::CtrlC],
            group: HelpGroup::Search,
            key_display: "Esc",
            label: "Cancel search",
        },
        KeymapEntry {
            action: ActionId::NextMatch,
            context: Normal,
            keys: &[Key::Char('n')],
            group: HelpGroup::Search,
            key_display: "n",
            label: "Next match",
        },
        KeymapEntry {
            action: ActionId::PrevMatch,
            context: Normal,
            keys: &[Key::Char('N')],
            group: HelpGroup::Search,
            key_display: "N",
            label: "Previous match",
        },
        KeymapEntry {
            action: ActionId::ToggleTree,
            context: Normal,
            keys: &[Key::Char('l')],
            group: Other,
            key_display: "l",
            label: "Toggle tree panel",
        },
        KeymapEntry {
            action: ActionId::SetMark,
            context: Normal,
            keys: &[Key::Char('m')],
            group: Selection,
            key_display: "m",
            label: "Set mark",
        },
        KeymapEntry {
            action: ActionId::YankToMark,
            context: Normal,
            keys: &[Key::Char('y')],
            group: Selection,
            key_display: "y",
            label: "Copy mark..cursor",
        },
        KeymapEntry {
            action: ActionId::OpenEditor,
            context: Normal,
            keys: &[Key::Char('e')],
            group: Other,
            key_display: "e",
            label: "Open in editor",
        },
        KeymapEntry {
            action: ActionId::Quit,
            context: Normal,
            keys: &[Key::Char('q'), Key::CtrlC],
            group: Other,
            key_display: "q",
            label: "Quit",
        },
        KeymapEntry {
            action: ActionId::ToggleTooltip,
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

fn keymap_entry(action: ActionId, context: KeyContext) -> Option<&'static KeymapEntry> {
    keymap_entries()
        .iter()
        .find(|e| e.action == action && e.context == context)
}

fn first_key_from_display(display: &str) -> &str {
    display.split('/').next().unwrap_or(display)
}

fn tooltip_pair(
    first: ActionId,
    second: ActionId,
    label: &str,
    context: KeyContext,
) -> Option<String> {
    let a = keymap_entry(first, context)?;
    let b = keymap_entry(second, context)?;
    debug_assert_eq!(a.group, b.group);
    Some(format!(
        "{}/{} {label}",
        first_key_from_display(a.key_display),
        first_key_from_display(b.key_display)
    ))
}

fn compact_label(label: &str) -> String {
    match label {
        "Center viewport" => "center".to_string(),
        "Toggle single file" => "single".to_string(),
        "Toggle full file context" => "context".to_string(),
        "Toggle tree panel" => "tree".to_string(),
        "Search" => "search".to_string(),
        "Set mark" => "mark".to_string(),
        "Copy mark..cursor" => "yank".to_string(),
        "Open in editor" => "edit".to_string(),
        "Quit" => "quit".to_string(),
        _ => label.to_ascii_lowercase(),
    }
}

fn tooltip_single(action: ActionId, context: KeyContext) -> Option<String> {
    let entry = keymap_entry(action, context)?;
    let label = compact_label(entry.label);
    Some(format!(
        "{} {label}",
        first_key_from_display(entry.key_display)
    ))
}

pub(crate) fn keymap_tooltip_lines() -> [String; 2] {
    let context = KeyContext::Normal;

    let line1 = [
        tooltip_pair(ActionId::ScrollDown, ActionId::ScrollUp, "scroll", context),
        tooltip_pair(
            ActionId::HalfPageDown,
            ActionId::HalfPageUp,
            "page",
            context,
        ),
        tooltip_pair(ActionId::Top, ActionId::Bottom, "top/bot", context),
        tooltip_single(ActionId::CenterViewport, context),
        tooltip_pair(ActionId::NextHunk, ActionId::PrevHunk, "hunk", context),
        tooltip_pair(ActionId::NextFile, ActionId::PrevFile, "file", context),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>()
    .join("  ");

    let line2 = [
        tooltip_single(ActionId::ToggleSingleFile, context),
        tooltip_single(ActionId::ToggleFullContext, context),
        tooltip_single(ActionId::ToggleTree, context),
        tooltip_single(ActionId::Search, context),
        tooltip_pair(ActionId::NextMatch, ActionId::PrevMatch, "match", context),
        tooltip_single(ActionId::SetMark, context),
        tooltip_single(ActionId::YankToMark, context),
        tooltip_single(ActionId::OpenEditor, context),
        tooltip_single(ActionId::Quit, context),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>()
    .join("  ");

    [line1, line2]
}

#[cfg_attr(not(test), allow(dead_code))]
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
