#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "{connection_string}/{db_name}",
            connection_string = self.connection_string_without_db(),
            db_name = self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{username}:{password}@{host}:{port}",
            host = self.host,
            password = self.password,
            port = self.port,
            username = self.username,
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.toml",
            config::FileFormat::Toml,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
