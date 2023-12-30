use rgb::RGB8;

/// Global clock config.  This is persisted to disk when modified, and can be set over the api.

#[derive(Clone, Debug)]
struct Config {
    light: RGB8,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            light: RGB8 {
                r: 25,
                g: 25,
                b: 25,
            },
        }
    }
}

struct ConfigHandler {
    config: Config
}

impl ConfigHandler {
    fn load() -> Config {
        log::info!("Loading config");
        Default::default()
    }
    pub fn new() -> Self {
        let config= ConfigHandler::load();
        Self { config }
    }

    pub fn get(&self) -> Config {
        self.config.clone()
    }

    fn persist(&self) {
        log::info!("Persisting config: {:?}", self.config);
    }

    pub fn set(&mut self, config: Config) {
        self.config = config;
        self.persist();
    }
}
