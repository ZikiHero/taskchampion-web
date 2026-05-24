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
    #[serde(default)]
    pub project: Option<String>,
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
    #[serde(default)]
    pub project: Option<String>,
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
    pub urgency: f64,
    pub due: Option<DateTime<Utc>>,
    pub project: Option<String>,
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
            urgency: compute_urgency(task),
            due: task.get_due(),
            project: task.get_value("project").map(|v| v.to_string()),
        }
    }
}

fn compute_urgency(task: &Task) -> f64 {
    const PRIORITY_H: f64 = 6.0;
    const PRIORITY_M: f64 = 3.9;
    const PRIORITY_L: f64 = 1.8;
    const PROJECT: f64 = 1.0;
    const WAITING: f64 = -3.0;
    const DUE: f64 = 12.0;

    let mut urgency = 0.0;

    urgency += match task.get_priority() {
        "H" => PRIORITY_H,
        "M" => PRIORITY_M,
        "L" => PRIORITY_L,
        _ => 0.0,
    };

    if task.get_value("project").is_some() {
        urgency += PROJECT;
    }

    if task.is_waiting() {
        urgency += WAITING;
    }

    if let Some(due) = task.get_due() {
        let days_until_due = due.signed_duration_since(Utc::now()).num_seconds() as f64 / 86_400.0;
        let due_factor = if days_until_due <= 0.0 {
            1.0
        } else if days_until_due <= 14.0 {
            ((14.0 - days_until_due) / 14.0).powf(1.4)
        } else {
            0.0
        };

        urgency += DUE * due_factor;
    }

    (urgency * 10.0).round() / 10.0
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub synced_at: String,
}
