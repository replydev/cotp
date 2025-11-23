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

#[derive(Eq, PartialEq, Debug, Default)]
pub enum Page {
    #[default]
    Main,
    Qrcode,
}
