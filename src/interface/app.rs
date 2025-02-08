use std::error;

use crate::interface::enums::Focus;
use crate::interface::enums::Page;
use crate::interface::enums::Page::{Main, Qrcode};
use crate::otp::otp_element::OTPDatabase;
use ratatui::layout::Rect;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::interface::stateful_table::{fill_table, StatefulTable};
use crate::utils::percentage;

use super::enums::PopupAction;
use super::popup::centered_rect;

const LARGE_APPLICATION_WIDTH: u16 = 75;

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

const DEFAULT_QRCODE_LABEL: &str = "Press enter to copy the OTP URI code";

/// Application.
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    title: String,
    pub(crate) table: StatefulTable,
    pub(crate) database: &'a mut OTPDatabase,
    progress: u16,
    /// Text to print replacing the percentage
    pub(crate) label_text: String,
    pub(crate) print_percentage: bool,
    pub(crate) current_page: Page,
    pub(crate) search_query: String,
    pub(crate) focus: Focus,
    pub(crate) popup: Popup,

    /// Info text in the `QRCode` page
    pub(crate) qr_code_page_label: &'static str,
}

pub struct Popup {
    pub(crate) text: String,
    pub(crate) action: PopupAction,
    pub(crate) percent_x: u16,
    pub(crate) percent_y: u16,
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new(database: &'a mut OTPDatabase) -> Self {
        let mut title = String::from(env!("CARGO_PKG_NAME"));
        title.push_str(" v");
        // Settings cotp version from env var defined in build.rs
        title.push_str(env!("COTP_VERSION"));
        Self {
            running: true,
            title,
            table: StatefulTable::new(database.elements_ref()),
            database,
            progress: percentage(),
            label_text: String::new(),
            print_percentage: true,
            current_page: Page::default(),
            search_query: String::new(),
            focus: Focus::MainPage,
            popup: Popup {
                text: String::new(),
                action: PopupAction::EditOtp,
                percent_x: 60,
                percent_y: 20,
            },
            qr_code_page_label: DEFAULT_QRCODE_LABEL,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.current_page = Page::default();
        self.print_percentage = true;
        self.qr_code_page_label = DEFAULT_QRCODE_LABEL;
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self, force_update: bool) {
        // Update progress bar
        let new_progress = percentage();
        // Check for new cycle
        if force_update || new_progress < self.progress {
            // Update codes
            self.table.items.clear();
            fill_table(&mut self.table, self.database.elements_ref());
        }
        self.progress = new_progress;
    }

    /// Renders the user interface widgets.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        match &self.current_page {
            Main => self.render_main_page(frame),
            Qrcode => self.render_qrcode_page(frame),
        }
    }

    fn render_qrcode_page(&self, frame: &mut Frame<'_>) {
        let paragraph = self
            .table
            .state
            .selected()
            .and_then(|index| self.database.elements_ref().get(index))
            .map_or_else(
                || {
                    Paragraph::new("No element is selected")
                        .block(Block::default().title("Nope").borders(Borders::ALL))
                        .style(Style::default().fg(Color::White).bg(Color::Reset))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: true })
                },
                |element| {
                    let title = if element.label.is_empty() {
                        element.issuer.clone()
                    } else {
                        format!("{} - {}", &element.issuer, &element.label)
                    };
                    Paragraph::new(format!(
                        "{}\n{}",
                        element.get_qrcode(),
                        self.qr_code_page_label
                    ))
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Reset))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                },
            );
        Self::render_paragraph(frame, paragraph);
    }

    fn render_paragraph(frame: &mut Frame<'_>, paragraph: Paragraph) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(frame.area());

        frame.render_widget(paragraph, rects[0]);
    }

    fn render_main_page(&mut self, frame: &mut Frame<'_>) {
        let height = frame.area().height;
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),          // Search bar
                    Constraint::Length(height - 8), // Table + Info Box
                    Constraint::Length(1),          // Progress bar
                ]
                .as_ref(),
            )
            .margin(2)
            .split(frame.area());

        let search_bar_title = "Press CTRL + F to search a code...";
        let search_bar = Paragraph::new(&*self.search_query)
            .block(
                Block::default()
                    .title(search_bar_title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if self.focus == Focus::SearchBar {
                        Color::LightRed
                    } else {
                        Color::White
                    })),
            )
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let progress_label = if self.print_percentage {
            format!("{}%", self.progress)
        } else {
            self.label_text.clone()
        };
        let progress_bar = Gauge::default()
            .block(Block::default())
            .gauge_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(self.progress)
            .label(progress_label);

        frame.render_widget(search_bar, rects[0]);
        self.render_table_box(frame, rects[1]);
        frame.render_widget(progress_bar, rects[2]);
        if self.focus == Focus::Popup {
            self.render_alert(frame);
        }
    }

    fn render_alert(&mut self, frame: &mut Frame<'_>) {
        let block = Block::default().title("Alert").borders(Borders::ALL);
        let paragraph = Paragraph::new(&*self.popup.text)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        let area = centered_rect(self.popup.percent_x, self.popup.percent_y, frame.area());
        frame.render_widget(Clear, area);
        //this clears out the background
        frame.render_widget(paragraph, area);
    }

    fn render_table_box(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let constraints = if Self::is_large_application(frame) {
            vec![Constraint::Percentage(80), Constraint::Percentage(20)]
        } else {
            vec![Constraint::Percentage(100)]
        };
        let chunks = Layout::default()
            .constraints(constraints)
            .direction(Direction::Horizontal)
            .split(area);

        let header_cells = ["Id", "Issuer", "Label", "OTP"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
        let header = Row::new(header_cells)
            .style(
                Style::default()
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .height(1)
            .bottom_margin(1);
        let rows = self.table.items.iter().map(|item| {
            Row::new(item.cells())
                .height(item.height())
                .bottom_margin(1)
        });

        const TABLE_WIDTHS: &[Constraint] = &[
            Constraint::Percentage(5),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(25),
        ];

        let t = Table::new(rows, TABLE_WIDTHS)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .title(self.title.as_str()),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("-> ");

        let selected_element = self
            .table
            .state
            .selected()
            .and_then(|i| self.database.get_element(i));

        let mut text = if let Some(element) = selected_element {
            format!(
                "
            Type: {}
            Algorithm: {}
            Counter: {}
            Pin: {}
            ",
                element.type_,
                element.algorithm,
                element
                    .counter
                    .map_or_else(|| String::from("N/A"), |e| e.to_string()),
                element.pin.clone().unwrap_or_else(|| String::from("N/A"))
            )
        } else {
            String::new()
        };

        text.push_str("\n\n         Press '?' to get help\n");
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Code info").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Reset))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_stateful_widget(t, chunks[0], &mut self.table.state);
        if Self::is_large_application(frame) {
            frame.render_widget(paragraph, chunks[1]);
        }
    }

    fn is_large_application(frame: &mut Frame<'_>) -> bool {
        frame.area().width >= LARGE_APPLICATION_WIDTH
    }
}
