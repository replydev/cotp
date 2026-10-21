//! Account editor. Metadata uses tui-input; secret buffers are masked and zeroized.
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::{Input, InputRequest, backend::crossterm::EventHandler};
use zeroize::{Zeroize, Zeroizing};

use super::{
    app::App,
    enums::{Focus, Page},
    row::code_for,
};
use crate::otp::{
    from_otp_uri::FromOtpUri, otp_algorithm::OTPAlgorithm, otp_element::OTPElement,
    otp_type::OTPType,
};

pub(crate) const SAVE: usize = 9;
pub(crate) const CANCEL: usize = 10;
const TYPES: [&str; 5] = ["TOTP", "HOTP", "STEAM", "YANDEX", "MOTP"];
const ALGORITHMS: [&str; 4] = ["SHA1", "SHA256", "SHA512", "MD5"];
pub(crate) const LABELS: [&str; 9] = [
    "Issuer",
    "Account label",
    "Secret",
    "Type",
    "Algorithm",
    "Digits",
    "Period (seconds)",
    "Counter",
    "PIN",
];

#[derive(Default)]
struct SecretInput {
    value: Zeroizing<String>,
    cursor: usize,
}

impl SecretInput {
    // Secrets and OTP URIs are ASCII. Rebuild on mutation so removed text is zeroized too.
    fn replace(&mut self, start: usize, end: usize, text: &str) {
        let value = format!("{}{}{}", &self.value[..start], text, &self.value[end..]);
        self.value = Zeroizing::new(value);
        self.cursor = start + text.len();
    }

    fn handle(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.value.zeroize();
                self.cursor = 0;
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => self.cursor = 0,
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.cursor = self.value.len()
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.value.len()),
            KeyCode::Backspace if self.cursor > 0 => self.replace(self.cursor - 1, self.cursor, ""),
            KeyCode::Delete if self.cursor < self.value.len() => {
                self.replace(self.cursor, self.cursor + 1, "")
            }
            KeyCode::Char(c)
                if c.is_ascii()
                    && !c.is_control()
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.replace(self.cursor, self.cursor, &c.to_string());
            }
            _ => {}
        }
    }
}

pub(crate) struct AccountForm {
    pub(crate) editing: Option<usize>,
    original: Option<OTPElement>,
    pub(crate) inputs: [Input; 9],
    secret: SecretInput,
    pin: SecretInput,
    pub(crate) active: usize,
    pub(crate) error: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct FormSnapshot {
    editing: Option<usize>,
    original: Option<OTPElement>,
    inputs: [String; 9],
    secret: String,
    pin: String,
    active: usize,
    error: Option<String>,
}
impl Drop for FormSnapshot {
    fn drop(&mut self) {
        self.secret.zeroize();
        self.pin.zeroize();
        self.inputs.zeroize();
    }
}

impl AccountForm {
    pub(crate) fn snapshot(&self) -> FormSnapshot {
        FormSnapshot {
            editing: self.editing,
            original: self.original.clone(),
            inputs: std::array::from_fn(|i| self.inputs[i].value().to_owned()),
            secret: self.secret.value.to_string(),
            pin: self.pin.value.to_string(),
            active: self.active,
            error: self.error.clone(),
        }
    }
    pub(crate) fn restore(mut state: FormSnapshot) -> Self {
        Self {
            editing: state.editing,
            original: state.original.take(),
            inputs: std::array::from_fn(|i| Input::new(std::mem::take(&mut state.inputs[i]))),
            secret: SecretInput {
                value: Zeroizing::new(std::mem::take(&mut state.secret)),
                cursor: 0,
            },
            pin: SecretInput {
                value: Zeroizing::new(std::mem::take(&mut state.pin)),
                cursor: 0,
            },
            active: state.active,
            error: state.error.take(),
        }
    }

    pub(crate) fn new(editing: Option<usize>, element: Option<&OTPElement>) -> Self {
        let mut form = Self {
            editing,
            original: element.cloned(),
            inputs: std::array::from_fn(|_| Input::default()),
            secret: SecretInput::default(),
            pin: SecretInput::default(),
            active: 0,
            error: None,
        };
        let defaults = ["", "", "", "TOTP", "SHA1", "6", "30", "0", ""];
        for (input, value) in form.inputs.iter_mut().zip(defaults) {
            *input = Input::from(value);
        }
        if let Some(element) = element {
            form.populate_metadata(element);
        }
        form
    }

    fn populate_metadata(&mut self, element: &OTPElement) {
        for (index, value) in [
            (0, element.issuer.clone()),
            (1, element.label.clone()),
            (3, element.type_.to_string()),
            (4, element.algorithm.to_string()),
            (5, element.digits.to_string()),
            (6, element.period.to_string()),
            (7, element.counter.unwrap_or(0).to_string()),
        ] {
            self.inputs[index] = Input::new(value);
        }
    }

    pub(crate) fn fields(&self) -> Vec<usize> {
        let kind = self.inputs[3].value();
        (0..9)
            .filter(|index| match index {
                4 => kind != "MOTP",
                6 => kind != "HOTP" && kind != "STEAM",
                7 => kind == "HOTP",
                8 => kind == "YANDEX" || kind == "MOTP",
                _ => true,
            })
            .collect()
    }

