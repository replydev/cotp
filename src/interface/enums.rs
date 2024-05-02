#[derive(Eq, PartialEq, Debug)]
pub enum Focus {
    MainPage,
    SearchBar,
    Popup,
}

#[derive(Eq, PartialEq, Debug)]
pub enum PopupAction {
    EditOtp,
    DeleteOtp,
    GeneralInfo,
    SaveBeforeQuit,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Page {
    Main,
    Qrcode,
}

impl Default for Page {
    fn default() -> Self {
        Self::Main
    }
}
