use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Cell, Clear, HighlightSpacing, LineGauge, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, Wrap,
    },
};

use super::{
    app::App,
    enums::{Action, Focus, Page, PopupAction},
    popup::centered_rect,
    row::{display_code, remaining},
    theme::*,
};

impl App<'_> {
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        self.hitboxes.clear();
        frame.render_widget(
            ratatui::widgets::Block::default().style(Style::default().bg(BACKGROUND).fg(TEXT)),
            frame.area(),
        );
        if self.sealed.is_some() {
            let error = self
                .notification
                .as_ref()
                .map(|(message, _, _)| message.as_str())
                .unwrap_or("");
            frame.render_widget(Paragraph::new(format!("cotp · Locked\n\nEnter your vault password and press Enter to unlock.\nPending edits are encrypted in memory.\n\nPassword: {}\n{}", "•".repeat(self.unlock_password.chars().count()), error)).wrap(Wrap { trim: true }), frame.area().inner(Margin::new(1, 1)));
            return;
        }
        if frame.area().width < 36 || frame.area().height < 12 {
            frame.render_widget(
                Paragraph::new("cotp\n\nEnlarge the terminal to 36 × 12.\nq quit")
                    .style(Style::default().fg(ACCENT))
                    .wrap(Wrap { trim: true }),
                frame.area(),
            );
            return;
        }
        let area = frame.area().inner(Margin::new(1, 1));
        let [header, search, content, status, shortcuts] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);
        self.render_header(frame, header);
        self.render_search(frame, search);
        if self.current_page == Page::Qrcode {
            self.render_qrcode(frame, content);
        } else if content.width >= 100 {
            let [accounts, detail] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(34)])
                    .spacing(1)
                    .areas(content);
            self.render_accounts(frame, accounts);
            self.render_detail(frame, detail);
        } else {
            self.render_accounts(frame, content);
        }
        self.render_status(frame, status);
        let hints = if self.current_page == Page::Qrcode && area.width < 65 {
            vec![("↵", "URI"), ("Esc", "back"), ("q", "quit")]
        } else if self.current_page == Page::Qrcode {
            vec![
                ("Enter", "copy URI"),
                ("Esc / Space", "back"),
                ("q", "quit"),
            ]
        } else if self.focus == Focus::SearchBar {
            vec![
                ("↑↓", "select"),
                ("Enter", "copy"),
                ("Esc", "done"),
                ("Ctrl+U", "clear"),
            ]
        } else if area.width < 65 {
            vec![
                ("↑", ""),
                ("↓", ""),
                ("↵", "copy"),
                ("/", "find"),
                ("?", ""),
            ]
        } else {
            vec![
                ("↑↓", "move"),
                ("Enter", "copy"),
                ("/", "search"),
                ("Space", "QR"),
                ("h", "hide"),
                ("Ctrl+L", "lock"),
                ("?", "help"),
                ("q", "quit"),
            ]
        };
        self.render_actions(frame, shortcuts, &hints, false);
        if self.focus == Focus::Popup {
            self.render_popup(frame);
        } else if self.focus == Focus::Form {
            self.render_form(frame);
        }
    }

    pub(crate) fn render_actions(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        hints: &[(&str, &str)],
        commands: bool,
    ) {
        let mut x = area.x;
        let mut y = area.y;
        for (key, label) in hints {
            let line = shortcut_line(&[(*key, *label)]);
            let width = line.width() as u16;
            if x + width > area.right() {
                x = area.x;
                y += 1;
            }
            if y >= area.bottom() {
                break;
            }
            let rect = Rect::new(x, y, width.min(area.width), 1);
            frame.render_widget(Paragraph::new(line), rect);
            let action = match *key {
                "Ctrl+U" => Some(Action::ClearSearch),
                "Ctrl+L" => None,
                "↑" => Some(Action::Key(KeyCode::Up)),
                "↓" | "↑↓" => Some(Action::Key(KeyCode::Down)),
                "Enter" | "↵" => Some(Action::Key(KeyCode::Enter)),
                "Esc" | "Esc / Enter" | "n / Esc" | "Esc / Space" => {
                    Some(Action::Key(KeyCode::Esc))
                }
                "Space" => Some(Action::Key(KeyCode::Char(' '))),
                key if key.len() == 1 => key.chars().next().map(|c| {
                    if commands {
                        Action::Command(KeyCode::Char(c))
                    } else {
                        Action::Key(KeyCode::Char(c))
                    }
                }),
                _ => None,
            };
            if let Some(action) = action {
                self.hitboxes.push((rect, action));
            }
            x += width;
        }
    }

    fn render_header(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let count = self.database.elements_ref().len();
        let mut summary_text = vec![Span::styled(
            format!(
                "{count} {}",
                if count == 1 { "account" } else { "accounts" }
            ),
            Style::default().fg(MUTED),
        )];
        if self.database.is_modified() {
            summary_text.push(Span::styled("  ● unsaved", Style::default().fg(WARNING)));
        }
        let summary_text = Line::from(summary_text);
        let summary_width = summary_text
            .width()
            .min(area.width.saturating_sub(9) as usize) as u16;
        let [brand, summary] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(summary_width)])
                .areas(area);
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  cotp  ",
                Style::default().fg(BACKGROUND).bg(ACCENT).bold(),
            )),
            brand,
        );
        frame.render_widget(
            Paragraph::new(summary_text).alignment(Alignment::Right),
            summary,
        );
        if area.height > 1 {
            let toolbar = Rect::new(area.x, area.y + 1, area.width, 1);
            self.render_actions(
                frame,
                toolbar,
                &[("a", "Add"), ("e", "Edit"), ("d", "Delete")],
                true,
            );
        }
    }

    fn render_search(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hitboxes.push((area, Action::Search));
        let active = self.focus == Focus::SearchBar;
        let block = panel(if active {
            "Search · Esc to finish"
        } else {
            "Search · / or Ctrl+F"
        })
        .border_style(Style::default().fg(if active { ACCENT } else { BORDER }));
        let inner = block.inner(area);
        // Display the end of long queries; width() accounts for wide Unicode characters.
        let mut query = self.search_query.as_str();
        let available = inner.width.saturating_sub(2) as usize;
        while Line::from(query).width() > available {
            query = &query[query.chars().next().map_or(0, char::len_utf8)..];
        }
        let text = if self.search_query.is_empty() && !active {
            Span::styled(" Find an issuer or account…", Style::default().fg(MUTED))
        } else {
            Span::styled(format!(" {query}"), Style::default().fg(TEXT))
        };
        frame.render_widget(Paragraph::new(text).block(block), area);
        if active && inner.width > 1 {
            frame.set_cursor_position((inner.x + 1 + Line::from(query).width() as u16, inner.y));
        }
    }

    fn render_accounts(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let count = self.table.items.len();
        let title = if self.search_query.is_empty() {
            "Accounts".to_string()
        } else {
            format!("Accounts · {count} matches")
        };
        let position = self.table.state.selected().map_or(0, |i| i + 1);
        let block = panel(title)
            .title_bottom(Line::from(format!(" {position} / {count} ")).right_aligned());
        if count == 0 {
            let text = if self.database.elements_ref().is_empty() {
                vec![
                    Line::from(""),
                    Line::from("Your vault is ready").style(Style::default().fg(ACCENT).bold()),
                    Line::from(""),
                    Line::from("Press a or click Add to create your first account."),
                    Line::from("Paste a secret key or an otpauth:// URI into the form."),
                    Line::from(""),
                    Line::from("Already use an authenticator? Try cotp import --help."),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from("No matching accounts").style(Style::default().fg(ACCENT).bold()),
                    Line::from(""),
                    Line::from("Try another issuer or account name."),
                    Line::from("Esc, then Esc again to clear the filter."),
                ]
            };
            frame.render_widget(
                Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }
        let compact = area.height < 8;
        let row_height = if compact { 1 } else { 2 };
        let row_margin = if compact { 0 } else { 1 };
        let header_height = if compact { 0 } else { 2 };
        let rows = self.table.items.iter().enumerate().map(|(index, item)| {
            let element = &self.database.elements_ref()[item.database_index];
            let issuer = if item.issuer.is_empty() {
                "Personal account"
            } else {
                &item.issuer
            };
            let account = Text::from(vec![
                Line::from(Span::styled(issuer, Style::default().fg(TEXT).bold())),
                Line::from(Span::styled(
                    item.label.as_str(),
                    Style::default().fg(MUTED),
                )),
            ]);
            let code = if item.has_error {
                "Unavailable".into()
            } else {
                display_code(&item.code, self.hide_codes)
            };
            let timing = remaining(element, self.now);
            let code_color = if item.has_error {
                DANGER
            } else if timing.is_some_and(|(left, _)| left <= 5) {
                WARNING
            } else {
                ACCENT
            };
            let cells = vec![
                Cell::from(account),
                Cell::from(code).style(Style::default().fg(code_color).bold()),
            ];
            Row::new(cells)
                .height(row_height)
                .bottom_margin(row_margin)
                .style(Style::default().bg(if index % 2 == 0 { SURFACE } else { BACKGROUND }))
        });
        let widths = [Constraint::Fill(1), Constraint::Length(13)];
        let headers = ["ACCOUNT", "CODE"];
        let mut table = Table::new(rows, widths)
            .block(block)
            .column_spacing(2)
            .row_highlight_style(Style::default().bg(SELECTED))
            .highlight_symbol(Span::styled("▎ ", Style::default().fg(ACCENT)))
            .highlight_spacing(HighlightSpacing::Always);
        if !compact {
            table = table.header(
                Row::new(headers)
                    .style(Style::default().fg(MUTED))
                    .bottom_margin(1),
            );
        }
        frame.render_stateful_widget(table, area, &mut self.table.state);
        let viewport = area
            .height
            .saturating_sub(2 + header_height)
            .saturating_add(row_margin) as usize
            / (row_height + row_margin) as usize;
        for (visible, index) in (self.table.state.offset()..count)
            .take(viewport)
            .enumerate()
        {
            self.hitboxes.push((
                Rect::new(
                    area.x + 1,
                    area.y + 1 + header_height + visible as u16 * (row_height + row_margin),
                    area.width.saturating_sub(2),
                    row_height,
                ),
                Action::Select(index),
            ));
        }
        if count > viewport {
            let mut state = ScrollbarState::new(count.saturating_sub(viewport))
                .position(self.table.state.offset());
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_style(Style::default().fg(ACCENT))
                    .track_style(Style::default().fg(BORDER)),
                area.inner(Margin::new(0, 1)),
                &mut state,
            );
        }
    }

    fn render_detail(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = panel("Selected account");
        let inner = block.inner(area).inner(Margin::new(1, 0));
        frame.render_widget(block, area);
        let Some(item) = self
            .table
            .state
            .selected()
            .and_then(|index| self.table.items.get(index))
        else {
            frame.render_widget(
                Paragraph::new("Select an account to see its details.")
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(MUTED)),
                inner,
            );
            return;
        };
        let element = &self.database.elements_ref()[item.database_index];
        let [identity, code_area, gauge_area, metadata] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .areas(inner);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(if item.issuer.is_empty() {
                    "Personal account"
                } else {
                    &item.issuer
                })
                .style(Style::default().fg(TEXT).bold()),
                Line::from(item.label.as_str()).style(Style::default().fg(MUTED)),
            ]),
            identity,
        );
        if item.has_error {
            frame.render_widget(
                Paragraph::new(item.code.as_str())
                    .style(Style::default().fg(DANGER))
                    .wrap(Wrap { trim: true }),
                code_area,
            );
        } else {
            frame.render_widget(
                Paragraph::new(large_code(&item.code, self.hide_codes, code_area.width))
                    .style(Style::default().fg(ACCENT).bold())
                    .alignment(Alignment::Center),
                code_area,
            );
        }
        if let Some((left, period)) = remaining(element, self.now) {
            frame.render_widget(
                LineGauge::default()
                    .ratio(left as f64 / period as f64)
                    .filled_symbol("━")
                    .unfilled_symbol("─")
                    .filled_style(Style::default().fg(if left <= 5 { WARNING } else { ACCENT }))
                    .unfilled_style(Style::default().fg(BORDER))
                    .label(Span::styled(
                        format!("{left}s "),
                        Style::default().fg(MUTED),
                    )),
                gauge_area,
            );
        } else {
            frame.render_widget(
                Paragraph::new("Counter based · + / − to change").style(Style::default().fg(MUTED)),
                gauge_area,
            );
        }
        let mut lines = vec![
            detail_line("Type", element.type_.to_string()),
            detail_line("Algorithm", element.algorithm.to_string()),
            detail_line("Digits", element.digits.to_string()),
        ];
        if let Some((_, period)) = remaining(element, self.now) {
            lines.push(detail_line("Period", format!("{period} seconds")));
        }
        if let Some(counter) = element.counter {
            lines.push(detail_line("Counter", counter.to_string()));
        }
        lines.extend([
            Line::from(""),
            shortcut_line(&[("Enter", "copy code")]),
            shortcut_line(&[("Space", "show QR code")]),
        ]);
        frame.render_widget(Paragraph::new(lines), metadata);
    }

    fn render_status(&self, frame: &mut Frame<'_>, area: Rect) {
        let (text, color) = if let Some((text, error, _)) = &self.notification {
            (text.clone(), if *error { DANGER } else { ACCENT })
        } else if self.hide_codes {
            ("Codes hidden · press h to reveal".into(), WARNING)
        } else if self.current_page == Page::Qrcode {
            ("This QR code contains the account secret.".into(), WARNING)
        } else if self.database.is_modified() {
            (
                "Unsaved changes · you will be asked to save when you quit".into(),
                WARNING,
            )
        } else if area.width < 100
            && let Some((left, _)) = self
                .table
                .selected_index()
                .and_then(|index| self.database.get_element(index))
                .and_then(|element| remaining(element, self.now))
        {
            (
                format!("Selected code · {left}s remaining"),
                if left <= 5 { WARNING } else { MUTED },
            )
        } else {
            ("Ready · codes refresh automatically".into(), MUTED)
        };
        frame.render_widget(Paragraph::new(text).style(Style::default().fg(color)), area);
    }

    fn render_qrcode(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = panel("Transfer account · QR code");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let Some(element) = self
            .table
            .selected_index()
            .and_then(|i| self.database.get_element(i))
        else {
            return;
        };
        let qr = element.get_qrcode();
        let width = qr
            .lines()
            .map(|line| Line::from(line).width())
            .max()
            .unwrap_or(0) as u16;
        let height = qr.lines().count() as u16;
        if inner.width < width || inner.height < height {
            frame.render_widget(Paragraph::new(format!("Enlarge the terminal to display the QR code ({width} × {height} cells).\n\nEnter copies the OTP URI.\nEsc returns to accounts."))
                .alignment(Alignment::Center).wrap(Wrap { trim: true }).style(Style::default().fg(MUTED)), inner);
        } else {
            frame.render_widget(
                Paragraph::new(qr).style(Style::default().fg(Color::White).bg(Color::Black)),
                centered_rect(width, height, inner),
            );
        }
    }

    fn render_help_body(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let actions = [
            ("↑ / k", "Previous account", KeyCode::Up),
            ("↓ / j", "Next account", KeyCode::Down),
            ("Home", "First account", KeyCode::Home),
            ("End", "Last account", KeyCode::End),
            ("PgUp / PgDn", "Move ten accounts", KeyCode::PageDown),
            ("a", "Add account", KeyCode::Char('a')),
            ("e", "Edit account", KeyCode::Char('e')),
            ("Enter", "Copy code / URI", KeyCode::Enter),
            ("/ or Ctrl+F", "Filter accounts", KeyCode::Char('/')),
            ("h", "Hide / reveal codes", KeyCode::Char('h')),
            ("Space", "Show / close QR", KeyCode::Char(' ')),
            ("+", "Increase HOTP counter", KeyCode::Char('+')),
            ("−", "Decrease HOTP counter", KeyCode::Char('-')),
            ("d / Delete", "Delete account", KeyCode::Char('d')),
            ("q / Ctrl+D", "Quit", KeyCode::Char('q')),
        ];
        for (row, (key, label, code)) in actions
            .iter()
            .skip(self.popup_scroll as usize)
            .take(area.height as usize)
            .enumerate()
        {
            let rect = Rect::new(area.x, area.y + row as u16, area.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!("{key:<12}"), Style::default().fg(ACCENT)),
                    Span::styled(*label, Style::default().fg(TEXT)),
                ])),
                rect,
            );
            self.hitboxes.push((rect, Action::Command(*code)));
        }
    }

    fn render_popup(&mut self, frame: &mut Frame<'_>) {
        self.hitboxes.clear();
        let (title, color, height, hints) = match self.popup.action {
            PopupAction::GeneralInfo => (
                "Shortcuts · click an action",
                ACCENT,
                25,
                vec![("↑↓", "scroll"), ("Esc / Enter", "close")],
            ),
            PopupAction::DeleteOtp => (
                "Delete account?",
                DANGER,
                11,
                vec![("y", "delete"), ("n / Esc", "cancel")],
            ),
            PopupAction::SaveBeforeQuit => (
                "Save your changes?",
                WARNING,
                10,
                vec![("y", "save & quit"), ("n", "discard"), ("Esc", "cancel")],
            ),
        };
        // Dim the dashboard while retaining its spatial context.
        let screen = frame.area();
        frame.buffer_mut().set_style(
            screen,
            Style::default().fg(MUTED).add_modifier(Modifier::DIM),
        );
        let area = centered_rect(62, height, frame.area());
        frame.render_widget(Clear, area);
        let block = panel(title).border_style(Style::default().fg(color));
        let inner = block.inner(area).inner(Margin::new(2, 1));
        frame.render_widget(block, area);
        let [body, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(inner);
        if self.popup.action == PopupAction::GeneralInfo {
            self.render_help_body(frame, body);
        } else {
            frame.render_widget(
                Paragraph::new(self.popup.text.as_str()).wrap(Wrap { trim: false }),
                body,
            );
        }
        self.render_actions(frame, footer, &hints, false);
    }
}

