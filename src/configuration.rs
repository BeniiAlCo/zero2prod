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
    pub require_ssl: bool,
    pub ca_cert: Secret<String>,
}

impl DatabaseSettings {
    pub fn get_ca_cert(&self) -> &Secret<String> {
        &self.ca_cert
    }

    pub fn without_db(&self) -> tokio_postgres::Config {
        let ssl_mode = if self.require_ssl {
            tokio_postgres::config::SslMode::Require
        } else {
            tokio_postgres::config::SslMode::Prefer
        };

        tokio_postgres::Config::new()
            .host(&self.host)
            .user(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .to_owned()
    }

    pub fn with_db(&self) -> tokio_postgres::Config {
        self.without_db().dbname(&self.database_name).to_owned()
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration/default"))
        .add_source(config::File::with_name(&format!(
            "configuration/{}",
            environment.as_str()
        )))
        .add_source(
            config::Environment::with_prefix("app")
                .prefix_separator("_")
                .separator("__"),
        )
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
