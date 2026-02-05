mod models;
mod handlers;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use std::env;
use taskchampion::{Replica, ServerConfig, SqliteStorage, Server};
use taskchampion::storage::AccessMode;
use tokio::sync::Mutex;
use tracing::{info, error};

use handlers::AppState;

// Public wrapper damit handlers.rs es nutzen kann
pub struct ServerWrapper(pub Box<dyn Server>);

unsafe impl Send for ServerWrapper {}
unsafe impl Sync for ServerWrapper {}

impl std::ops::Deref for ServerWrapper {
    type Target = Box<dyn Server>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ServerWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Taskchampion Service...");
    
    let server_dir = env::var("TC_SERVER_DIR")
        .unwrap_or_else(|_| "./server-data".to_string());
    let auto_sync = env::var("AUTO_SYNC")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    
    info!("📁 Server directory: {}", server_dir);
    info!("🔄 Auto-sync: {}", auto_sync);
    
    let storage = SqliteStorage::new(
        "./local-cache.db",
        AccessMode::ReadWrite,
        true
    ).await?;
    
    let mut replica = Replica::new(storage);
    
    let server_config = ServerConfig::Local { 
        server_dir: server_dir.into() 
    };
    let server = server_config.into_server().await?;
    let mut server_wrapper = ServerWrapper(server);
    
    info!("🔄 Performing initial sync...");
    match replica.sync(&mut *server_wrapper, false).await {
        Ok(_) => info!("✅ Initial sync completed"),
        Err(e) => error!("⚠️  Initial sync failed (continuing anyway): {}", e),
    }
    
    let state = AppState {
        replica: Arc::new(Mutex::new(replica)),
        server: Arc::new(Mutex::new(server_wrapper)),
        auto_sync,
    };
    
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/tasks", get(handlers::list_tasks))
        .route("/tasks", post(handlers::create_task))
        .route("/tasks/:uuid", get(handlers::get_task))
        .route("/tasks/:uuid", put(handlers::update_task))
        .route("/tasks/:uuid", delete(handlers::delete_task))
        .route("/sync", post(handlers::trigger_sync))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("🎉 Server running on http://0.0.0.0:3001");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
