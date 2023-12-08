#![feature(array_chunks)]

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle}, mono_font::{MonoTextStyle, ascii::{FONT_4X6, FONT_5X7}}, text::Text,
};
use esp_idf_hal::{
    gpio::{OutputPin, PinDriver},
    prelude::*, delay::Delay,
};

use max7219::{MAX7219};
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

    let raw_display = MAX7219::from_pins(8, data, cs, clk).unwrap();
    let mut screen = Screen::from_display(raw_display, 8);
    screen.begin().unwrap();

    // let style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
    // Text::new("oops", Point::new(0,0), style).draw(&mut screen).unwrap();
    // screen.flush().unwrap();

    // let circle = Circle::new(Point::new(12, 4), 6)
    //     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));

    // match circle.draw(&mut screen) {
    //     Ok(_) => screen.flush().unwrap(),
    //     Err(e) => log::error!("{:?}", e),
    // }


    let delay = Delay::new_default();


    for x in 0..32 {
        for y in 0..16 {
            // log::info!("{x}, {y}");
            screen.blit(x, y, true);
            screen.flush().unwrap();
            delay.delay_ms(20);
        }
    }

    log::info!("flush");
    // Update the display
    // screen.flush().unwrap();

    log::info!("Done");
}
