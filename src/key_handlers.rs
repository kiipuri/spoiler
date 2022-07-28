use crossterm::event::KeyCode;

use crate::app::{App, FocusableWidget};

pub fn handler(key: KeyCode, app: &mut App) {
    match key {
        KeyCode::Char('k') | KeyCode::Up => handle_up(app),
        KeyCode::Char('j') | KeyCode::Down => handle_down(app),
        _ => (),
    }
}

fn handle_up(app: &mut App) {
    match app.focused_widget {
        FocusableWidget::TorrentList => app.previous(),
    }
}

fn handle_down(app: &mut App) {
    match app.focused_widget {
        FocusableWidget::TorrentList => app.next(),
    }
}
