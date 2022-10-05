mod app;
mod config;
mod conversion;
mod io_handler;
mod key_handlers;
mod tree;
mod ui;

use crate::ui::draw;
use app::{get_all_torrents, App};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use io_handler::{Events, InputEvent};
use key_handlers::handler;
use log::LevelFilter;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> io::Result<()> {
    tui_logger::init_logger(LevelFilter::Error).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let app = Arc::new(Mutex::new(App::new().await));
    let app_ui = Arc::clone(&app);

    tokio::spawn(async move {
        loop {
            let app = app.clone();
            get_all_torrents(&app).await;
            drop(app);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    start_ui(&app_ui).await?;
    Ok(())
}

async fn start_ui(app: &Arc<Mutex<App<'static>>>) -> io::Result<()> {
    setup_terminal()?;
    let mut terminal = start_terminal(io::stdout())?;

    let tick_rate = Duration::from_millis(200);
    let mut events = Events::new(tick_rate).await;

    loop {
        let app_mutex = app.clone();
        {
            let mut app = app_mutex.lock().unwrap();
            terminal.draw(|f| {
                draw(f, &mut app);
            })?;
        }

        match events.next().await {
            InputEvent::Input(key) => {
                let mut app = app_mutex.lock().unwrap();
                handler(key, &mut app).await;
            }
            InputEvent::Tick => {}
        }

        {
            let app = app_mutex.lock().unwrap();

            if app.should_quit {
                events.close();
                break;
            }
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
