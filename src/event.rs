use crate::config::Config;

#[allow(dead_code)] // TODO working out what granularity to use.
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
    ChangeConfig(Config),
    // Internal
    Flash,
}
