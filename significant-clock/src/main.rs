#![feature(array_chunks)]
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use esp_idf_hal::{
    delay::Delay,
    gpio::{OutputPin, PinDriver},
    prelude::*,
};

use max7219::{connectors::Connector, DataError, DecodeMode, MAX7219};
mod screen;
use screen::Screen;

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

    let raw_display = MAX7219::from_pins(4, data, cs, clk).unwrap();
    let mut screen = Screen::from_display(raw_display, 4);
    screen.begin().unwrap();

    let circle = Circle::new(Point::new(12, 4), 5)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));

    circle.draw(&mut screen).unwrap();

    log::info!("flush");
    // Update the display
    screen.flush().unwrap();

    log::info!("Done");
}
