use tui::widgets::TableState;

use crate::otp::otp_element::{OTPElement, OTPType};

pub struct StatefulTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<Vec<String>>,
}

impl StatefulTable {
    pub fn new(elements: &[OTPElement]) -> StatefulTable {
        let mut table = StatefulTable {
            state: TableState::default(),
            items: vec![],
        };
        fill_table(&mut table, elements);
        table
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn fill_table(table: &mut StatefulTable, elements: &[OTPElement]) {
    for (i, element) in elements.iter().enumerate() {
        let label = match element.type_ {
            OTPType::Hotp => match element.counter {
                Some(result) => {
                    element.label.to_owned() + (format!(" ({} counter)", result).as_str())
                }
                None => element.label.to_owned(),
            },
            _ => element.label.to_owned(),
        };
        table.items.push(vec![
            (i + 1).to_string(),
            element.issuer.to_owned(),
            label,
            element.get_otp_code().unwrap(),
        ]);
    }
}
