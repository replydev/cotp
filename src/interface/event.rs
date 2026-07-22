use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};

use crate::interface::app::AppResult;

/// Terminal events the dashboard reacts to. Everything else read from the
/// terminal (mouse, resize, focus, paste) is ignored at the source.
#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    // Event sender channel.
    //sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    // Event handler thread.
    //handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(tick_rate);

                if event::poll(timeout).expect("no events available") {
                    let send_result = match event::read().expect("unable to read event") {
                        // Workaround to fix double input on Windows
                        // Please check https://github.com/crossterm-rs/crossterm/issues/752
                        CrosstermEvent::Key(e) if e.kind == KeyEventKind::Press => {
                            sender.send(Event::Key(e))
                        }
                        // Mouse, resize, focus and paste events are irrelevant
                        // to the dashboard: drop them here instead of routing
                        // dead variants through the channel.
                        _ => Ok(()),
                    };
                    if send_result.is_err() {
                        // The receiver has been dropped: the dashboard has
                        // exited, so stop the event thread gracefully.
                        break;
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if sender.send(Event::Tick).is_err() {
                        // Receiver dropped, see above.
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self { receiver }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }
}
