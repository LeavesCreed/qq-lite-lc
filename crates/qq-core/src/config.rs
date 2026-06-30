use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub endpoint: String,
    pub access_token: Option<String>,
    pub reconnect: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            endpoint: "ws://127.0.0.1:3001".to_owned(),
            access_token: None,
            reconnect: true,
        }
    }
}
