mod app;
mod conversion;
mod key_handlers;
mod ui;

use app::{get_all_torrents_loop, App};
use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use key_handlers::handler;
use log::{error, LevelFilter};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::ui::draw;

enum InputEvent {
    Input(KeyEvent),
    Tick,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tui_logger::init_logger(LevelFilter::Error).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    setup_terminal()?;
    let mut terminal = start_terminal(io::stdout())?;

    let app = Arc::new(Mutex::new(App::new().await));
    let app_ui = Arc::clone(&app);
    let app_events = Arc::clone(&app);

    tokio::spawn(async move {
        loop {
            let app = app.clone();
            get_all_torrents_loop(app).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
            if poll(Duration::from_millis(200)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if let Err(_) = tx.send(InputEvent::Input(key)).await {
                        error!("event poll errored");
                    };
                }
            } else {
                if let Err(_) = tx.send(InputEvent::Tick).await {
                    error!("event poll errored");
                }
            }
        }
    });

    loop {
        let app = app_ui.clone();
        let app = app.lock().unwrap();

        terminal.draw(|f| {
            draw(f, &app);
        })?;

        if app.should_quit {
            break;
        }

        drop(app);
        match rx.recv().await.unwrap_or(InputEvent::Tick) {
            InputEvent::Input(key) => match key.code {
                KeyCode::Char('q') => app_events.lock().unwrap().should_quit = true,
                _ => handler(key.code, &mut app_events.lock().unwrap()),
            },
            InputEvent::Tick => (),
        }
    }

    terminal.clear()?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}

fn setup_terminal() -> io::Result<()> {
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
