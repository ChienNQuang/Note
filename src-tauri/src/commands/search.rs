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
    let limit = limit.unwrap_or(50) as i64;
    db.search_nodes(&query, limit).await
}

#[tauri::command]
pub async fn search_nodes_by_tags(
    db: State<'_, DatabaseService>,
    tags: Vec<String>,
    limit: Option<usize>,
) -> AppResult<Vec<Node>> {
    let limit = limit.unwrap_or(50) as i64;
    db.search_nodes_by_tags(&tags, limit).await
}

#[tauri::command]
pub async fn search_nodes_by_properties(
    db: State<'_, DatabaseService>,
    property_key: String,
    property_value: String,
    limit: Option<usize>,
) -> AppResult<Vec<Node>> {
    let limit = limit.unwrap_or(50) as i64;
    db.search_nodes_by_properties(&property_key, &property_value, limit).await
}

#[tauri::command]
pub async fn get_root_nodes(
    db: State<'_, DatabaseService>,
) -> AppResult<Vec<Node>> {
    db.get_root_nodes().await
}