    pub(crate) fn navigate(&mut self, backwards: bool) {
        let mut order = self.fields();
        order.extend([SAVE, CANCEL]);
        let position = order
            .iter()
            .position(|index| *index == self.active)
            .unwrap_or(0);
        self.active = order[(position + if backwards { order.len() - 1 } else { 1 }) % order.len()];
    }

    pub(crate) fn is_choice(index: usize) -> bool {
        index == 3 || index == 4
    }

    pub(crate) fn cycle(&mut self, backwards: bool) {
        let options: &[&str] = if self.active == 3 {
            &TYPES
        } else {
            &ALGORITHMS
        };
        let current = self.inputs[self.active].value();
        let position = options
            .iter()
            .position(|value| *value == current)
            .unwrap_or(0);
        let next = (position + if backwards { options.len() - 1 } else { 1 }) % options.len();
        self.inputs[self.active] = Input::from(options[next]);
        if self.active == 3 {
            let digits = match options[next] {
                "STEAM" => "5",
                "YANDEX" => "8",
                _ => "6",
            };
            self.inputs[5] = Input::from(digits);
            if options[next] == "YANDEX" {
                self.inputs[4] = Input::from("SHA256");
            }
            if options[next] == "MOTP" {
                self.inputs[6] = Input::from("10");
            }
        }
    }

    pub(crate) fn display_input(&self, index: usize) -> Input {
        match index {
            2 => Input::new("•".repeat(self.secret.value.len())).with_cursor(self.secret.cursor),
            8 => Input::new("•".repeat(self.pin.value.len())).with_cursor(self.pin.cursor),
            _ => self.inputs[index].clone(),
        }
    }

    pub(crate) fn hint(&self, index: usize) -> &'static str {
        match index {
            0 => "Service name, for example GitHub",
            1 => "Email, username, or account name",
            2 if self.editing.is_some() => "Leave blank to keep the current secret",
            2 => "Secret key; you can also paste an otpauth:// URI",
            3 | 4 => "Click, ← / → or Space to change",
            5 => "Number of characters in the generated code (1–10)",
            6 => "Must match the service's refresh period",
            7 => "Must match the service's HOTP counter",
            8 if self.editing.is_some() => "Leave blank to keep the current PIN",
            8 => "Required for Yandex and MOTP",
            _ => "",
        }
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.active >= SAVE || Self::is_choice(self.active) {
            return;
        }
        if self.active == 2 && text.trim().starts_with("otpauth://") {
            if !["totp", "hotp", "steam", "yandex", "motp"]
                .iter()
                .any(|kind| text.trim().starts_with(&format!("otpauth://{kind}/")))
            {
                self.error = Some("Unsupported OTP URI account type.".into());
                return;
            }
            match OTPElement::from_otp_uri(text.trim()) {
                Ok(element) => {
                    self.populate_metadata(&element);
                    self.secret.value = Zeroizing::new(element.secret.clone());
                    self.secret.cursor = self.secret.value.len();
                    self.error = None;
                }
                Err(_) => {
                    self.error =
                        Some("Invalid OTP URI. Check the account and secret parameters.".into())
                }
            }
            return;
        }
        if self.active == 2 || self.active == 8 {
            let clean = Zeroizing::new(
                text.chars()
                    .filter(|c| c.is_ascii() && !c.is_control())
                    .collect::<String>(),
            );
            let input = if self.active == 2 {
                &mut self.secret
            } else {
                &mut self.pin
            };
            input.replace(input.cursor, input.cursor, &clean);
        } else {
            for c in text.chars().filter(|c| !c.is_control()) {
                self.inputs[self.active].handle(InputRequest::InsertChar(c));
            }
        }
    }

    pub(crate) fn input(&mut self, key: KeyEvent) {
        if self.active >= SAVE {
            return;
        }
        if Self::is_choice(self.active) {
            if matches!(
                key.code,
                KeyCode::Left | KeyCode::Right | KeyCode::Char(' ')
            ) {
                self.cycle(key.code == KeyCode::Left);
            }
        } else if self.active == 2 {
            self.secret.handle(key);
        } else if self.active == 8 {
            self.pin.handle(key);
        } else {
            self.inputs[self.active].handle_event(&Event::Key(key));
        }
    }

