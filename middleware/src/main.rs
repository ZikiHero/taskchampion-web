// main.rs

mod models;
mod handlers;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use std::env;
use taskchampion::{
    Replica, ServerConfig, SqliteStorage,
    Server, Uuid,
    storage::AccessMode,
};
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use handlers::AppState;

// ============================================
// SERVER WRAPPER (Thread-Safe)
// ============================================

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

impl ServerWrapper {
    pub fn new_in_memory() -> Self {
        struct DummyServer;
        #[async_trait::async_trait(?Send)]
        impl Server for DummyServer {
            async fn get_child_version(&mut self, _parent_version_id: taskchampion::server::VersionId) -> Result<taskchampion::server::GetVersionResult, taskchampion::Error> {
                todo!()
            }
            async fn add_version(&mut self, _parent_version_id: taskchampion::server::VersionId, _history_segment: Vec<u8>) -> Result<(taskchampion::server::AddVersionResult, taskchampion::server::SnapshotUrgency), taskchampion::Error> {
                todo!()
            }
            async fn add_snapshot(&mut self, _version_id: taskchampion::server::VersionId, _data: Vec<u8>) -> Result<(), taskchampion::Error> {
                todo!()
            }
            async fn get_snapshot(&mut self) -> Result<Option<(taskchampion::server::VersionId, Vec<u8>)>, taskchampion::Error> {
                todo!()
            }
        }
        Self(Box::new(DummyServer))
    }
}

// ============================================
// MAIN
// ============================================

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    info!("🚀 Starting Taskchampion Encryption Middleware...");

    // ============================================
    // LOAD & VALIDATE CONFIG
    // ============================================

    let client_id = env::var("TC_CLIENT_ID")
        .expect("❌ TC_CLIENT_ID environment variable must be set!");

    let client_uuid = Uuid::parse_str(&client_id)
        .expect("❌ TC_CLIENT_ID must be a valid UUID!");

    let encryption_password = env::var("TC_ENCRYPTION_PASSWORD")
        .expect("❌ TC_ENCRYPTION_PASSWORD environment variable must be set!");

    let sync_server_url = env::var("TC_SYNC_SERVER_URL")
        .expect("❌ TC_SYNC_SERVER_URL environment variable must be set!");

    let db_path = env::var("TC_DB_PATH")
        .unwrap_or_else(|_| "./local-cache.db".to_string());

    let auto_sync = env::var("TC_AUTO_SYNC")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);

    let server_port = env::var("TC_SERVER_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .expect("❌ TC_SERVER_PORT must be a valid port number!");

    // ============================================
    // DERIVE ENCRYPTION KEY
    // ============================================

    info!("🔐 Deriving encryption key from password...");
    let encryption_secret = encryption_password.as_bytes().to_vec();

    // ============================================
    // LOG CONFIGURATION (Safe - No Secrets!)
    // ============================================

    info!("📊 Configuration:");
    info!("  • Sync Server: {}", sync_server_url);
    info!("  • Client ID:   {}", client_uuid);
    info!("  • Database:    {}", db_path);
    info!("  • Port:        {}", server_port);
    info!("  • Encryption:  ✅ ENABLED");
    info!("  • Auto-Sync:   {}", if auto_sync { "✅ ON" } else { "⏸️  OFF" });

    // ============================================
    // INITIALIZE STORAGE
    // ============================================

    info!("💾 Initializing SQLite storage...");
    let storage = SqliteStorage::new(
        db_path.clone(),
        AccessMode::ReadWrite,
        true
    ).await.map_err(|e| {
        error!("❌ Failed to initialize storage at {}: {}", db_path, e);
        e
    })?;

    let mut replica = Replica::new(storage);

    // ============================================
    // CONFIGURE ENCRYPTED SERVER CONNECTION
    // ============================================

    info!("🔌 Connecting to sync server...");
    let server_config = ServerConfig::Remote {
        url: sync_server_url.parse().map_err(|e| {
            error!("❌ Invalid sync server URL: {}", e);
            e
        })?,
        client_id: client_uuid,
        encryption_secret,
    };

    let server = server_config.into_server().await.map_err(|e| {
        error!("❌ Failed to connect to sync server: {}", e);
        e
    })?;

    let mut server_wrapper = ServerWrapper(server);

    // ============================================
    // INITIAL SYNC
    // ============================================

    info!("🔄 Performing initial encrypted sync...");
    match replica.sync(&mut *server_wrapper, false).await {
        Ok(_) => info!("✅ Initial sync completed successfully"),
        Err(e) => {
            warn!("⚠️  Initial sync failed: {}", e);
            warn!("    This may happen if:");
            warn!("    • Sync server is unreachable");
            warn!("    • Client ID is not registered");
            warn!("    • Wrong encryption password");
            warn!("    • Network issues");
            warn!("    Continuing anyway - sync available via POST /sync");
        },
    }

    // ============================================
    // BUILD APPLICATION STATE
    // ============================================

    let state = AppState {
        replica: Arc::new(Mutex::new(replica)),
        server: Arc::new(Mutex::new(server_wrapper)),
        auto_sync,
    };

    // ============================================
    // BUILD API ROUTER
    // ============================================

    let app = Router::new()
        // ============ SYSTEM ============
        .route("/health", get(handlers::health))
        .route("/sync", post(handlers::trigger_sync))

        // ============ TASKS ============
        .route("/tasks", get(handlers::list_tasks))
        .route("/tasks", post(handlers::create_task))
        .route("/tasks/:uuid", get(handlers::get_task))
        .route("/tasks/:uuid", put(handlers::update_task))
        .route("/tasks/:uuid", delete(handlers::delete_task))

        // ============ PROJECTS ============
        .route("/projects", get(handlers::list_projects))
        .route("/projects/:name", get(handlers::get_project_stats))
        .route("/projects/:name/details", get(handlers::get_project_details))
        .route("/projects/:name/tasks", get(handlers::get_project_tasks))
        .route("/projects/:name/tasks", post(handlers::create_project_task))

        // ============ TAGS ============
        .route("/tags", get(handlers::list_tags))
        .route("/tags/:name", get(handlers::get_tag_stats))
        .route("/tags/:name/details", get(handlers::get_tag_details))
        .route("/tags/:name/tasks", get(handlers::get_tag_tasks))
        .route("/tags/:name/tasks", post(handlers::create_task_with_tag))

        .with_state(state);

    // ============================================
    // START SERVER
    // ============================================

    let bind_addr = format!("0.0.0.0:{}", server_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.map_err(|e| {
        error!("❌ Failed to bind to {}: {}", bind_addr, e);
        e
    })?;

    info!("✅ Server ready!");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🎉 Rust Encryption Middleware RUNNING");
    info!("🌐 Listening on: http://{}", bind_addr);
    info!("🔐 End-to-End Encryption: ✅ ACTIVE");
    info!("📡 Ready for Go backend requests");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    axum::serve(listener, app).await.map_err(|e| {
        error!("❌ Server error: {}", e);
        e
    })?;

    Ok(())
}
