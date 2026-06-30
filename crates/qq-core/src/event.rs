use crate::{conversation::Conversation, id::LocalMessageId, message::{Message, SendState}};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed { reason: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DomainEvent {
    ConnectionChanged(ConnectionState),
    ConversationUpdated(Conversation),
    MessageReceived(Message),
    MessageSendStateChanged {
        local_id: LocalMessageId,
        state: SendState,
    },
    StoreSynced,
}
