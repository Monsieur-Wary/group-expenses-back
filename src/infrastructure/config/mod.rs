use anyhow::Context;
use std::env;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    application_port: u16,
    database: DatabaseSettings,
    security: SecuritySettings,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        // Necessary env vars
        let hash_salt = env::var("HASH_SALT").context("HASH_SALT env var is not defined!")?;
        let secret_key = env::var("SECRET_KEY").context("SECRET_KEY env var is not defined!")?;

        // Default settings
        let mut settings = Settings {
            application_port: 8000,
            database: DatabaseSettings {
                username: "postgres".to_string(),
                password: "password".to_string(),
                port: 5432,
                host: "localhost".to_string(),
                name: "group-expenses".to_string(),
            },
            security: SecuritySettings {
                hash_salt,
                secret_key,
                token_expiration_time: 3600,
            },
        };

        if let Ok(application_port) = env::var("APPLICATION_PORT")
            .context("Couldn't read application port env variable.")
            .and_then(|p| {
                p.parse()
                    .context("Couldn't parse application port env var.")
            })
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
            .context("Couldn't read db port env variable.")
            .and_then(|p| p.parse().context("Couldn't parse db port env var."))
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

    pub fn security(&self) -> &SecuritySettings {
        &self.security
    }
}

#[derive(serde::Deserialize, Clone)]
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

#[derive(serde::Deserialize, Clone)]
pub struct SecuritySettings {
    hash_salt: String,
    secret_key: String,
    token_expiration_time: i64,
}

impl SecuritySettings {
    pub fn hash_salt(&self) -> &[u8] {
        self.hash_salt.as_bytes()
    }

    pub fn secret_key(&self) -> &[u8] {
        self.secret_key.as_bytes()
    }

    pub fn token_expiration_time(&self) -> i64 {
        self.token_expiration_time
    }
}
