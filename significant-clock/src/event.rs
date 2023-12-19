#[derive(Debug)]
pub enum SignificanceMode {
    On,
    Off,
}

#[derive(Debug)]
pub enum Event {
    // Network
    APActivated,
    APDisactivated,
    NetworkConnecting,
    NetworkConnected,
    // NTP
    ClockSynced,
    // display
    ChangeBrightness(u8),
    ShowStatic(String),
    Hide,
    Show,
    // clock
    SetSingificanceMode(SignificanceMode)
}
