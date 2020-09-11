use anyhow::Context;
use std::env;

#[derive(serde::Deserialize)]
pub struct Settings {
    application_port: u16,
    database: DatabaseSettings,
    hash_salt: String,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let hash_salt = env::var("HASH_SALT").context("HASH_SALT env var is not defined!")?;

        let mut settings = Settings {
            application_port: 8000,
            database: DatabaseSettings {
                username: "postgres".to_string(),
                password: "password".to_string(),
                port: 5432,
                host: "localhost".to_string(),
                name: "group-expenses".to_string(),
            },
            hash_salt,
        };

        if let Ok(application_port) = env::var("APPLICATION_PORT")
            .map_err(anyhow::Error::new)
            .and_then(|p| p.parse().map_err(anyhow::Error::new))
        {
            settings.application_port = application_port;
        }

        if let Ok(username) = env::var("DB_USERNAME") {
            settings.database.username = username;
        }

        if let Ok(password) = env::var("DB_PASSWORD") {
            settings.database.password = password;
        }

        if let Ok(port) = env::var("DB_PORT")
            .map_err(anyhow::Error::new)
            .and_then(|p| p.parse().map_err(anyhow::Error::new))
        {
            settings.database.port = port;
        }

        if let Ok(host) = env::var("DB_HOST") {
            settings.database.host = host;
        }

        Ok(settings)
    }

    pub fn application_port(&self) -> u16 {
        self.application_port
    }

    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", &self.application_port)
    }

    pub fn database(&self) -> &DatabaseSettings {
        &self.database
    }

    pub fn hash_salt(&self) -> &[u8] {
        self.hash_salt.as_bytes()
    }
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    username: String,
    password: String,
    port: u16,
    host: String,
    name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }
}
