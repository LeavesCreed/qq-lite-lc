use crate::{conversation::Conversation, event::{ConnectionState, DomainEvent}, id::{ConversationId, LocalMessageId}, message::{Message, SendState}, view::{ConversationView, TimelineView}};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ClientState {
    pub connection: ConnectionState,
    conversations: HashMap<ConversationId, Conversation>,
    messages: HashMap<ConversationId, Vec<Message>>,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            conversations: HashMap::new(),
            messages: HashMap::new(),
        }
    }
}

impl ClientState {
    pub fn apply(&mut self, event: DomainEvent) {
        match event {
            DomainEvent::ConnectionChanged(connection) => {
                self.connection = connection;
            }
            DomainEvent::ConversationUpdated(conversation) => {
                self.conversations.insert(conversation.id.clone(), conversation);
            }
            DomainEvent::MessageReceived(message) => {
                let preview = message.preview();
                if let Some(conversation) = self.conversations.get_mut(&message.conversation_id) {
                    conversation.last_message_preview = Some(preview);
                    conversation.updated_at_ms = conversation.updated_at_ms.max(message.sent_at_ms);
                }
                let messages = self.messages.entry(message.conversation_id.clone()).or_default();
                if let Some(existing) = messages.iter_mut().find(|existing| existing.local_id == message.local_id) {
                    *existing = message;
                } else {
                    messages.push(message);
                    messages.sort_by(|a, b| a.sent_at_ms.cmp(&b.sent_at_ms));
                }
            }
            DomainEvent::MessageSendStateChanged { local_id, state } => {
                self.update_send_state(&local_id, state);
            }
            DomainEvent::StoreSynced => {}
        }
    }

    pub fn conversation_views(&self) -> Vec<ConversationView> {
        let mut views = self.conversations.values().cloned().map(ConversationView::from).collect::<Vec<_>>();
        views.sort_by(|a, b| b.updated_at_ms.cmp(&a.updated_at_ms));
        views
    }

    pub fn timeline_view(&self, conversation_id: &ConversationId) -> TimelineView {
        TimelineView {
            conversation_id: conversation_id.clone(),
            messages: self.messages
                .get(conversation_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }

    fn update_send_state(&mut self, local_id: &LocalMessageId, state: SendState) {
        for messages in self.messages.values_mut() {
            if let Some(message) = messages.iter_mut().find(|message| &message.local_id == local_id) {
                message.send_state = state;
                return;
            }
        }
    }
}
