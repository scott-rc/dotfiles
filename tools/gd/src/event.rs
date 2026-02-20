use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;

use crate::nvim::bridge::RedrawEvent;

pub enum AppCmd {
    NextFile,
    PrevFile,
    Quit,
}

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Redraw(Vec<RedrawEvent>),
    Cmd(AppCmd),
}

pub async fn next(
    redraw_rx: &mut mpsc::UnboundedReceiver<Vec<RedrawEvent>>,
    cmd_rx: &mut mpsc::UnboundedReceiver<AppCmd>,
) -> Option<AppEvent> {
    tokio::select! {
        biased;

        cmd = cmd_rx.recv() => {
            cmd.map(AppEvent::Cmd)
        }

        // Prioritize nvim redraw events for responsiveness
        events = redraw_rx.recv() => {
            events.map(AppEvent::Redraw)
        }

        result = tokio::task::spawn_blocking(event::read) => {
            match result {
                Ok(Ok(Event::Key(k))) if k.kind == KeyEventKind::Press => Some(AppEvent::Key(k)),
                Ok(Ok(Event::Resize(w, h))) => Some(AppEvent::Resize(w, h)),
                Ok(Ok(_) | Err(_)) | Err(_) => None,
            }
        }
    }
}
