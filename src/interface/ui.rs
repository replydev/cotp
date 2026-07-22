use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Backend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Wrap};
use ratatui::{Frame, Terminal};

use crate::interface::app::{App, AppResult};
use crate::interface::enums::Focus;
use crate::interface::enums::Page::{Main, Qrcode};
use crate::interface::event::EventHandler;
use crate::interface::popup::centered_rect;

const LARGE_APPLICATION_WIDTH: u16 = 75;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> AppResult<()>
    where
        <B as Backend>::Error: 'static,
    {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: render
    pub fn draw(&mut self, app: &mut App) -> AppResult<()>
    where
        <B as Backend>::Error: 'static,
    {
        self.terminal.draw(|frame| render(app, frame))?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> AppResult<()>
    where
        <B as Backend>::Error: 'static,
    {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    match &app.current_page {
        Main => render_main_page(app, frame),
        Qrcode => render_qrcode_page(app, frame),
    }
}

fn render_qrcode_page(app: &mut App, frame: &mut Frame<'_>) {
    let selected_index = app
        .table
        .state
        .selected()
        .filter(|index| *index < app.database.elements_ref().len());

    let paragraph = if let Some(index) = selected_index {
        // Building the QR code (URI + matrix + unicode rendering) is
        // expensive, so cache the rendered string and rebuild it only
        // when the selection changes
        let cache_is_valid = matches!(&app.qrcode_cache, Some((cached, _)) if *cached == index);
        if !cache_is_valid {
            let qrcode = app.database.elements_ref()[index].get_qrcode();
            app.qrcode_cache = Some((index, qrcode));
        }
        let element = &app.database.elements_ref()[index];
        let qrcode = app
            .qrcode_cache
            .as_ref()
            .map(|(_, qrcode)| qrcode.as_str())
            .unwrap_or_default();
        let title = if element.label.is_empty() {
            element.issuer.clone()
        } else {
            format!("{} - {}", element.issuer, element.label)
        };
        Paragraph::new(format!("{}\n{}", qrcode, app.qr_code_page_label))
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
    render_paragraph(frame, paragraph);
}

fn render_paragraph(frame: &mut Frame<'_>, paragraph: Paragraph) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(frame.area());

    frame.render_widget(paragraph, rects[0]);
}

fn render_main_page(app: &mut App, frame: &mut Frame<'_>) {
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
    let search_bar = Paragraph::new(&*app.search_query)
        .block(
            Block::default()
                .title(search_bar_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.focus == Focus::SearchBar {
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
    let progress = app.progress();
    let progress_label = if app.print_percentage {
        format!("{progress}%")
    } else {
        app.label_text.clone()
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
    render_table_box(app, frame, rects[1]);
    frame.render_widget(progress_bar, rects[2]);
    if app.focus == Focus::Popup {
        render_alert(app, frame);
    }
}

fn render_alert(app: &mut App, frame: &mut Frame<'_>) {
    let block = Block::default().title("Alert").borders(Borders::ALL);
    let paragraph = Paragraph::new(&*app.popup.text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    let area = centered_rect(app.popup.percent_x, app.popup.percent_y, frame.area());
    frame.render_widget(Clear, area);
    //this clears out the background
    frame.render_widget(paragraph, area);
}

fn render_table_box(app: &mut App, frame: &mut Frame<'_>, area: Rect) {
    let constraints = if is_large_application(frame) {
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
    let rows = app.table.items.iter().map(|item| {
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
                .title(app.title.as_str()),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("-> ");

    let selected_element = app
        .table
        .state
        .selected()
        .and_then(|i| app.database.get_element(i));

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
    frame.render_stateful_widget(t, chunks[0], &mut app.table.state);
    if is_large_application(frame) {
        frame.render_widget(paragraph, chunks[1]);
    }
}

fn is_large_application(frame: &mut Frame<'_>) -> bool {
    frame.area().width >= LARGE_APPLICATION_WIDTH
}
