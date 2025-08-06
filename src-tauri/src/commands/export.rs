use tauri::State;
use crate::services::DatabaseService;
use crate::errors::AppResult;
use std::path::PathBuf;

#[tauri::command]
pub async fn export_to_json(
    db: State<'_, DatabaseService>,
    path: String,
) -> AppResult<()> {
    let path = PathBuf::from(path);
    db.export_to_json(&path).await
}

#[tauri::command]
pub async fn import_from_json(
    db: State<'_, DatabaseService>,
    path: String,
) -> AppResult<()> {
    let path = PathBuf::from(path);
    db.import_from_json(&path).await
}

#[tauri::command]
pub async fn export_node_to_markdown(
    db: State<'_, DatabaseService>,
    node_id: String,
) -> AppResult<String> {
    db.export_node_to_markdown(&node_id).await
}

#[tauri::command]
pub async fn export_all_to_markdown(
    db: State<'_, DatabaseService>,
) -> AppResult<String> {
    db.export_all_to_markdown().await
}