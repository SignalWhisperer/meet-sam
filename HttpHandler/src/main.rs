use aws_sdk_dynamodb::model::AttributeValue;
use common::{Message, MessageCommand, MessageCommandType, MessageHead, MessageRequest};
use lambda_http::{http::Method, run, service_fn, Body, Request, Response};
use serde::Serialize;
use serde_dynamo::from_items;
use uuid::Uuid;

fn truncate(input: &str, length: usize) -> &str {
    match input.char_indices().nth(length) {
        None => input,
        Some((index, _)) => &input[..index],
    }
}

pub async fn list_messages(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
) -> Result<Vec<MessageHead>> {
    let result = client.scan().table_name(table).send().await?;

    if let Some(items) = result.items {
        let messages: Vec<MessageHead> = from_items(items)?;
        Ok(messages)
    } else {
        Ok(Vec::new())
    }
}

pub async fn get_message(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
    message_id: &str,
) -> Result<Vec<Message>> {
    let result = client
        .query()
        .table_name(table)
        .key_condition_expression("#message_id = :message_id")
        .expression_attribute_names("#message_id", "message_id")
        .expression_attribute_values(":message_id", AttributeValue::S(message_id.to_owned()))
        .send()
        .await?;

    if let Some(items) = result.items {
        let messages: Vec<Message> = from_items(items)?;
        Ok(messages)
    } else {
        Ok(Vec::new())
    }
}

#[derive(Debug, Serialize)]
struct MessageListResponse {
    messages: Vec<MessageHead>,
}

#[derive(Debug, Serialize)]
struct MessageGetResponse {
    messages: Vec<Message>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

async fn function_handler(event: Request) -> Result<Response<Body>> {
    let shared_config = aws_config::load_from_env().await;
    let sns = aws_sdk_sns::Client::new(&shared_config);
    let ddb = aws_sdk_dynamodb::Client::new(&shared_config);
    let dispatch_message_topic = std::env::var("DispatchMessageTopic")?;
    let message_store_table = std::env::var("MessageStoreTable")?;

    tracing::info!("Received request: {event:?}");

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json");

    let out = match *event.method() {
        Method::GET => {
            if event.uri().path() == "/" {
                tracing::info!("Request to list messages");

                let messages = list_messages(&ddb, &message_store_table).await?;
                let message_list = MessageListResponse { messages };

                serde_json::to_string(&message_list)?
            } else if let Some(message_id) = event.uri().path().strip_prefix("/") {
                tracing::info!("Request to get message: {message_id}");

                let messages = get_message(&ddb, &message_store_table, &message_id).await?;
                let message_get = MessageGetResponse { messages };

                serde_json::to_string(&message_get)?
            } else {
                r#"{"message":"Invalid message ID"}"#.into()
            }
        }
        Method::POST => {
            if let Body::Text(body) = event.body() {
                tracing::info!("Request to create new message");

                let request = serde_json::from_str::<MessageRequest>(body)?;
                let payload = MessageCommand {
                    command: MessageCommandType::Put {
                        message_id: Uuid::new_v4().to_string(),
                        from: truncate(&request.from, 255).into(),
                        subject: truncate(&request.subject, 255).into(),
                        contents: truncate(&request.contents, 1024).into(),
                    },
                };

                let pub_ack = sns
                    .publish()
                    .topic_arn(dispatch_message_topic)
                    .message(serde_json::to_string(&payload)?)
                    .send()
                    .await?;

                tracing::info!("Published SNS message: {pub_ack:?}");

                r#"{"message":"Message sent"}"#.into()
            } else {
                r#"{"message":"Invalid request body type"}"#.into()
            }
        }
        Method::DELETE => {
            if let Some(message_id) = event.uri().path().strip_prefix("/") {
                tracing::info!("Request to delete message: {message_id}");

                let payload = MessageCommand {
                    command: MessageCommandType::Delete {
                        message_id: message_id.to_owned(),
                    },
                };

                let pub_ack = sns
                    .publish()
                    .topic_arn(dispatch_message_topic)
                    .message(serde_json::to_string(&payload)?)
                    .send()
                    .await?;

                tracing::info!("Published SNS message: {pub_ack:?}");

                r#"{"message":"Message sent"}"#.into()
            } else {
                r#"{"message":"Invalid message ID"}"#.into()
            }
        }
        _ => r#"{"message":"Unsupported method"}"#.into(),
    };

    Ok(resp.body(out.into()).map_err(Box::new)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
