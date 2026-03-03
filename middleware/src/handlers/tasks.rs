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

use axum::extract::Query;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TaskFilter {
    pub tag: Option<String>,
    pub project: Option<String>,
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(filter): Query<TaskFilter>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            tracing::info!("Found {} total tasks in replica", all_tasks.len());
            let tasks: Vec<TaskResponse> = all_tasks
                .values()
                .filter(|task| {
                    let is_not_deleted = task.get_status() != Status::Deleted;
                    
                    // Apply tag filter
                    let matches_tag = if let Some(ref tag_filter) = filter.tag {
                        task.get_tags().any(|t| t.to_string() == *tag_filter)
                    } else {
                        true
                    };

                    // Apply project filter
                    let matches_project = if let Some(ref project_filter) = filter.project {
                        task.get_value("project").map(|v| v == project_filter).unwrap_or(false)
                    } else {
                        true
                    };

                    is_not_deleted && matches_tag && matches_project
                })
                .map(TaskResponse::from_task)
                .collect();
            
            tracing::info!("Returning {} non-deleted tasks", tasks.len());

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

        if let Some(project) = payload.project {
            let value = if project.is_empty() { None } else { Some(project) };
            task.set_value("project".to_string(), value, &mut ops)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::helpers::create_test_state;
    use taskchampion::Status;

    async fn add_test_task_with_tag(state: &AppState, description: &str, tag: Option<&str>, project: Option<&str>) {
        let mut replica = state.replica.lock().await;
        let mut ops = Operations::new();
        let mut task = replica.create_task(taskchampion::Uuid::new_v4(), &mut ops).await.unwrap();
        task.set_description(description.to_string(), &mut ops).unwrap();
        task.set_status(Status::Pending, &mut ops).unwrap();
        if let Some(t) = tag {
            let tag_obj = taskchampion::Tag::try_from(t).unwrap();
            task.add_tag(&tag_obj, &mut ops).unwrap();
        }
        if let Some(p) = project {
            task.set_value("project".to_string(), Some(p.to_string()), &mut ops).unwrap();
        }
        replica.commit_operations(ops).await.unwrap();
    }

    #[tokio::test]
    async fn test_list_tasks_filtering() {
        let state = create_test_state().await;
        
        add_test_task_with_tag(&state, "Task 1", Some("urgent"), Some("work")).await;
        add_test_task_with_tag(&state, "Task 2", Some("home"), Some("personal")).await;
        add_test_task_with_tag(&state, "Task 3", None, Some("work")).await;

        // Test filter by tag
        let filter = TaskFilter { tag: Some("urgent".to_string()), project: None };
        let response = list_tasks(State(state.clone()), Query(filter)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test filter by project
        let filter = TaskFilter { tag: None, project: Some("work".to_string()) };
        let response = list_tasks(State(state.clone()), Query(filter)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // Test filter by both
        let filter = TaskFilter { tag: Some("urgent".to_string()), project: Some("work".to_string()) };
        let response = list_tasks(State(state.clone()), Query(filter)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // Test filter by non-existent tag
        let filter = TaskFilter { tag: Some("nonexistent".to_string()), project: None };
        let response = list_tasks(State(state.clone()), Query(filter)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
