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
    pub framebuffer: [u8; 8 * 8], // TODO static? and maybe bitmask?
    cols: u32,
    rows: u32,
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
            framebuffer: [0; 8 * 8],
            cols: 8 * 4,
            rows: 8 * 2,
        }
    }


    pub fn flush(&mut self) -> Result<(), DataError> {
        for (i, chunk) in self.framebuffer.array_chunks::<8>().enumerate() {
            self.display.write_raw(i, chunk)?;
        }
        Ok(())
    }



    pub fn blit(&mut self, x: u32, y: u32, on: bool) {
        log::info!("({x}, {y})");
        let col = (x / 8);
        let x = x % 8;
        // let row = 1;
        let row = (y / 8);
        let y = y % 8;
        let segment = col + (row * 4); // cols per row: calculate and store.
        let posn = (segment * 8 + y) as usize;
        log::info!("Row {row} Col {col} segment {segment}");

        let mut row = self.framebuffer[posn];
        let mask = 0b1000_0000 >> x;
        if on {
            row |= mask;
        } else {
            row &= !mask;
        }
        self.framebuffer[posn] = row;
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
            // if let Ok((x @ 0..=self.cols, y @ 0..=self.rows)) = coord.try_into() {
            if let Ok((x, y)) = coord.try_into() {
                if x < self.cols && y < self.rows {
                    self.blit(x, y, color.is_on());
                }
                // if (0 <= x < self.cols) & (0 <= y < self.rows) {
                //     self.blit(x, y, color.is_on());
                // }
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
        Size::new(self.cols, self.rows)
    }
}
