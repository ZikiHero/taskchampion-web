use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use taskchampion::Status;
use tracing::error;

use crate::models::TaskResponse;
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

    let all_tasks = match replica.all_tasks().await {
        Ok(tasks) => tasks,
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    for task in all_tasks.values() {
        if let Some(project) = task.get_value("project") {
            projects.insert(project.to_string());
        }
    }

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

    let all_tasks = match replica.all_tasks().await {
        Ok(tasks) => tasks,
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    for task in all_tasks.values() {
        if let Some(project) = task.get_value("project") {
            if project == project_name {
                task_count += 1;

                match task.get_status() {
                    taskchampion::Status::Pending => pending_count += 1,
                    taskchampion::Status::Completed => completed_count += 1,
                    taskchampion::Status::Deleted => deleted_count += 1,
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

    let all_tasks = match replica.all_tasks().await {
        Ok(tasks) => tasks,
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
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

    let all_tasks = match replica.all_tasks().await {
        Ok(tasks) => tasks,
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    for task in all_tasks.values() {
        if let Some(project) = task.get_value("project") {
            if project == project_name {
                task_count += 1;

                match task.get_status() {
                    taskchampion::Status::Pending => {
                        pending_count += 1;
                        // Only include pending tasks in preview
                        if tasks_preview.len() < 10 {
                            tasks_preview.push(TaskResponse::from_task(task));
                        }
                    }
                    taskchampion::Status::Completed => completed_count += 1,
                    taskchampion::Status::Deleted => deleted_count += 1,
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
#[derive(Deserialize)]
pub struct CreateProjectTaskRequest {
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub async fn create_project_task(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
    Json(payload): Json<CreateProjectTaskRequest>,
) -> impl IntoResponse {
    let mut replica = state.replica.lock().await;
    let mut ops = taskchampion::Operations::new();

    // Create new task
    let mut task = match replica.create_task(taskchampion::Uuid::new_v4(), &mut ops).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create task: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let result: Result<(), StatusCode> = (|| {
        // Set basic properties
        task.set_status(Status::Pending, &mut ops)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        task.set_description(payload.description, &mut ops)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Set project
        task.set_value("project".to_string(), Some(project_name), &mut ops)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Set tags if provided
        apply_tags(&mut task, payload.tags, &mut ops)?;

        Ok(())
    })();

    if let Err(status) = result {
        return status.into_response();
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
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use taskchampion::{Replica, SqliteStorage, storage::AccessMode};

    async fn create_test_state() -> AppState {
        let storage = SqliteStorage::new(":memory:".to_string(), AccessMode::ReadWrite, true).await.unwrap();
        let replica = Replica::new(storage);
        let server = crate::ServerWrapper::new_in_memory();

        AppState {
            replica: Arc::new(Mutex::new(replica)),
            server: Arc::new(Mutex::new(server)),
            auto_sync: false,
        }
    }

    #[tokio::test]
    async fn test_list_projects_empty() {
        let state = create_test_state().await;
        let response = list_projects(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_projects_with_data() {
        let state = create_test_state().await;

        // Create tasks with projects
        {
            let mut replica = state.replica.lock().await;

            let mut ops = taskchampion::Operations::new();
            let mut task1 = replica
                .create_task(taskchampion::Uuid::new_v4(), &mut ops)
                .await
                .unwrap();
            task1.set_value("project".to_string(), Some("work".to_string()), &mut ops).unwrap();
            
            let mut task2 = replica
                .create_task(taskchampion::Uuid::new_v4(), &mut ops)
                .await
                .unwrap();
            task2.set_value("project".to_string(), Some("home".to_string()), &mut ops).unwrap();
            
            replica.commit_operations(ops).await.unwrap();
        }

        let response = list_projects(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_project_stats() {
        let state = create_test_state().await;

        // Create tasks in project
        {
            let mut replica = state.replica.lock().await;

            let mut ops = taskchampion::Operations::new();
            let mut task1 = replica
                .create_task(taskchampion::Uuid::new_v4(), &mut ops)
                .await
                .unwrap();
            task1.set_value("project".to_string(), Some("work".to_string()), &mut ops).unwrap();
            task1.set_status(taskchampion::Status::Pending, &mut ops).unwrap();

            let mut task2 = replica
                .create_task(taskchampion::Uuid::new_v4(), &mut ops)
                .await
                .unwrap();
            task2.set_value("project".to_string(), Some("work".to_string()), &mut ops).unwrap();
            task2.set_status(taskchampion::Status::Completed, &mut ops).unwrap();
            
            replica.commit_operations(ops).await.unwrap();
        }

        let response = get_project_stats(State(state), Path("work".to_string())).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_project_tasks() {
        let state = create_test_state().await;

        // Create tasks
        {
            let mut replica = state.replica.lock().await;

            let mut ops = taskchampion::Operations::new();
            let mut task1 = replica
                .create_task(taskchampion::Uuid::new_v4(), &mut ops)
                .await
                .unwrap();
            task1.set_value("project".to_string(), Some("work".to_string()), &mut ops).unwrap();

            let mut task2 = replica
                .create_task(taskchampion::Uuid::new_v4(), &mut ops)
                .await
                .unwrap();
            task2.set_value("project".to_string(), Some("home".to_string()), &mut ops).unwrap();
            
            replica.commit_operations(ops).await.unwrap();
        }

        let response = get_project_tasks(State(state), Path("work".to_string())).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
