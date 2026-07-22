use std::error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::interface::enums::Focus;
use crate::interface::enums::Page;
use crate::otp::otp_element::{OTPDatabase, OTPElement};

use crate::interface::stateful_table::{StatefulTable, fill_table};
use crate::utils::percentage;

use super::enums::PopupAction;

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

const DEFAULT_QRCODE_LABEL: &str = "Press enter to copy the OTP URI code";

/// Application.
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    pub(crate) title: String,
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
    pub(crate) qrcode_cache: Option<(usize, String)>,
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
    pub(crate) fn progress(&self) -> u16 {
        self.table
            .state
            .selected()
            .and_then(|index| self.database.elements_ref().get(index))
            .map_or_else(percentage, |element| period_percentage(element.period))
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
