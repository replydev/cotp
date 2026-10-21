use std::{
    error,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::{
    enums::{Action, Focus, Page, PopupAction},
    stateful_table::StatefulTable,
};
use crate::otp::otp_element::OTPDatabase;

pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

pub struct App<'a> {
    pub running: bool,
    pub(crate) table: StatefulTable,
    pub(crate) database: &'a mut OTPDatabase,
    pub(crate) current_page: Page,
    pub(crate) search_query: String,
    pub(crate) focus: Focus,
    pub(crate) popup: Popup,
    pub(crate) popup_scroll: u16,
    pub(crate) notification: Option<(String, bool, Instant)>,
    pub(crate) hide_codes: bool,
    pub(crate) now: u64,
    pub(crate) form: Option<super::form::AccountForm>,
    pub(crate) hitboxes: Vec<(ratatui::layout::Rect, Action)>,
}

pub struct Popup {
    pub(crate) text: String,
    pub(crate) action: PopupAction,
}

impl<'a> App<'a> {
    pub fn new(database: &'a mut OTPDatabase) -> Self {
        Self {
            running: true,
            table: StatefulTable::new(database.elements_ref()),
            database,
            current_page: Page::default(),
            search_query: String::new(),
            focus: Focus::MainPage,
            popup: Popup {
                text: String::new(),
                action: PopupAction::GeneralInfo,
            },
            popup_scroll: 0,
            notification: None,
            hide_codes: crate::config::settings().hide_codes,
            now: unix_seconds(),
            form: None,
            hitboxes: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.current_page = Page::Main;
        self.focus = Focus::MainPage;
    }

    pub(crate) fn notify(&mut self, message: impl Into<String>, error: bool) {
        self.notification = Some((message.into(), error, Instant::now()));
    }

    pub(crate) fn filter(&mut self) {
        self.table
            .refresh(self.database.elements_ref(), &self.search_query);
    }

    pub fn tick(&mut self, force_update: bool) {
        let now = unix_seconds();
        if force_update || now != self.now {
            self.filter();
        }
        self.now = now;
        if self
            .notification
            .as_ref()
            .is_some_and(|(_, _, start)| start.elapsed() >= Duration::from_secs(4))
        {
            self.notification = None;
        }
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
