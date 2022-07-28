use crossterm::event::KeyCode;

use crate::app::{App, FloatingWidget, FocusableWidget, Route};

pub fn handler(key: KeyCode, app: &mut App) {
    match key {
        KeyCode::Char('k') | KeyCode::Up => handle_up(app),
        KeyCode::Char('j') | KeyCode::Down => handle_down(app),
        KeyCode::Char('l') | KeyCode::Right => handle_right(app),
        KeyCode::Char('h') | KeyCode::Left => handle_left(app),
        KeyCode::Char('?') | KeyCode::F(1) => handle_help(app),
        KeyCode::Esc => handle_esc(app),
        _ => (),
    }
}

fn handle_up(app: &mut App) {
    match app.focused_widget {
        FocusableWidget::TorrentList => app.previous(),
        _ => (),
    }
}

fn handle_down(app: &mut App) {
    match app.focused_widget {
        FocusableWidget::TorrentList => app.next(),
        _ => (),
    }
}

fn handle_right(app: &mut App) {
    match app.focused_widget {
        FocusableWidget::TorrentList => {
            app.stack_push(Route::TorrentInfo);
        }
        _ => (),
    }
}

fn handle_left(app: &mut App) {
    match app.navigation_stack.last() {
        Some(Route::TorrentInfo) => {
            app.stack_pop();
        }
        _ => (),
    }
}

fn handle_help(app: &mut App) {
    app.focused_widget = FocusableWidget::Help;
}

fn handle_esc(app: &mut App) {
    if let Some(i) = app.navigation_stack.last() {
        // app.focused_widget = i;
    }
}
