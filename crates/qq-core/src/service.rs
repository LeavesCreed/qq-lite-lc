use crate::{command::ClientCommand, error::{CoreError, CoreResult}, event::DomainEvent, state::ClientState, view::{ConversationView, TimelineView}, ConversationId};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Clone)]
pub struct ClientCore {
    state: Arc<Mutex<ClientState>>,
    events: broadcast::Sender<DomainEvent>,
}

impl ClientCore {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(512);
        Self {
            state: Arc::new(Mutex::new(ClientState::default())),
            events,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.events.subscribe()
    }

    pub async fn dispatch_event(&self, event: DomainEvent) -> CoreResult<()> {
        self.state.lock().await.apply(event.clone());
        self.events.send(event).map_err(|_| CoreError::EventChannelClosed)?;
        Ok(())
    }

    pub async fn handle_local_command(&self, command: ClientCommand) -> CoreResult<()> {
        match command {
            ClientCommand::LoadConversations | ClientCommand::LoadMessages { .. } => Ok(()),
            other => Err(CoreError::InvalidCommand(format!("{other:?} must be handled by an adapter"))),
        }
    }

    pub async fn conversations(&self) -> Vec<ConversationView> {
        self.state.lock().await.conversation_views()
    }

    pub async fn timeline(&self, conversation_id: &ConversationId) -> TimelineView {
        self.state.lock().await.timeline_view(conversation_id)
    }
}

impl Default for ClientCore {
    fn default() -> Self {
        Self::new()
    }
}
