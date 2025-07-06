use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub event_type: String,
    pub code: u16,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    InputEvent(InputEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub id: String,
    pub message: Message,
    pub timestamp: u64,
}

impl Packet {
    pub fn new(message: Message) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn input_event(event_type: String, code: u16, value: i32) -> Self {
        Self::new(Message::InputEvent(InputEvent {
            event_type,
            code,
            value,
        }))
    }
}
