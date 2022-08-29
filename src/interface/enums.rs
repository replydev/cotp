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
}

#[derive(Eq, PartialEq, Debug)]
pub enum Page {
    Main,
    Qrcode,
    Info,
}
