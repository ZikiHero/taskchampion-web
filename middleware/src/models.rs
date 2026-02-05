use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use taskchampion::{Task, Uuid};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub description: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub due: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub due: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub uuid: Uuid,
    pub description: String,
    pub status: String,
    pub entry: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub priority: Option<String>,
    pub due: Option<DateTime<Utc>>,
}

impl TaskResponse {
    pub fn from_task(task: &Task) -> Self {
        Self {
            uuid: task.get_uuid(),
            description: task.get_description().to_string(),
            status: format!("{:?}", task.get_status()).to_lowercase(),
            entry: task.get_entry(),
            modified: task.get_modified(),
            tags: task.get_tags().map(|t| t.to_string()).collect(),
            priority: {
                let p = task.get_priority().to_string();
                if p.is_empty() { None } else { Some(p) }
            },
            due: task.get_due(),
        }
    }
}

// ← DIESE STRUCT WAR FEHLEND!
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub synced_at: String,
}
