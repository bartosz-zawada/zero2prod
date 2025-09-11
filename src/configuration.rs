use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name,
        )
        .into()
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize reader
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.ron",
            config::FileFormat::Ron,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
