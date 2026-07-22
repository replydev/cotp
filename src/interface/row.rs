use ratatui::style::Color::{Black, Yellow};
use ratatui::style::Style;
use ratatui::widgets::Cell;

pub(crate) struct Row {
    pub(crate) id: String,
    pub(crate) issuer: String,
    pub(crate) label: String,
    pub(crate) otp_code: String,
    has_error: bool,
}

impl Row {
    pub(crate) fn new(
        id: String,
        issuer: String,
        label: String,
        otp_code: String,
        has_error: bool,
    ) -> Self {
        Row {
            id,
            issuer,
            label,
            otp_code,
            has_error,
        }
    }

    fn columns(&self) -> [&String; 4] {
        [&self.id, &self.issuer, &self.label, &self.otp_code]
    }

    pub fn height(&self) -> u16 {
        (self
            .columns()
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1) as u16
    }

    pub fn cells(&self) -> Vec<Cell<'_>> {
        self.columns()
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
