use tauri::State;
use std::sync::Mutex;

use crate::models::{Page, CreatePageRequest, UpdatePageRequest, Block, CreateBlockRequest, UpdateBlockRequest, MoveBlockRequest};
use crate::services::DatabaseService;
use crate::errors::AppError;

/// Create a new page
#[tauri::command]
pub async fn create_page(
    db: State<'_, Mutex<DatabaseService>>,
    title: String,
) -> Result<Page, AppError> {
    let db = db.lock().unwrap();
    
    let request = CreatePageRequest {
        title,
        properties: None,
        tags: None,
        status: None,
    };
    
    db.create_page(request)
}

/// Get a page by ID
#[tauri::command]
pub async fn get_page(
    db: State<'_, Mutex<DatabaseService>>,
    page_id: String,
) -> Result<Page, AppError> {
    let db = db.lock().unwrap();
    db.get_page(&page_id)
}

/// Update a page
#[tauri::command]
pub async fn update_page(
    db: State<'_, Mutex<DatabaseService>>,
    page_id: String,
    title: Option<String>,
    properties: Option<std::collections::HashMap<String, serde_json::Value>>,
    tags: Option<Vec<String>>,
    status: Option<String>,
) -> Result<Page, AppError> {
    let db = db.lock().unwrap();
    
    // Parse status if provided
    let parsed_status = if let Some(status_str) = status {
        Some(status_str.parse().map_err(|e: String| {
            AppError::InvalidPageData(format!("Invalid status: {}", e))
        })?)
    } else {
        None
    };
    
    let request = UpdatePageRequest {
        title,
        properties,
        tags,
        status: parsed_status,
    };
    
    db.update_page(&page_id, request)
}

/// Delete a page
#[tauri::command]
pub async fn delete_page(
    db: State<'_, Mutex<DatabaseService>>,
    page_id: String,
) -> Result<(), AppError> {
    let db = db.lock().unwrap();
    db.delete_page(&page_id)
}

/// List all pages
#[tauri::command]
pub async fn list_pages(
    db: State<'_, Mutex<DatabaseService>>,
) -> Result<Vec<Page>, AppError> {
    let db = db.lock().unwrap();
    db.list_pages()
}

/// Get page statistics (for Phase 1 dashboard)
#[tauri::command]
pub async fn get_page_stats(
    db: State<'_, Mutex<DatabaseService>>,
) -> Result<PageStats, AppError> {
    let db = db.lock().unwrap();
    let pages = db.list_pages()?;
    
    let total_pages = pages.len();
    let draft_pages = pages.iter().filter(|p| matches!(p.status, crate::models::PageStatus::Draft)).count();
    let published_pages = pages.iter().filter(|p| matches!(p.status, crate::models::PageStatus::Published)).count();
    
    // Calculate most recent activity
    let recent_page = pages.first().map(|p| p.title.clone());
    
    Ok(PageStats {
        total_pages,
        draft_pages,
        published_pages,
        recent_page,
    })
}

// ===== BLOCK OPERATIONS =====

/// Create a new block
#[tauri::command]
pub async fn create_block(
    db: State<'_, Mutex<DatabaseService>>,
    page_id: String,
    text: String,
    parent_id: Option<String>,
    order: Option<i32>,
    block_type: Option<String>,
    properties: Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<Block, AppError> {
    let db = db.lock().unwrap();
    
    // Parse block type
    let parsed_block_type = if let Some(type_str) = block_type {
        Some(type_str.parse().map_err(|e: String| {
            AppError::InvalidBlockData(format!("Invalid block type: {}", e))
        })?)
    } else {
        None
    };
    
    let request = CreateBlockRequest {
        page_id,
        text,
        parent_id,
        order,
        block_type: parsed_block_type,
        properties,
    };
    
    db.create_block(request)
}

/// Get a block by ID
#[tauri::command]
pub async fn get_block(
    db: State<'_, Mutex<DatabaseService>>,
    block_id: String,
) -> Result<Block, AppError> {
    let db = db.lock().unwrap();
    db.get_block(&block_id)
}

/// Update a block
#[tauri::command]
pub async fn update_block(
    db: State<'_, Mutex<DatabaseService>>,
    block_id: String,
    text: Option<String>,
    properties: Option<std::collections::HashMap<String, serde_json::Value>>,
    block_type: Option<String>,
) -> Result<Block, AppError> {
    let db = db.lock().unwrap();
    
    // Parse block type if provided
    let parsed_block_type = if let Some(type_str) = block_type {
        Some(type_str.parse().map_err(|e: String| {
            AppError::InvalidBlockData(format!("Invalid block type: {}", e))
        })?)
    } else {
        None
    };
    
    let request = UpdateBlockRequest {
        text,
        properties,
        block_type: parsed_block_type,
    };
    
    db.update_block(&block_id, request)
}

/// Delete a block
#[tauri::command]
pub async fn delete_block(
    db: State<'_, Mutex<DatabaseService>>,
    block_id: String,
) -> Result<(), AppError> {
    let db = db.lock().unwrap();
    db.delete_block(&block_id)
}

/// Get all blocks for a page
#[tauri::command]
pub async fn get_page_blocks(
    db: State<'_, Mutex<DatabaseService>>,
    page_id: String,
) -> Result<Vec<Block>, AppError> {
    let db = db.lock().unwrap();
    db.get_page_blocks(&page_id)
}

/// Move a block to a new position
#[tauri::command]
pub async fn move_block(
    db: State<'_, Mutex<DatabaseService>>,
    block_id: String,
    new_parent_id: Option<String>,
    new_order: i32,
) -> Result<Block, AppError> {
    let db = db.lock().unwrap();
    
    let request = MoveBlockRequest {
        new_parent_id,
        new_order,
    };
    
    db.move_block(&block_id, request)
}

/// Page statistics structure
#[derive(serde::Serialize)]
pub struct PageStats {
    pub total_pages: usize,
    pub draft_pages: usize,
    pub published_pages: usize,
    pub recent_page: Option<String>,
} 