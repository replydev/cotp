use super::{
    app::App,
    enums::Action,
    form::{AccountForm, CANCEL, LABELS, SAVE},
    popup::centered_rect,
    theme::*,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

impl App<'_> {
    pub(crate) fn render_form(&mut self, frame: &mut Frame<'_>) {
        self.hitboxes.clear();
        let Some(form) = self.form.as_ref() else {
            return;
        };
        let screen = frame.area();
        frame.buffer_mut().set_style(
            screen,
            Style::default().fg(MUTED).add_modifier(Modifier::DIM),
        );
        let height = (form.fields().len() as u16 * 3 + 9).min(34);
        let area = centered_rect(78, height, screen);
        frame.render_widget(Clear, area);
        let block = panel(if form.editing.is_some() {
            "Edit account"
        } else {
            "Add account"
        })
        .border_style(Style::default().fg(ACCENT));
        let inner = block.inner(area).inner(Margin::new(1, 0));
        frame.render_widget(block, area);
        let [intro, fields_area, hint, message, buttons, shortcuts] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        frame.render_widget(
            Paragraph::new("Account details\nSave changes to your vault when you quit.")
                .style(Style::default().fg(MUTED)),
            intro,
        );
        let fields = form.fields();
        let visible = (fields_area.height / 3).max(1) as usize;
        let active_position = fields
            .iter()
            .position(|index| *index == form.active)
            .unwrap_or(fields.len().saturating_sub(1));
        let offset = active_position.saturating_sub(visible - 1);
        for (row, index) in fields.iter().skip(offset).take(visible).enumerate() {
            let index = *index;
            let rect = Rect::new(
                fields_area.x,
                fields_area.y + row as u16 * 3,
                fields_area.width,
                3.min(fields_area.height),
            );
            if rect.bottom() > fields_area.bottom() {
                break;
            }
            let active = form.active == index;
            let input = form.display_input(index);
            let block = panel(format!(
                "{}{}",
                LABELS[index],
                if index == 2 || index == 8 {
                    " · masked"
                } else {
                    ""
                }
            ))
            .border_style(Style::default().fg(if active { ACCENT } else { BORDER }));
            let input_area = block.inner(rect);
            let width = input_area.width.saturating_sub(2) as usize;
            let scroll = input.visual_scroll(width);
            let value = if AccountForm::is_choice(index) {
                format!("‹ {} ›", input.value())
            } else if input.value().is_empty()
                && form.editing.is_some()
                && (index == 2 || index == 8)
            {
                "Unchanged".into()
            } else {
                input.value().to_owned()
            };
            frame.render_widget(
                Paragraph::new(value)
                    .style(Style::default().fg(if active { TEXT } else { MUTED }))
                    .scroll((0, scroll.min(u16::MAX as usize) as u16))
                    .block(block),
                rect,
            );
            self.hitboxes.push((rect, Action::Field(index)));
            if active && !AccountForm::is_choice(index) && input_area.width > 0 {
                frame.set_cursor_position((
                    input_area.x + input.visual_cursor().saturating_sub(scroll).min(width) as u16,
                    input_area.y,
                ));
            }
        }
        frame.render_widget(
            Paragraph::new(form.hint(form.active)).style(Style::default().fg(MUTED)),
            hint,
        );
        let message_text = form
            .error
            .as_deref()
            .unwrap_or("Tab moves between fields and buttons. Enter activates a button.");
        frame.render_widget(
            Paragraph::new(message_text)
                .style(Style::default().fg(if form.error.is_some() { DANGER } else { MUTED }))
                .wrap(Wrap { trim: true }),
            message,
        );
        let [save, cancel, _] = Layout::horizontal([
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(buttons);
        for (rect, label, index, action) in [
            (save, "Save account", SAVE, Action::Save),
            (cancel, "Cancel", CANCEL, Action::Cancel),
        ] {
            let active = form.active == index;
            frame.render_widget(
                Paragraph::new(
                    Line::from(Span::styled(
                        format!("[ {label} ]"),
                        Style::default()
                            .fg(if active { BACKGROUND } else { ACCENT })
                            .bg(if active { ACCENT } else { SELECTED })
                            .bold(),
                    ))
                    .centered(),
                ),
                rect,
            );
            self.hitboxes.push((rect, action));
        }
        frame.render_widget(
            Paragraph::new("Ctrl+S save · Esc cancel · Tab / Shift+Tab move")
                .style(Style::default().fg(MUTED)),
            shortcuts,
        );
    }
}
