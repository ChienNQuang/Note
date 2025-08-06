// Module declarations
pub mod errors;
pub mod models;
pub mod utils;
pub mod services;
pub mod commands;

// Re-exports for easier access
pub use errors::{AppError, AppResult};
pub use models::*;
pub use services::{DatabaseService, LinkService};
pub use commands::nodes::*;
pub use commands::search::*;
pub use commands::stats::*;
pub use commands::export::*;

// Basic commands
#[tauri::command]
async fn greet(name: &str) -> Result<String, AppError> {
    Ok(format!("Hello, {}! Welcome to the refactored Note app.", name))
}

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

    // Create tokio runtime for async initialization
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    
    // Initialize services asynchronously
    let (db_service, link_service) = runtime.block_on(async {
        let db_service = DatabaseService::new()
            .await
            .expect("Failed to initialize database service");
        let link_service = LinkService::new(db_service.clone());
        (db_service, link_service)
    });
    
    tracing::info!("Services initialized successfully");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(db_service)
        .manage(link_service)
        .invoke_handler(tauri::generate_handler![
            // Basic commands
            greet,
            get_app_version,
            // Node commands
            create_node,
            get_node,
            get_node_with_children,
            update_node,
            delete_node,
            move_node,
            // Journal commands
            get_daily_note,
            get_or_create_daily_note,
            // Linking commands
            get_linked_references,
            get_unlinked_references,
            // Search commands
            search_nodes,
            search_nodes_by_tags,
            search_nodes_by_properties,
            get_root_nodes,
            // Stats commands
            get_database_stats,
            get_node_stats,
            get_link_stats,
            // Export commands
            export_to_json,
            import_from_json,
            export_node_to_markdown,
            export_all_to_markdown,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}