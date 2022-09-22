use crate::{
    app::{FloatingWidget, RouteId},
    conversion::{convert_bytes, convert_rate, get_percentage, status_string},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
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
        FloatingWidget::RemoveTorrent => draw_delete_torrent(f, &app),
        FloatingWidget::ModifyColumns => draw_modify_columns(f, &app),
        _ => (),
    }
}

fn draw_torrent_list<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let block = Block::default().title("Torrents").borders(Borders::ALL);
    let (header_rows, rows) = app.get_torrent_rows();

    let mut state = TableState::default();
    state.select(app.selected_torrent);

    let mut columns_count = 0;
    let mut widths = Vec::new();

    for column in &app.all_info_columns {
        if column.show {
            columns_count += 1;
        }
    }

    for _ in 0..columns_count {
        widths.push(Constraint::Ratio(1, columns_count));
    }

    let table = Table::new(rows)
        .header(Row::new(header_rows))
        .block(block.clone())
        .widths(&widths)
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
    let area = floating_rect(f, 50, 15);
    let rows = vec![
        Row::new(vec!["j / Down", "Move down"]),
        Row::new(vec!["k / Up", "Move up"]),
        Row::new(vec!["l", "Open torrent / Move right"]),
        Row::new(vec!["h", "Move left"]),
        Row::new(vec!["p", "Pause/unpause torrent"]),
        Row::new(vec!["r", "Rename torrent"]),
        Row::new(vec!["d", "Delete torrent"]),
        Row::new(vec!["t", "Toggle torrent files deletion"]),
        Row::new(vec!["Enter", "Confirm"]),
        Row::new(vec!["Esc", "Go back"]),
        Row::new(vec!["q", "Exit"]),
    ];
    let table = Table::new(rows)
        .block(block)
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);
    f.render_widget(Clear, area);
    f.render_widget(table, area);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f, 100, 3);
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
    let area = floating_rect(f, 100, app.torrent_files.len() as u32 + 2);
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

    let area = floating_rect(f, 100, 9);
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

fn draw_delete_torrent<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f, 45, 6);
    let mut text = Text::from(Spans::from(vec![
        Span::from("Delete "),
        Span::styled(
            app.get_selected_torrent_name(),
            Style::default().add_modifier(Modifier::ITALIC),
        ),
        Span::from("?"),
    ]));

    if app.delete_files {
        text.extend(Text::from(Spans::from(vec![Span::styled(
            "Delete files on disk",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )])));
    } else {
        text.extend(Text::from(Spans::from(vec![Span::styled(
            "Delete torrent only",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )])));
    }

    text.extend(Text::raw("\nPress T to toggle deletion"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Delete torrent");

    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(tui::layout::Alignment::Center),
        area,
    );
}

fn draw_modify_columns<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f, 100, 20);
    let mut items = Vec::new();

    for column in &app.all_info_columns {
        let list_item = ListItem::new(column.column.to_str());
        if column.show {
            items.push(list_item.style(Style::default().bg(Color::Green).fg(Color::Black)));
        } else {
            items.push(list_item.style(Style::default().bg(Color::Blue).fg(Color::Black)));
        }
    }

    let list = List::new(items).highlight_style(Style::default().bg(Color::Red));
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Modify columns");
    let mut state = ListState::default();
    state.select(app.selected_column);
    f.render_widget(Clear, area);
    f.render_stateful_widget(list.block(block), area, &mut state);
}

fn floating_rect<B: Backend>(f: &mut Frame<B>, width: u32, height: u32) -> Rect {
    let float_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(
                ((100 as f32 - height as f32 / f.size().height as f32 * 100 as f32) / 2 as f32)
                    .ceil() as u16,
            ),
            Constraint::Length(height as u16),
            Constraint::Percentage(
                ((100 as f32 - height as f32 / f.size().height as f32 * 100 as f32) / 2 as f32)
                    .ceil() as u16,
            ),
        ])
        .split(f.size());

    Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(
                ((100 as f32 - width as f32 / f.size().width as f32 * 100 as f32) / 2 as f32)
                    .round() as u16,
            ),
            Constraint::Length(width as u16),
            Constraint::Percentage(
                ((100 as f32 - width as f32 / f.size().width as f32 * 100 as f32) / 2 as f32)
                    .round() as u16,
            ),
        ])
        .split(float_layout[1])[1]
}
