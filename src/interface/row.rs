use tui::style::Color::{Black, Yellow};
use tui::style::Style;
use tui::widgets::Cell;

pub(crate) struct Row {
    pub(crate) values: Vec<String>,
    has_error: bool,
}

impl Row {
    pub(crate) fn new(values: Vec<String>, has_error: bool) -> Self {
        Row { values, has_error }
    }
    pub fn height(&self) -> u16 {
        (self
            .values
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1) as u16
    }

    pub fn cells(&self) -> Vec<Cell> {
        self.values
            .iter()
            .map(|c| {
                let style = if self.has_error {
                    Style::default().bg(Yellow).fg(Black)
                } else {
                    Style::default()
                };
                Cell::from(c.as_str()).style(style)
            })
            .collect()
    }
}
