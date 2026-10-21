use crate::interface::row::Row;
use crate::otp::otp_element::OTPElement;
use ratatui::widgets::TableState;

pub struct StatefulTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<Row>,
}

impl StatefulTable {
    pub fn new(elements: &[OTPElement]) -> Self {
        let mut table = Self {
            state: TableState::default(),
            items: vec![],
        };
        table.refresh(elements, "");
        table
    }

    pub(crate) fn selected_index(&self) -> Option<usize> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
            .map(|row| row.database_index)
    }

    pub(crate) fn refresh(&mut self, elements: &[OTPElement], query: &str) {
        let selected = self.selected_index();
        let previous_position = self.state.selected().unwrap_or(0);
        let query = query.to_lowercase();
        self.items = elements
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                format!("{} {}", element.issuer, element.label)
                    .to_lowercase()
                    .contains(&query)
            })
            .map(|(index, element)| Row::new(index, element))
            .collect();
        let position = selected
            .and_then(|index| {
                self.items
                    .iter()
                    .position(|row| row.database_index == index)
            })
            .or_else(|| {
                (!self.items.is_empty()).then(|| previous_position.min(self.items.len() - 1))
            });
        self.state.select(position);
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            let next = self
                .state
                .selected()
                .map_or(0, |i| (i + 1) % self.items.len());
            self.state.select(Some(next));
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            let previous = self
                .state
                .selected()
                .map_or(0, |i| if i == 0 { self.items.len() - 1 } else { i - 1 });
            self.state.select(Some(previous));
        }
    }

    pub(crate) fn move_by(&mut self, delta: isize) {
        if !self.items.is_empty() {
            let index = self
                .state
                .selected()
                .unwrap_or(0)
                .saturating_add_signed(delta)
                .min(self.items.len() - 1);
            self.state.select(Some(index));
        }
    }
}
