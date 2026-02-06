use axum::http::StatusCode;
use std::future::Future;
use std::task::{Context, Poll};
use taskchampion::{Uuid, Operations, Tag, Status, Task, Replica, SqliteStorage};
use tracing::error;

use crate::models::CreateTaskRequest;
use super::AppState;

// ============================================
// UNSAFE SEND WRAPPER
// ============================================

pub struct UnsafeSendFuture<F>(pub F);
unsafe impl<F> Send for UnsafeSendFuture<F> {}

impl<F: Future> Future for UnsafeSendFuture<F> {
    type Output = F::Output;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
    }
}

// ============================================
// HELPER FUNCTIONS
// ============================================

pub async fn get_task_from_replica(
    replica: &mut Replica<SqliteStorage>,
    uuid: Uuid,
) -> Result<Task, StatusCode> {
    match replica.get_task(uuid).await {
        Ok(Some(task)) if task.get_status() != Status::Deleted => Ok(task),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn commit_and_sync(
    mut replica: tokio::sync::MutexGuard<'_, Replica<SqliteStorage>>,
    ops: Operations,
    state: AppState,
    sync_err_msg: &str,
) -> Result<(), StatusCode> {
    replica.commit_operations(ops).await.map_err(|e| {
        error!("Failed to commit operations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    drop(replica);

    if let Err(e) = auto_sync_if_enabled(state).await {
        error!("{}: {}", sync_err_msg, e);
    }
    Ok(())
}

pub async fn perform_sync(state: &AppState) -> Result<(), String> {
    let mut replica = state.replica.lock().await;
    let mut server = state.server.lock().await;

    let sync_fut = replica.sync(&mut **server, false);
    match UnsafeSendFuture(sync_fut).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("❌ Sync failed: {}", e);
            Err(format!("Sync failed: {}", e))
        }
    }
}

pub fn map_status(status_str: &str) -> Result<Status, StatusCode> {
    match status_str.to_lowercase().as_str() {
        "pending" => Ok(Status::Pending),
        "completed" => Ok(Status::Completed),
        "deleted" => Ok(Status::Deleted),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

pub fn map_priority(priority_str: &str) -> Result<crate::handlers::Priority, StatusCode> {
    crate::handlers::Priority::try_from(priority_str)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

pub fn apply_tags(
    task: &mut Task,
    tags: Vec<String>,
    ops: &mut Operations
) -> Result<(), StatusCode> {
    // Clear existing user-defined tags first
    let current_tags: Vec<Tag> = task.get_tags().collect();
    for tag in current_tags {
        if tag.is_user() {
            task.remove_tag(&tag, ops)
                .map_err(|e| {
                    error!("apply_tags remove_tag failed: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
    }

    // Add new tags
    for tag_name in tags {
        let tag = Tag::try_from(tag_name.as_str())
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        if !tag.is_user() {
             return Err(StatusCode::BAD_REQUEST);
        }

        task.add_tag(&tag, ops)
            .map_err(|e| {
                error!("apply_tags add_tag failed for {}: {}", tag_name, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }
    Ok(())
}

pub async fn create_new_task(
    replica: &mut Replica<SqliteStorage>,
    ops: &mut Operations,
) -> Result<Task, StatusCode> {
    match replica.create_task(Uuid::new_v4(), ops).await {
        Ok(t) => Ok(t),
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn fill_task_from_create_request(
    task: &mut Task,
    payload: CreateTaskRequest,
    ops: &mut Operations,
) -> Result<(), StatusCode> {
    // Set basic properties
    task.set_status(Status::Pending, ops).map_err(|e| {
        error!("Failed to set status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    task.set_description(payload.description, ops).map_err(|e| {
        error!("Failed to set description: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Apply tags
    if let Some(tags) = payload.tags {
        apply_tags(task, tags, ops)?;
    }

    // Apply optional fields
    if let Some(priority_str) = payload.priority {
        let priority = map_priority(&priority_str)?;
        task.set_priority(priority.into(), ops).map_err(|e| {
            error!("Failed to set priority: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    if let Some(due) = payload.due {
        task.set_due(Some(due), ops).map_err(|e| {
            error!("Failed to set due: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    Ok(())
}

pub async fn get_all_tasks(
    replica: &mut Replica<SqliteStorage>,
) -> Result<std::collections::HashMap<Uuid, Task>, StatusCode> {
    match replica.all_tasks().await {
        Ok(tasks) => Ok(tasks),
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn auto_sync_if_enabled(state: AppState) -> Result<(), String> {
    if !state.auto_sync {
        return Ok(());
    }

    tracing::info!("🔄 Auto-sync triggered...");
    perform_sync(&state).await?;
    tracing::info!("✅ Auto-sync completed");
    Ok(())
}

#[cfg(test)]
pub async fn create_test_state() -> AppState {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use taskchampion::{Replica, SqliteStorage, storage::AccessMode};

    let storage = SqliteStorage::new("testdata".to_string(), AccessMode::ReadWrite, true).await.unwrap();
    let replica = Replica::new(storage);
    let server = crate::ServerWrapper::new_in_memory();

    AppState {
        replica: Arc::new(Mutex::new(replica)),
        server: Arc::new(Mutex::new(server)),
        auto_sync: false,
    }
}

#[cfg(test)]
pub async fn add_test_task(
    state: &AppState,
    description: &str,
    project: Option<&str>,
    status: Option<Status>,
) {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();
    let mut task = replica
        .create_task(Uuid::new_v4(), &mut ops)
        .await
        .unwrap();
    
    task.set_description(description.to_string(), &mut ops).unwrap();
    
    if let Some(p) = project {
        task.set_value("project".to_string(), Some(p.to_string()), &mut ops).unwrap();
    }
    
    if let Some(s) = status {
        task.set_status(s, &mut ops).unwrap();
    }
    
    replica.commit_operations(ops).await.unwrap();
}
