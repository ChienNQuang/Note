use tauri::State;
use crate::models::{Node, CreateNodeRequest, UpdateNodeRequest, NodeWithChildren};
use crate::services::{DatabaseService, LinkService};
use crate::errors::AppResult;

#[tauri::command]
pub async fn create_node(
    db: State<'_, DatabaseService>,
    link_service: State<'_, LinkService>,
    data: CreateNodeRequest,
) -> AppResult<Node> {
    let node = db.inner().create_node(data)?;
    link_service.update_links_for_node(&node)?;
    Ok(node)
}

#[tauri::command]
pub async fn get_node(
    db: State<'_, DatabaseService>,
    node_id: String,
) -> AppResult<Node> {
    db.inner().get_node(&node_id)
}

#[tauri::command]
pub async fn get_node_with_children(
    db: State<'_, DatabaseService>,
    node_id: String,
) -> AppResult<NodeWithChildren> {
    db.inner().get_node_with_children(&node_id)
}

#[tauri::command]
pub async fn update_node(
    db: State<'_, DatabaseService>,
    link_service: State<'_, LinkService>,
    node_id: String,
    data: UpdateNodeRequest,
) -> AppResult<Node> {
    let node = db.inner().update_node(&node_id, data)?;
    link_service.update_links_for_node(&node)?;
    Ok(node)
}

#[tauri::command]
pub async fn delete_node(
    db: State<'_, DatabaseService>,
    node_id: String,
) -> AppResult<()> {
    db.inner().delete_node(&node_id)
}

#[tauri::command]
pub async fn move_node(
    db: State<'_, DatabaseService>,
    node_id: String,
    new_parent_id: Option<String>,
    new_order: i32,
) -> AppResult<()> {
    db.inner().move_node(&node_id, new_parent_id, new_order)
}

#[tauri::command]
pub async fn get_daily_note(
    db: State<'_, DatabaseService>,
    date: String,
) -> AppResult<Node> {
    db.inner().get_daily_note(&date)
}

#[tauri::command]
pub async fn get_or_create_daily_note(
    db: State<'_, DatabaseService>,
    date: String,
) -> AppResult<Node> {
    db.inner().get_or_create_daily_note(&date)
}

#[tauri::command]
pub async fn get_linked_references(
    link_service: State<'_, LinkService>,
    node_id: String,
) -> AppResult<Vec<Node>> {
    link_service.get_linked_references(&node_id)
}

#[tauri::command]
pub async fn get_unlinked_references(
    db: State<'_, DatabaseService>,
    link_service: State<'_, LinkService>,
    node_id: String,
) -> AppResult<Vec<Node>> {
    let node = db.inner().get_node(&node_id)?;
    link_service.get_unlinked_references(&node_id, &node.content)
} 