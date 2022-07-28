use tui::{
    backend::Backend,
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
    Frame,
};

use super::app::App;

pub fn draw_overview<B: Backend>(f: &mut Frame<B>, app: &App) {
    let block = Block::default().title("A Block").borders(Borders::ALL);
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
