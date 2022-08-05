use crossterm::event::KeyCode;

use crate::app::{App, FloatingWidget, FocusableWidget, Route, RouteId};

pub fn handler(key: KeyCode, app: &mut App) {
    if key == KeyCode::Esc {
        handle_esc(app);
    }

    if !matches!(app.floating_widget, FloatingWidget::None) {
        return;
    }

    match key {
        KeyCode::Char('k') | KeyCode::Up => handle_up(app),
        KeyCode::Char('j') | KeyCode::Down => handle_down(app),
        KeyCode::Char('l') | KeyCode::Right => handle_right(app),
        KeyCode::Char('h') | KeyCode::Left => handle_left(app),
        KeyCode::Char('?') | KeyCode::F(1) => handle_help(app),
        // KeyCode::Esc => handle_esc(app),
        _ => (),
    }
}

fn handle_up(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => app.previous(),
        _ => (),
    }
}

fn handle_down(app: &mut App) {
    if !matches!(app.floating_widget, FloatingWidget::None) {
        return;
    }

    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => app.next(),
        _ => (),
    }
}

fn handle_right(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => {
            app.stack_push(Route {
                id: RouteId::TorrentInfo,
                focused_widget: FocusableWidget::Tabs,
            });
        }
        Some(FocusableWidget::Tabs) => app.next_tab(),
        _ => (),
    }
}

fn handle_left(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::Tabs) => {
            app.previous_tab();
            return;
        }
        _ => (),
    }

    match app.last_route_id() {
        Some(RouteId::TorrentInfo) => {
            app.stack_pop();
        }
        _ => (),
    }
}

fn handle_help(app: &mut App) {
    app.floating_widget = FloatingWidget::Help;
}

fn handle_esc(app: &mut App) {
    if matches!(app.floating_widget, FloatingWidget::None) {
        match app.last_route_focused_widget() {
            Some(FocusableWidget::Tabs) => {
                app.stack_pop();
                return;
            }
            _ => (),
        }
    }

    app.floating_widget = FloatingWidget::None;
}
