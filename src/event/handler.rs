use crossterm::event::{self, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse,
    Resize,
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        std::thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(event::Event::Key(key)) => {
                            if event_tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(event::Event::Mouse(_)) => {
                            if event_tx.send(Event::Mouse).is_err() {
                                break;
                            }
                        }
                        Ok(event::Event::Resize(_, _)) => {
                            if event_tx.send(Event::Resize).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                } else if event_tx.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
