use crate::app::{FloatingWidget, FocusableWidget, RouteId, Torrent};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};

use super::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.last_route_id() {
        Some(RouteId::TorrentList) => draw_torrent_list(f, app),
        Some(RouteId::TorrentInfo) => draw_torrent_info(f, app),
        _ => (),
    }

    match app.floating_widget {
        FloatingWidget::Help => draw_help(f, app),
        _ => (),
    }
}

fn draw_torrent_list<B: Backend>(f: &mut Frame<B>, app: &App) {
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

fn draw_torrent_info<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // let torrent = &app.torrents[app.selected_torrent.unwrap()];
    let torrent = Torrent::default();
    // draw_torrent_info_overview(f, app, torrent);
    draw_torrent_info_files(f, app, &torrent);
}

fn draw_torrent_info_overview<B: Backend>(f: &mut Frame<B>, app: &App, torrent: &Torrent) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let info_block = Block::default().title("Information").borders(Borders::ALL);
    let info_rows = vec![
        Row::new(vec!["Name", &torrent.name]),
        Row::new(vec!["Total size", &torrent.total_size]),
        Row::new(vec!["Percent done", &torrent.percent_done]),
        Row::new(vec!["Path", &torrent.location]),
        Row::new(vec!["Magnet", &torrent.magnet]),
    ];

    let info_table = Table::new(info_rows)
        .block(info_block)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

    let transfer_block = Block::default().title("Transfer").borders(Borders::ALL);
    let transfer_rows = vec![
        Row::new(vec!["Size", &torrent.total_size]),
        Row::new(vec!["Downloaded", &torrent.downloaded]),
        Row::new(vec!["Download speed", &torrent.download_speed]),
        Row::new(vec!["Download limit", &torrent.download_limit]),
        Row::new(vec!["Ratio", &torrent.ratio]),
        Row::new(vec!["State", &torrent.state]),
        Row::new(vec!["Peers", &torrent.peers]),
    ];

    let transfer_table = Table::new(transfer_rows)
        .block(transfer_block)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

    f.render_widget(info_table, chunks[0]);
    f.render_widget(transfer_table, chunks[1]);
}

fn draw_torrent_info_files<B: Backend>(f: &mut Frame<B>, app: &mut App, torrent: &Torrent) {
    app.get_torrent_files(torrent.id);

    let block = Block::default().title("Files").borders(Borders::ALL);
    let mut rows = vec![];
    for file in &app.torrent_files {
        rows.push(Row::new(vec![file.name.to_owned(), file.done.to_owned()]));
    }

    let table = Table::new(rows).block(block).widths(&[
        Constraint::Percentage(50),
        Constraint::Percentage(50),
        // Constraint::Percentage(16),
        // Constraint::Percentage(16),
        // Constraint::Percentage(16),
        // Constraint::Percentage(16),
    ]);
    f.render_widget(table, f.size());
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
