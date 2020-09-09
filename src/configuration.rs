#[derive(serde::Deserialize, Default)]
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

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", &self.application_port)
    }
}
