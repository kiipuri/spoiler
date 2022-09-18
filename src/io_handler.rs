use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crossterm::event::{self, poll, Event, KeyEvent};
use log::error;

pub enum InputEvent {
    Input(KeyEvent),
    Tick,
}

pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    stop_capture: Arc<AtomicBool>,
}

impl Events {
    pub async fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let event_stop_capture = stop_capture.clone();
        tokio::spawn(async move {
            loop {
                if poll(tick_rate).unwrap() {
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

                if event_stop_capture.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Events { rx, stop_capture }
    }

    pub async fn next(&mut self) -> InputEvent {
        self.rx.recv().await.unwrap()
    }

    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed)
    }
}
