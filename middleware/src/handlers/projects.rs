use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use taskchampion::Status;
use tracing::error;

use crate::models::{CreateTaskRequest, TaskResponse};
use super::{AppState, helpers::*};

#[derive(Serialize)]
pub struct ProjectStats {
    pub name: String,
    pub task_count: usize,
    pub pending_count: usize,
    pub completed_count: usize,
    pub deleted_count: usize,
}

#[derive(Serialize)]
pub struct ProjectDetails {
    pub name: String,
    pub task_count: usize,
    pub pending_count: usize,
    pub completed_count: usize,
    pub deleted_count: usize,
    pub tasks_preview: Vec<TaskResponse>,
}

/// GET /projects - List all unique projects
pub async fn list_projects(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut projects = std::collections::HashSet::new();
    match get_all_tasks(&mut replica).await {
        Ok(all_tasks) => {
            tracing::info!("list_projects: Found {} total tasks", all_tasks.len());
            for task in all_tasks.values() {
                if let Some(project) = task.get_value("project") {
                    projects.insert(project.to_string());
                }
            }
        },
        Err(status) => return status.into_response(),
    };

    let mut project_list: Vec<String> = projects.into_iter().collect();
    project_list.sort();

    Json(project_list).into_response()
}

/// GET /projects/:name - Get project details and stats
pub async fn get_project_stats(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut task_count = 0;
    let mut pending_count = 0;
    let mut completed_count = 0;
    let mut deleted_count = 0;

    let all_tasks = match get_all_tasks(&mut replica).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };

    for task in all_tasks.values() {
        if let Some(project) = task.get_value("project") {
            if project == project_name {
                task_count += 1;

                match task.get_status() {
                    Status::Pending => pending_count += 1,
                    Status::Completed => completed_count += 1,
                    Status::Deleted => deleted_count += 1,
                    _ => {}
                }
            }
        }
    }

    if task_count == 0 {
        return StatusCode::NOT_FOUND.into_response();
    }

    Json(ProjectStats {
        name: project_name,
        task_count,
        pending_count,
        completed_count,
        deleted_count,
    }).into_response()
}

/// GET /projects/:name/tasks - Get all tasks in project
pub async fn get_project_tasks(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut tasks = Vec::new();

    let all_tasks = match get_all_tasks(&mut replica).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };

    for task in all_tasks.values() {
        if let Some(project) = task.get_value("project") {
            if project == project_name {
                tasks.push(TaskResponse::from_task(task));
            }
        }
    }

    // Sort by entry time (newest first)
    tasks.sort_by(|a, b| b.entry.cmp(&a.entry));

    Json(tasks).into_response()
}

/// GET /projects/:name/details - Get project with task preview
pub async fn get_project_details(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;

    let mut task_count = 0;
    let mut pending_count = 0;
    let mut completed_count = 0;
    let mut deleted_count = 0;
    let mut tasks_preview = Vec::new();

    let all_tasks = match get_all_tasks(&mut replica).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };

    for task in all_tasks.values() {
        if let Some(project) = task.get_value("project") {
            if project == project_name {
                task_count += 1;

                match task.get_status() {
                    Status::Pending => {
                        pending_count += 1;
                        // Only include pending tasks in preview
                        if tasks_preview.len() < 10 {
                            tasks_preview.push(TaskResponse::from_task(task));
                        }
                    }
                    Status::Completed => completed_count += 1,
                    Status::Deleted => deleted_count += 1,
                    _ => {}
                }
            }
        }
    }

    if task_count == 0 {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Sort preview by entry time (newest first)
    tasks_preview.sort_by(|a, b| b.entry.cmp(&a.entry));

    Json(ProjectDetails {
        name: project_name,
        task_count,
        pending_count,
        completed_count,
        deleted_count,
        tasks_preview,
    }).into_response()
}

/// POST /projects/:name/tasks - Create task in project
pub async fn create_project_task(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = taskchampion::Operations::new();

    let mut task = match create_new_task(&mut replica, &mut ops).await {
        Ok(t) => t,
        Err(status) => return status.into_response(),
    };

    if let Err(status) = fill_task_from_create_request(&mut task, payload, &mut ops) {
        return status.into_response();
    }

    // Set project
    if let Err(e) = task.set_value("project".to_string(), Some(project_name), &mut ops) {
        error!("Failed to set project: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let response = TaskResponse::from_task(&task);

    // Commit and sync
    if let Err(status) = commit_and_sync(
        replica,
        ops,
        state.clone(),
        "Auto-sync failed after project task creation"
    ).await {
        return status.into_response();
    }

    (StatusCode::CREATED, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use taskchampion::Status;

    #[tokio::test]
    async fn test_list_projects_empty() {
        let state = create_test_state().await;
        let response = list_projects(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_projects_with_data() {
        let state = create_test_state().await;

        add_test_task(&state, "Task 1", Some("work"), None).await;
        add_test_task(&state, "Task 2", Some("home"), None).await;

        let response = list_projects(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_project_stats() {
        let state = create_test_state().await;

        add_test_task(&state, "Task 1", Some("work"), Some(Status::Pending)).await;
        add_test_task(&state, "Task 2", Some("work"), Some(Status::Completed)).await;

        let response = get_project_stats(State(state), Path("work".to_string())).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_project_tasks() {
        let state = create_test_state().await;

        add_test_task(&state, "Task 1", Some("work"), None).await;
        add_test_task(&state, "Task 2", Some("home"), None).await;

        let response = get_project_tasks(State(state), Path("work".to_string())).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
