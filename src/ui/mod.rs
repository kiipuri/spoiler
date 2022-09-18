use crate::{
    app::{FloatingWidget, RouteId},
    conversion::{convert_bytes, convert_rate, get_percentage, status_string},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs,
    },
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
        FloatingWidget::AddTorrent => draw_add_torrent(f, &app),
        FloatingWidget::AddTorrentConfirm => draw_add_torrent_confirm(f, &app),
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
    let area = floating_rect(f, 10);
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
    let area = floating_rect(f, 3);
    let input = Paragraph::new(app.input.as_ref()).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Rename torrent"),
    );

    f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1);
    f.render_widget(Clear, area);
    f.render_widget(input, area);
}

fn draw_add_torrent<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f, app.torrent_files.len() as u32 + 2);
    let mut rows = Vec::new();
    for file in &app.torrent_files {
        rows.push(ListItem::new(file.to_str().unwrap()));
    }
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Add torrent"))
        .highlight_style(Style::default().bg(Color::Red));

    let mut state = ListState::default();
    state.select(app.selected_torrent_file);

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_add_torrent_confirm<B: Backend>(f: &mut Frame<B>, app: &App) {
    let torrent = lava_torrent::torrent::v1::Torrent::read_from_file(
        app.torrent_files[app.selected_torrent_file.unwrap()].as_path(),
    )
    .unwrap();

    let text = Text::from(Spans::from(vec![
        Span::raw("Press "),
        Span::styled("P", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to toggle paused, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to add torrent"),
    ]));

    let area = floating_rect(f, 9);
    let chunks = Layout::default()
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .margin(1)
        .split(area);
    let rows = vec![
        Row::new(vec![
            "Filename",
            app.torrent_files[app.selected_torrent_file.unwrap()]
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        ]),
        Row::new(vec!["Torrent name".to_string(), torrent.name.to_string()]),
        Row::new(vec!["Size".to_string(), convert_bytes(torrent.length)]),
        Row::new(vec![
            "Info hash".to_string(),
            torrent.info_hash().to_string(),
        ]),
        Row::new(vec!["Start paused".to_string(), app.add_paused.to_string()]),
    ];
    let table = Table::new(rows).widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);
    let block = Block::default().borders(Borders::ALL).title("Add torrent");

    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(text).alignment(tui::layout::Alignment::Center),
        chunks[0],
    );
    f.render_widget(table, chunks[1]);
    f.render_widget(block, area);
}

fn floating_rect<B: Backend>(f: &mut Frame<B>, height: u32) -> Rect {
    let float_layout = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage(
                ((100 as f32 - height as f32 / f.size().height as f32 * 100 as f32) / 2 as f32)
                    .round() as u16,
            ),
            Constraint::Length(height as u16),
            Constraint::Percentage(
                ((100 as f32 - height as f32 / f.size().height as f32 * 100 as f32) / 2 as f32)
                    .round() as u16,
            ),
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
