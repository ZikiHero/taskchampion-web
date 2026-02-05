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
impl<F: std::future::Future> std::future::Future for UnsafeSendFuture<F> {
    type Output = F::Output;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
    }
}

async fn auto_sync_if_enabled(
    state: AppState,
) -> Result<(), String> {
    if !state.auto_sync {
        return Ok(());
    }
    
    info!("🔄 Auto-sync triggered...");
    let mut replica = state.replica.lock().await;
    let mut server = state.server.lock().await;
    
    let sync_fut = replica.sync(&mut **server, false);
    match UnsafeSendFuture(sync_fut).await {
        Ok(_) => {
            info!("✅ Auto-sync completed");
            Ok(())
        }
        Err(e) => {
            error!("❌ Auto-sync failed: {}", e);
            Err(format!("Sync failed: {}", e))
        }
    }
}

// ============================================
// HANDLERS - ALLE mit impl IntoResponse
// ============================================

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
    
    let mut replica = state.replica.lock().await;
    let mut server = state.server.lock().await;
    
    let sync_fut = replica.sync(&mut **server, false);
    match UnsafeSendFuture(sync_fut).await {
        Ok(_) => {
            info!("✅ Manual sync completed");
            (StatusCode::OK, Json(SyncResponse {
                success: true,
                message: "Sync completed successfully".to_string(),
                synced_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("❌ Manual sync failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(SyncResponse {
                success: false,
                message: format!("Sync failed: {}", e),
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
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    
    match replica.get_task(uuid).await {
        Ok(Some(task)) => {
            if task.get_status() == Status::Deleted {
                return StatusCode::NOT_FOUND.into_response();
            }
            (StatusCode::OK, Json(TaskResponse::from_task(&task))).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
        for tag_name in tags {
            let tag = match Tag::try_from(tag_name.as_str()) {
                Ok(t) => t,
                Err(_) => return StatusCode::BAD_REQUEST.into_response(),
            };
            if let Err(_) = task.add_tag(&tag, &mut ops) {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    
    if let Err(_) = replica.commit_operations(ops).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    let response = TaskResponse::from_task(&task);
    
    drop(replica);
    
    if let Err(_) = auto_sync_if_enabled(state.clone()).await {
        // Log but don't fail the request
        error!("Auto-sync failed after create");
    }
    
    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();
    
    let mut task = match replica.get_task(uuid).await {
        Ok(Some(t)) => t,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    if let Some(description) = payload.description {
        if let Err(_) = task.set_description(description, &mut ops) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    
    if let Some(status_str) = payload.status {
        let status = match status_str.to_lowercase().as_str() {
            "pending" => Status::Pending,
            "completed" => Status::Completed,
            "deleted" => Status::Deleted,
            _ => return StatusCode::BAD_REQUEST.into_response(),
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
        
        for tag_name in tags {
            let tag = match Tag::try_from(tag_name.as_str()) {
                Ok(t) => t,
                Err(_) => return StatusCode::BAD_REQUEST.into_response(),
            };
            if let Err(_) = task.add_tag(&tag, &mut ops) {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    
    if let Err(_) = replica.commit_operations(ops).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    let task = match replica.get_task(uuid).await {
        Ok(Some(t)) => t,
        _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    let response = TaskResponse::from_task(&task);
    
    drop(replica);
    
    if let Err(_) = auto_sync_if_enabled(state.clone()).await {
        error!("Auto-sync failed after update");
    }
    
    (StatusCode::OK, Json(response)).into_response()
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();
    
    let mut task = match replica.get_task(uuid).await {
        Ok(Some(t)) => t,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    if let Err(_) = task.set_status(Status::Deleted, &mut ops) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    if let Err(_) = replica.commit_operations(ops).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    drop(replica);
    
    if let Err(_) = auto_sync_if_enabled(state.clone()).await {
        error!("Auto-sync failed after delete");
    }

    StatusCode::NO_CONTENT.into_response()
}
