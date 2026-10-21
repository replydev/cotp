use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub(crate) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height.min(area.height))])
        .flex(Flex::Center)
        .areas(area);
    Layout::horizontal([Constraint::Length(width.min(area.width))])
        .flex(Flex::Center)
        .areas::<1>(area)[0]
}
