use esp_idf_hal::{
    delay::Delay,
    gpio::{OutputPin, PinDriver},
    prelude::*,
};
use max7219::{connectors::PinConnector, MAX7219};

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
    let mut display = MAX7219::from_pins(4, data, cs, clk).unwrap();

    let delay = Delay::new_default();

    log::info!("Turning on");
    display.power_on().unwrap();
    delay.delay_ms(500);

    log::info!("test mode");
    for i in 0..4 {
        log::info!("testing {}", i);
        display.test(i, true).unwrap();
        delay.delay_ms(500);
    }

    for intensity in 0x0..0xF {
        log::info!("Intensity: {}", intensity);
        for i in 0..4 {
            display.set_intensity(i, intensity).unwrap();
        }
        delay.delay_ms(500);
    }

    log::info!("Done");
}
