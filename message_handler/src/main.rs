use aws_lambda_events::{chrono::Utc, event::sns::SnsEvent};
use aws_sdk_dynamodb::model::AttributeValue;
use common::{MessageCommand, MessageCommandType};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use uuid::Uuid;

async fn handle_message(
    ddb: &aws_sdk_dynamodb::Client,
    message_store_table: &str,
    message: MessageCommand,
) {
    match message.command {
        MessageCommandType::Put {
            from,
            subject,
            contents,
        } => {
            let message_id = Uuid::new_v4().to_string();

            let res = ddb
                .put_item()
                .table_name(message_store_table)
                .item("message_id", AttributeValue::S(message_id.clone()))
                .item("from", AttributeValue::S(from))
                .item("subject", AttributeValue::S(subject))
                .item("contents", AttributeValue::S(contents))
                .item("timestamp", AttributeValue::S(Utc::now().to_rfc3339()))
                .send()
                .await;

            match res {
                Err(error) => tracing::error!("Failed to create message {message_id}: {error}"),
                Ok(_) => tracing::info!("Created message {message_id}"),
            }
        }
        MessageCommandType::Delete { message_id } => {
            let res = ddb
                .delete_item()
                .table_name(message_store_table)
                .key("message_id", AttributeValue::S(message_id.clone()))
                .send()
                .await;

            match res {
                Err(error) => tracing::error!("Failed to delete message {message_id}: {error}"),
                Ok(_) => tracing::info!("Deleted message {message_id}"),
            }
        }
    }
}

async fn function_handler(event: LambdaEvent<SnsEvent>) -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let ddb = aws_sdk_dynamodb::Client::new(&shared_config);
    let message_store_table = std::env::var("MessageStoreTable")?;

    tracing::info!("Received Lambda Event: {event:?}");

    let messages = event
        .payload
        .records
        .iter()
        .filter_map(|record| serde_json::from_str::<MessageCommand>(&record.sns.message).ok())
        .collect::<Vec<MessageCommand>>();

    for message in messages {
        handle_message(&ddb, &message_store_table, message).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
