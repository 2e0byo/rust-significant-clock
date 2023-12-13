use chrono::{DateTime, Utc};

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use esp_idf_hal::{
    delay::Delay,
    gpio::{OutputPin, PinDriver},
    prelude::*,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi::{ClientConfiguration, EspWifi},
};

use screen::Screen;
use std::time::SystemTime;
use u8g2_fonts::{
    self, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

use max7219::{connectors::Connector, MAX7219};
mod screen;

use crate::screen::{ScreenBuilder, ScreenConfig, Segment};

fn show_time<T>(screen: &mut Screen<T>)
where
    T: Connector,
{
    screen.clear();
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let hm = dt.format("%H:%M");
    let s = dt.format("%S");

    let large_font = FontRenderer::new::<fonts::u8g2_font_5x7_tf>();
    // let small_font = FontRenderer::new::<fonts::u8g2_font_squeezed_r7_tr>();
    let tiny_font = FontRenderer::new::<fonts::u8g2_font_u8glib_4_tf>();
    large_font
        .render_aligned(
            format_args!("{}", hm),
            screen.bounding_box().center(),
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(BinaryColor::On),
            screen,
        )
        .unwrap();
    tiny_font
        .render_aligned(
            format_args!("{}", s),
            screen.bounding_box().bottom_right().unwrap() + Point::new(1, 2),
            VerticalPosition::Bottom,
            HorizontalAlignment::Right,
            FontColor::Transparent(BinaryColor::On),
            screen,
        )
        .unwrap();

    screen.flush().unwrap();
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let data = PinDriver::output(peripherals.pins.gpio27.downgrade_output()).unwrap();
    let cs = PinDriver::output(peripherals.pins.gpio26.downgrade_output()).unwrap();
    let clk = PinDriver::output(peripherals.pins.gpio25.downgrade_output()).unwrap();

    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs)).unwrap();
    let client_config = ClientConfiguration {
        ssid: "***REMOVED***".into(),
        password: "***REMOVED***".into(),
        auth_method: esp_idf_svc::wifi::AuthMethod::None, // personal?
        ..Default::default()
    };
    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::Client(client_config))
        .unwrap();
    wifi.start().unwrap();
    wifi.connect().unwrap();

    let delay = Delay::new_default();
    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
        delay.delay_ms(1);
    }
    log::info!("Connected!");

    let ntp = EspSntp::new_default().unwrap();
    while ntp.get_sync_status() != SyncStatus::Completed {} // TODO async this all
    log::info!("Time synchronised");

    let mut raw_display = MAX7219::from_pins(8, data, cs, clk).unwrap();
    let segments = vec![
        Segment::normal(0),
        Segment::normal(1),
        Segment::normal(2),
        Segment::normal(3),
        Segment::inverted(7),
        Segment::inverted(6),
        Segment::inverted(5),
        Segment::inverted(4),
    ];
    let config = ScreenConfig {
        n_displays: 8,
        cols: 4 * 8,
        rows: 2 * 8,
        segments,
        row_length: 4,
    };

    let mut screen = ScreenBuilder::new(config)
        .to_screen(&mut raw_display)
        .unwrap();

    // let style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
    // Text::new("12:45:36", Point::new(0, 6), style)
    //     .draw(&mut screen)
    //     .unwrap();
    // screen.flush().unwrap();
    // Circle::new(Point::new(2, 2), 4)
    //     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    //     .draw(&mut screen)
    //     .unwrap();

    // Circle::new(Point::new(12, 4), 6)
    //     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    //     .draw(&mut screen).unwrap();

    screen.set_brightness(1);
    loop {
        show_time(&mut screen);
        delay.delay_ms(1_000);
    }

    log::info!("Done");
}
