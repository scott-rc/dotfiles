//! Keymap and help tests.

use super::super::keymap::{keymap_help_lines, keymap_lookup};
use super::super::types::{ActionId, KeyContext};

#[test]
fn keymap_center_viewport_is_z() {
    assert_eq!(
        keymap_lookup(tui::pager::Key::Char('z'), KeyContext::Normal),
        Some(ActionId::CenterViewport)
    );
    assert_eq!(
        keymap_lookup(tui::pager::Key::Char(' '), KeyContext::Normal),
        None
    );
}

#[test]
fn keymap_normal_navigation_keys() {
    use tui::pager::Key;
    let cases = [
        (Key::Char('j'), ActionId::ScrollDown),
        (Key::Char('k'), ActionId::ScrollUp),
        (Key::Char('d'), ActionId::HalfPageDown),
        (Key::Char('u'), ActionId::HalfPageUp),
        (Key::Char('g'), ActionId::Top),
        (Key::Char('G'), ActionId::Bottom),
        (Key::Char('z'), ActionId::CenterViewport),
        (Key::Char(']'), ActionId::NextHunk),
        (Key::Char('['), ActionId::PrevHunk),
        (Key::Char('}'), ActionId::NextFile),
        (Key::Char('{'), ActionId::PrevFile),
        (Key::Char('s'), ActionId::ToggleSingleFile),
        (Key::Char('o'), ActionId::ToggleFullContext),
    ];
    for (key, expected) in cases {
        assert_eq!(
            keymap_lookup(key, KeyContext::Normal),
            Some(expected),
            "Normal context: {key:?} should map to {expected:?}"
        );
    }
}

#[test]
fn keymap_normal_other_keys() {
    use tui::pager::Key;
    let cases = [
        (Key::Char('/'), ActionId::Search),
        (Key::Char('n'), ActionId::NextMatch),
        (Key::Char('N'), ActionId::PrevMatch),
        (Key::Char('l'), ActionId::ToggleTree),
        (Key::Char('m'), ActionId::SetMark),
        (Key::Char('y'), ActionId::YankToMark),
        (Key::Char('e'), ActionId::OpenEditor),
        (Key::Char('q'), ActionId::Quit),
        (Key::Char('?'), ActionId::ToggleTooltip),
    ];
    for (key, expected) in cases {
        assert_eq!(
            keymap_lookup(key, KeyContext::Normal),
            Some(expected),
            "Normal context: {key:?} should map to {expected:?}"
        );
    }
}

#[test]
fn keymap_search_keys() {
    use tui::pager::Key;
    let cases = [
        (Key::Enter, ActionId::SearchSubmit),
        (Key::Escape, ActionId::SearchCancel),
        (Key::CtrlC, ActionId::SearchCancel),
    ];
    for (key, expected) in cases {
        assert_eq!(
            keymap_lookup(key, KeyContext::Search),
            Some(expected),
            "Search context: {key:?} should map to {expected:?}"
        );
    }
}

#[test]
fn keymap_normal_keys_not_in_search() {
    use tui::pager::Key;
    assert_eq!(
        keymap_lookup(Key::Char('j'), KeyContext::Search),
        None,
        "j (ScrollDown) should not fire in Search context"
    );
    assert_eq!(
        keymap_lookup(Key::Char('n'), KeyContext::Search),
        None,
        "n (NextMatch) should not fire in Search context"
    );
}

#[test]
fn keymap_alternate_keys() {
    use tui::pager::Key;
    assert_eq!(
        keymap_lookup(Key::Down, KeyContext::Normal),
        Some(ActionId::ScrollDown)
    );
    assert_eq!(
        keymap_lookup(Key::Up, KeyContext::Normal),
        Some(ActionId::ScrollUp)
    );
    assert_eq!(
        keymap_lookup(Key::Home, KeyContext::Normal),
        Some(ActionId::Top)
    );
    assert_eq!(
        keymap_lookup(Key::End, KeyContext::Normal),
        Some(ActionId::Bottom)
    );
    assert_eq!(
        keymap_lookup(Key::Enter, KeyContext::Normal),
        Some(ActionId::ScrollDown)
    );
    assert_eq!(
        keymap_lookup(Key::CtrlC, KeyContext::Normal),
        Some(ActionId::Quit)
    );
}

#[test]
fn help_includes_all_primary_runtime_actions() {
    let help_text = keymap_help_lines().join(" ");
    let required = [
        "]",
        "Next hunk",
        "[",
        "Previous hunk",
        "}",
        "Next file",
        "{",
        "Previous file",
        "s",
        "Toggle single file",
        "o",
        "Toggle full file context",
        "/",
        "Search",
        "n",
        "Next match",
        "N",
        "Previous match",
        "l",
        "Toggle tree panel",
        "m",
        "Set mark",
        "y",
        "Copy mark..cursor",
        "e",
        "Open in editor",
        "q",
        "Quit",
        "?",
        "Toggle key hints",
    ];
    for s in required {
        assert!(
            help_text.contains(s),
            "help must include runtime action: {s:?}"
        );
    }
}

#[test]
fn help_groups_are_present() {
    let help = keymap_help_lines();
    let joined = help.join("\n");
    for group in [
        "Navigation",
        "Diff Navigation",
        "Search",
        "Selection",
        "Other",
    ] {
        assert!(joined.contains(group), "help must include group: {group:?}");
    }
}
