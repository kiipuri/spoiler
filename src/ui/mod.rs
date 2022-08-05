use crate::{
    app::{FloatingWidget, RouteId},
    conversion::{convert_bytes, convert_rate, get_percentage},
};
use log::error;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Row, Table, TableState, Tabs},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use super::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.last_route_id() {
        Some(RouteId::TorrentList) => draw_torrent_list(f, app),
        Some(RouteId::TorrentInfo) => draw_torrent_info(f, app),
        _ => (),
    }

    match app.floating_widget {
        FloatingWidget::Help => draw_help(f),
        _ => (),
    }
}

fn draw_torrent_list<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let block = Block::default().title("Torrents").borders(Borders::ALL);
    let mut rows = vec![];
    let torrents: Vec<String> = app
        .torrents
        .arguments
        .torrents
        .iter()
        .map(|it| format!("{}", &it.name.as_ref().unwrap()))
        .collect();

    for torrent in torrents {
        rows.push(Row::new(vec![torrent]));
    }

    let mut state = TableState::default();
    state.select(app.selected_torrent);

    let table = Table::new(rows)
        .block(block.clone())
        .widths(&[
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .highlight_style(Style::default().bg(Color::Red));
    f.render_stateful_widget(table, chunks[0], &mut state);

    let logs = TuiLoggerWidget::default().block(block.clone().title("Logs"));
    f.render_widget(logs, chunks[1]);
}

fn draw_torrent_info<B: Backend>(f: &mut Frame<B>, app: &App) {
    let tabs = Tabs::new(vec![
        Spans::from(Span::styled("Overview", Style::default())),
        Spans::from(Span::styled("Files", Style::default())),
        Spans::from(Span::styled("tab 3", Style::default())),
    ])
    .block(Block::default().borders(Borders::ALL).title("tabs"))
    .highlight_style(Style::default().fg(Color::Yellow))
    .select(app.selected_tab);
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    f.render_widget(tabs, chunks[0]);

    match app.selected_tab {
        0 => draw_torrent_info_overview(f, app, chunks[1]),
        1 => draw_torrent_info_files(f, app, chunks[1]),
        _ => (),
    }
}

fn draw_torrent_info_overview<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let torrent = &app.torrents.arguments.torrents[app.selected_torrent.unwrap()];
    // let thing = torrent.total_size.as_ref().cloned();
    let size = torrent.total_size.as_ref().unwrap();
    let size = convert_bytes(size.to_owned());
    let percent = torrent.percent_done.as_ref().unwrap();
    let percent = get_percentage(percent.to_owned());

    let info_block = Block::default().title("Information").borders(Borders::ALL);
    let info_rows = vec![
        Row::new(vec!["Name", torrent.name.as_ref().unwrap()]),
        Row::new(vec!["Total size", size.as_str()]),
        Row::new(vec!["Percent done", percent.as_str()]),
        Row::new(vec!["Path", torrent.download_dir.as_ref().unwrap()]),
        Row::new(vec!["Magnet", torrent.hash_string.as_ref().unwrap()]),
    ];

    let info_table = Table::new(info_rows)
        .block(info_block)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);
    let download_speed = torrent.rate_download.as_ref().unwrap();
    let download_speed = convert_rate(download_speed.to_owned());

    let upload_speed = torrent.rate_upload.as_ref().unwrap();
    let upload_speed = convert_rate(upload_speed.to_owned());

    let transfer_block = Block::default().title("Transfer").borders(Borders::ALL);
    let transfer_rows = vec![
        Row::new(vec!["Download speed", download_speed.as_str()]),
        Row::new(vec!["Download limit", upload_speed.as_str()]),
        // Row::new(vec!["Ratio", torrent.ratio]),
        // Row::new(vec!["State", torrent.state]),
        // Row::new(vec!["Peers", torrent.peers]),
    ];

    let transfer_table = Table::new(transfer_rows)
        .block(transfer_block)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

    f.render_widget(info_table, chunks[0]);
    f.render_widget(transfer_table, chunks[1]);
}

fn draw_torrent_info_files<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().title("Files").borders(Borders::ALL);
    let mut rows = vec![];
    for file in app.torrents.arguments.torrents[app.selected_torrent.unwrap()]
        .files
        .as_ref()
        .unwrap()
    {
        rows.push(Row::new(vec![file.name.as_str()]));
    }

    let table = Table::new(rows)
        .block(block)
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(table, area);
}

fn draw_help<B: Backend>(f: &mut Frame<B>) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    let area = floating_rect(f);
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

fn floating_rect<B: Backend>(f: &mut Frame<B>) -> Rect {
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
