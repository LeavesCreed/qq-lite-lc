use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventEnvelope {
    pub post_type: Option<String>,
    pub message_type: Option<String>,
    pub sub_type: Option<String>,
    pub time: Option<i64>,
    pub self_id: Option<i64>,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    pub message_id: Option<i64>,
    pub message: Option<Vec<MessageSegment>>,
    pub raw_message: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageSegment {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub data: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionRequest {
    pub action: String,
    pub params: Value,
    pub echo: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionResponse {
    pub status: Option<String>,
    pub retcode: Option<i64>,
    pub data: Option<Value>,
    pub message: Option<String>,
    pub wording: Option<String>,
    pub echo: Option<String>,
}
