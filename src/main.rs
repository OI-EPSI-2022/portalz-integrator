use std::time::Instant;
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use dotenv::dotenv;
use crate::configuration::{get_configuration};
use secrecy::{ExposeSecret};
use log::{error, info};
use crate::models::ApiDeclaration;
use serde::{Deserialize, Serialize};

mod configuration;
mod models;

fn main() -> Result<()> {
    dotenv().ok();

    env_logger::init();
    // Open connection.
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
    let mut connection = Connection::open(&rabbitmq_url)?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    let queue = channel.queue_declare("api_declaration", QueueDeclareOptions::default())?;

    // Start a consumer.
    let consumer = queue.consume(ConsumerOptions::default())?;
    info!("Connected to queue {} in {:?}", configuration.rabbitmq.host, start_connection.elapsed());
    info!("Waiting for messages. Press Ctrl-C to exit.");

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => handle_message(delivery, &consumer)?,
            other => error!("Unexpected message: {:?}", other),
        }
    }

    connection.close()
}

fn handle_message(delivery: amiquip::Delivery, consumer: &amiquip::Consumer) -> Result<()> {
    let api_declaration: Result<ApiDeclaration, _> = serde_json::from_slice(&delivery.body);
    match api_declaration {
        Ok(declaration) => {
            info!("Received message: {:?}", declaration);
        }
        Err(e) => error!("Error while parsing message: {}", e)
    }
    consumer.ack(delivery)?;
    Ok(())
}
