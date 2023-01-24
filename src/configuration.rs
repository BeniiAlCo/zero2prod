use secrecy::{ExposeSecret, Secret};
use std::env;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "user={} password={} host={} port={} dbname={}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "user={} password={} host={} port={}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "production".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration/default"))
        .add_source(config::File::with_name(&format!(
            "configuration/{}",
            environment.as_str()
        )))
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?;

    settings.try_deserialize()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local' or `production'."
            )),
        }
    }
}
