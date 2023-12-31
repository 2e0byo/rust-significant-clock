use std::{
    fs::{self, File},
    path::Path,
};

use crate::leds::Pixel;
use anyhow::Result;
use rgb::RGB8;
use serde::{Deserialize, Serialize};

trait Persist<'a>
where
    Self: Default,
    Self: Serialize,
    for<'de> Self: Deserialize<'de>,
{
    fn load(path: &Path) -> Self {
        File::open(path)
            .and_then(|reader| Ok(serde_json::from_reader(reader)?))
            .unwrap_or_default()
    }

    fn save(&self, path: &Path) -> Result<()> {
        let serialised = serde_json::to_string(&self)?;
        fs::write(path, serialised)?;
        Ok(())
    }
}

impl Persist<'_> for Config {}

/// Global clock config.  This is persisted to disk when modified, and can be set over the api.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub lamp_brightness: Pixel,
    pub significant_mode: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            lamp_brightness: RGB8 {
                r: 25,
                g: 25,
                b: 25,
            }
            .into(),
            significant_mode: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    lamp_on: bool,
    alarm_on: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            lamp_on: true,
            alarm_on: false,
        }
    }
}

pub struct Handler<T> {
    current: T,
    path: Box<Path>,
}

impl<T> Handler<T>
where
    T: for<'a> Persist<'a>,
    T: Clone,
{
    pub fn new(path: &Path) -> Self {
        let current: T = Persist::load(path);
        Self {
            current,
            path: path.into(),
        }
    }

    pub fn get(&self) -> T {
        self.current.clone()
    }

    pub fn set(&mut self, val: T) {
        self.current = val;
        if let Err(e) = self.current.save(&self.path) {
            log::warn!("Error persisting: {e:?}");
        }
    }
}

pub type ConfigHandler = Handler<Config>;
