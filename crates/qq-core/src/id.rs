use serde::{Deserialize, Serialize};
use std::{fmt, sync::atomic::{AtomicU64, Ordering}};
use uuid::Uuid;

static LOCAL_COUNTER: AtomicU64 = AtomicU64::new(1);

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(ConversationId);
id_type!(LocalMessageId);
id_type!(RemoteMessageId);
id_type!(UserId);

impl LocalMessageId {
    pub fn generated() -> Self {
        let counter = LOCAL_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("local-{}-{}", Uuid::new_v4(), counter))
    }
}
