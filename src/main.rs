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
mod database;
mod rabbitmq;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    env_logger::init();

    // Load configuration
    let config = configuration::get_configuration().expect("Failed to read configuration.");
    // Create RabbitMQ connection
    let mut rabbitmq_connection = rabbitmq::open_connection().await?;
    // Open database connection
    let database = database::open_connection()
        .await
        .expect("Failed to connect to database.");

    // Create RabbitMQ consumer
    let channel = rabbitmq_connection.open_channel(None)?;
    let queue = channel.queue_declare("api_declaration", QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;

    info!("Waiting for messages. Press Ctrl-C to exit.");

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                handle_message(&delivery)?;
                consumer.ack(delivery)?;
            },
            other => {
                error!("Unexpected message: {:?}", other);
            },
        }
    }

    rabbitmq_connection.close()
}

fn handle_message(delivery: &amiquip::Delivery) -> Result<()> {
    let api_declaration: Result<ApiDeclaration, _> = serde_json::from_slice(&delivery.body);
    match api_declaration {
        Ok(declaration) => {
            info!("Received message: {:?}", declaration);
        }
        Err(e) => error!("Error while parsing message: {}", e)
    }
    Ok(())
}
