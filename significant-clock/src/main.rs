#![feature(never_type)]

use anyhow::{Context, Result};
use crossbeam_channel::bounded;

use esp_idf_hal::{
    gpio::{OutputPin, PinDriver},
    prelude::*,
};
use esp_idf_svc::{
    sntp::{EspSntp, SntpConf},
    wifi::ClientConfiguration,
};

use max7219::MAX7219;
mod clock;
mod event;
mod ntp;
mod screen;
mod wifi;

use crate::event::Event;
use crate::wifi::*;
use crate::{
    clock::screen_loop,
    screen::{ScreenBuilder, ScreenConfig, Segment},
};

use std::thread;

struct Resources {
    peripherals: Peripherals,
}

impl Resources {
    pub fn new() -> Result<Resources> {
        let peripherals = Peripherals::take()?;
        Ok(Resources { peripherals })
    }

    pub fn start_tasks(&self) -> Result<()> {
        Ok(())
    }
}

fn main() -> Result<!> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let data = PinDriver::output(peripherals.pins.gpio26.downgrade_output())?;
    let cs = PinDriver::output(peripherals.pins.gpio33.downgrade_output())?;
    let clk = PinDriver::output(peripherals.pins.gpio25.downgrade_output())?;

    let buzz = PinDriver::output(peripherals.pins.gpio27.downgrade_output())?;

    let mut red = PinDriver::output(peripherals.pins.gpio22.downgrade_output())?;
    let mut green = PinDriver::output(peripherals.pins.gpio23.downgrade_output())?;
    let mut blue = PinDriver::output(peripherals.pins.gpio21.downgrade_output())?;

    red.set_high();
    green.set_high();
    blue.set_high();

    // left = 34;
    // rigth button = 35//


    let segments = vec![
        Segment::inverted(7),
        Segment::inverted(6),
        Segment::inverted(5),
        Segment::inverted(4),
        Segment::inverted(3),
        Segment::inverted(2),
        Segment::inverted(1),
        Segment::inverted(0),
    ];
    let config = ScreenConfig {
        n_displays: 8,
        cols: 4 * 8,
        rows: 2 * 8,
        segments,
        row_length: 4,
    };

    let raw_display = MAX7219::from_pins(8, data, cs, clk).unwrap();
    let screen = ScreenBuilder::new(config).to_screen(raw_display).unwrap();

    let (msg_tx, msg_rx) = bounded::<Event>(1);
    let screen_rx = msg_rx.clone();

    let _ = thread::Builder::new()
        .stack_size(4096)
        .spawn(move || screen_loop(screen, screen_rx));

    let wifi_builder = WifiBuilder::from_modem(peripherals.modem)?;

    let client_config = ClientConfiguration {
        ssid: "***REMOVED***".into(),
        password: "***REMOVED***".into(),
        auth_method: esp_idf_svc::wifi::AuthMethod::None, // personal?
        ..Default::default()
    };
    let wifi = wifi_builder
        .with_client_config(client_config)
        .build()
        .context("Failed to setup wifi")?;

    let rx = msg_rx.clone();
    let tx = msg_tx.clone();
    let _ = thread::Builder::new()
        .stack_size(4096)
        .spawn(move || wifi_loop(wifi, rx, tx));
    let tx = msg_tx.clone();
    let _sntp = EspSntp::new_with_callback(&SntpConf::default(), move |_| {
        let _ = tx.try_send(Event::ClockSynced);
    });

    let _ = msg_tx.send(Event::ChangeBrightness(0));
    let _ = msg_tx.send(Event::ChangeBrightness(5));

    log::info!("Booted");
    loop {
        log::info!("Waiting for next message");
        match msg_rx.recv() {
            Ok(msg) => log::info!("Broadcast message {msg:?}"),
            Err(e) => log::error!("Error receiving message: {e:?}"),
        }
    }
}
