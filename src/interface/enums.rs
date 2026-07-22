#[derive(Eq, PartialEq, Debug)]
pub enum Focus {
    MainPage,
    SearchBar,
    Popup,
}

#[derive(Eq, PartialEq, Debug, Default)]
pub enum PopupAction {
    DeleteOtp,
    #[default]
    GeneralInfo,
    SaveBeforeQuit,
}

#[derive(Eq, PartialEq, Debug, Default)]
pub enum Page {
    #[default]
    Main,
    Qrcode,
}
