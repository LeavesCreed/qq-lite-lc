use crate::id::{ConversationId, LocalMessageId, RemoteMessageId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SendState {
    Received,
    Pending,
    Sent,
    Failed { reason: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RichNode {
    Text { text: String },
    Unsupported { kind: String, summary: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub local_id: LocalMessageId,
    pub remote_id: Option<RemoteMessageId>,
    pub conversation_id: ConversationId,
    pub sender_id: UserId,
    pub sender_name: Option<String>,
    pub direction: MessageDirection,
    pub sent_at_ms: i64,
    pub nodes: Vec<RichNode>,
    pub raw_json: Option<Value>,
    pub send_state: SendState,
}

impl Message {
    pub fn outgoing_text(conversation_id: ConversationId, sender_id: UserId, text: impl Into<String>, sent_at_ms: i64) -> Self {
        Self {
            local_id: LocalMessageId::generated(),
            remote_id: None,
            conversation_id,
            sender_id,
            sender_name: None,
            direction: MessageDirection::Outgoing,
            sent_at_ms,
            nodes: vec![RichNode::Text { text: text.into() }],
            raw_json: None,
            send_state: SendState::Pending,
        }
    }

    pub fn preview(&self) -> String {
        self.nodes
            .iter()
            .map(|node| match node {
                RichNode::Text { text } => text.as_str(),
                RichNode::Unsupported { summary, .. } => summary.as_str(),
            })
            .collect::<Vec<_>>()
            .join("")
    }
}
