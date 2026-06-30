use async_trait::async_trait;
use qq_core::{ports::ChatStore, Conversation, ConversationId, ConversationKind, CoreError, CoreResult, LocalMessageId, Message, MessageDirection, RemoteMessageId, RichNode, SendState, UserId};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Row, SqlitePool};
use std::path::Path;

#[derive(Clone)]
pub struct SqliteChatStore {
    pool: SqlitePool,
}

impl SqliteChatStore {
    pub async fn connect(database_url: &str) -> CoreResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|error| CoreError::Store(error.to_string()))?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    pub async fn connect_file(path: impl AsRef<Path>) -> CoreResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|error| CoreError::Store(error.to_string()))?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    pub async fn migrate(&self) -> CoreResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                last_message_preview TEXT,
                updated_at_ms INTEGER NOT NULL,
                unread_count INTEGER NOT NULL DEFAULT 0
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|error| CoreError::Store(error.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                local_id TEXT PRIMARY KEY NOT NULL,
                remote_id TEXT,
                conversation_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                sender_name TEXT,
                direction TEXT NOT NULL,
                sent_at_ms INTEGER NOT NULL,
                nodes_json TEXT NOT NULL,
                raw_json TEXT,
                send_state_json TEXT NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id)
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|error| CoreError::Store(error.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_conversation_time ON messages(conversation_id, sent_at_ms);")
            .execute(&self.pool)
            .await
            .map_err(|error| CoreError::Store(error.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl ChatStore for SqliteChatStore {
    async fn upsert_conversation(&self, conversation: &Conversation) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO conversations (id, kind, title, last_message_preview, updated_at_ms, unread_count)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                title = excluded.title,
                last_message_preview = excluded.last_message_preview,
                updated_at_ms = excluded.updated_at_ms,
                unread_count = excluded.unread_count;
            "#,
        )
        .bind(conversation.id.as_str())
        .bind(kind_to_str(&conversation.kind))
        .bind(&conversation.title)
        .bind(&conversation.last_message_preview)
        .bind(conversation.updated_at_ms)
        .bind(i64::from(conversation.unread_count))
        .execute(&self.pool)
        .await
        .map_err(|error| CoreError::Store(error.to_string()))?;
        Ok(())
    }

    async fn insert_message(&self, message: &Message) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO messages
                (local_id, remote_id, conversation_id, sender_id, sender_name, direction, sent_at_ms, nodes_json, raw_json, send_state_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
            "#,
        )
        .bind(message.local_id.as_str())
        .bind(message.remote_id.as_ref().map(|id| id.as_str()))
        .bind(message.conversation_id.as_str())
        .bind(message.sender_id.as_str())
        .bind(&message.sender_name)
        .bind(format!("{:?}", message.direction))
        .bind(message.sent_at_ms)
        .bind(serde_json::to_string(&message.nodes).map_err(|error| CoreError::Store(error.to_string()))?)
        .bind(message.raw_json.as_ref().map(|value| value.to_string()))
        .bind(serde_json::to_string(&message.send_state).map_err(|error| CoreError::Store(error.to_string()))?)
        .execute(&self.pool)
        .await
        .map_err(|error| CoreError::Store(error.to_string()))?;
        Ok(())
    }

    async fn list_conversations(&self) -> CoreResult<Vec<Conversation>> {
        let rows = sqlx::query(
            r#"
            SELECT id, kind, title, last_message_preview, updated_at_ms, unread_count
            FROM conversations
            ORDER BY updated_at_ms DESC;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| CoreError::Store(error.to_string()))?;

        rows.into_iter()
            .map(|row| {
                Ok(Conversation {
                    id: ConversationId::new(row.try_get::<String, _>("id").map_err(store_error)?),
                    kind: parse_kind(&row.try_get::<String, _>("kind").map_err(store_error)?),
                    title: row.try_get("title").map_err(store_error)?,
                    last_message_preview: row.try_get("last_message_preview").map_err(store_error)?,
                    updated_at_ms: row.try_get("updated_at_ms").map_err(store_error)?,
                    unread_count: row.try_get::<i64, _>("unread_count").map_err(store_error)? as u32,
                })
            })
            .collect()
    }

    async fn list_messages(&self, conversation_id: &ConversationId, limit: usize) -> CoreResult<Vec<Message>> {
        let rows = sqlx::query(
            r#"
            SELECT local_id, remote_id, conversation_id, sender_id, sender_name, direction, sent_at_ms, nodes_json, raw_json, send_state_json
            FROM messages
            WHERE conversation_id = ?1
            ORDER BY sent_at_ms DESC
            LIMIT ?2;
            "#,
        )
        .bind(conversation_id.as_str())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| CoreError::Store(error.to_string()))?;

        let mut messages = rows.into_iter()
            .map(|row| {
                let nodes_json: String = row.try_get("nodes_json").map_err(store_error)?;
                let raw_json: Option<String> = row.try_get("raw_json").map_err(store_error)?;
                let send_state_json: String = row.try_get("send_state_json").map_err(store_error)?;
                let remote_id: Option<String> = row.try_get("remote_id").map_err(store_error)?;

                Ok(Message {
                    local_id: LocalMessageId::new(row.try_get::<String, _>("local_id").map_err(store_error)?),
                    remote_id: remote_id.map(RemoteMessageId::new),
                    conversation_id: ConversationId::new(row.try_get::<String, _>("conversation_id").map_err(store_error)?),
                    sender_id: UserId::new(row.try_get::<String, _>("sender_id").map_err(store_error)?),
                    sender_name: row.try_get("sender_name").map_err(store_error)?,
                    direction: parse_direction(&row.try_get::<String, _>("direction").map_err(store_error)?),
                    sent_at_ms: row.try_get("sent_at_ms").map_err(store_error)?,
                    nodes: serde_json::from_str::<Vec<RichNode>>(&nodes_json).map_err(|error| CoreError::Store(error.to_string()))?,
                    raw_json: raw_json.map(|value| serde_json::from_str(&value)).transpose().map_err(|error| CoreError::Store(error.to_string()))?,
                    send_state: serde_json::from_str::<SendState>(&send_state_json).map_err(|error| CoreError::Store(error.to_string()))?,
                })
            })
            .collect::<CoreResult<Vec<_>>>()?;
        messages.reverse();
        Ok(messages)
    }
}

fn kind_to_str(kind: &ConversationKind) -> &'static str {
    match kind {
        ConversationKind::Private => "private",
        ConversationKind::Group => "group",
    }
}

fn parse_kind(value: &str) -> ConversationKind {
    match value {
        "group" => ConversationKind::Group,
        _ => ConversationKind::Private,
    }
}

fn parse_direction(value: &str) -> MessageDirection {
    match value {
        "Outgoing" => MessageDirection::Outgoing,
        _ => MessageDirection::Incoming,
    }
}

fn store_error(error: sqlx::Error) -> CoreError {
    CoreError::Store(error.to_string())
}
