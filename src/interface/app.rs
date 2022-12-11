use std::error;

use crate::interface::enums::Focus;
use crate::interface::enums::Page;
use crate::interface::enums::Page::{Main, Qrcode};
use crate::otp::otp_element::OTPDatabase;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Wrap};

use crate::interface::table::{fill_table, StatefulTable};
use crate::utils::percentage;

use super::enums::PopupAction;
use super::popup::centered_rect;

const LARGE_APPLICATION_WIDTH: u16 = 75;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    title: String,
    pub(crate) table: StatefulTable,
    pub(crate) database: OTPDatabase,
    progress: u16,
    /// Text to print replacing the percentage
    pub(crate) label_text: String,
    pub(crate) print_percentage: bool,
    pub(crate) current_page: Page,
    pub(crate) search_query: String,
    pub(crate) focus: Focus,
    pub(crate) popup_text: String,
    pub(crate) popup_action: PopupAction,
    pub(crate) popup_percent_x: u16,
    pub(crate) popup_percent_y: u16,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(database: OTPDatabase) -> Self {
        let mut title = String::from(env!("CARGO_PKG_NAME"));
        title.push_str(" v");
        title.push_str(env!("CARGO_PKG_VERSION"));
        Self {
            running: true,
            title,
            table: StatefulTable::new(database.elements_ref()),
            database,
            progress: percentage(),
            label_text: String::from(""),
            print_percentage: true,
            current_page: Main,
            search_query: String::from(""),
            focus: Focus::MainPage,
            popup_text: String::from(""),
            popup_action: PopupAction::EditOtp,
            popup_percent_x: 60,
            popup_percent_y: 20,
        }
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
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        match &self.current_page {
            Main => self.render_main_page(frame),
            Qrcode => self.render_qrcode_page(frame),
        }
    }

    fn render_qrcode_page<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let paragraph = if let Some(i) = self.table.state.selected() {
            if let Some(element) = self.database.elements_ref().get(i) {
                let title = if element.label.is_empty() {
                    element.issuer.to_owned()
                } else {
                    format!("{} - {}", &element.issuer, &element.label)
                };
                Paragraph::new(element.get_qrcode())
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
            } else {
                Paragraph::new("No element is selected")
                    .block(Block::default().title("Nope").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
            }
        } else {
            Paragraph::new("No element is selected")
                .block(Block::default().title("Nope").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
        };
        self.render_paragraph(frame, paragraph);
    }

    fn render_paragraph<B: Backend>(&self, frame: &mut Frame<'_, B>, paragraph: Paragraph) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(frame.size());

        frame.render_widget(paragraph, rects[0]);
    }

    fn render_main_page<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let height = frame.size().height;
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),              // Search bar
                    Constraint::Length(height - 3 - 6), // Table + Info Box
                    Constraint::Length(6),              // Progress bar
                ]
                .as_ref(),
            )
            .margin(2)
            .split(frame.size());

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
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let progress_label = if self.print_percentage {
            format!("{}%", self.progress)
        } else {
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

        frame.render_widget(search_bar, rects[0]);
        //frame.render_stateful_widget(t, rects[1], &mut self.table.state);
        self.render_table_box(frame, rects[1]);
        frame.render_widget(progress_bar, rects[2]);
        if self.focus == Focus::Popup {
            let block = Block::default().title("Alert").borders(Borders::ALL);
            let paragraph = Paragraph::new(&*self.popup_text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            let area = centered_rect(self.popup_percent_x, self.popup_percent_y, frame.size());
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(paragraph, area);
        }
    }

    fn render_table_box<B: Backend>(&mut self, frame: &mut Frame<'_, B>, area: Rect) {
        // TODO If terminal is too little do not show info box
        let constraints = if self.is_large_application(frame) {
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
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .title(self.title.as_str()),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("-> ")
            .widths(&[
                Constraint::Percentage(5),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
            ]);

        let selected_element = match self.table.state.selected() {
            Some(index) => self.database.get_element(index),
            None => None,
        };
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
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| String::from("N/A")),
                element.pin.clone().unwrap_or_else(|| String::from("N/A"))
            )
        } else {
            String::from("")
        };

        text.push_str(
            "
        
        Press I to get help
        ",
        );
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Code info").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_stateful_widget(t, chunks[0], &mut self.table.state);
        if self.is_large_application(frame) {
            frame.render_widget(paragraph, chunks[1]);
        }
    }

    fn is_large_application<B: Backend>(&self, frame: &mut Frame<'_, B>) -> bool {
        frame.size().width >= LARGE_APPLICATION_WIDTH
    }
}
