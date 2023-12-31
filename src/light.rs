use embedded_hal::pwm::SetDutyCycle;

use crate::{leds::Leds, config::ConfigHandler};

enum States {
    OFF,
    FADING,
    ON,
}


pub struct Lamp<T: SetDutyCycle> {
    leds: Leds<T>,
    config_handler: ConfigHandler,

}

impl <T: SetDutyCycle> Lamp <T> {
    pub fn new(leds: Leds<T>, config_handler: ConfigHandler ) -> Lamp<T> {
        Lamp {leds, config_handler}
    }

    pub fn on(&mut self) -> Result<(), T::Error>{
        let config = self.config_handler.get();
        self.leds.fade(config.lamp_brightness)
    }

    pub fn off(&mut self) -> Result<(), T::Error>{
        let config = self.config_handler.get();
        self.leds.off()
    }

}
