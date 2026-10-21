use std::io;

use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::prelude::Backend;

use crate::interface::app::{App, AppResult};
use crate::interface::event::EventHandler;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
    active: bool,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self {
            terminal,
            events,
            active: false,
        }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> AppResult<()>
    where
        <B as Backend>::Error: 'static,
    {
        terminal::enable_raw_mode()?;
        self.active = true;
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous_hook(info);
        }));
        crossterm::execute!(
            io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        )?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::app::App::render
    pub fn draw(&mut self, app: &mut App) -> AppResult<()>
    where
        <B as Backend>::Error: 'static,
    {
        self.terminal.draw(|frame| app.render(frame))?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> AppResult<()>
    where
        <B as Backend>::Error: 'static,
    {
        terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            DisableBracketedPaste
        )?;
        self.terminal.show_cursor()?;
        self.active = false;
        Ok(())
    }
}

fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        io::stderr(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste,
        crossterm::cursor::Show
    );
}

impl<B: Backend> Drop for Tui<B> {
    fn drop(&mut self) {
        if self.active {
            restore_terminal();
        }
    }
}
