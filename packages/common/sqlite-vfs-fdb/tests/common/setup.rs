use foundationdb::{api::NetworkAutoStop, Database};
use lazy_static::lazy_static;
use std::sync::Arc;
use uuid::Uuid;
use tracing;
use tracing_subscriber;

lazy_static! {
    // Initialize logging once
    static ref LOGGING: () = {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    };

    // Hold onto the network object for the lifetime of the program
    static ref NETWORK: NetworkAutoStop = {
        tracing::info!("Initializing FoundationDB for tests...");
        unsafe { foundationdb::boot() }
    };

    // Store a shared database connection
    static ref DATABASE: Arc<Database> = {
        // Make sure network is initialized first by referencing it
        let _ = &*NETWORK;
        // Ensure logging is set up
        let _ = &*LOGGING;

        Arc::new(foundationdb::Database::default().expect("Failed to connect to FoundationDB"))
    };
}

// Helper function to set up FDB for tests
pub fn setup_fdb() -> Arc<Database> {
    // This will initialize everything if it hasn't been done yet
    DATABASE.clone()
}

// Generate a unique database name for tests
#[allow(dead_code)]
pub fn test_db_name(prefix: &str) -> String {
    format!("{}_{}_{}", "test", prefix, Uuid::new_v4())
}