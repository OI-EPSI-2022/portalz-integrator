use std::time::Instant;
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use crate::configuration::{get_configuration};
use secrecy::{ExposeSecret};
use log::{error, info};

mod configuration;

fn main() -> Result<()> {
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

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                info!("({:>3}) Received [{}]", i, body);
                consumer.ack(delivery)?;
            }
            other => {
                error!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}
