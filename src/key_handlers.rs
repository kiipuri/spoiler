use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, FloatingWidget, FocusableWidget, InputMode, Route, RouteId};

pub async fn handler(key: KeyEvent, app: &mut App) {
    // if !matches!(app.floating_widget, FloatingWidget::None) {
    //     if key == KeyCode::Esc {
    //         handle_esc(app);
    //     }
    //     return;
    // }

    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('K') | KeyCode::Char('k') | KeyCode::Up => handle_up(key, app),
            KeyCode::Char('J') | KeyCode::Char('j') | KeyCode::Down => handle_down(key, app),
            KeyCode::Char('l') | KeyCode::Right => handle_right(app),
            KeyCode::Char('h') | KeyCode::Left => handle_left(app),
            KeyCode::Char('?') | KeyCode::F(1) => handle_help(app),
            KeyCode::Char('p') => handle_pause(app).await,
            KeyCode::Char('r') => handle_rename(app).await,
            KeyCode::Char('a') => handle_add(app).await,
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('d') => handle_remove(app).await,
            KeyCode::Char('t') => handle_toggle(app),
            KeyCode::Char('c') => handle_columns(app),
            KeyCode::Enter => handle_enter(app).await,
            KeyCode::Esc => handle_esc(app),
            _ => (),
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                app.rename_torrent().await;
                app.input_mode = InputMode::Normal;
                app.floating_widget = FloatingWidget::None
            }
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Esc => handle_esc(app),
            _ => (),
        },
    }
}

fn handle_up(key: KeyEvent, app: &mut App) {
    match app.floating_widget {
        FloatingWidget::AddTorrent => {
            app.previous_torrent_file();
            return;
        }
        FloatingWidget::AddTorrentConfirm | FloatingWidget::Help => return,
        FloatingWidget::ModifyColumns => match key.modifiers {
            KeyModifiers::SHIFT => {
                app.move_column_up();
                app.previous_column();
                return;
            }
            _ => {
                app.previous_column();
                return;
            }
        },
        _ => (),
    }

    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => app.previous(),
        Some(FocusableWidget::FileList) => app.previous_file(),
        _ => (),
    }
}

fn handle_down(key: KeyEvent, app: &mut App) {
    match app.floating_widget {
        FloatingWidget::AddTorrent => {
            app.next_torrent_file();
            return;
        }
        FloatingWidget::AddTorrentConfirm | FloatingWidget::Help => return,
        FloatingWidget::ModifyColumns => match key.modifiers {
            KeyModifiers::SHIFT => {
                app.move_column_down();
                app.next_column();
                return;
            }
            _ => {
                app.next_column();
                return;
            }
        },
        _ => (),
    }

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
    match app.floating_widget {
        FloatingWidget::AddTorrent => {
            app.floating_widget = FloatingWidget::AddTorrentConfirm;
            return;
        }
        FloatingWidget::AddTorrentConfirm => return,
        _ => (),
    }

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
    if let FloatingWidget::AddTorrentConfirm = app.floating_widget {
        app.floating_widget = FloatingWidget::AddTorrent;
    }

    match app.last_route_focused_widget() {
        Some(FocusableWidget::Tabs) => {
            app.previous_tab();
        }
        _ => (),
    }
}

fn handle_help(app: &mut App) {
    if !matches!(app.floating_widget, FloatingWidget::None) {
        return;
    }

    app.floating_widget = FloatingWidget::Help;
}

async fn handle_pause(app: &mut App) {
    match app.floating_widget {
        FloatingWidget::AddTorrentConfirm => {
            app.toggle_add_torrent_paused();
            return;
        }
        _ => (),
    }
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => {
            app.toggle_torrent_pause().await;
        }
        _ => (),
    }
}

async fn handle_rename(app: &mut App) {
    if !matches!(app.floating_widget, FloatingWidget::None) {
        return;
    }

    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => {
            app.floating_widget = FloatingWidget::Input;
            app.input_mode = InputMode::Editing;
        }
        _ => (),
    }
}

async fn handle_add(app: &mut App) {
    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => {
            app.floating_widget = FloatingWidget::AddTorrent;
            app.get_torrent_files();
        }
        _ => (),
    }
}

async fn handle_remove(app: &mut App) {
    if !matches!(app.floating_widget, FloatingWidget::None) {
        return;
    }

    match app.last_route_focused_widget() {
        Some(FocusableWidget::TorrentList) => {
            app.floating_widget = FloatingWidget::RemoveTorrent;
        }
        _ => (),
    }
}

fn handle_toggle(app: &mut App) {
    match app.floating_widget {
        FloatingWidget::RemoveTorrent => {
            app.delete_files = !app.delete_files;
        }
        _ => (),
    }
}

fn handle_columns(app: &mut App) {
    if !matches!(app.floating_widget, FloatingWidget::None) {
        return;
    }

    app.floating_widget = FloatingWidget::ModifyColumns;
}

async fn handle_enter(app: &mut App) {
    match app.floating_widget {
        FloatingWidget::AddTorrentConfirm => {
            app.add_torrent().await;
            app.floating_widget = FloatingWidget::None;
        }
        FloatingWidget::RemoveTorrent => {
            app.remove_torrent().await;
            app.floating_widget = FloatingWidget::None;
        }
        FloatingWidget::ModifyColumns => {
            app.toggle_show_column();
        }
        _ => (),
    }

    match app.last_route_focused_widget() {
        Some(FocusableWidget::FileList) => {
            app.torrent_collapse_files[app.selected_torrent_file.unwrap()].collapse =
                !app.torrent_collapse_files[app.selected_torrent_file.unwrap()].collapse;
            for file in &app.torrent_collapse_files {
                log::error!("{} | {}", file.collapse, file.path.path().to_str().unwrap());
            }
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
    app.input_mode = InputMode::Normal;
}
