use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use taskchampion::{Uuid, Operations, Status};
use tracing::error;

use crate::models::{CreateTaskRequest, TaskResponse, UpdateTaskRequest};
use super::{AppState, helpers::*};

// ============================================
// TASK HANDLERS
// ============================================

pub async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            let tasks: Vec<TaskResponse> = all_tasks
                .values()
                .filter(|task| task.get_status() != Status::Deleted)
                .map(TaskResponse::from_task)
                .collect();

            Json(tasks).into_response()
        }
        Err(status) => status.into_response(),
    }
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    match get_task_from_replica(&mut replica, uuid).await {
        Ok(task) => Json(TaskResponse::from_task(&task)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = Operations::new();

    let mut task = match create_new_task(&mut replica, &mut ops).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };

    if let Err(status) = fill_task_from_create_request(&mut task, payload, &mut ops) {
        return status.into_response();
    }

    let response = TaskResponse::from_task(&task);

    if let Err(status) = commit_and_sync(
        replica,
        ops,
        state.clone(),
        "Auto-sync failed after create"
    ).await {
        return status.into_response();
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

    let mut task = match get_task_from_replica(&mut replica, uuid).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };

    let result: Result<(), StatusCode> = (|| {
        if let Some(description) = payload.description {
            task.set_description(description, &mut ops)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        if let Some(status_str) = payload.status {
            let status = map_status(&status_str)?;
            task.set_status(status, &mut ops)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        if let Some(tags) = payload.tags {
            apply_tags(&mut task, tags, &mut ops)?;
        }

        if let Some(priority_str) = payload.priority {
            let priority = map_priority(&priority_str)?;
            task.set_priority(priority.into(), &mut ops)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        if let Some(due) = payload.due {
            task.set_due(Some(due), &mut ops)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        Ok(())
    })();

    if let Err(status) = result {
        return status.into_response();
    }

    if let Err(status) = commit_and_sync(
        replica,
        ops,
        state.clone(),
        "Auto-sync failed after update"
    ).await {
        return status.into_response();
    }

    let mut replica = state.replica.lock().await;
    match get_task_from_replica(&mut replica, uuid).await {
        Ok(task) => Json(TaskResponse::from_task(&task)).into_response(),
        Err(status) => status.into_response(),
    }
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

    if let Err(e) = task.set_status(Status::Deleted, &mut ops) {
        error!("Failed to set task status to deleted: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(status) = commit_and_sync(
        replica,
        ops,
        state.clone(),
        "Auto-sync failed after delete"
    ).await {
        return status.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
