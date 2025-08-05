use tauri::State;
use crate::models::Node;
use crate::services::DatabaseService;
use crate::errors::AppResult;

#[tauri::command]
pub async fn search_nodes(
    db: State<'_, DatabaseService>,
    query: String,
    limit: Option<usize>,
) -> AppResult<Vec<Node>> {
    db.search_nodes(&query, limit)
}

#[tauri::command]
pub async fn find_nodes_by_tag(
    db: State<'_, DatabaseService>,
    tag: String,
) -> AppResult<Vec<Node>> {
    db.find_nodes_by_tag(&tag)
}

#[tauri::command]
pub async fn get_root_nodes(
    db: State<'_, DatabaseService>,
) -> AppResult<Vec<Node>> {
    db.get_root_nodes()
}

#[tauri::command]
pub async fn get_recent_nodes(
    db: State<'_, DatabaseService>,
    limit: usize,
) -> AppResult<Vec<Node>> {
    db.get_recent_nodes(limit)
}