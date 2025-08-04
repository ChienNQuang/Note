// Module declarations
pub mod errors;
pub mod models;
pub mod utils;
pub mod services;
pub mod commands;

// Re-exports for easier access
pub use errors::{AppError, AppResult};
pub use models::*;
pub use services::DatabaseService;
pub use commands::*;

// Tauri commands - start with a basic test command
#[tauri::command]
async fn greet(name: &str) -> Result<String, AppError> {
    Ok(format!("Hello, {}! Welcome to the Note app.", name))
}

// Health check command
#[tauri::command]
async fn health_check() -> Result<String, AppError> {
    Ok("Note app backend is running".to_string())
}

// Get app version
#[tauri::command]
async fn get_app_version() -> Result<String, AppError> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "note=debug,tauri=info".into()),
        )
        .init();

    tracing::info!("Starting Note application");

    // Initialize database service
    let database_service = DatabaseService::new()
        .expect("Failed to initialize database service");
    
    tracing::info!("Database service initialized successfully");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(std::sync::Mutex::new(database_service))
        .invoke_handler(tauri::generate_handler![
            // Basic commands
            greet,
            health_check,
            get_app_version,
            // Page management commands
            create_page,
            get_page,
            update_page,
            delete_page,
            list_pages,
            get_page_stats,
            // Block management commands
            create_block,
            get_block,
            update_block,
            delete_block,
            get_page_blocks,
            move_block,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
