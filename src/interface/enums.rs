#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
pub enum Focus {
    MainPage,
    SearchBar,
    Popup,
    Form,
}

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
pub enum PopupAction {
    DeleteOtp,
    GeneralInfo,
    SaveBeforeQuit,
}

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default)]
pub enum Page {
    #[default]
    Main,
    Qrcode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Action {
    Key(crossterm::event::KeyCode),
    Command(crossterm::event::KeyCode),
    Search,
    ClearSearch,
    Select(usize),
    Field(usize),
    Save,
    Cancel,
}
