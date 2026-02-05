// handlers/system.rs

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tracing::info;

use crate::models::SyncResponse;
use super::{AppState, helpers::perform_sync};

// ============================================
// SYSTEM HANDLERS
// ============================================

pub async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn trigger_sync(State(state): State<AppState>) -> impl IntoResponse {
    info!("🔄 Manual sync triggered...");

    match perform_sync(&state).await {
        Ok(_) => {
            info!("✅ Manual sync completed");
            (StatusCode::OK, Json(SyncResponse {
                success: true,
                message: "Sync completed successfully".to_string(),
                synced_at: chrono::Utc::now().to_rfc3339(),
            })).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(SyncResponse {
                success: false,
                message: e,
                synced_at: chrono::Utc::now().to_rfc3339(),
            })).into_response()
        }
    }
}
