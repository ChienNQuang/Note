use tauri::State;
use crate::services::DatabaseService;
use crate::services::database::stats::{DatabaseStats, NodeStats};
use crate::errors::AppResult;

#[tauri::command]
pub async fn get_database_stats(
    db: State<'_, DatabaseService>,
) -> AppResult<DatabaseStats> {
    db.get_database_stats().await
}

#[tauri::command]
pub async fn get_node_stats(
    db: State<'_, DatabaseService>,
    node_id: String,
) -> AppResult<NodeStats> {
    db.get_node_stats(&node_id).await
}

#[tauri::command]
pub async fn get_link_stats(
    db: State<'_, DatabaseService>,
) -> AppResult<Vec<(String, i64)>> {
    db.get_link_stats().await
}