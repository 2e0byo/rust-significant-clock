use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crossbeam_channel::Receiver;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use esp_idf_hal::delay::Delay;

use std::time::SystemTime;

use max7219::connectors::Connector;
use u8g2_fonts::{
    self, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

use crate::event::Event;
use crate::screen::Screen;

fn show_time<T>(screen: &mut Screen<T>) -> Result<()>
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
        .unwrap(); // infallible
    let bottom_rhc = screen
        .bounding_box()
        .bottom_right()
        .context("Screen has no bottom rhc")?;

    tiny_font
        .render_aligned(
            format_args!("{}", s),
            bottom_rhc + Point::new(1, 2),
            VerticalPosition::Bottom,
            HorizontalAlignment::Right,
            FontColor::Transparent(BinaryColor::On),
            screen,
        )
        .unwrap(); // infallible

    screen.flush();

    Ok(())
}

pub fn screen_loop<T>(mut screen: Screen<T>, rx: Receiver<Event>) -> !
where
    T: Connector,
{
    let delay = Delay::new_default();
    loop {
        if let Err(e) = show_time(&mut screen) {
            log::error!("Show time failed: {e:?}")
        };
        match rx.try_recv() {
            Ok(Event::ChangeBrightness(val)) => {
                let _ = screen.set_brightness(val);
            }
            _ => (),
        };
        delay.delay_ms(100);
    }
}
