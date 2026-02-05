use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use taskchampion::{Replica, SqliteStorage, Uuid, Operations, Tag, Status};
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::models::{CreateTaskRequest, TaskResponse, UpdateTaskRequest, SyncResponse};
use crate::ServerWrapper;

#[derive(Clone)]
pub struct AppState {
    pub replica: Arc<Mutex<Replica<SqliteStorage>>>,
    pub server: Arc<Mutex<ServerWrapper>>,
    pub auto_sync: bool,
}

// ============================================
// HELPER: Auto-Sync
// ============================================

use std::task::{Context, Poll};

struct UnsafeSendFuture<F>(F);
unsafe impl<F> Send for UnsafeSendFuture<F> {}
impl<F: Future> Future for UnsafeSendFuture<F> {
    type Output = F::Output;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
    }
}

async fn get_task_from_replica(
    replica: &mut Replica<SqliteStorage>,
    uuid: Uuid,
) -> Result<taskchampion::Task, StatusCode> {
    match replica.get_task(uuid).await {
        Ok(Some(task)) => {
            if task.get_status() == Status::Deleted {
                return Err(StatusCode::NOT_FOUND);
            }
            Ok(task)
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn commit_and_sync(
    mut replica: tokio::sync::MutexGuard<'_, Replica<SqliteStorage>>,
    ops: Operations,
    state: AppState,
    sync_err_msg: &str,
) -> Result<(), StatusCode> {
    replica.commit_operations(ops).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    drop(replica);
    
    if let Err(_) = auto_sync_if_enabled(state).await {
        error!("{}", sync_err_msg);
    }
    Ok(())
}

async fn perform_sync(state: &AppState) -> Result<(), String> {
    let mut replica = state.replica.lock().await;
    let mut server = state.server.lock().await;
    
    let sync_fut = replica.sync(&mut **server, false);
    match UnsafeSendFuture(sync_fut).await {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            error!("❌ Sync failed: {}", e);
            Err(format!("Sync failed: {}", e))
        }
    }
}

fn map_status(status_str: &str) -> Result<Status, StatusCode> {
    match status_str.to_lowercase().as_str() {
        "pending" => Ok(Status::Pending),
        "completed" => Ok(Status::Completed),
        "deleted" => Ok(Status::Deleted),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn apply_tags(task: &mut taskchampion::Task, tags: Vec<String>, ops: &mut Operations) -> Result<(), StatusCode> {
    for tag_name in tags {
        let tag = Tag::try_from(tag_name.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;
        task.add_tag(&tag, ops).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(())
}

async fn auto_sync_if_enabled(
    state: AppState,
) -> Result<(), String> {
    if !state.auto_sync {
        return Ok(());
    }
    
    info!("🔄 Auto-sync triggered...");
    match perform_sync(&state).await {
        Ok(_) => {
            info!("✅ Auto-sync completed");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// ============================================
// HANDLERS - ALLE mit impl IntoResponse
// ============================================

async fn get_task_response(
    replica: &mut Replica<SqliteStorage>,
    uuid: Uuid,
) -> axum::response::Response {
    match get_task_from_replica(replica, uuid).await {
        Ok(task) => (StatusCode::OK, Json(TaskResponse::from_task(&task))).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn trigger_sync(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("🔄 Manual sync triggered...");
    
    match perform_sync(&state).await {
        Ok(_) => {
            info!("✅ Manual sync completed");
            (StatusCode::OK, Json(SyncResponse {
                success: true,
                message: "Sync completed successfully".to_string(),
                synced_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(SyncResponse {
                success: false,
                message: e,
                synced_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
    }
}

pub async fn list_tasks(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    
    match replica.all_tasks().await {
        Ok(all_tasks) => {
            let tasks: Vec<TaskResponse> = all_tasks
                .iter()
                .filter(|(_uuid, task)| task.get_status() != Status::Deleted)
                .map(|(_uuid, task)| TaskResponse::from_task(task))
                .collect();
            
            (StatusCode::OK, Json(tasks)).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> axum::response::Response {
    let mut replica = state.replica.lock().await;
    get_task_response(&mut replica, uuid).await
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();
    
    let mut task = match replica.create_task(Uuid::new_v4(), &mut ops).await {
        Ok(t) => t,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    if let Err(_) = task.set_status(Status::Pending, &mut ops) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    if let Err(_) = task.set_description(payload.description.clone(), &mut ops) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    if let Some(tags) = payload.tags {
        if let Err(status) = apply_tags(&mut task, tags, &mut ops) {
            return status.into_response();
        }
    }
    
    let response = TaskResponse::from_task(&task);

    if let Err(status) = commit_and_sync(replica, ops, state.clone(), "Auto-sync failed after create").await {
        return status.into_response();
    }
    
    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> axum::response::Response {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();
    
    let mut task = match get_task_from_replica(&mut replica, uuid).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };
    
    if let Some(description) = payload.description {
        if let Err(_) = task.set_description(description, &mut ops) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    
    if let Some(status_str) = payload.status {
        let status = match map_status(&status_str) {
            Ok(s) => s,
            Err(e) => return e.into_response(),
        };
        if let Err(_) = task.set_status(status, &mut ops) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    
    if let Some(tags) = payload.tags {
        let current_tags: Vec<Tag> = task.get_tags().collect();
        
        for tag in &current_tags {
            if let Err(_) = task.remove_tag(tag, &mut ops) {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        
        if let Err(status) = apply_tags(&mut task, tags, &mut ops) {
            return status.into_response();
        }
    }
    
    if let Err(status) = commit_and_sync(replica, ops, state.clone(), "Auto-sync failed after update").await {
        return status.into_response();
    }
    
    let mut replica = state.replica.lock().await;
    get_task_response(&mut replica, uuid).await
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();
    
    let mut task = match get_task_from_replica(&mut replica, uuid).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };
    
    if let Err(_) = task.set_status(Status::Deleted, &mut ops) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    if let Err(status) = commit_and_sync(replica, ops, state.clone(), "Auto-sync failed after delete").await {
        return status.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
