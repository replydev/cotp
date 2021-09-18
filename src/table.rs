use tui::widgets::TableState;
use crate::otp::otp_element::OTPElement;
use crate::otp::otp_helper::get_good_otp_code;

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
        let i = 0;
        for element in elements{
            table.items.push(vec![(i+1).to_string(),element.issuer(),element.label(),get_good_otp_code(element)])
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