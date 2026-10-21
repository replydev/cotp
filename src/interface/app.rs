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
    pub(crate) sealed: Option<String>,
    pub(crate) unlock_password: zeroize::Zeroizing<String>,
    pub(crate) last_activity: Instant,
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Popup {
    pub(crate) text: String,
    pub(crate) action: PopupAction,
}

impl<'a> App<'a> {
    pub fn new(database: &'a mut OTPDatabase) -> Self {
        Self {
            running: true,
            sealed: None,
            unlock_password: zeroize::Zeroizing::new(String::new()),
            last_activity: Instant::now(),
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
        if self.sealed.is_some() {
            return;
        }
        let timeout = crate::config::settings().idle_timeout_seconds;
        if timeout > 0 && self.last_activity.elapsed() >= Duration::from_secs(timeout) {
            self.lock_session();
            return;
        }
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

#[derive(serde::Serialize, serde::Deserialize)]
struct Snapshot {
    database: OTPDatabase,
    modified: bool,
    form: Option<super::form::FormSnapshot>,
    search: String,
    focus: Focus,
    page: Page,
    popup: Popup,
    popup_scroll: u16,
    selected: Option<usize>,
}
impl App<'_> {
    pub(crate) fn lock_session(&mut self) {
        use zeroize::Zeroize;
        if self.sealed.is_some() {
            return;
        }
        let result = (|| -> color_eyre::Result<String> {
            // Serialize by reference, preserving the unsaved flag omitted by the on-disk format.
            #[derive(serde::Serialize)]
            struct State<'a> {
                database: &'a OTPDatabase,
                modified: bool,
                form: Option<super::form::FormSnapshot>,
                search: &'a str,
                focus: &'a Focus,
                page: &'a Page,
                popup: &'a Popup,
                popup_scroll: u16,
                selected: Option<usize>,
            }
            let state = State {
                database: self.database,
                modified: self.database.is_modified(),
                form: self.form.as_ref().map(|f| f.snapshot()),
                search: &self.search_query,
                focus: &self.focus,
                page: &self.current_page,
                popup: &self.popup,
                popup_scroll: self.popup_scroll,
                selected: self.table.selected_index(),
            };
            let plaintext = zeroize::Zeroizing::new(serde_json::to_string(&state)?);
            crate::persistence::seal(&plaintext)
        })();
        match result {
            Ok(cipher) => {
                self.sealed = Some(cipher);
                *self.database = OTPDatabase::default();
                self.form = None;
                self.table = StatefulTable::new(&[]);
                self.search_query.zeroize();
                self.popup.text.zeroize();
                if let Some((message, _, _)) = self.notification.as_mut() {
                    message.zeroize();
                }
                self.notification = None;
                self.hitboxes.clear();
                crate::persistence::lock();
            }
            Err(error) => self.notify(format!("Could not lock: {error}"), true),
        }
    }
    pub(crate) fn unlock_session(&mut self) {
        use zeroize::Zeroize;
        let Some(cipher) = self.sealed.clone() else {
            return;
        };
        let result = (|| -> color_eyre::Result<()> {
            let (text, key, salt) =
                crate::crypto::cryptography::decrypt_string(&cipher, &self.unlock_password)?;
            let text = zeroize::Zeroizing::new(text);
            let key = zeroize::Zeroizing::new(key);
            let state: Snapshot = serde_json::from_str(&text)?;
            crate::persistence::unlock(&key, &salt)?;
            *self.database = state.database;
            self.database.needs_modification = state.modified;
            self.form = state.form.map(super::form::AccountForm::restore);
            self.search_query = state.search;
            self.focus = state.focus;
            self.current_page = state.page;
            self.popup = state.popup;
            self.popup_scroll = state.popup_scroll;
            self.filter();
            if let Some(selected) = state.selected {
                self.table.state.select(
                    self.table
                        .items
                        .iter()
                        .position(|row| row.database_index == selected),
                );
            }
            Ok(())
        })();
        self.unlock_password.zeroize();
        match result {
            Ok(()) => {
                self.sealed = None;
                self.last_activity = Instant::now();
                self.filter();
                self.notify("Unlocked · pending edits restored", false);
            }
            Err(_) => self.notify("Could not unlock. Check your password.", true),
        }
    }
}
