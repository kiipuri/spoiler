use crate::{
    app::{FloatingWidget, RouteId},
    conversion::{convert_bytes, convert_rate, get_percentage, status_string},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState, Tabs},
    Frame,
};
use tui_logger::TuiLoggerWidget;
use unicode_width::UnicodeWidthStr;

use super::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.last_route_id() {
        Some(RouteId::TorrentList) => draw_torrent_list(f, app),
        Some(RouteId::TorrentInfo) => draw_torrent_info(f, app),
        _ => (),
    }

    match app.floating_widget {
        FloatingWidget::Help => draw_help(f),
        FloatingWidget::Input => draw_input(f, &app),
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

    for torrent in &app.torrents.arguments.torrents {
        rows.push(Row::new(vec![
            torrent.name.as_ref().unwrap().to_owned(),
            status_string(&torrent.status.as_ref().unwrap()).to_string(),
            get_percentage(torrent.percent_done.as_ref().unwrap().to_owned()),
            convert_rate(*torrent.rate_download.as_ref().unwrap()),
            convert_rate(*torrent.rate_upload.as_ref().unwrap()),
            format!("{:.2}", torrent.upload_ratio.as_ref().unwrap()),
        ]));
    }

    let mut state = TableState::default();
    state.select(app.selected_torrent);

    let table = Table::new(rows)
        .header(Row::new(vec![
            "Name",
            "Status",
            "Progress",
            "Down Speed",
            "Up Speed",
            "Ratio",
        ]))
        .block(block.clone())
        .widths(&[
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 8),
            Constraint::Ratio(1, 8),
            Constraint::Ratio(1, 8),
            Constraint::Ratio(1, 8),
            Constraint::Ratio(1, 8),
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

    let info_block = Block::default().title("Information").borders(Borders::ALL);
    let info_rows = vec![
        Row::new(vec!["Name", torrent.name.as_ref().unwrap()]),
        Row::new(vec![
            "Total size".to_string(),
            convert_bytes(*torrent.total_size.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Percent done".to_string(),
            get_percentage(*torrent.percent_done.as_ref().unwrap()),
        ]),
        Row::new(vec!["Path", torrent.download_dir.as_ref().unwrap()]),
    ];

    let info_table = Table::new(info_rows)
        .block(info_block)
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

    let transfer_block = Block::default().title("Transfer").borders(Borders::ALL);
    let transfer_rows = vec![
        Row::new(vec![
            "Download speed".to_string(),
            convert_rate(*torrent.rate_download.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Downloaded".to_string(),
            convert_bytes(
                *torrent.size_when_done.as_ref().unwrap()
                    - *torrent.left_until_done.as_ref().unwrap(),
            ),
        ]),
        Row::new(vec![
            "Upload speed".to_string(),
            convert_rate(*torrent.rate_upload.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Uploaded".to_string(),
            convert_bytes(*torrent.uploaded_ever.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Ratio".to_string(),
            format!("{:.2}", torrent.upload_ratio.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Status",
            status_string(torrent.status.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Eta".to_string(),
            torrent.eta.as_ref().unwrap().to_string(),
        ]),
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
    let priorities = app.torrents.arguments.torrents[app.selected_torrent.unwrap()]
        .priorities
        .as_ref()
        .unwrap()
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>();

    let mut index = 0;
    for file in app.torrents.arguments.torrents[app.selected_torrent.unwrap()]
        .files
        .as_ref()
        .unwrap()
    {
        rows.push(Row::new(vec![
            file.name.as_str(),
            priorities[index].as_str(),
        ]));
        index += 1;
    }

    let mut state = TableState::default();
    state.select(app.selected_file);

    let table = Table::new(rows)
        .header(Row::new(vec!["Filename", "Priority"]))
        .block(block)
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
        .highlight_style(Style::default().bg(Color::Red));
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_help<B: Backend>(f: &mut Frame<B>) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    let area = floating_rect(f);
    let rows = vec![
        Row::new(vec!["j / Down", "Move down"]),
        Row::new(vec!["k / Up", "Move up"]),
        Row::new(vec!["l", "Open torrent / Move right"]),
        Row::new(vec!["h", "Move left"]),
        Row::new(vec!["p", "Pause/unpause torrent"]),
        Row::new(vec!["r", "Rename torrent"]),
        Row::new(vec!["Esc", "Go back"]),
        Row::new(vec!["q", "Exit"]),
    ];
    let table = Table::new(rows)
        .block(block)
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(Clear, area);
    f.render_widget(table, area);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f);
    let input = Paragraph::new(app.input.as_ref()).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Rename torrent"),
    );

    f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1);
    f.render_widget(Clear, area);
    f.render_widget(input, area);
}

fn floating_rect<B: Backend>(f: &mut Frame<B>) -> Rect {
    let float_layout = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(3),
            // Constraint::Percentage(50),
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
