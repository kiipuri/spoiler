use crate::{
    app::{FloatingWidget, RouteId},
    conversion::{
        convert_bytes, convert_rate, convert_secs, date, get_status_percentage, status_string,
    },
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::{
        Axis, Block, Borders, Chart, Clear, Dataset, List, ListItem, ListState, Paragraph, Row,
        Table, TableState, Tabs,
    },
    Frame,
};
use tui_logger::TuiLoggerWidget;
use tui_tree_widget::Tree;
use unicode_width::UnicodeWidthStr;

use super::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.last_route_id() {
        Some(RouteId::TorrentList) => draw_torrent_list(f, app),
        Some(RouteId::TorrentInfo) => draw_torrent_info(f, app),
        _ => (),
    }

    match app.floating_widget {
        FloatingWidget::Help => draw_help(f, app),
        FloatingWidget::Input => draw_input(f, app),
        FloatingWidget::AddTorrent => draw_add_torrent(f, app),
        FloatingWidget::AddTorrentConfirm => draw_add_torrent_confirm(f, app),
        FloatingWidget::RemoveTorrent => draw_delete_torrent(f, app),
        FloatingWidget::ModifyColumns => draw_modify_columns(f, app),
        _ => (),
    }
}

fn draw_torrent_list<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(f.size().height - 17),
                Constraint::Length(13),
                Constraint::Length(4),
            ]
            .as_ref(),
        )
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
        .style(app.config.get_style())
        .highlight_style(app.config.get_highlight_style());
    f.render_stateful_widget(table, chunks[0], &mut state);

    {
        let session_block = Block::default()
            .style(app.config.get_style())
            .borders(Borders::ALL)
            .title("Session Stats");

        let table = Table::new(vec![
            Row::new(vec![
                "Down:".to_string(),
                convert_rate(app.session_stats.as_ref().unwrap().download_speed),
                "Up:".to_string(),
                convert_rate(app.session_stats.as_ref().unwrap().upload_speed),
                "Downloaded:".to_string(),
                convert_bytes(
                    app.session_stats
                        .as_ref()
                        .unwrap()
                        .current_stats
                        .downloaded_bytes,
                ),
                "Uploaded:".to_string(),
                convert_bytes(
                    app.session_stats
                        .as_ref()
                        .unwrap()
                        .current_stats
                        .uploaded_bytes,
                ),
            ]),
            Row::new(vec![
                "Slow Mode:".to_string(),
                app.session.as_ref().unwrap().alt_speed_enabled.to_string(),
                "Slow Mode Down:".to_string(),
                convert_rate(app.session.as_ref().unwrap().alt_speed_down * 1000),
                "Slow Mode Up:".to_string(),
                convert_rate(app.session.as_ref().unwrap().alt_speed_up * 1000),
            ]),
        ])
        .widths(&[
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(16),
            Constraint::Min(14),
            Constraint::Min(14),
            Constraint::Min(14),
            Constraint::Min(14),
            Constraint::Min(14),
        ])
        .block(session_block);
        f.render_widget(table, chunks[2]);
    }

    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        let info_block = Block::default().title("Information").borders(Borders::ALL);
        let transfer_block = Block::default().title("Transfer").borders(Borders::ALL);

        if app.torrents.is_empty() {
            return;
        }

        let sel_torrent = &app.torrents[app.selected_torrent.unwrap()];
        let info_rows = vec![
            Row::new(vec!["Name".to_string(), app.get_selected_torrent_name()]),
            Row::new(vec![
                "Status",
                status_string(sel_torrent.status.as_ref().unwrap()),
            ]),
            Row::new(vec![
                "Total Size".to_string(),
                convert_bytes(sel_torrent.total_size.unwrap()),
            ]),
            Row::new(vec![
                "Progress".to_string(),
                get_status_percentage(sel_torrent),
            ]),
            Row::new(vec![
                "Save Path".to_string(),
                sel_torrent.download_dir.as_ref().unwrap().to_string(),
            ]),
            Row::new(vec![
                "Date Added".to_string(),
                date(sel_torrent.added_date.unwrap()),
            ]),
            Row::new(vec![
                "Date Completed".to_string(),
                date(sel_torrent.done_date.unwrap()),
            ]),
            Row::new(vec![
                "Info Hash".to_string(),
                sel_torrent.hash_string.as_ref().unwrap().to_string(),
            ]),
            Row::new(vec![
                "Comment".to_string(),
                sel_torrent.comment.as_ref().unwrap().to_string(),
            ]),
            Row::new(vec![
                "Created On".to_string(),
                sel_torrent.creator.as_ref().unwrap().to_string(),
            ]),
            Row::new(vec![
                "Pieces".to_string(),
                sel_torrent.piece_count.as_ref().unwrap().to_string(),
            ]),
        ];

        let transfer_rows = vec![
            Row::new(vec![
                "Time Active".to_string(),
                convert_secs(
                    sel_torrent.seconds_downloading.unwrap() + sel_torrent.seconds_seeding.unwrap(),
                ),
            ]),
            Row::new(vec![
                "Downloaded".to_string(),
                convert_bytes(
                    sel_torrent.size_when_done.as_ref().unwrap()
                        - sel_torrent.left_until_done.as_ref().unwrap(),
                ),
            ]),
            Row::new(vec![
                "Download Speed".to_string(),
                convert_rate(sel_torrent.rate_download.unwrap()),
            ]),
            Row::new(vec![
                "Download Limit".to_string(),
                convert_rate(sel_torrent.download_limit.unwrap() * 1000),
            ]),
            Row::new(vec![
                "Uploaded".to_string(),
                convert_bytes(*sel_torrent.uploaded_ever.as_ref().unwrap()),
            ]),
            Row::new(vec![
                "Upload Speed".to_string(),
                convert_rate(*sel_torrent.rate_upload.as_ref().unwrap()),
            ]),
            Row::new(vec![
                "Upload Limit".to_string(),
                convert_rate(*sel_torrent.upload_limit.as_ref().unwrap() * 1000),
            ]),
            Row::new(vec![
                "Ratio".to_string(),
                format!("{:.2}", sel_torrent.upload_ratio.as_ref().unwrap()),
            ]),
            Row::new(vec![
                "Eta".to_string(),
                convert_secs(*sel_torrent.eta.as_ref().unwrap()),
            ]),
            Row::new(vec![
                "Connections".to_string(),
                sel_torrent.peers_connected.unwrap().to_string(),
            ]),
        ];

        let info_table = Table::new(info_rows)
            .block(info_block)
            .style(app.config.get_style())
            .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);
        f.render_widget(info_table, chunks[0]);

        let transfer_table = Table::new(transfer_rows)
            .block(transfer_block)
            .style(app.config.get_style())
            .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);
        f.render_widget(transfer_table, chunks[1]);
    }
}

