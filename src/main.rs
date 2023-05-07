use std::time::Instant;
use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use dotenv::dotenv;
use crate::configuration::{get_configuration};
use secrecy::{ExposeSecret};
use log::{error, info};
use crate::models::{ApiDeclaration, WorkflowInputs, WorkflowParameters};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

mod configuration;
mod models;
mod database;
mod rabbitmq;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    env_logger::init();

    // Load configuration
    let config = get_configuration().expect("Failed to read configuration.");
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
                let parsed_declaration = handle_message(&delivery)?;
                save_api_declaration(&parsed_declaration, &database).await.expect("Failed to save api declaration.");
                send_creation_request(&parsed_declaration).await.expect("Failed to send creation request.");
                consumer.ack(delivery)?;
            },
            other => {
                error!("Unexpected message: {:?}", other);
            },
        }
    }

    rabbitmq_connection.close()
}

fn handle_message(delivery: &amiquip::Delivery) -> Result<ApiDeclaration> {
    let api_declaration: Result<ApiDeclaration, _> = serde_json::from_slice(&delivery.body);
    match &api_declaration {
        Ok(declaration) => {
            info!("Received message: {:?}", declaration);
        }
        Err(e) => error!("Error while parsing message: {}", e)
    }
    Ok(api_declaration.unwrap())
}

async fn save_api_declaration(declaration: &ApiDeclaration, database: &sqlx::PgPool) -> Result<()> {
    let response = sqlx::query(
            "INSERT INTO portalz_operations (name, spec, project_id, created_by)\
            VALUES ($1, $2, \'92ce5187-8d93-40de-a50f-91137cb467f9\', \'db1e7024-e6b8-4c4c-bee1-23e3808963a1\')"
    )
        .bind(declaration.name.to_string())
        .bind(serde_json::to_value(declaration).unwrap())
    .execute(database).await;
    match response {
        Ok(_) => info!("Saved api declaration: {:?}", declaration),
        Err(e) => error!("Failed to save api declaration: {}", e),
    }
    Ok(())
}

async fn send_creation_request(declaration: &ApiDeclaration) -> Result<()> {
    let client = reqwest::Client::new();
    let request_url = "https://api.github.com/repos/OI-EPSI-2022/portalz-target-template/actions/workflows/general.yml/dispatches";
    let body = WorkflowParameters {
        reference: "main".to_string(),
        inputs: WorkflowInputs {
            specs_file: serde_json::to_string(declaration).unwrap(),
            id: "8d79a9f9-afd3-4d5d-8403-3fecbff98dbb".to_string(),
        }
    };
    info!("{:?}", serde_json::to_value(&body).unwrap());
    let response = client
        .post(request_url)
        .bearer_auth(get_configuration().unwrap().github.token.expose_secret())
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "portalz-integrator")
        .json(&serde_json::to_value(&body).unwrap())
        .send()
        .await;
    match response {
        Ok(resp) => info!("Triggered job: {:?}", resp.text().await),
        Err(e) => error!("Failed to trigger workflow: {}", e),
    }
    Ok(())
}
