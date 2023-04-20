use std::time::{Duration, Instant};
use log::info;
use secrecy::ExposeSecret;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, PgPool};
use crate::configuration::{get_configuration};

pub async fn open_connection() -> Result<PgPool, sqlx::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    info!("Connecting to database {}...", configuration.database.host);
    let start_connection = Instant::now();
    let options = PgConnectOptions::new()
        .username(&configuration.database.auth.username)
        .password(&configuration.database.auth.password.expose_secret())
        .host(&configuration.database.host)
        .port(configuration.database.port)
        .database(&configuration.database.name);
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .max_connections(5)
        .connect_with(options)
        .await?;
    info!("Connected to database {} in {:?}", configuration.database.host, start_connection.elapsed());
    return Ok(pool);
}