fn draw_torrent_info<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let tabs = Tabs::new(vec![
        Spans::from(Span::styled("Speed", Style::default())),
        Spans::from(Span::styled("Files", Style::default())),
        Spans::from(Span::styled("Logs", Style::default())),
    ])
    .block(Block::default().borders(Borders::ALL).title("tabs"))
    .style(app.config.get_style())
    .highlight_style(app.config.get_style().fg(Color::Yellow))
    .select(app.selected_tab);
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    f.render_widget(tabs, chunks[0]);

    match app.selected_tab {
        0 => draw_speed_chart(f, app, chunks[1]),
        1 => draw_torrent_info_files(f, app, chunks[1]),
        2 => logs(f, app, chunks[1]),
        _ => (),
    }
}

fn draw_speed_chart<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let speed_block = Block::default().title("Speed").borders(Borders::ALL);

    let datasets = vec![
        Dataset::default()
            .name("Download")
            .marker(symbols::Marker::Braille)
            .style(app.config.get_style().fg(Color::LightGreen))
            .graph_type(tui::widgets::GraphType::Line)
            .data(&app.data.download),
        Dataset::default()
            .name("Upload")
            .marker(symbols::Marker::Braille)
            .style(app.config.get_style().fg(Color::Blue))
            .graph_type(tui::widgets::GraphType::Line)
            .data(&app.data.upload),
    ];
    let chart = Chart::new(datasets)
        .block(speed_block)
        .style(app.config.get_style())
        .x_axis(Axis::default().bounds([0.0, 10.0]))
        .y_axis(Axis::default().bounds([0.0, app.data.height]).labels(vec![
            Span::raw(""),
            Span::raw(convert_rate(app.data.height as i64)),
        ]));
    f.render_widget(chart, area);
}