fn detail_line(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<12}"), Style::default().fg(MUTED)),
        Span::styled(value, Style::default().fg(TEXT)),
    ])
}

fn shortcut_line(items: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (key, action) in items {
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default().fg(ACCENT).bold(),
        ));
        spans.push(Span::styled(
            format!("{action}  "),
            Style::default().fg(MUTED),
        ));
    }
    Line::from(spans)
}

fn large_code(code: &str, hidden: bool, width: u16) -> Text<'static> {
    if hidden
        || !code.bytes().all(|byte| byte.is_ascii_digit())
        || code.len() * 4 + 1 > width as usize
    {
        return Text::from(format!("\n{}", display_code(code, hidden)));
    }
    const DIGITS: [[&str; 3]; 10] = [
        ["█▀█", "█ █", "▀▀▀"],
        ["▀█ ", " █ ", "▀▀▀"],
        ["▀▀█", "█▀▀", "▀▀▀"],
        ["▀▀█", " ▀█", "▀▀▀"],
        ["█ █", "▀▀█", "  ▀"],
        ["█▀▀", "▀▀█", "▀▀▀"],
        ["█▀▀", "█▀█", "▀▀▀"],
        ["▀▀█", "  █", "  ▀"],
        ["█▀█", "█▀█", "▀▀▀"],
        ["█▀█", "▀▀█", "▀▀▀"],
    ];
    Text::from(
        (0..3)
            .map(|row| {
                let mut line = String::new();
                for (i, byte) in code.bytes().enumerate() {
                    if i == code.len() / 2 {
                        line.push(' ');
                    }
                    line.push_str(DIGITS[(byte - b'0') as usize][row]);
                    line.push(' ');
                }
                Line::from(line)
            })
            .collect::<Vec<_>>(),
    )
}
