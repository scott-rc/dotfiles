//! Keymap and help tests.

use super::super::keymap::{keymap_help_lines, keymap_lookup};
use super::super::types::{ActionId, KeyContext};

#[test]
fn keymap_full_context_toggle_is_z() {
    assert_eq!(
        keymap_lookup(tui::pager::Key::Char('z'), KeyContext::Normal),
        Some(ActionId::ToggleFullContext)
    );
    assert_eq!(
        keymap_lookup(tui::pager::Key::Char(' '), KeyContext::Normal),
        None
    );
}

#[test]
fn help_includes_full_context_toggle_z() {
    let help = keymap_help_lines();
    let has_z = help.iter().any(|l| l.contains('z'));
    let has_label = help.iter().any(|l| l.contains("Toggle full file context"));
    assert!(has_z, "help must show z for full-context toggle");
    assert!(has_label, "help must describe full file context toggle");
}

#[test]
fn keymap_normal_navigation_keys() {
    use tui::pager::Key;
    let cases = [
        (Key::Char('j'), ActionId::ScrollDown),
        (Key::Char('k'), ActionId::ScrollUp),
        (Key::Char('d'), ActionId::NextHunk),
        (Key::Char('u'), ActionId::PrevHunk),
        (Key::Char('D'), ActionId::NextFile),
        (Key::Char('U'), ActionId::PrevFile),
        (Key::Char('a'), ActionId::ToggleSingleFile),
        (Key::Char('z'), ActionId::ToggleFullContext),
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
        (Key::Char('e'), ActionId::ToggleTree),
        (Key::Char('v'), ActionId::EnterVisual),
        (Key::Char('E'), ActionId::OpenEditor),
        (Key::Char('q'), ActionId::Quit),
        (Key::Char('?'), ActionId::Help),
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
fn keymap_tree_keys() {
    use tui::pager::Key;
    let cases = [
        (Key::Char('h'), ActionId::ReturnToDiff),
        (Key::Char('e'), ActionId::TreeClose),
        (Key::Char('j'), ActionId::TreeNavDown),
        (Key::Char('k'), ActionId::TreeNavUp),
        (Key::Char('g'), ActionId::TreeFirst),
        (Key::Char('G'), ActionId::TreeLast),
        (Key::Enter, ActionId::TreeSelect),
        (Key::Char('a'), ActionId::ToggleSingleFile),
        (Key::Char('d'), ActionId::NextHunk),
        (Key::Char('u'), ActionId::PrevHunk),
        (Key::Char('q'), ActionId::Quit),
    ];
    for (key, expected) in cases {
        assert_eq!(
            keymap_lookup(key, KeyContext::Tree),
            Some(expected),
            "Tree context: {key:?} should map to {expected:?}"
        );
    }
}

#[test]
fn keymap_visual_keys() {
    use tui::pager::Key;
    let cases = [
        (Key::Char('j'), ActionId::VisualExtendDown),
        (Key::Char('k'), ActionId::VisualExtendUp),
        (Key::Char('y'), ActionId::VisualCopy),
        (Key::Escape, ActionId::VisualCancel),
        (Key::Char('q'), ActionId::Quit),
    ];
    for (key, expected) in cases {
        assert_eq!(
            keymap_lookup(key, KeyContext::Visual),
            Some(expected),
            "Visual context: {key:?} should map to {expected:?}"
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
fn keymap_tree_only_keys_not_in_normal() {
    use tui::pager::Key;
    assert_eq!(
        keymap_lookup(Key::Char('h'), KeyContext::Normal),
        None,
        "h is ReturnToDiff in Tree only — should be None in Normal"
    );
}

#[test]
fn keymap_visual_only_keys_not_in_normal() {
    use tui::pager::Key;
    assert_eq!(
        keymap_lookup(Key::Char('y'), KeyContext::Normal),
        None,
        "y is VisualCopy in Visual only — should be None in Normal"
    );
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
fn help_includes_all_primary_runtime_actions() {
    let help_text = keymap_help_lines().join(" ");
    let required = [
        "d", "Next hunk", "u", "Previous hunk", "D", "Next file", "U", "Previous file",
        "a", "Toggle single file", "z", "Toggle full file context", "/", "Search",
        "n", "Next match", "N", "Previous match", "e", "Toggle tree panel",
        "v", "Enter visual mode", "E", "Open in editor", "q", "Quit",
    ];
    for s in required {
        assert!(
            help_text.contains(s),
            "help must include runtime action: {s:?}"
        );
    }
}
