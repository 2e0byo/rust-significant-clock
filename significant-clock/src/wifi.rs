use anyhow::Context;
use anyhow::Result;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use embedded_svc::wifi::{AccessPointConfiguration, Configuration};
use esp_idf_hal::delay::Delay;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, EspWifi},
};

use crate::event::Event;

#[derive(Default)]
pub struct WifiBuilder {
    wifi: Option<BlockingWifi<EspWifi<'static>>>,
    ap_config: AccessPointConfiguration,
    client_config: ClientConfiguration,
}

enum CurrentConfig {
    Client(ClientConfiguration),
    AccessPoint(AccessPointConfiguration),
}

impl WifiBuilder {
    pub fn from_modem(modem: Modem) -> Result<WifiBuilder> {
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;
        let wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs))?;
        let blocking_wifi = BlockingWifi::wrap(wifi, sysloop)?;
        Ok(WifiBuilder {
            wifi: Some(blocking_wifi),
            ..Default::default()
        })
    }

    pub fn with_ap_config(self, ap_config: AccessPointConfiguration) -> WifiBuilder {
        WifiBuilder { ap_config, ..self }
    }

    pub fn with_client_config(self, client_config: ClientConfiguration) -> WifiBuilder {
        WifiBuilder {
            client_config,
            ..self
        }
    }

    pub fn build(self) -> Option<Wifi> {
        match self.wifi {
            Some(wifi) => {
                let mut wifi = Wifi {
                    wifi,
                    ap_config: self.ap_config,
                    client_config: self.client_config,
                };
                wifi.as_client().ok()?;
                Some(wifi)
            }
            None => {
                log::error!("No wifi currently");
                None
            }
        }
    }
}

pub struct Wifi {
    wifi: BlockingWifi<EspWifi<'static>>,
    ap_config: AccessPointConfiguration,
    client_config: ClientConfiguration,
}

impl Wifi {
    pub fn as_client(&mut self) -> Result<()> {
        self.wifi
            .set_configuration(&Configuration::Client(self.client_config.clone()))?;
        self.wifi.start()?;
        Ok(())
    }

    pub fn as_ap(&mut self) -> Result<()> {
        self.wifi
            .set_configuration(&Configuration::AccessPoint(self.ap_config.clone()))?;
        Ok(())
    }

    pub fn connect(&mut self) -> Result<()> {
        log::info!("Trying to connect to wifi");
        self.wifi.disconnect()?;
        self.wifi.connect()?;
        log::info!("Connected successfully");
        Ok(())
    }

    pub fn try_connect(&mut self, attempts: usize) -> Result<()> {
        for _ in 0..attempts {
            match self.connect() {
                Ok(_) => return Ok(()),
                Err(e) => log::error!("{e:?}"),
            }
            // if let Ok(_) = self.connect() {
            //     return Ok(());
            // };
        }
        match self.wifi.is_connected() {
            Ok(true) => Some(()),
            Ok(false) | Err(_) => None,
        }
        .context("failed to connect")
    }
}

pub fn wifi_loop(mut wifi: Wifi, _rx: Receiver<Event>, tx: Sender<Event>) -> ! {
    if let Ok(()) = wifi.try_connect(10) {
        tx.send(Event::NetworkConnected).unwrap()
    };
    // if let Ok(_) = wifi.try_connect(10) {
    //     tx.send(Event::NetworkConnected)
    // };
    let delay = Delay::new_default();
    loop {
        delay.delay_ms(100);
    }
}
