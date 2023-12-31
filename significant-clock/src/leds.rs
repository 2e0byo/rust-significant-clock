use anyhow::Result;
use embedded_hal::pwm::SetDutyCycle;
use esp_idf_hal::delay::Ets;
use rgb::{RGB, RGB8};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Pixel(RGB8);

impl Deref for Pixel {
    type Target = RGB8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Pixel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<Pixel> for RGB<f32> {
    fn into(self) -> Pixel {
        Pixel(RGB8 {
            r: self.r as u8,
            g: self.g as u8,
            b: self.b as u8,
        })
    }
}

impl Into<Pixel> for RGB8 {
    fn into(self) -> Pixel {
        Pixel(self)
    }
}

pub struct Leds<T: SetDutyCycle> {
    red: T,
    green: T,
    blue: T,
    current: Pixel,
}

fn step(current: u8, other: u8, steps: u8) -> f32 {
    let multiplier = if current < other { 1. } else { -1. };
    multiplier * (current.abs_diff(other) as f32) / steps as f32
}

impl<T: SetDutyCycle> Leds<T> {
    pub fn new(red: T, green: T, blue: T) -> Self {
        let current = RGB8::default();
        Self {
            red,
            green,
            blue,
            current: current.into(),
        }
    }

    pub fn set(&mut self, val: Pixel) {
        self.current = val;
    }

    pub fn flush(&mut self) -> Result<(), T::Error> {
        self.red
            .set_duty_cycle_fraction(self.current.r.into(), 255)?;
        self.green
            .set_duty_cycle_fraction(self.current.g.into(), 255)?;
        self.blue
            .set_duty_cycle_fraction(self.current.b.into(), 255)?;

        Ok(())
    }

    pub fn fade(&mut self, target: Pixel) -> Result<(), T::Error> {
        let steps = 50;
        let increment: RGB<f32> = RGB {
            r: step(self.current.r, target.r, steps),
            g: step(self.current.g, target.g, steps),
            b: step(self.current.b, target.b, steps),
        };
        let current: RGB<f32> = self.current.0.into();

        for step in 0..steps {
            log::info!("step: {step:?}");
            let val = current + (increment * step as f32);
            self.set(val.into());
            self.flush()?;
            Ets::delay_ms(1);
        }
        self.set(target);
        self.flush()?;
        Ok(())
    }

    pub fn off(&mut self) -> Result<(), T::Error> {
        self.fade(RGB8 { r: 0, g: 0, b: 0 }.into())
    }

    pub fn on(&mut self) -> Result<(), T::Error> {
        self.fade(
            RGB8 {
                r: 255,
                g: 255,
                b: 255,
            }
            .into(),
        )
    }

    pub fn flash(&mut self) -> Result<(), T::Error> {
        let current = self.current;
        for _ in 0..2 {
            self.off()?;
            self.on()?;
        }
        self.fade(current)?;
        Ok(())
    }
}
