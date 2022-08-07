use std::time::Duration;

use crossterm::event::{self, poll, Event, KeyEvent};
use log::error;

pub enum InputEvent {
    Input(KeyEvent),
    Tick,
}

pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    // _tx: tokio::sync::mpsc::Sender<InputEvent>,
}

impl Events {
    pub async fn new(tick_rate: Duration) -> Events {
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        let event_tx = _tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1)).await;
                if poll(tick_rate).unwrap() {
                    if let Event::Key(key) = event::read().unwrap() {
                        if let Err(_) = event_tx.send(InputEvent::Input(key)).await {
                            error!("event poll errored");
                        };
                    }
                } else {
                    if let Err(_) = event_tx.send(InputEvent::Tick).await {
                        error!("event poll errored");
                    }
                }
            }
        });

        Events { rx }
    }

    pub async fn next(&mut self) -> InputEvent {
        self.rx.recv().await.unwrap()
    }
}
