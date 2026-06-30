use crate::dto::{ActionRequest, EventEnvelope, MessageSegment};
use qq_core::{Conversation, ConversationId, ConversationKind, DomainEvent, LocalMessageId, Message, MessageDirection, RemoteMessageId, RichNode, SendState, UserId};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn conversation_event_from_message(event: &EventEnvelope) -> Option<DomainEvent> {
    if event.post_type.as_deref() != Some("message") {
        return None;
    }

    let message_type = event.message_type.as_deref()?;
    let user_id = event.user_id?;
    let timestamp_ms = event.time.unwrap_or_default() * 1000;
    let (conversation_id, kind, title) = match (message_type, event.group_id) {
        ("group", Some(group_id)) => (
            ConversationId::new(format!("group:{group_id}")),
            ConversationKind::Group,
            format!("Group {group_id}"),
        ),
        ("private", _) => (
            ConversationId::new(format!("private:{user_id}")),
            ConversationKind::Private,
            format!("User {user_id}"),
        ),
        _ => return None,
    };

    Some(DomainEvent::ConversationUpdated(Conversation {
        id: conversation_id,
        kind,
        title,
        last_message_preview: None,
        updated_at_ms: timestamp_ms,
        unread_count: 0,
    }))
}

pub fn message_event_from_message(event: EventEnvelope) -> Option<DomainEvent> {
    if event.post_type.as_deref() != Some("message") {
        return None;
    }

    let message_type = event.message_type.as_deref()?;
    let user_id = event.user_id?;
    let timestamp_ms = event.time.unwrap_or_default() * 1000;
    let conversation_id = match (message_type, event.group_id) {
        ("group", Some(group_id)) => ConversationId::new(format!("group:{group_id}")),
        ("private", _) => ConversationId::new(format!("private:{user_id}")),
        _ => return None,
    };
    let nodes = event.message.as_deref().map(segments_to_nodes).unwrap_or_else(|| {
        vec![RichNode::Text {
            text: event.raw_message.clone().unwrap_or_default(),
        }]
    });

    Some(DomainEvent::MessageReceived(Message {
        local_id: LocalMessageId::generated(),
        remote_id: event.message_id.map(|id| RemoteMessageId::new(id.to_string())),
        conversation_id,
        sender_id: UserId::new(user_id.to_string()),
        sender_name: None,
        direction: MessageDirection::Incoming,
        sent_at_ms: timestamp_ms,
        nodes,
        raw_json: serde_json::to_value(&event).ok(),
        send_state: SendState::Received,
    }))
}

pub fn events_from_message(event: EventEnvelope) -> Vec<DomainEvent> {
    let mut events = Vec::new();
    if let Some(conversation) = conversation_event_from_message(&event) {
        events.push(conversation);
    }
    if let Some(message) = message_event_from_message(event) {
        events.push(message);
    }
    events
}

pub fn text_message_action(conversation_id: &ConversationId, text: &str) -> ActionRequest {
    let id = conversation_id.as_str();
    let (action, target_key, target_id) = if let Some(group_id) = id.strip_prefix("group:") {
        ("send_group_msg", "group_id", group_id)
    } else if let Some(user_id) = id.strip_prefix("private:") {
        ("send_private_msg", "user_id", user_id)
    } else {
        ("send_msg", "user_id", id)
    };

    let mut params = json!({
        "message": [
            {
                "type": "text",
                "data": { "text": text }
            }
        ]
    });
    if let Value::Object(map) = &mut params {
        let target = target_id
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or_else(|_| Value::String(target_id.to_owned()));
        map.insert(target_key.to_owned(), target);
    }

    ActionRequest {
        action: action.to_owned(),
        params,
        echo: Uuid::new_v4().to_string(),
    }
}

fn segments_to_nodes(segments: &[MessageSegment]) -> Vec<RichNode> {
    segments.iter().map(segment_to_node).collect()
}

fn segment_to_node(segment: &MessageSegment) -> RichNode {
    match segment.kind.as_str() {
        "text" => RichNode::Text {
            text: string_data(segment, "text").unwrap_or_default(),
        },
        kind => RichNode::Unsupported {
            kind: kind.to_owned(),
            summary: format!("[{kind}]"),
        },
    }
}

fn string_data(segment: &MessageSegment, key: &str) -> Option<String> {
    match segment.data.get(key) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(value) => Some(value.to_string()),
        None => None,
    }
}