fn draw_torrent_info_files<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let items = Tree::new(&*app.tree.items)
        .block(Block::default().title("Files").borders(Borders::ALL))
        .style(app.config.get_style())
        .highlight_style(app.config.get_highlight_style());
    f.render_stateful_widget(items, area, &mut app.tree.state);
}

fn logs<B: Backend>(f: &mut Frame<B>, _app: &mut App, area: Rect) {
    let logs =
        TuiLoggerWidget::default().block(Block::default().title("Logs").borders(Borders::ALL));
    f.render_widget(logs, area);
}

fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App) {
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
        Row::new(vec!["c", "Modify torrent field columns"]),
        Row::new(vec!["Enter", "Confirm"]),
        Row::new(vec!["Esc", "Go back"]),
        Row::new(vec!["q", "Exit"]),
    ];
    let table = Table::new(rows)
        .block(block)
        .style(app.config.get_style())
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);
    f.render_widget(Clear, area);
    f.render_widget(table, area);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f, 100, 3);
    let input = Paragraph::new(app.input.as_ref()).block(
        Block::default()
            .borders(Borders::ALL)
            .style(app.config.get_style())
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
        .style(app.config.get_style())
        .highlight_style(app.config.get_highlight_style());

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
        Row::new(vec!["Info hash".to_string(), torrent.info_hash()]),
        Row::new(vec!["Start paused".to_string(), app.add_paused.to_string()]),
    ];
    let table = Table::new(rows)
        .style(app.config.get_style())
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add torrent")
        .style(app.config.get_style());

    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(text)
            .alignment(tui::layout::Alignment::Center)
            .style(app.config.get_style()),
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
            .alignment(tui::layout::Alignment::Center)
            .style(app.config.get_style()),
        area,
    );
}

fn draw_modify_columns<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = floating_rect(f, 40, 15);
    let chunks = Layout::default()
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .margin(1)
        .split(area);
    let text = Paragraph::new("Press Shift+j/k to reorder columns")
        .alignment(tui::layout::Alignment::Center);
    let mut items = Vec::new();

    for column in &app.all_info_columns {
        let list_item = ListItem::new(column.column.as_str());
        if column.show {
            items.push(
                list_item.style(
                    app.config
                        .get_style()
                        .bg(app.config.bg_column_show)
                        .fg(app.config.fg_column_show),
                ),
            );
        } else {
            items.push(
                list_item.style(
                    app.config
                        .get_style()
                        .bg(app.config.bg_column_hide)
                        .fg(app.config.fg_column_hide),
                ),
            );
        }
    }

    let list = List::new(items)
        .style(app.config.get_style())
        .highlight_style(app.config.get_highlight_style());
    let block = Block::default()
        .borders(Borders::ALL)
        .style(app.config.get_style())
        .title("Modify columns");
    let mut state = ListState::default();
    state.select(app.selected_column);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(text, chunks[0]);
    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn floating_rect<B: Backend>(f: &mut Frame<B>, width: u32, height: u32) -> Rect {
    let float_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(
                ((100_f32 - height as f32 / f.size().height as f32 * 100_f32) / 2_f32).ceil()
                    as u16,
            ),
            Constraint::Length(height as u16),
            Constraint::Percentage(
                ((100_f32 - height as f32 / f.size().height as f32 * 100_f32) / 2_f32).ceil()
                    as u16,
            ),
        ])
        .split(f.size());

    Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(
                ((100_f32 - width as f32 / f.size().width as f32 * 100_f32) / 2_f32).round() as u16,
            ),
            Constraint::Length(width as u16),
            Constraint::Percentage(
                ((100_f32 - width as f32 / f.size().width as f32 * 100_f32) / 2_f32).round() as u16,
            ),
        ])
        .split(float_layout[1])[1]
}
