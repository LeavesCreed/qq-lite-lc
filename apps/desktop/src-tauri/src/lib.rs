use qq_core::{ports::ChatStore, ClientCommand, ClientCore, ConnectionConfig, ConversationId, ConversationView, MessageView};
use qq_napcat::NapCatGateway;
use qq_store::SqliteChatStore;
use std::path::PathBuf;
use tauri::{Manager, State};

#[derive(Clone)]
struct AppState {
    core: ClientCore,
    gateway: NapCatGateway,
    store: SqliteChatStore,
}

#[tauri::command]
async fn connect(endpoint: String, access_token: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    state.gateway.execute(ClientCommand::Connect(ConnectionConfig {
        endpoint,
        access_token,
        reconnect: true,
    })).await.map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_conversations(state: State<'_, AppState>) -> Result<Vec<ConversationView>, String> {
    let stored = state.store.list_conversations().await.map_err(|error| error.to_string())?;
    for conversation in stored {
        state.core.dispatch_event(qq_core::DomainEvent::ConversationUpdated(conversation)).await.map_err(|error| error.to_string())?;
    }
    Ok(state.core.conversations().await)
}

#[tauri::command]
async fn list_messages(conversation_id: String, state: State<'_, AppState>) -> Result<Vec<MessageView>, String> {
    let id = ConversationId::new(conversation_id);
    let stored = state.store.list_messages(&id, 200).await.map_err(|error| error.to_string())?;
    for message in stored {
        state.core.dispatch_event(qq_core::DomainEvent::MessageReceived(message)).await.map_err(|error| error.to_string())?;
    }
    Ok(state.core.timeline(&id).await.messages)
}

#[tauri::command]
async fn send_text_message(conversation_id: String, text: String, state: State<'_, AppState>) -> Result<(), String> {
    state.gateway.execute(ClientCommand::SendTextMessage {
        conversation_id: ConversationId::new(conversation_id),
        text,
    }).await.map_err(|error| error.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database_path = app.path().app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("qq-lite-lc.sqlite");
            if let Some(parent) = database_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let store = tauri::async_runtime::block_on(SqliteChatStore::connect_file(&database_path))?;
            let core = ClientCore::new();
            let gateway = NapCatGateway::new();
            let mut gateway_events = gateway.subscribe();
            let bridge_core = core.clone();
            let bridge_store = store.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = gateway_events.recv().await {
                    match &event {
                        qq_core::DomainEvent::ConversationUpdated(conversation) => {
                            let _ = bridge_store.upsert_conversation(conversation).await;
                        }
                        qq_core::DomainEvent::MessageReceived(message) => {
                            let _ = bridge_store.insert_message(message).await;
                        }
                        _ => {}
                    }
                    let _ = bridge_core.dispatch_event(event).await;
                }
            });

            app.manage(AppState { core, gateway, store });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect,
            list_conversations,
            list_messages,
            send_text_message
        ])
        .run(tauri::generate_context!())
        .expect("failed to run QQ Lite LC desktop app");
}
