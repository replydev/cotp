use std::error;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Cell, Gauge, Row, Table};

use crate::otp::otp_element::OTPElement;
use crate::interface::table::{fill_table, StatefulTable};
use crate::utils::percentage;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    title: String,
    pub(crate) table: StatefulTable,
    pub(crate) elements: Vec<OTPElement>,
    progress: u16,
    pub(crate) label_text: String,
    pub(crate) print_percentage: bool,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(elements: Vec<OTPElement>) -> Self {
        let mut title = String::from(env!("CARGO_PKG_NAME"));
        title.push_str(" v");
        title.push_str(env!("CARGO_PKG_VERSION"));
        Self { running: true, title, table: StatefulTable::new(&elements), elements, progress: percentage(),label_text: String::from(""), print_percentage: true, }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        // Update progress bar
        let new_progress = percentage();
        // Check for new cycle
        if new_progress < self.progress {
            // Update codes
            self.table.items.clear();
            fill_table(&mut self.table, &self.elements);
        }
        self.progress = new_progress;
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
            .margin(2)
            .split(frame.size());
        let header_cells = ["Id", "Issuer", "Label", "OTP"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
        let header = Row::new(header_cells)
            .style(
                Style::default()
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            )
            .height(1)
            .bottom_margin(1);
        let rows = self.table.items.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(height as u16).bottom_margin(1)
        });
        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::TOP | Borders::BOTTOM).title(self.title.as_str()))
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD))
            .highlight_symbol("-> ")
            .widths(&[
                Constraint::Percentage(5),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
            ]);

        let progress_label = if self.print_percentage {
            format!("{}%", self.progress)
        }
        else{
            self.label_text.to_owned()
        };
        let progress_bar = Gauge::default()
            .block(Block::default())
            .gauge_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(self.progress as u16)
            .label(progress_label);

        frame.render_stateful_widget(t, rects[0], &mut self.table.state);
        frame.render_widget(progress_bar, rects[1]);
    }
}