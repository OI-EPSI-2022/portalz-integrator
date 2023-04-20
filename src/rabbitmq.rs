use std::time::Instant;
use amiquip::{Connection};
use log::info;
use secrecy::ExposeSecret;
use crate::configuration::get_configuration;

pub async fn open_connection() -> Result<Connection, amiquip::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let rabbitmq_url = format!(
        "{}://{}:{}@{}:{}/{}",
        configuration.rabbitmq.protocol,
        configuration.rabbitmq.auth.username,
        configuration.rabbitmq.auth.password.expose_secret(),
        configuration.rabbitmq.host,
        configuration.rabbitmq.port,
        configuration.rabbitmq.auth.username
    );
    info!("Connecting to queue {}...", configuration.rabbitmq.host);
    let start_connection = Instant::now();

    let connection = Connection::open(&rabbitmq_url)?;
    info!("Connected to queue {} in {:?}", configuration.rabbitmq.host, start_connection.elapsed());

    return Ok(connection);
}