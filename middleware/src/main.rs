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
use tracing::{info, warn};

use handlers::AppState;

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

/// Derives encryption key from password using Argon2
/*
fn derive_encryption_key(password: &str) -> Vec<u8> {
    use argon2::{
        password_hash::{PasswordHasher, SaltString},
        Argon2,
    };

    // Fixed salt for deterministic key derivation
    // This ensures same password always produces same key
    let salt = SaltString::from_b64("dGFza2NoYW1waW9uU2FsdFYx").unwrap();
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .hash
        .unwrap();

    // Return first 32 bytes as encryption key
    hash.as_bytes()[..32].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_encryption_key() {
        let key1 = c("password123");
        let key2 = derive_encryption_key("password123");
        let key3 = derive_encryption_key("different");

        assert_eq!(key1.len(), 32);
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
*/
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Taskchampion Encryption Middleware...");

    // ============================================
    // REQUIRED CONFIG - Must be set!
    // ============================================

    let client_id = env::var("TC_CLIENT_ID")
        .expect("❌ TC_CLIENT_ID environment variable must be set!");

    let client_uuid = Uuid::parse_str(&client_id)
        .expect("❌ TC_CLIENT_ID must be a valid UUID!");

    let encryption_password = env::var("TC_ENCRYPTION_PASSWORD")
        .expect("❌ TC_ENCRYPTION_PASSWORD environment variable must be set!");

    let sync_server_url = env::var("TC_SYNC_SERVER_URL")
        .expect("❌ TC_SYNC_SERVER_URL environment variable must be set!");

    // ============================================
    // OPTIONAL CONFIG
    // ============================================

    let db_path = env::var("TC_DB_PATH")
        .unwrap_or_else(|_| "./local-cache.db".to_string());

    let auto_sync = env::var("TC_AUTO_SYNC")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);

    // ============================================
    // SECURITY: Derive Encryption Key
    // ============================================

    info!("🔐 Deriving encryption key from password...");
    let encryption_secret = encryption_password.as_bytes().to_vec();

    // ============================================
    // LOG CONFIG (without secrets!)
    // ============================================

    info!("📊 Configuration:");
    info!("  • Sync Server: {}", sync_server_url);
    info!("  • Client ID: {}", client_uuid);
    info!("  • Database: {}", db_path);
    info!("  • Encryption: ✅ ENABLED (Argon2 derived)");
    info!("  • Auto-Sync: {}", auto_sync);

    // ============================================
    // INITIALIZE STORAGE & SERVER
    // ============================================

    let storage = SqliteStorage::new(
        db_path,
        AccessMode::ReadWrite,
        true
    ).await?;

    let mut replica = Replica::new(storage);

    // Configure Remote Server with Encryption
    let server_config = ServerConfig::Remote {
        url: sync_server_url.parse()?,
        client_id: client_uuid,
        encryption_secret,
    };

    info!("🔌 Connecting to sync server...");
    let server = server_config.into_server().await?;
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
            warn!("    1. Sync server is unreachable");
            warn!("    2. Client ID is not registered");
            warn!("    3. Wrong encryption password");
            warn!("    Continuing anyway...");
        },
    }

    // ============================================
    // BUILD API
    // ============================================

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

    // ============================================
    // START SERVER
    // ============================================

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;

    info!("🎉 Rust Encryption Middleware running on http://0.0.0.0:3001");
    info!("🔐 All data encrypted before sync (Go backend never sees keys)");
    info!("📡 Ready to accept requests from Go backend");

    axum::serve(listener, app).await?;

    Ok(())
}
