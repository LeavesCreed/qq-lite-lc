use crate::{dto::{ActionRequest, EventEnvelope}, mapper::{events_from_message, text_message_action}};
use futures_util::{SinkExt, StreamExt};
use qq_core::{ClientCommand, ConnectionConfig, ConnectionState, CoreError, CoreResult, DomainEvent};
use tokio::{sync::{broadcast, mpsc}, task::JoinHandle, time::{sleep, Duration}};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, http::header::AUTHORIZATION, Message}};

#[derive(Clone)]
pub struct NapCatGateway {
    commands: mpsc::Sender<ClientCommand>,
    events: broadcast::Sender<DomainEvent>,
}

impl NapCatGateway {
    pub fn new() -> Self {
        let (commands, command_rx) = mpsc::channel(256);
        let (events, _) = broadcast::channel(512);
        tokio::spawn(run_gateway(command_rx, events.clone()));
        Self { commands, events }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.events.subscribe()
    }

    pub async fn execute(&self, command: ClientCommand) -> CoreResult<()> {
        self.commands.send(command).await.map_err(|_| CoreError::CommandChannelClosed)
    }
}

impl Default for NapCatGateway {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_gateway(mut commands: mpsc::Receiver<ClientCommand>, events: broadcast::Sender<DomainEvent>) {
    let mut active_config: Option<ConnectionConfig> = None;
    let mut connection_task: Option<JoinHandle<()>> = None;

    while let Some(command) = commands.recv().await {
        match command {
            ClientCommand::Connect(config) => {
                if let Some(task) = connection_task.take() {
                    task.abort();
                }
                active_config = Some(config.clone());
                publish(&events, DomainEvent::ConnectionChanged(ConnectionState::Connecting));
                let task_events = events.clone();
                connection_task = Some(tokio::spawn(async move {
                    loop {
                        match connect_once(config.clone(), task_events.clone()).await {
                            Ok(()) => publish(&task_events, DomainEvent::ConnectionChanged(ConnectionState::Disconnected)),
                            Err(error) => publish(&task_events, DomainEvent::ConnectionChanged(ConnectionState::Failed { reason: error.to_string() })),
                        }
                        if !config.reconnect {
                            break;
                        }
                        publish(&task_events, DomainEvent::ConnectionChanged(ConnectionState::Reconnecting));
                        sleep(Duration::from_secs(2)).await;
                    }
                }));
            }
            ClientCommand::Disconnect => {
                active_config = None;
                if let Some(task) = connection_task.take() {
                    task.abort();
                }
                publish(&events, DomainEvent::ConnectionChanged(ConnectionState::Disconnected));
            }
            ClientCommand::SendTextMessage { conversation_id, text } => {
                let Some(config) = active_config.clone() else {
                    publish(&events, DomainEvent::ConnectionChanged(ConnectionState::Failed {
                        reason: "not connected to NapCat".to_owned(),
                    }));
                    continue;
                };

                let action = text_message_action(&conversation_id, &text);
                if let Err(error) = send_action(config, action).await {
                    publish(&events, DomainEvent::ConnectionChanged(ConnectionState::Failed { reason: error.to_string() }));
                }
            }
            ClientCommand::LoadConversations | ClientCommand::LoadMessages { .. } => {}
        }
    }
}

async fn connect_once(config: ConnectionConfig, events: broadcast::Sender<DomainEvent>) -> CoreResult<()> {
    let mut request = config.endpoint.as_str().into_client_request().map_err(|error| CoreError::Gateway(error.to_string()))?;
    if let Some(token) = config.access_token.as_deref() {
        request.headers_mut().insert(
            AUTHORIZATION,
            format!("Bearer {token}").parse().map_err(|error| CoreError::Gateway(format!("invalid access token header: {error}")))?,
        );
    }

    let (stream, _) = connect_async(request).await.map_err(|error| CoreError::Gateway(error.to_string()))?;
    publish(&events, DomainEvent::ConnectionChanged(ConnectionState::Connected));

    let (_, mut read) = stream.split();
    while let Some(frame) = read.next().await {
        let frame = frame.map_err(|error| CoreError::Gateway(error.to_string()))?;
        if let Message::Text(text) = frame {
            match serde_json::from_str::<EventEnvelope>(&text) {
                Ok(event) => {
                    for event in events_from_message(event) {
                        publish(&events, event);
                    }
                }
                Err(error) => publish(&events, DomainEvent::ConnectionChanged(ConnectionState::Failed {
                    reason: format!("failed to decode NapCat event: {error}"),
                })),
            }
        }
    }

    Ok(())
}

async fn send_action(config: ConnectionConfig, action: ActionRequest) -> CoreResult<()> {
    let mut request = config.endpoint.as_str().into_client_request().map_err(|error| CoreError::Gateway(error.to_string()))?;
    if let Some(token) = config.access_token.as_deref() {
        request.headers_mut().insert(
            AUTHORIZATION,
            format!("Bearer {token}").parse().map_err(|error| CoreError::Gateway(format!("invalid access token header: {error}")))?,
        );
    }

    let (mut stream, _) = connect_async(request).await.map_err(|error| CoreError::Gateway(error.to_string()))?;
    let payload = serde_json::to_string(&action).map_err(|error| CoreError::Gateway(error.to_string()))?;
    stream.send(Message::Text(payload.into())).await.map_err(|error| CoreError::Gateway(error.to_string()))?;
    Ok(())
}

fn publish(events: &broadcast::Sender<DomainEvent>, event: DomainEvent) {
    let _ = events.send(event);
}
