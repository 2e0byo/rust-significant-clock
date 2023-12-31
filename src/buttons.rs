use embedded_hal::digital::InputPin;
use esp_idf_hal::delay::Delay;

use rgb::RGB8;

use crossbeam_channel::Sender;

use crate::{config::Config, event::Event, leds::Pixel};

type ActionFn = fn(&Actions) -> ();

pub struct Actions {
    pub short_left: ActionFn,
    pub long_left: ActionFn,
    pub short_right: ActionFn,
    pub long_right: ActionFn,
}

impl Default for Actions {
    fn default() -> Actions {
        Actions {
            short_left: |_| (),
            short_right: |_| (),
            long_left: |this| (this.short_left)(this),
            long_right: |this| (this.short_right)(this),
        }
    }
}

pub struct Buttons<T: InputPin> {
    left_button: T,
    right_button: T,
    config: Config,
}

impl<T: InputPin> Buttons<T> {
    pub fn new(left_button: T, right_button: T, config: Config) -> Self {
        // TODO work out where to put actions
        Self {
            left_button,
            right_button,
            config,
        }
    }

    pub fn run(&mut self, tx: Sender<Event>) -> ! {
        // TODO allow both
        let delay = Delay::new_default();
        let step: Pixel = RGB8 {
            r: 25,
            g: 25,
            b: 25,
        }
        .into();
        loop {
            if let Ok(true) = self.left_button.is_high() {
                while let Ok(true) = self.left_button.is_high() {
                    delay.delay_ms(1);
                }
                self.config.lamp_brightness = self.config.lamp_brightness.safe_add(step);
                // FIXME why do we need to send all events twice?  Maybe log response here.
                let _ = tx.send(Event::ChangeConfig(self.config.clone()));
                let _ = tx.send(Event::ChangeConfig(self.config.clone()));
            }

            if let Ok(true) = self.right_button.is_high() {
                while let Ok(true) = self.right_button.is_high() {
                    delay.delay_ms(1);
                }
                self.config.lamp_brightness = self.config.lamp_brightness.safe_sub(step);
                let _ = tx.send(Event::ChangeConfig(self.config.clone()));
                let _ = tx.send(Event::ChangeConfig(self.config.clone()));
            }
            delay.delay_ms(1);
        }
    }
}
