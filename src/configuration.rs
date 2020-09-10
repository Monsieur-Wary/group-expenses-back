#[derive(serde::Deserialize, Default)]
pub struct Settings {
    application_port: u16,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        std::env::var("application_port")
            .map_err(anyhow::Error::new)
            .and_then(|p| p.parse::<u16>().map_err(anyhow::Error::new))
            .or(Ok(DEFAULT_APPLICATION_PORT))
            .map(|application_port| Settings { application_port })
    }

    pub fn application_port(&self) -> u16 {
        self.application_port
    }

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", &self.application_port)
    }
}

pub const DEFAULT_APPLICATION_PORT: u16 = 8000;
