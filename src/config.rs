use secrecy::{ExposeSecret, SecretBox};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct Config {
    pub db: DatabaseConfig,
    pub app: AppConfig,
    pub email_client: EmailClientConfig,
}

#[derive(serde::Deserialize)]
pub struct EmailClientConfig {
    pub base_url: String,
    pub sender_email: String,
    pub auth_token: SecretBox<String>,
    pub timeout_ms: u64,
}

impl EmailClientConfig {
    pub fn parse_sender_email(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn parse_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms)
    }
}

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub username: String,
    pub password: SecretBox<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub require_ssl: bool,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> PgConnectOptions {
        let options = self.connection_string_without_db().database(&self.name);
        options.log_statements(tracing_log::log::LevelFilter::Trace) // set log level to trace
    }

    pub fn connection_string_without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }
}

pub enum Env {
    Local,
    Prod,
}

impl Env {
    pub fn as_str(&self) -> &str {
        match self {
            Env::Local => "local",
            Env::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Env {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Env::Local),
            "prod" => Ok(Env::Prod),
            _ => Err(format!("{} is not a valid environment", value)),
        }
    }
}

pub fn read_config() -> Result<Config, config::ConfigError> {
    tracing::info!("Reading config ...");
    let directory = std::env::current_dir().expect("Failed to determine current directory");
    let config_directory = directory.join("config");

    let env: Env = std::env::var("ZERO2PROD_APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse: ZERO2PROD_APP_ENV");

    let base_file = config_directory.join("base.yaml");
    let env_file = config_directory.join(format!("{}.yaml", env.as_str()));

    let base_env_source = config::File::from(base_file);
    let env_source = config::File::from(env_file);
    let env_var_source = config::Environment::with_prefix("ZERO2PROD_APP")
        .prefix_separator("_")
        .separator("__");

    let config = config::Config::builder()
        .add_source(base_env_source)
        .add_source(env_source)
        .add_source(env_var_source)
        .build()?;
    config.try_deserialize::<Config>()
}
