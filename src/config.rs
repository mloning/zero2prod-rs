#[derive(serde::Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn read_config() -> Result<Config, config::ConfigError> {
    let source = config::File::new("config.yaml", config::FileFormat::Yaml);
    let config = config::Config::builder().add_source(source).build()?;
    config.try_deserialize::<Config>()
}
