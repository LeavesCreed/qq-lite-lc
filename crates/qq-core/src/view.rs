use crate::{conversation::{Conversation, ConversationKind}, id::{ConversationId, LocalMessageId, RemoteMessageId, UserId}, message::{Message, MessageDirection, RichNode, SendState}};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationView {
    pub id: ConversationId,
    pub kind: ConversationKind,
    pub title: String,
    pub last_message_preview: Option<String>,
    pub updated_at_ms: i64,
    pub unread_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageView {
    pub local_id: LocalMessageId,
    pub remote_id: Option<RemoteMessageId>,
    pub conversation_id: ConversationId,
    pub sender_id: UserId,
    pub sender_name: Option<String>,
    pub direction: MessageDirection,
    pub sent_at_ms: i64,
    pub nodes: Vec<RichNode>,
    pub send_state: SendState,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimelineView {
    pub conversation_id: ConversationId,
    pub messages: Vec<MessageView>,
}

impl From<Conversation> for ConversationView {
    fn from(value: Conversation) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            title: value.title,
            last_message_preview: value.last_message_preview,
            updated_at_ms: value.updated_at_ms,
            unread_count: value.unread_count,
        }
    }
}

impl From<Message> for MessageView {
    fn from(value: Message) -> Self {
        Self {
            local_id: value.local_id,
            remote_id: value.remote_id,
            conversation_id: value.conversation_id,
            sender_id: value.sender_id,
            sender_name: value.sender_name,
            direction: value.direction,
            sent_at_ms: value.sent_at_ms,
            nodes: value.nodes,
            send_state: value.send_state,
        }
    }
}
