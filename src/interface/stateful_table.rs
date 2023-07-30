use crate::interface::row::Row;
use ratatui::widgets::TableState;

use crate::otp::{otp_element::OTPElement, otp_type::OTPType};

pub struct StatefulTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<Row>,
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
        let selected = if self.items.is_empty() {
            None
        } else {
            Some(match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            })
        };
        self.state.select(selected);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().checked_sub(1)
                } else {
                    Some(i - 1)
                }
            }
            None => {
                if self.items.is_empty() {
                    None
                } else {
                    Some(0)
                }
            }
        };
        self.state.select(i);
    }
}

pub fn fill_table(table: &mut StatefulTable, elements: &[OTPElement]) {
    for (i, element) in elements.iter().enumerate() {
        let label = match element.type_ {
            OTPType::Hotp => match element.counter {
                Some(result) => {
                    element.label.to_owned() + (format!(" ({result} counter)").as_str())
                }
                None => element.label.to_owned(),
            },
            _ => element.label.to_owned(),
        };
        let result = element.get_otp_code();

        let error = result.is_err();
        table.items.push(Row::new(
            vec![
                (i + 1).to_string(),
                element.issuer.to_owned(),
                label,
                match result {
                    Ok(code) => code,
                    Err(e) => e.to_string(),
                },
            ],
            error,
        ));
    }
}
