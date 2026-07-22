use std::error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::interface::enums::Focus;
use crate::interface::enums::Page;
use crate::interface::enums::Page::{Main, Qrcode};
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Wrap};

use crate::interface::stateful_table::{StatefulTable, fill_table};
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
    /// Time step of each element at the last tick, used to detect when an
    /// element crosses its own period boundary and its code must be renewed
    last_steps: Vec<u64>,
    /// Text to print replacing the percentage
    pub(crate) label_text: String,
    pub(crate) print_percentage: bool,
    pub(crate) current_page: Page,
    pub(crate) search_query: String,
    pub(crate) focus: Focus,
    pub(crate) popup: Popup,

    /// Info text in the `QRCode` page
    pub(crate) qr_code_page_label: &'static str,

    /// Cached rendered QR code for the `QRCode` page, keyed by the index of
    /// the element it was generated from
    qrcode_cache: Option<(usize, String)>,
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
            last_steps: element_steps(database.elements_ref()),
            database,
            label_text: String::new(),
            print_percentage: true,
            current_page: Page::default(),
            search_query: String::new(),
            focus: Focus::MainPage,
            popup: Popup {
                text: String::new(),
                action: PopupAction::default(),
                percent_x: 60,
                percent_y: 20,
            },
            qr_code_page_label: DEFAULT_QRCODE_LABEL,
            qrcode_cache: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.current_page = Page::default();
        self.print_percentage = true;
        self.qr_code_page_label = DEFAULT_QRCODE_LABEL;
        self.qrcode_cache = None;
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self, force_update: bool) {
        let steps = element_steps(self.database.elements_ref());
        // Regenerate the codes when any element crossed its own period
        // boundary, so elements with a period != 30 seconds (e.g. 60s TOTP,
        // 10s MOTP) are refreshed on time too
        if force_update || steps != self.last_steps {
            // Update codes
            self.table.items.clear();
            fill_table(&mut self.table, self.database.elements_ref());
            // Elements may have changed (e.g. HOTP counter increment or
            // deletion), so the cached QR code may be stale
            self.qrcode_cache = None;
        }
        self.last_steps = steps;
    }

    /// Percentage of the current period cycle elapsed for the selected
    /// element, falling back to the global 30 seconds cycle if no element is
    /// selected
    fn progress(&self) -> u16 {
        self.table
            .state
            .selected()
            .and_then(|index| self.database.elements_ref().get(index))
            .map_or_else(percentage, |element| period_percentage(element.period))
    }

    /// Renders the user interface widgets.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        match &self.current_page {
            Main => self.render_main_page(frame),
            Qrcode => self.render_qrcode_page(frame),
        }
    }

    fn render_qrcode_page(&mut self, frame: &mut Frame<'_>) {
        let selected_index = self
            .table
            .state
            .selected()
            .filter(|index| *index < self.database.elements_ref().len());

        let paragraph = if let Some(index) = selected_index {
            // Building the QR code (URI + matrix + unicode rendering) is
            // expensive, so cache the rendered string and rebuild it only
            // when the selection changes
            let cache_is_valid =
                matches!(&self.qrcode_cache, Some((cached, _)) if *cached == index);
            if !cache_is_valid {
                let qrcode = self.database.elements_ref()[index].get_qrcode();
                self.qrcode_cache = Some((index, qrcode));
            }
            let element = &self.database.elements_ref()[index];
            let qrcode = self
                .qrcode_cache
                .as_ref()
                .map(|(_, qrcode)| qrcode.as_str())
                .unwrap_or_default();
            let title = if element.label.is_empty() {
                element.issuer.clone()
            } else {
                format!("{} - {}", element.issuer, element.label)
            };
            Paragraph::new(format!("{}\n{}", qrcode, self.qr_code_page_label))
                .block(Block::default().title(title).borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
        } else {
            Paragraph::new("No element is selected")
                .block(Block::default().title("Nope").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
        };
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
                    Constraint::Length(3),                        // Search bar
                    Constraint::Length(height.saturating_sub(8)), // Table + Info Box
                    Constraint::Length(1),                        // Progress bar
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

        // The gauge tracks the period of the selected element, so a 60s TOTP
        // or a 10s MOTP shows its actual remaining time
        let progress = self.progress();
        let progress_label = if self.print_percentage {
            format!("{progress}%")
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
            .percent(progress)
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
            Period: {} {}
            Counter: {}
            Pin: {}
            ",
                element.type_,
                element.algorithm,
                element.period,
                if element.period == 1u64 {
                    "second"
                } else {
                    "seconds"
                },
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

/// Milliseconds elapsed since the Unix epoch
fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// Index of the current time step for the given period in seconds (the T
/// value of RFC 6238). A period of 0 is treated as 1 to avoid a division by
/// zero
fn current_step(period: u64) -> u64 {
    (current_millis() / 1000) / period.max(1)
}

/// Percentage of the current cycle elapsed for the given period in seconds
fn period_percentage(period: u64) -> u16 {
    let period_millis = period.max(1) * 1000;
    ((current_millis() % period_millis) * 100 / period_millis) as u16
}

/// The current time step of every element, in database order
fn element_steps(elements: &[OTPElement]) -> Vec<u64> {
    elements
        .iter()
        .map(|element| current_step(element.period))
        .collect()
}
