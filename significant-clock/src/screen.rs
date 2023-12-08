use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};

use max7219::{connectors::Connector, DataError, DecodeMode, MAX7219};

pub struct Screen<T>
where
    T: Connector,
{
    display: MAX7219<T>,
    n_displays: usize,
    framebuffer: [u8; 8 * 4], // TODO static? and maybe bitmask?
}


impl<T> Screen<T>
where
    T: Connector,
{

    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), DataError> {
        for n in 0..self.n_displays {
            self.display.set_intensity(n, brightness)?;
        }
        Ok(())
    }

    // TODO make self an enum and walk the state machine here.
    pub fn begin(&mut self) -> Result<(), DataError> {
        self.display.power_on()?;
        for n in 0..self.n_displays {
            self.display.set_decode_mode(n, DecodeMode::NoDecode)?;
            self.display.clear_display(n)?;
            self.display.set_intensity(n, 0x04)?;
        }
        Ok(())
    }

    pub fn from_display(display: MAX7219<T>, digits: usize) -> Screen<T> {
        Screen {
            display,
            n_displays: digits,
            framebuffer: [0; 8 * 4],
        }
    }

    fn write_raw(&mut self, data: Vec<DisplayData>) -> Result<(), DataError> {
        for n in 0..self.n_displays {
            self.display.write_raw(n, &data[n])?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), DataError> {
        for (i, chunk) in self.framebuffer.array_chunks::<8>().enumerate() {
            self.display.write_raw(i, chunk)?;
        }
        Ok(())
    }

    pub fn blit(&mut self, x: u32, y: u32, on: bool) {
        let segment = x / 8;
        let x = x % 8;
        let row_index = (segment + y) as usize;
        let mut row = self.framebuffer[row_index];
        // let mask = 0x80 >> x;
        let mask = 1 << x;
        if on {
            row |= mask;
        } else {
            let mask = 1 << x;
            row &= !mask;
        }
        self.framebuffer[row_index] = row;
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
            if let Ok((x @ 0..=32, y @ 0..=8)) = coord.try_into() {
                self.blit(x, y, color.is_on());
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
        Size::new(32, 8)
    }
}
