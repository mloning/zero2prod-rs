use secrecy::{ExposeSecret, SecretBox};

#[derive(serde::Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub app: AppConfig,
}

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub username: String,
    pub password: SecretBox<String>,
    pub port: u16,
    pub host: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        )))
    }
    pub fn connection_string_without_db(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )))
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
    let directory = std::env::current_dir().expect("Failed to determine current directory");
    let config_directory = directory.join("config");

    let env: Env = std::env::var("ZERO2PROD_APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse: ZERO2PROD_APP_ENV");

    let base_file = config_directory.join("base.yaml");
    let env_file = config_directory.join(format!("{}.yaml", env.as_str()));

    let base_source = config::File::from(base_file);
    let env_source = config::File::from(env_file);

    let config = config::Config::builder()
        .add_source(base_source)
        .add_source(env_source)
        .build()?;
    config.try_deserialize::<Config>()
}
