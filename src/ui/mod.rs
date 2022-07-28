use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{FocusableWidget, Route};

use super::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.navigation_stack.last() {
        Some(Route::Overview) => draw_overview(f, app),
        Some(Route::TorrentInfo) => draw_torrent_info(f, app),
        _ => (),
    }

    match app.focused_widget {
        FocusableWidget::Help => draw_help(f, app),
        _ => (),
    }
}

fn draw_overview<B: Backend>(f: &mut Frame<B>, app: &App) {
    let block = Block::default().title("Torrents").borders(Borders::ALL);
    let mut rows = vec![];
    for torrent in &app.torrents {
        rows.push(Row::new(vec![
            torrent.name.to_owned(),
            torrent.id.to_string(),
        ]));
    }

    let mut state = TableState::default();
    state.select(app.selected_torrent);

    let table = Table::new(rows)
        .block(block)
        .widths(&[
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .highlight_style(Style::default().bg(Color::Red));
    f.render_stateful_widget(table, f.size(), &mut state);

    // let paragraph = Paragraph::new(format!("{:#?}", &app.torrents));
    // f.render_widget(paragraph, f.size());
}

fn draw_torrent_info<B: Backend>(f: &mut Frame<B>, app: &App) {
    let block = Block::default().title("Torrent info").borders(Borders::ALL);
    let torrent = &app.torrents[app.selected_torrent.unwrap()];
    let list = List::new(vec![
        ListItem::new(torrent.id.to_string()),
        ListItem::new(torrent.name.to_owned()),
    ])
    .block(block);

    f.render_widget(list, f.size());
}

fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    let area = floating_rect(f, app);
    let rows = vec![
        Row::new(vec!["j / Down", "Move down"]),
        Row::new(vec!["k / Up", "Move up"]),
        Row::new(vec!["l", "Open torrent / Move right"]),
        Row::new(vec!["h", "Go back / Move left"]),
        Row::new(vec!["q", "Exit"]),
    ];
    let table = Table::new(rows)
        .block(block)
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(Clear, area);
    f.render_widget(table, area);
}

fn floating_rect<B: Backend>(f: &mut Frame<B>, app: &App) -> Rect {
    let float_layout = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(f.size());

    Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(float_layout[1])[1]
}
