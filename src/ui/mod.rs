use crate::{
    app::{FloatingWidget, RouteId},
    conversion::{convert_bytes, convert_rate, convert_secs, date, get_percentage, status_string},
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
use tui_tree_widget::{Tree, TreeItem, TreeState};
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
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let ver_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let info_transfer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(ver_chunks[1]);

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
    f.render_stateful_widget(table, ver_chunks[0], &mut state);

    let info_block = Block::default().title("Information").borders(Borders::ALL);
    let transfer_block = Block::default().title("Transfer").borders(Borders::ALL);
    let sel_torrent = &app.torrents.arguments.torrents[app.selected_torrent.unwrap()];
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
            get_percentage(sel_torrent.percent_done.unwrap()),
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
            "Created on".to_string(),
            sel_torrent.creator.as_ref().unwrap().to_string(),
        ]),
        Row::new(vec![
            "Pieces".to_string(),
            sel_torrent.piece_count.as_ref().unwrap().to_string(),
        ]),
    ];

    let transfer_rows = vec![
        Row::new(vec![
            "Time active".to_string(),
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
            "Download speed".to_string(),
            convert_rate(sel_torrent.rate_download.unwrap()),
        ]),
        Row::new(vec![
            "Download limit".to_string(),
            convert_rate(sel_torrent.download_limit.unwrap() * 1000),
        ]),
        Row::new(vec![
            "Uploaded".to_string(),
            convert_bytes(*sel_torrent.uploaded_ever.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Upload speed".to_string(),
            convert_rate(*sel_torrent.rate_upload.as_ref().unwrap()),
        ]),
        Row::new(vec![
            "Upload limit".to_string(),
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

    log::error!("{:?}", sel_torrent.seconds_seeding.as_ref().unwrap());

    let info_table = Table::new(info_rows)
        .block(info_block)
        .style(app.config.get_style())
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(info_table, info_transfer_chunks[0]);

    let transfer_table = Table::new(transfer_rows)
        .block(transfer_block)
        .style(app.config.get_style())
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(transfer_table, info_transfer_chunks[1]);

    let logs = TuiLoggerWidget::default().block(block.clone().title("Logs"));
    f.render_widget(logs, chunks[1]);
}

fn draw_torrent_info<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let tabs = Tabs::new(vec![
        Spans::from(Span::styled("Overview", Style::default())),
        Spans::from(Span::styled("Files", Style::default())),
        Spans::from(Span::styled("tab 3", Style::default())),
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
        .style(app.config.get_style())
        .block(info_block)
        .style(app.config.get_style())
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
        .style(app.config.get_style())
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

    f.render_widget(info_table, chunks[0]);
    f.render_widget(transfer_table, chunks[1]);
}

fn draw_torrent_info_files<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let _logs = TuiLoggerWidget::default();

    let tree_items = vec![
        TreeItem::new_leaf("a"),
        TreeItem::new(
            "b",
            vec![
                TreeItem::new_leaf("c"),
                TreeItem::new_leaf("d"),
                TreeItem::new_leaf("e"),
            ],
        ),
    ];

    let items = Tree::new(tree_items)
        .block(Block::default().title("tree widget"))
        .style(app.config.get_style())
        .highlight_style(app.config.get_highlight_style());
    let mut state = TreeState::default();
    f.render_stateful_widget(items, area, &mut state);

    // app.show_tree();
    // let mut rows = Vec::new();
    // let mut collapse = false;
    // let mut depth = 1;
    //
    // for entry in &app.torrent_collapse_files {
    //     if entry.collapse && !collapse {
    //         collapse = true;
    //         depth = entry.path.depth();
    //     } else if collapse {
    //         if entry.path.depth() == depth {
    //             collapse = false;
    //         }
    //     }
    //
    //     if !collapse {
    //         rows.push(Row::new(vec![entry.path.path().to_str().unwrap()]));
    //     }
    // }

    // let table = Table::new(rows)
    //     .widths(&[Constraint::Percentage(60), Constraint::Percentage(40)])
    //     .highlight_style(Style::default().bg(Color::Red));
    // let mut state = TableState::default();
    // state.select(app.selected_file);

    // f.render_stateful_widget(table, area, &mut state);
}
// fn draw_torrent_info_files<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
//     let block = Block::default().title("Files").borders(Borders::ALL);
//     let mut rows = vec![];
//     let priorities = app.torrents.arguments.torrents[app.selected_torrent.unwrap()]
//         .priorities
//         .as_ref()
//         .unwrap()
//         .iter()
//         .map(|f| f.to_string())
//         .collect::<Vec<String>>();
//
//     let mut index = 0;
//     for file in app.torrents.arguments.torrents[app.selected_torrent.unwrap()]
//         .files
//         .as_ref()
//         .unwrap()
//     {
//         rows.push(Row::new(vec![
//             file.name.as_str(),
//             priorities[index].as_str(),
//         ]));
//         index += 1;
//     }
//
//     let mut state = TableState::default();
//     state.select(app.selected_file);
//
//     let table = Table::new(rows)
//         .header(Row::new(vec!["Filename", "Priority"]))
//         .block(block)
//         .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
//         .highlight_style(Style::default().bg(Color::Red));
//     f.render_stateful_widget(table, area, &mut state);
// }

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
        let list_item = ListItem::new(column.column.to_str());
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
