use tui::widgets::TableState;

use crate::otp::otp_element::OTPElement;
use crate::otp::otp_helper::get_otp_code;

pub struct StatefulTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<Vec<String>>,
}

impl StatefulTable {
    pub fn new(elements: &Vec<OTPElement>) -> StatefulTable {
        let mut table = StatefulTable {
            state: TableState::default(),
            items: vec![],
        };
        let mut i = 0;
        for element in elements {
            // TODO remove unwrap and check exception
            let label = match element.type_().as_str() {
                "HOTP" => {
                    match element.counter() {
                        Some(result) => element.label() + (format!(" ({} counter)",result).as_str()),
                        None => element.label(),
                    }
                },
                _ => element.label(),
            };
            table.items.push(vec![(i + 1).to_string(), label, element.issuer(), get_otp_code(element).unwrap()]);
            i += 1;
        }
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