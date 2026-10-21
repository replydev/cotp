//! Shared palette and component styling for the terminal dashboard.
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType},
};

pub const BACKGROUND: Color = Color::Rgb(13, 19, 29);
pub const SURFACE: Color = Color::Rgb(20, 29, 42);
pub const SELECTED: Color = Color::Rgb(28, 57, 67);
pub const BORDER: Color = Color::Rgb(48, 65, 83);
pub const TEXT: Color = Color::Rgb(224, 233, 242);
pub const MUTED: Color = Color::Rgb(144, 162, 181);
pub const ACCENT: Color = Color::Rgb(94, 234, 212);
pub const WARNING: Color = Color::Rgb(251, 191, 91);
pub const DANGER: Color = Color::Rgb(251, 113, 133);

pub fn panel(title: impl Into<String>) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().fg(TEXT).bg(SURFACE))
        .title(Line::styled(
            format!(" {} ", title.into()),
            Style::default().fg(MUTED),
        ))
}
