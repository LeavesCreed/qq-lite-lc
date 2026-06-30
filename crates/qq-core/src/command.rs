use crate::{config::ConnectionConfig, id::ConversationId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientCommand {
    Connect(ConnectionConfig),
    Disconnect,
    LoadConversations,
    LoadMessages {
        conversation_id: ConversationId,
        limit: usize,
    },
    SendTextMessage {
        conversation_id: ConversationId,
        text: String,
    },
}
