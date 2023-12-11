#![feature(array_chunks)]

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_4X6, FONT_5X7},
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use esp_idf_hal::{
    delay::Delay,
    gpio::{OutputPin, PinDriver},
    prelude::*,
};

use max7219::MAX7219;
mod screen;

use crate::screen::{ScreenBuilder, ScreenConfig, Segment};

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

    let style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
    Text::new("12:45:36", Point::new(6, 12), style)
        .draw(&mut screen)
        .unwrap();
    // screen.flush().unwrap();
    Circle::new(Point::new(2, 2), 4)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut screen)
        .unwrap();

    // Circle::new(Point::new(12, 4), 6)
    //     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    //     .draw(&mut screen).unwrap();

    screen.flush().unwrap();

    // match circle.draw(&mut screen) {
    //     Ok(_) => screen.flush().unwrap(),
    //     Err(e) => log::error!("{:?}", e),
    // }

    let delay = Delay::new_default();

    loop {
        for x in 0..32 {
            for y in 0..16 {
                screen.clear();
                screen.blit(x, y, true);
                match screen.flush() {
                    Ok(_) => (),
                    Err(e) => log::info!("{e:?}"),
                }
                delay.delay_us(1);
            }
        }
    }

    // log::info!("flush");
    // // Update the display
    // // screen.flush().unwrap();

    log::info!("Done");
}
