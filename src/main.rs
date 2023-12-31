#![feature(never_type)]

use anyhow::{Context, Result};
use crossbeam_channel::bounded;

use esp_idf_hal::{
    gpio::{InputPin, OutputPin, PinDriver},
    ledc::{config::TimerConfig, *},
    prelude::*,
};
use esp_idf_svc::{
    sntp::{EspSntp, SntpConf},
    wifi::ClientConfiguration,
};

use max7219::MAX7219;
mod buttons;
mod clock;
mod config;
mod event;
mod lamp;
mod leds;
mod pins;
mod screen;
mod secrets;
mod wifi;

use crate::{
    buttons::Buttons,
    clock::screen_loop,
    screen::{ScreenBuilder, ScreenConfig, Segment},
};
use crate::{config::ConfigHandler, lamp::Lamp, wifi::*};
use crate::{event::Event, leds::Leds};

use std::{path::Path, thread};

fn main() -> Result<!> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let config_handler = ConfigHandler::new(Path::new("config.json"));
    let config = config_handler.get();

    let peripherals = Peripherals::take()?;
    let data = PinDriver::output(peripherals.pins.gpio26.downgrade_output())?;
    let cs = PinDriver::output(peripherals.pins.gpio33.downgrade_output())?;
    let clk = PinDriver::output(peripherals.pins.gpio25.downgrade_output())?;

    let _buzz = PinDriver::output(peripherals.pins.gpio27.downgrade_output())?;

    let screen = {
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
        let raw_display = MAX7219::from_pins(8, data, cs, clk)
            .ok() // hack for non convertable error types.
            .context("Failed to get display")?;
        ScreenBuilder::new(config).build(raw_display)?
    };

    let (msg_tx, msg_rx) = bounded::<Event>(8);

    let _screen_task = {
        let rx = msg_rx.clone();
        let tx = msg_tx.clone();
        let config = config.clone();
        thread::Builder::new()
            .stack_size(4096)
            .spawn(move || screen_loop(screen, rx, tx, config))
    };

    let _lamp_task = {
        let timer_driver = LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &TimerConfig::default().frequency(25.kHz().into()),
        )?;
        let red = LedcDriver::new(
            peripherals.ledc.channel0,
            &timer_driver,
            peripherals.pins.gpio22,
        )?;
        let green = LedcDriver::new(
            peripherals.ledc.channel1,
            &timer_driver,
            peripherals.pins.gpio23,
        )?;
        let blue = LedcDriver::new(
            peripherals.ledc.channel2,
            &timer_driver,
            peripherals.pins.gpio21,
        )?;

        let leds = Leds::new(red, green, blue);
        let mut lamp = Lamp::new(leds, config.clone());
        let rx = msg_rx.clone();

        thread::Builder::new()
            .stack_size(4096)
            .spawn(move || lamp.run(rx))
    };

    let _wifi_task = {
        let wifi_builder = WifiBuilder::from_modem(peripherals.modem)?;

        let client_config = ClientConfiguration {
            ssid: secrets::SSID.into(),
            password: secrets::PASSWORD.into(),
            auth_method: esp_idf_svc::wifi::AuthMethod::None, // personal?
            ..Default::default()
        };
        let wifi = wifi_builder
            .with_client_config(client_config)
            .build()
            .context("Failed to setup wifi")?;

        let rx = msg_rx.clone();
        let tx = msg_tx.clone();
        thread::Builder::new()
            .stack_size(4096)
            .spawn(move || wifi_loop(wifi, rx, tx))
    };

    let tx = msg_tx.clone();
    let _sntp = EspSntp::new_with_callback(&SntpConf::default(), move |_| {
        let _ = tx.try_send(Event::ClockSynced);
    });

    let _button_task = {
        let left_button = PinDriver::input(peripherals.pins.gpio34.downgrade_input())?;
        let right_button = PinDriver::input(peripherals.pins.gpio35.downgrade_input())?;
        let mut buttons = Buttons::new(left_button, right_button, config.clone());
        let tx = msg_tx.clone();

        thread::Builder::new()
            .stack_size(4096)
            .spawn(move || buttons.run(tx))
    };

    // Send startup messages
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
