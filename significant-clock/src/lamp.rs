use crossbeam_channel::Receiver;
use embedded_hal::pwm::SetDutyCycle;

use crate::{config::Config, event::Event, leds::Leds};

enum States {
    OFF,
    FADING,
    ON,
}

pub struct Lamp<T: SetDutyCycle> {
    leds: Leds<T>,
    config: Config,
}

impl<T: SetDutyCycle> Lamp<T> {
    pub fn new(leds: Leds<T>, config: Config) -> Lamp<T> {
        Lamp { leds, config }
    }

    fn on(&mut self) -> Result<(), T::Error> {
        self.leds.fade(self.config.lamp_brightness)
    }

    fn off(&mut self) -> Result<(), T::Error> {
        self.leds.off()
    }

    fn sync(&mut self) -> Result<(), T::Error> {
        if self.config.lamp_on {
            self.on()
        } else {
            self.off()
        }
    }

    pub fn run(&mut self, rx: Receiver<Event>) -> ! {
        loop {
            match rx.recv() {
                Ok(Event::Flash) => {
                    let _ = self.leds.flash();
                },
                Ok(Event::ChangeConfig(config)) => {
                    self.config = config;
                    let _ = self.sync();
                },
                _ => (),
            }
        }
    }
}
