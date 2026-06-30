use crate::{command::ClientCommand, conversation::Conversation, error::CoreResult, event::DomainEvent, id::ConversationId, message::Message};
use async_trait::async_trait;

#[async_trait]
pub trait ChatGateway: Send + Sync {
    async fn execute(&self, command: ClientCommand) -> CoreResult<()>;
}

#[async_trait]
pub trait ChatStore: Send + Sync {
    async fn upsert_conversation(&self, conversation: &Conversation) -> CoreResult<()>;
    async fn insert_message(&self, message: &Message) -> CoreResult<()>;
    async fn list_conversations(&self) -> CoreResult<Vec<Conversation>>;
    async fn list_messages(&self, conversation_id: &ConversationId, limit: usize) -> CoreResult<Vec<Message>>;
}

pub trait EventSink: Send + Sync {
    fn publish(&self, event: DomainEvent) -> CoreResult<()>;
}
