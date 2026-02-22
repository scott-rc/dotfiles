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
