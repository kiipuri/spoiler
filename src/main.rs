mod app;
mod key_handlers;
mod ui;

use std::io::{self, Write};

use app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use key_handlers::handler;
use tui::{backend::CrosstermBackend, Terminal};
use ui::draw;

fn main() -> Result<(), io::Error> {
    setup_terminal()?;

    let mut terminal = start_terminal(io::stdout())?;
    let mut app = App {
        ..Default::default()
    };
    app.get_all_torrents();
    app.torrents
        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    loop {
        terminal.draw(|f| {
            draw(f, &mut app);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                _ => {}
            }
            handler(key.code, &mut app);
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<(), io::Error> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn start_terminal<W: Write>(buf: W) -> io::Result<Terminal<CrosstermBackend<W>>> {
    let backend = CrosstermBackend::new(buf);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    Ok(terminal)
}
