// handlers/mod.rs

mod helpers;
mod system;
mod tasks;
mod projects;
mod tags;

pub use system::*;
pub use tasks::*;
pub use projects::*;
pub use tags::*;

use std::sync::Arc;
use taskchampion::{Replica, SqliteStorage};
use tokio::sync::Mutex;

use crate::ServerWrapper;

// ============================================
// PRIORITY ENUM
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    #[serde(rename = "H")]
    High,
    #[serde(rename = "M")]
    Medium,
    #[serde(rename = "L")]
    Low,
    #[serde(rename = "")]
    None,
}

impl TryFrom<&str> for Priority {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "H" | "HIGH" => Ok(Priority::High),
            "M" | "MEDIUM" => Ok(Priority::Medium),
            "L" | "LOW" => Ok(Priority::Low),
            "" | "NONE" => Ok(Priority::None),
            _ => Err(()),
        }
    }
}

impl From<Priority> for String {
    fn from(priority: Priority) -> Self {
        match priority {
            Priority::High => "H".to_string(),
            Priority::Medium => "M".to_string(),
            Priority::Low => "L".to_string(),
            Priority::None => "".to_string(),
        }
    }
}

// ============================================
// APP STATE
// ============================================

#[derive(Clone)]
pub struct AppState {
    pub replica: Arc<Mutex<Replica<SqliteStorage>>>,
    pub server: Arc<Mutex<ServerWrapper>>,
    pub auto_sync: bool,
}
