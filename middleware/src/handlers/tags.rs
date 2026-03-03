use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use taskchampion::{Operations, Status, Tag};
use tracing::error;

use crate::models::{CreateTaskRequest, TaskResponse};
use super::{AppState, helpers::*};

// ============================================
// DATA STRUCTURES
// ============================================

#[derive(Serialize, Debug)]
pub struct TagStats {
    pub name: String,
    pub task_count: usize,
    pub pending_count: usize,
    pub completed_count: usize,
    pub deleted_count: usize,
}

#[derive(Serialize)]
pub struct TagDetails {
    pub name: String,
    pub task_count: usize,
    pub pending_count: usize,
    pub completed_count: usize,
    pub deleted_count: usize,
    pub tasks_preview: Vec<TaskResponse>,
}

// ============================================
// TAG HANDLERS
// ============================================

/// GET /tags - List all unique tags
pub async fn list_tags(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut tags = std::collections::HashSet::new();

    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            tracing::info!("list_tags: Found {} total tasks", all_tasks.len());
            for task in all_tasks.values() {
                // Only count tags from non-deleted tasks
                if task.get_status() != Status::Deleted {
                    for tag in task.get_tags() {
                        tags.insert(tag.to_string());
                    }
                }
            }

            let mut tag_list: Vec<String> = tags.into_iter().collect();
            tag_list.sort();

            Json(tag_list).into_response()
        }
        Err(e) => {
            error!("Failed to list tags: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /tags/:name - Get tag statistics
pub async fn get_tag_stats(
    State(state): State<AppState>,
    Path(tag_name): Path<String>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut task_count = 0;
    let mut pending_count = 0;
    let mut completed_count = 0;
    let mut deleted_count = 0;

    let tag_to_find = match Tag::try_from(tag_name.as_str()) {
        Ok(t) => t,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            for task in all_tasks.values() {
                if task.get_tags().any(|t| t == tag_to_find) {
                    task_count += 1;

                    match task.get_status() {
                        Status::Pending => pending_count += 1,
                        Status::Completed => completed_count += 1,
                        Status::Deleted => deleted_count += 1,
                        _ => {}
                    }
                }
            }

            // Return 404 if tag doesn't exist
            if task_count == 0 {
                return StatusCode::NOT_FOUND.into_response();
            }

            Json(TagStats {
                name: tag_name,
                task_count,
                pending_count,
                completed_count,
                deleted_count,
            }).into_response()
        }
        Err(e) => {
            error!("Failed to get tag stats: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /tags/:name/details - Get tag details with task preview
pub async fn get_tag_details(
    State(state): State<AppState>,
    Path(tag_name): Path<String>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut task_count = 0;
    let mut pending_count = 0;
    let mut completed_count = 0;
    let mut deleted_count = 0;
    let mut tasks_preview = Vec::new();

    let tag_to_find = match Tag::try_from(tag_name.as_str()) {
        Ok(t) => t,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            let mut matching_tasks: Vec<_> = all_tasks
                .values()
                .filter(|task| task.get_tags().any(|t| t == tag_to_find))
                .collect();

            // Return 404 if tag doesn't exist
            if matching_tasks.is_empty() {
                return StatusCode::NOT_FOUND.into_response();
            }

            // Sort by entry time (newest first)
            matching_tasks.sort_by(|a, b| {
                b.get_entry().cmp(&a.get_entry())
            });

            for task in &matching_tasks {
                task_count += 1;

                match task.get_status() {
                    Status::Pending => pending_count += 1,
                    Status::Completed => completed_count += 1,
                    Status::Deleted => deleted_count += 1,
                    _ => {}
                }

                // Only include first 5 tasks in preview
                if tasks_preview.len() < 5 && task.get_status() != Status::Deleted {
                    tasks_preview.push(TaskResponse::from_task(task));
                }
            }

            // Sort preview by entry time (newest first)
            tasks_preview.sort_by(|a, b| b.entry.cmp(&a.entry));

            Json(TagDetails {
                name: tag_name,
                task_count,
                pending_count,
                completed_count,
                deleted_count,
                tasks_preview,
            }).into_response()
        }
        Err(e) => {
            error!("Failed to get tag details: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /tags/:name/tasks - Get all tasks with this tag
pub async fn get_tag_tasks(
    State(state): State<AppState>,
    Path(tag_name): Path<String>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let tag_to_find = match Tag::try_from(tag_name.as_str()) {
        Ok(t) => t,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            let mut matching_tasks: Vec<_> = all_tasks
                .values()
                .filter(|task| {
                    task.get_status() != Status::Deleted &&
                        task.get_tags().any(|t| t == tag_to_find)
                })
                .collect();

            // Return 404 if tag doesn't exist
            if matching_tasks.is_empty() {
                return StatusCode::NOT_FOUND.into_response();
            }

            // Sort by entry time (newest first)
            matching_tasks.sort_by(|a, b| {
                b.get_entry().cmp(&a.get_entry())
            });

            let tasks: Vec<TaskResponse> = matching_tasks
                .iter()
                .map(|task| TaskResponse::from_task(task))
                .collect();

            Json(tasks).into_response()
        }
        Err(e) => {
            error!("Failed to get tag tasks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// POST /tags/:name/tasks - Create a new task with this tag
pub async fn create_task_with_tag(
    State(state): State<AppState>,
    Path(tag_name): Path<String>,
    Json(mut payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    // Validate tag name
    if let Err(_) = Tag::try_from(tag_name.as_str()) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // Add tag to payload
    let mut tags = payload.tags.unwrap_or_default();
    if !tags.contains(&tag_name) {
        tags.push(tag_name);
    }
    payload.tags = Some(tags);

    // Reuse the existing create_task handler logic
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
        "Auto-sync failed after create with tag"
    ).await {
        return status.into_response();
    }

    Json(response).into_response()
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_list_tags_empty() {
        let state = create_test_state().await;
        let response = list_tags(State(state)).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_tag_stats_not_found() {
        let state = create_test_state().await;
        let response = get_tag_stats(
            State(state),
            Path("nonexistent".to_string())
        ).await.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_task_with_tag() {
        let state = create_test_state().await;

        let payload = CreateTaskRequest {
            description: "Test task".to_string(),
            tags: None,
            priority: None,
            due: None,
            project: None,
        };

        let response = create_task_with_tag(
            State(state.clone()),
            Path("urgent".to_string()),
            Json(payload)
        ).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
