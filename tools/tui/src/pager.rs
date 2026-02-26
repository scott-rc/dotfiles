use std::io::Write;
use std::process::{Command, Stdio};

// Terminal control constants
pub const ALT_SCREEN_ON: &str = "\x1b[?1049h";
pub const ALT_SCREEN_OFF: &str = "\x1b[?1049l";
pub const CURSOR_HIDE: &str = "\x1b[?25l";
pub const CURSOR_SHOW: &str = "\x1b[?25h";
pub const CLEAR_LINE: &str = "\x1b[2K";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Char(char),
    Tab,
    BackTab,
    Enter,
    Escape,
    Backspace,
    CtrlC,
    CtrlD,
    CtrlH,
    CtrlL,
    CtrlU,
    Up,
    Down,
    Left,
    Right,
    AltLeft,
    AltRight,
    AltBackspace,
    PageUp,
    PageDown,
    Home,
    End,
    Unknown,
}

pub fn crossterm_to_key(key_event: crossterm::event::KeyEvent) -> Key {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mods = key_event.modifiers;
    match key_event.code {
        // Ctrl combos
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlC,
        KeyCode::Char('d') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlD,
        KeyCode::Char('h') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlH,
        KeyCode::Char('l') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlL,
        KeyCode::Char('u') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlU,
        // Alt combos
        KeyCode::Char('b') | KeyCode::Left if mods.contains(KeyModifiers::ALT) => Key::AltLeft,
        KeyCode::Char('f') | KeyCode::Right if mods.contains(KeyModifiers::ALT) => Key::AltRight,
        KeyCode::Backspace if mods.contains(KeyModifiers::ALT) => Key::AltBackspace,
        // Plain chars
        KeyCode::Char(c) => Key::Char(c),
        // Nav keys
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        // Special keys
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        _ => Key::Unknown,
    }
}

pub fn get_term_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}

pub fn move_to(out: &mut impl Write, row: u16, col: u16) {
    let _ = write!(out, "\x1b[{};{}H", row + 1, col + 1);
}

pub fn copy_to_clipboard(text: &str) -> bool {
    let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() else {
        return false;
    };
    let Some(mut stdin) = child.stdin.take() else {
        return false;
    };
    if stdin.write_all(text.as_bytes()).is_err() {
        return false;
    }
    drop(stdin);
    child.wait().is_ok()
}

pub fn open_in_editor(path: &str, line: Option<usize>, read_only: bool) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    let basename = editor.rsplit('/').next().unwrap_or(&editor);
    let is_vim = basename == "vim" || basename == "nvim";

    let mut args: Vec<String> = Vec::new();
    if read_only && is_vim {
        args.push("-R".to_string());
    }
    if is_vim && let Some(l) = line {
        args.push(format!("+{l}"));
    }
    args.push(path.to_string());

    let _ = Command::new(&editor)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_crossterm_to_key_ctrl_combos() {
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Key::CtrlC
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('d'), KeyModifiers::CONTROL)),
            Key::CtrlD
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('h'), KeyModifiers::CONTROL)),
            Key::CtrlH
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('l'), KeyModifiers::CONTROL)),
            Key::CtrlL
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('u'), KeyModifiers::CONTROL)),
            Key::CtrlU
        );
    }

    #[test]
    fn test_crossterm_to_key_alt_combos() {
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('b'), KeyModifiers::ALT)),
            Key::AltLeft
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('f'), KeyModifiers::ALT)),
            Key::AltRight
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Left, KeyModifiers::ALT)),
            Key::AltLeft
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Right, KeyModifiers::ALT)),
            Key::AltRight
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Backspace, KeyModifiers::ALT)),
            Key::AltBackspace
        );
    }

    #[test]
    fn test_crossterm_to_key_nav_and_special() {
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Up, KeyModifiers::NONE)),
            Key::Up
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Down, KeyModifiers::NONE)),
            Key::Down
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Left, KeyModifiers::NONE)),
            Key::Left
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Right, KeyModifiers::NONE)),
            Key::Right
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::PageUp, KeyModifiers::NONE)),
            Key::PageUp
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::PageDown, KeyModifiers::NONE)),
            Key::PageDown
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Home, KeyModifiers::NONE)),
            Key::Home
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::End, KeyModifiers::NONE)),
            Key::End
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Tab, KeyModifiers::NONE)),
            Key::Tab
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Enter, KeyModifiers::NONE)),
            Key::Enter
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Esc, KeyModifiers::NONE)),
            Key::Escape
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Backspace, KeyModifiers::NONE)),
            Key::Backspace
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Char('q'), KeyModifiers::NONE)),
            Key::Char('q')
        );
        assert_eq!(
            crossterm_to_key(make_key(KeyCode::Insert, KeyModifiers::NONE)),
            Key::Unknown
        );
    }
}
