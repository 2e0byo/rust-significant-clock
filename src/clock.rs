use anyhow::{Context, Result};
use chrono::{DateTime, Local, Timelike};
use crossbeam_channel::{Receiver, Sender};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use esp_idf_hal::{
    delay::Delay,
    sys::{setenv, tzset},
};

use std::ffi::CString;

use max7219::connectors::Connector;
use u8g2_fonts::{
    self, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

use crate::screen::Screen;
use crate::{config::Config, event::Event};

fn is_significant(time: DateTime<Local>) -> bool {
    let internal_pattern = |time: DateTime<Local>| {
        // TODO use an iterator here not vector
        let numbers: Vec<u8> = time
            .format("%H%M%S")
            .to_string()
            .chars()
            .map(|c| c.try_into().unwrap())
            .collect();
        let diffs: Vec<i8> = numbers
            .windows(2)
            .map(|window| window[1] as i8 - window[0] as i8)
            .collect();
        diffs[0] == -diffs[4] && diffs[1] == -diffs[3] || diffs == vec![1, 1, 1, 1, 1]
    };

    time.minute() == time.hour() && time.hour() == time.second() // 12:12:12
        || internal_pattern(time) // 12:34:56 || 12:33:21
}

fn flash(tx: &Sender<Event>) {
    let _ = tx.try_send(Event::Flash);
}

fn show_time<T>(screen: &mut Screen<T>, significant_mode: bool, tx: &Sender<Event>) -> Result<()>
where
    T: Connector,
{
    screen.clear();

    let dt = Local::now();
    if significant_mode && is_significant(dt) {
        flash(tx);
    }

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

    let _ = screen.flush(); // provide coercable error type.

    Ok(())
}

pub fn screen_loop<T>(
    mut screen: Screen<T>,
    rx: Receiver<Event>,
    tx: Sender<Event>,
    config: Config,
) -> !
where
    T: Connector,
{
    // TODO move init somewhere else.
    unsafe {
        let tz = CString::new("TZ").unwrap();
        // let zone = CString::new("CET-1CEST,M3.5.0,M10.5.0/3").unwrap();
        let zone = CString::new("GMT0BST,M3.5.0/1,M10.5.0").unwrap();
        setenv(tz.as_ptr(), zone.as_ptr(), 1);
        tzset();
    }
    let delay = Delay::new_default();
    let mut config = config;
    loop {
        if let Err(e) = show_time(&mut screen, config.significant_mode, &tx) {
            log::error!("Show time failed: {e:?}")
        };
        match rx.try_recv() {
            Ok(Event::ChangeBrightness(val)) => {
                let _ = screen.set_brightness(val);
            }
            Ok(Event::ChangeConfig(new_config)) => config = new_config,
            _ => (),
        };
        delay.delay_ms(100);
    }
}
