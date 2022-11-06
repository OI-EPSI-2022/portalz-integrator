use config::{Config, File};
use secrecy::{Secret};
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub rabbitmq: RabbitMQSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct AuthSettings {
    pub username: String,
    pub password: Secret<String>
}

#[derive(serde::Deserialize, Clone)]
pub struct RabbitMQSettings {
    pub protocol: String,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub auth: AuthSettings
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let config = Config::builder()
        .add_source(File::from(configuration_directory.join("base")).required(true))
        .add_source(
            config::Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_"),
        )
        .build()?;

    config.try_deserialize()
}
