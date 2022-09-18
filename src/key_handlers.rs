use crossterm::event::KeyCode;

use crate::app::{App, FloatingWidget, FocusableWidget, Route, RouteId};

pub async fn handler(key: KeyCode, app: &mut App) {
    if !matches!(app.floating_widget, FloatingWidget::None) {
        if key == KeyCode::Esc {
            handle_esc(app);
        }
        return;
    }

    match key {
        KeyCode::Char('k') | KeyCode::Up => handle_up(app),
        KeyCode::Char('j') | KeyCode::Down => handle_down(app),
        KeyCode::Char('l') | KeyCode::Right => handle_right(app),
        KeyCode::Char('h') | KeyCode::Left => handle_left(app),
        KeyCode::Char('?') | KeyCode::F(1) => handle_help(app),
        KeyCode::Char('p') => handle_pause(app).await,
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc => handle_esc(app),
        _ => (),
    }
}

fn handle_up(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => app.previous(),
        Some(FocusableWidget::FileList) => app.previous_file(),
        _ => (),
    }
}

fn handle_down(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => app.next(),
        Some(FocusableWidget::Tabs) => {
            if app.selected_tab != 1 {
                return;
            }
            let index = app.navigation_stack.len() - 1;
            app.navigation_stack[index].focused_widget = FocusableWidget::FileList;
            app.selected_file = Some(0);
        }
        Some(FocusableWidget::FileList) => app.next_file(),
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
        // Some(FocusableWidget::FileList) => app.increment_priority(),
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
}

fn handle_help(app: &mut App) {
    app.floating_widget = FloatingWidget::Help;
}

async fn handle_pause(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => {
            app.toggle_torrent_pause().await;
        }
        _ => (),
    }
}

fn handle_esc(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::Tabs) => {
            app.stack_pop();
            return;
        }
        Some(FocusableWidget::FileList) => {
            let index = app.navigation_stack.len() - 1;
            app.navigation_stack[index].focused_widget = FocusableWidget::Tabs;
            app.selected_file = None;
        }
        _ => (),
    }

    app.floating_widget = FloatingWidget::None;
}
