#[derive(serde::Deserialize)]
pub struct Settings {
    application_port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name("configuration"))?;
        settings.try_into()
    }

    pub fn application_port(&self) -> u16 {
        self.application_port
    }
}