    pub(crate) fn build(&self) -> Result<OTPElement, (usize, String)> {
        let fail = |field, message: &str| (field, message.to_string());
        let label = self.inputs[1].value().trim();
        if label.is_empty() {
            return Err(fail(1, "Enter an account label."));
        }
        let digits = self.inputs[5]
            .value()
            .parse::<u64>()
            .ok()
            .filter(|n| (1..=10).contains(n))
            .ok_or_else(|| fail(5, "Digits must be a number from 1 to 10."))?;
        let kind = OTPType::from(self.inputs[3].value());
        let period = match kind {
            OTPType::Steam | OTPType::Hotp => 30,
            _ => self.inputs[6]
                .value()
                .parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or_else(|| fail(6, "Period must be a whole number greater than zero."))?,
        };
        let counter = if kind == OTPType::Hotp {
            Some(
                self.inputs[7]
                    .value()
                    .parse::<u64>()
                    .map_err(|_| fail(7, "Counter must be a non-negative whole number."))?,
            )
        } else {
            None
        };
        let secret = if self.secret.value.is_empty() {
            self.original
                .as_ref()
                .map(|e| e.secret.as_str())
                .unwrap_or("")
        } else {
            &self.secret.value
        };
        let secret = Zeroizing::new(
            secret
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
        );
        if secret.is_empty() {
            return Err(fail(2, "Enter the account's secret key."));
        }
        let pin = if matches!(kind, OTPType::Yandex | OTPType::Motp) {
            let pin = if self.pin.value.is_empty() {
                self.original
                    .as_ref()
                    .and_then(|e| e.pin.as_deref())
                    .unwrap_or("")
            } else {
                &self.pin.value
            };
            if pin.is_empty() {
                return Err(fail(8, "A PIN is required for this account type."));
            }
            Some(pin.to_string())
        } else {
            None
        };
        let element = OTPElement {
            id: self
                .original
                .as_ref()
                .map(|e| e.id.clone())
                .unwrap_or_default(),
            alias: self.original.as_ref().and_then(|e| e.alias.clone()),
            tags: self
                .original
                .as_ref()
                .map(|e| e.tags.clone())
                .unwrap_or_default(),
            favorite: self.original.as_ref().is_some_and(|e| e.favorite),
            order: self.original.as_ref().and_then(|e| e.order),
            secret: if kind == OTPType::Motp {
                secret.to_lowercase()
            } else {
                secret.to_uppercase()
            },
            issuer: self.inputs[0].value().trim().into(),
            label: label.into(),
            digits,
            type_: kind,
            algorithm: OTPAlgorithm::from(self.inputs[4].value()),
            period,
            counter,
            pin,
        };
        let decoded = if kind == OTPType::Motp {
            hex::decode(&element.secret)
                .map_err(|_| fail(2, "MOTP requires a hexadecimal secret key."))?
        } else {
            data_encoding::BASE32_NOPAD
                .decode(element.secret.as_bytes())
                .map_err(|_| fail(2, "Enter a valid Base32 secret key (A–Z and 2–7)."))?
        };
        let _decoded = Zeroizing::new(decoded);
        code_for(&element).map_err(|error| fail(2, &format!("Cannot generate a code: {error}")))?;
        Ok(element)
    }
}

impl App<'_> {
    pub(crate) fn open_form(&mut self, edit: bool) {
        let index = if edit {
            self.table.selected_index()
        } else {
            None
        };
        if edit && index.is_none() {
            self.notify("Select an account to edit", true);
            return;
        }
        self.form = Some(AccountForm::new(
            index,
            index.and_then(|i| self.database.get_element(i)),
        ));
        self.current_page = Page::Main;
        self.focus = Focus::Form;
    }

    pub(crate) fn cancel_form(&mut self) {
        self.form = None;
        self.focus = Focus::MainPage;
    }

    pub(crate) fn save_form(&mut self) {
        let Some(form) = self.form.as_mut() else {
            return;
        };
        match form.build() {
            Ok(element) => {
                let editing = form.editing;
                let index = editing.unwrap_or(self.database.elements_ref().len());
                if let Some(existing) = editing.and_then(|i| self.database.mut_element(i)) {
                    *existing = element;
                    self.database.mark_modified();
                } else {
                    if let Err(error) = self.database.add_element(element) {
                        form.error = Some(format!("Could not add account: {error}"));
                        return;
                    }
                }
                self.cancel_form();
                self.search_query.clear();
                self.filter();
                self.table.state.select(
                    self.table
                        .items
                        .iter()
                        .position(|row| row.database_index == index),
                );
                self.notify(
                    if editing.is_some() {
                        "Account updated · changes pending save"
                    } else {
                        "Account added · changes pending save"
                    },
                    false,
                );
            }
            Err((field, error)) => {
                form.active = field;
                form.error = Some(error);
            }
        }
    }
}

pub(crate) fn handle_form_key(key: KeyEvent, app: &mut App) {
    if key.code == KeyCode::Esc
        || key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
    {
        app.cancel_form();
        return;
    }
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.save_form();
        return;
    }
    let Some(form) = app.form.as_mut() else {
        return;
    };
    match key.code {
        KeyCode::Tab | KeyCode::Down => form.navigate(false),
        KeyCode::BackTab | KeyCode::Up => form.navigate(true),
        KeyCode::Enter if form.active == SAVE => app.save_form(),
        KeyCode::Enter if form.active == CANCEL => app.cancel_form(),
        KeyCode::Enter => form.navigate(false),
        _ => form.input(key),
    }
}

impl Drop for AccountForm {
    fn drop(&mut self) {
        for input in &mut self.inputs {
            input.value_and_reset().zeroize();
        }
        if let Some(error) = self.error.as_mut() {
            error.zeroize();
        }
    }
}
