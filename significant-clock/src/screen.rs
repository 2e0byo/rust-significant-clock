use std::iter;

use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

use max7219::{connectors::Connector, DataError, DecodeMode, MAX7219};

#[derive(Debug)]
pub struct Segment {
    pub invert_x: bool,
    pub invert_y: bool,
    pub physical_posn: u8,
}

impl Segment {
    pub fn inverted(physical_posn: u8) -> Segment {
        Segment {
            invert_x: true,
            invert_y: true,
            physical_posn,
        }
    }
    pub fn normal(physical_posn: u8) -> Segment {
        Segment {
            invert_x: false,
            invert_y: false,
            physical_posn,
        }
    }
}

#[derive(Debug)]
pub struct ScreenConfig {
    pub n_displays: usize,
    pub cols: u32,
    pub rows: u32,
    pub segments: Vec<Segment>,
    pub row_length: usize,
}

#[derive(Debug)]
pub struct ScreenBuilder {
    config: ScreenConfig,
    framebuffer: Vec<u8>,
}

pub struct Screen<T>
where
    T: Connector,
{
    config: ScreenConfig,
    framebuffer: Vec<u8>,
    last_framebuffer: Vec<u8>,
    display: MAX7219<T>,
}

// this will become a library
#[allow(dead_code)]
impl ScreenBuilder {
    pub fn new(config: ScreenConfig) -> ScreenBuilder {
        let len = config.n_displays;
        ScreenBuilder {
            config,
            framebuffer: iter::repeat(0).take(len * 8).collect(),
        }
    }

    pub fn to_screen<T>(self, display: MAX7219<T>) -> Result<Screen<T>, DataError>
    where
        T: Connector,
    {
        let last_framebuffer = self.framebuffer.clone();
        let mut screen = Screen {
            config: self.config,
            framebuffer: self.framebuffer,
            last_framebuffer,
            display,
        };
        screen.display.power_on()?;
        for n in 0..screen.config.n_displays {
            screen.display.set_decode_mode(n, DecodeMode::NoDecode)?;
            screen.display.clear_display(n)?;
            screen.display.set_intensity(n, 0x04)?;
        }
        Ok(screen)
    }
}

// this will become a library
#[allow(dead_code)]
impl<T> Screen<T>
where
    T: Connector,
{
    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), DataError> {
        for n in 0..self.config.n_displays {
            self.display.set_intensity(n, brightness)?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), DataError> {
        let updates = iter::zip(self.framebuffer.chunks(8), self.last_framebuffer.chunks(8))
            .enumerate()
            .map(|(display, (new, old))| {
                iter::zip(new, old)
                    .enumerate()
                    .filter_map(move |(row, (new, old))| match new == old {
                        true => None,
                        false => Some((display, row + 1, new)),
                    })
            })
            .flatten();

        for (display, row, new) in updates {
            self.display.write_raw_byte(display, row as u8, *new)?;
        }

        self.last_framebuffer = self.framebuffer.clone();
        Ok(())
    }

    pub fn blit(&mut self, x: u32, y: u32, on: bool) {
        let col = x as usize / 8;
        let x = x % 8;
        let row = y as usize / 8;
        let y = y % 8;
        let segment_no = col + (row * self.config.row_length);

        let segment = &self.config.segments[segment_no];
        let x = if segment.invert_x { 7 - x } else { x };
        let y = if segment.invert_y { 7 - y } else { y };

        let row_index = (segment.physical_posn * 8) as usize + y as usize;

        let mut row = self.framebuffer[row_index];
        let mask = 0b1000_0000 >> x;
        if on {
            row |= mask;
        } else {
            row &= !mask;
        }
        self.framebuffer[row_index] = row;
    }

    pub fn clear(&mut self) {
        self.framebuffer = iter::repeat(0).take(self.framebuffer.len()).collect();
    }
}

impl<T> DrawTarget for Screen<T>
where
    T: Connector,
{
    type Error = core::convert::Infallible;
    type Color = BinaryColor;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<BinaryColor>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x, y)) = coord.try_into() {
                if x < self.config.cols && y < self.config.rows {
                    self.blit(x, y, color.is_on());
                }
            }
        }

        Ok(())
    }
}

impl<T> OriginDimensions for Screen<T>
where
    T: Connector,
{
    fn size(&self) -> Size {
        Size::new(self.config.cols, self.config.rows)
    }
}
