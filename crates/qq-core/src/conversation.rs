use crate::id::ConversationId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConversationKind {
    Private,
    Group,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub kind: ConversationKind,
    pub title: String,
    pub last_message_preview: Option<String>,
    pub updated_at_ms: i64,
    pub unread_count: u32,
}

impl Conversation {
    pub fn new(id: ConversationId, kind: ConversationKind, title: impl Into<String>, updated_at_ms: i64) -> Self {
        Self {
            id,
            kind,
            title: title.into(),
            last_message_preview: None,
            updated_at_ms,
            unread_count: 0,
        }
    }
}
