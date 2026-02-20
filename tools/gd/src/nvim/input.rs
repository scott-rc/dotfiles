use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn to_nvim_key(event: &KeyEvent) -> Option<String> {
    use crossterm::event::KeyEventKind;

    if event.kind == KeyEventKind::Release || event.kind == KeyEventKind::Repeat {
        return None;
    }

    let mods = event.modifiers;
    let ctrl = mods.contains(KeyModifiers::CONTROL);
    let shift = mods.contains(KeyModifiers::SHIFT);
    let alt = mods.contains(KeyModifiers::ALT);

    let base = match event.code {
        KeyCode::Char(c) => {
            if ctrl || alt {
                let lo = c.to_ascii_lowercase();
                return Some(wrap_mods(ctrl, shift, alt, &lo.to_string()));
            }
            if c == '<' {
                return Some("<lt>".into());
            }
            return Some(c.to_string());
        }
        KeyCode::F(n) => format!("F{n}"),
        KeyCode::Enter => "CR".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Tab if shift => return Some("<S-Tab>".into()),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Backspace => "BS".into(),
        KeyCode::Delete => "Del".into(),
        KeyCode::Insert => "Insert".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::BackTab => return Some("<S-Tab>".into()),
        KeyCode::Null
        | KeyCode::CapsLock | KeyCode::ScrollLock | KeyCode::NumLock
        | KeyCode::PrintScreen | KeyCode::Pause | KeyCode::Menu
        | KeyCode::KeypadBegin
        | KeyCode::Modifier(_) | KeyCode::Media(_) => return None,
    };

    if ctrl || shift || alt {
        Some(wrap_mods(ctrl, shift, alt, &base))
    } else {
        Some(format!("<{base}>"))
    }
}

fn wrap_mods(ctrl: bool, shift: bool, alt: bool, key: &str) -> String {
    let mut prefix = String::new();
    if ctrl {
        prefix.push_str("C-");
    }
    if shift {
        prefix.push_str("S-");
    }
    if alt {
        prefix.push_str("A-");
    }
    format!("<{prefix}{key}>")
}
