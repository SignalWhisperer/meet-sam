use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRequest {
    pub from: String,
    pub subject: String,
    pub contents: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageCommand {
    pub command: MessageCommandType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageCommandType {
    Put {
        from: String,
        subject: String,
        contents: String,
    },
    Delete {
        message_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: String,
    pub from: String,
    pub subject: String,
    pub contents: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageHead {
    pub message_id: String,
    pub from: String,
    pub subject: String,
    pub timestamp: DateTime<Utc>,
}

impl From<Message> for MessageHead {
    fn from(message: Message) -> Self {
        Self {
            message_id: message.message_id,
            from: message.from,
            subject: message.subject,
            timestamp: message.timestamp,
        }
    }
}
