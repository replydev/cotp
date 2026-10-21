use crate::interface::app::AppResult;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize,
    Paste(String),
}

/// Poll on the UI thread so terminal errors propagate and no reader outlives the TUI.
#[derive(Debug)]
pub struct EventHandler {
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate),
            last_tick: Instant::now(),
        }
    }

    pub fn next(&mut self) -> AppResult<Event> {
        loop {
            let timeout = self.tick_rate.saturating_sub(self.last_tick.elapsed());
            if timeout.is_zero() {
                self.last_tick = Instant::now();
                return Ok(Event::Tick);
            }
            if event::poll(timeout)? {
                match event::read()? {
                    CrosstermEvent::Key(key) if key.kind != KeyEventKind::Release => {
                        return Ok(Event::Key(key));
                    }
                    CrosstermEvent::Mouse(mouse) => return Ok(Event::Mouse(mouse)),
                    CrosstermEvent::Paste(text) => return Ok(Event::Paste(text)),
                    CrosstermEvent::Resize(_, _) => return Ok(Event::Resize),
                    _ => {}
                }
            }
        }
    }
}
