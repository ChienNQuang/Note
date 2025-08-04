use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use dirs;

use crate::models::{Page, Block, PageStatus, BlockType, CreatePageRequest, UpdatePageRequest, CreateBlockRequest, UpdateBlockRequest, MoveBlockRequest};
use crate::errors::{AppError, AppResult};
use crate::utils::{generate_page_id, generate_block_id, generate_id};

pub struct DatabaseService {
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseService {
    /// Create a new database service with connection to SQLite
    pub fn new() -> AppResult<Self> {
        // Get app data directory
        let app_dir = dirs::data_dir()
            .ok_or_else(|| AppError::Internal("Could not find data directory".to_string()))?
            .join("note-app");
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&app_dir)?;
        
        let db_path = app_dir.join("note.db");
        
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::DatabaseConnectionFailed(e.to_string()))?;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let service = DatabaseService {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        // Initialize database schema
        service.initialize_schema()?;
        
        // Create default user if none exists
        service.ensure_default_user()?;
        
        Ok(service)
    }
    
    /// Initialize the database schema (Phase 1 - simplified schema)
    fn initialize_schema(&self) -> AppResult<()> {
        let conn = self.connection.lock().unwrap();
        
        // Create users table first (for foreign key constraints)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                preferences TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Create pages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pages (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                properties TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT NOT NULL,
                status TEXT DEFAULT 'draft',
                FOREIGN KEY (created_by) REFERENCES users(id)
            )",
            [],
        )?;
        
        // Create blocks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL,
                text TEXT NOT NULL,
                parent_id TEXT,
                order_index INTEGER NOT NULL,
                properties TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT NOT NULL,
                version INTEGER DEFAULT 1,
                block_type TEXT DEFAULT 'text',
                
                FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE,
                FOREIGN KEY (parent_id) REFERENCES blocks(id) ON DELETE CASCADE,
                FOREIGN KEY (created_by) REFERENCES users(id)
            )",
            [],
        )?;
        
        // Create page_blocks junction table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS page_blocks (
                page_id TEXT NOT NULL,
                block_id TEXT NOT NULL,
                is_root BOOLEAN DEFAULT FALSE,
                
                PRIMARY KEY (page_id, block_id),
                FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE,
                FOREIGN KEY (block_id) REFERENCES blocks(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_page_id ON blocks(page_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_parent_id ON blocks(parent_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_order ON blocks(page_id, order_index)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_page_blocks_page ON page_blocks(page_id)", [])?;
        
        Ok(())
    }
    
    /// Ensure a default user exists for Phase 1 (single-user)
    fn ensure_default_user(&self) -> AppResult<()> {
        let conn = self.connection.lock().unwrap();
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users",
            [],
            |row| row.get(0)
        )?;
        
        if count == 0 {
            let user_id = generate_id();
            conn.execute(
                "INSERT INTO users (id, name, email, preferences) VALUES (?1, ?2, ?3, ?4)",
                params![
                    user_id,
                    "Local User",
                    None::<String>,
                    "{\"theme\":\"system\",\"font_size\":14,\"auto_save_interval\":30,\"git_auto_commit\":true,\"collaboration_enabled\":false,\"shortcuts\":{}}"
                ],
            )?;
        }
        
        Ok(())
    }
    
    /// Get the default user ID for Phase 1
    pub fn get_default_user_id(&self) -> AppResult<String> {
        let conn = self.connection.lock().unwrap();
        
        let user_id: String = conn.query_row(
            "SELECT id FROM users LIMIT 1",
            [],
            |row| row.get(0)
        )?;
        
        Ok(user_id)
    }
    
    // ===== PAGE OPERATIONS =====
    
    /// Create a new page
    pub fn create_page(&self, request: CreatePageRequest) -> AppResult<Page> {
        let conn = self.connection.lock().unwrap();
        let page_id = generate_page_id();
        let user_id = self.get_default_user_id()?;
        let now = Utc::now();
        
        let properties_json = serde_json::to_string(&request.properties.unwrap_or_default())?;
        let status = request.status.unwrap_or(PageStatus::Draft);
        
        conn.execute(
            "INSERT INTO pages (id, title, properties, created_at, updated_at, created_by, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                page_id,
                request.title,
                properties_json,
                now,
                now,
                user_id,
                status.to_string()
            ],
        )?;
        
        // Return the created page
        self.get_page(&page_id)
    }
    
    /// Get a page by ID
    pub fn get_page(&self, page_id: &str) -> AppResult<Page> {
        let conn = self.connection.lock().unwrap();
        
        let page = conn.query_row(
            "SELECT id, title, properties, created_at, updated_at, created_by, status 
             FROM pages WHERE id = ?1",
            params![page_id],
            |row| {
                let properties_str: String = row.get(2)?;
                let properties: std::collections::HashMap<String, serde_json::Value> = 
                    serde_json::from_str(&properties_str).unwrap_or_default();
                
                let created_at: DateTime<Utc> = row.get(3)?;
                let updated_at: DateTime<Utc> = row.get(4)?;
                let status_str: String = row.get(6)?;
                let status = status_str.parse().unwrap_or(PageStatus::Draft);
                
                Ok(Page {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    root_blocks: Vec::new(), // Will be populated separately
                    properties,
                    file_path: None, // Phase 2 feature
                    created_at,
                    updated_at,
                    created_by: row.get(5)?,
                    last_edited_by: row.get(5)?, // Same as created_by for now
                    tags: Vec::new(), // Will be extracted from properties
                    status,
                })
            }
        ).map_err(|_| AppError::FileNotFound(format!("Page with ID {} not found", page_id)))?;
        
        Ok(page)
    }
    
    /// Update a page
    pub fn update_page(&self, page_id: &str, request: UpdatePageRequest) -> AppResult<Page> {
        let conn = self.connection.lock().unwrap();
        let now = Utc::now();
        
        // Pre-allocate strings to ensure they live long enough
        let mut title_str = String::new();
        let mut properties_json = String::new();
        let mut status_str = String::new();
        
        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        
        if let Some(title) = &request.title {
            updates.push("title = ?");
            title_str = title.clone();
            params.push(&title_str);
        }
        
        if let Some(properties) = &request.properties {
            updates.push("properties = ?");
            properties_json = serde_json::to_string(properties)?;
            params.push(&properties_json);
        }
        
        if let Some(status) = &request.status {
            updates.push("status = ?");
            status_str = status.to_string();
            params.push(&status_str);
        }
        
        if updates.is_empty() {
            return self.get_page(page_id); // No updates needed
        }
        
        // Always update the timestamp
        updates.push("updated_at = ?");
        params.push(&now);
        
        // Add page_id parameter
        params.push(&page_id);
        
        let query = format!(
            "UPDATE pages SET {} WHERE id = ?",
            updates.join(", ")
        );
        
        let affected = conn.execute(&query, params.as_slice())?;
        
        if affected == 0 {
            return Err(AppError::FileNotFound(format!("Page with ID {} not found", page_id)));
        }
        
        self.get_page(page_id)
    }
    
    /// Delete a page
    pub fn delete_page(&self, page_id: &str) -> AppResult<()> {
        let conn = self.connection.lock().unwrap();
        
        let affected = conn.execute("DELETE FROM pages WHERE id = ?1", params![page_id])?;
        
        if affected == 0 {
            return Err(AppError::FileNotFound(format!("Page with ID {} not found", page_id)));
        }
        
        Ok(())
    }
    
    /// List all pages
    pub fn list_pages(&self) -> AppResult<Vec<Page>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, title, properties, created_at, updated_at, created_by, status 
             FROM pages ORDER BY updated_at DESC"
        )?;
        
        let page_iter = stmt.query_map([], |row| {
            let properties_str: String = row.get(2)?;
            let properties: std::collections::HashMap<String, serde_json::Value> = 
                serde_json::from_str(&properties_str).unwrap_or_default();
            
            let created_at: DateTime<Utc> = row.get(3)?;
            let updated_at: DateTime<Utc> = row.get(4)?;
            let status_str: String = row.get(6)?;
            let status = status_str.parse().unwrap_or(PageStatus::Draft);
            
            Ok(Page {
                id: row.get(0)?,
                title: row.get(1)?,
                root_blocks: Vec::new(),
                properties,
                file_path: None,
                created_at,
                updated_at,
                created_by: row.get(5)?,
                last_edited_by: row.get(5)?,
                tags: Vec::new(),
                status,
            })
        })?;
        
        let mut pages = Vec::new();
        for page in page_iter {
            pages.push(page?);
        }
        
        Ok(pages)
    }
    
    // ===== BLOCK OPERATIONS (Prepared for Phase 1.3) =====
    
    /// Create a new block (placeholder for Phase 1.3)
    pub fn create_block(&self, request: CreateBlockRequest) -> AppResult<Block> {
        let conn = self.connection.lock().unwrap();
        let block_id = generate_block_id();
        let user_id = self.get_default_user_id()?;
        let now = Utc::now();
        
        let properties_json = serde_json::to_string(&request.properties.unwrap_or_default())?;
        let block_type = request.block_type.unwrap_or(BlockType::Text);
        let order = request.order.unwrap_or(0);
        
        conn.execute(
            "INSERT INTO blocks (id, page_id, text, parent_id, order_index, properties, created_at, updated_at, created_by, block_type) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                block_id,
                request.page_id,
                request.text,
                request.parent_id,
                order,
                properties_json,
                now,
                now,
                user_id,
                block_type.to_string()
            ],
        )?;
        
        // Add to page_blocks junction table if it's a root block
        if request.parent_id.is_none() {
            conn.execute(
                "INSERT INTO page_blocks (page_id, block_id, is_root) VALUES (?1, ?2, ?3)",
                params![request.page_id, block_id, true],
            )?;
        }
        
        self.get_block(&block_id)
    }
    
    /// Get a block by ID (placeholder for Phase 1.3)
    pub fn get_block(&self, block_id: &str) -> AppResult<Block> {
        let conn = self.connection.lock().unwrap();
        
        let block = conn.query_row(
            "SELECT id, page_id, text, parent_id, order_index, properties, created_at, updated_at, created_by, version, block_type 
             FROM blocks WHERE id = ?1",
            params![block_id],
            |row| {
                let properties_str: String = row.get(5)?;
                let properties: std::collections::HashMap<String, serde_json::Value> = 
                    serde_json::from_str(&properties_str).unwrap_or_default();
                
                let created_at: DateTime<Utc> = row.get(6)?;
                let updated_at: DateTime<Utc> = row.get(7)?;
                let block_type_str: String = row.get(10)?;
                let block_type = block_type_str.parse().unwrap_or(BlockType::Text);
                
                Ok(Block {
                    id: row.get(0)?,
                    text: row.get(2)?,
                    page_id: row.get(1)?,
                    parent_id: row.get(3)?,
                    children: Vec::new(), // Will be populated separately
                    order: row.get(4)?,
                    properties,
                    created_at,
                    updated_at,
                    created_by: row.get(8)?,
                    last_edited_by: row.get(8)?,
                    version: row.get(9)?,
                    block_type,
                    vector_clock: None, // Phase 3 feature
                    operation_id: None, // Phase 3 feature
                })
            }
        ).map_err(|_| AppError::FileNotFound(format!("Block with ID {} not found", block_id)))?;
        
        Ok(block)
    }
    
    /// Get all blocks for a page (placeholder for Phase 1.3)
    pub fn get_page_blocks(&self, page_id: &str) -> AppResult<Vec<Block>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, page_id, text, parent_id, order_index, properties, created_at, updated_at, created_by, version, block_type 
             FROM blocks WHERE page_id = ?1 ORDER BY order_index"
        )?;
        
        let block_iter = stmt.query_map(params![page_id], |row| {
            let properties_str: String = row.get(5)?;
            let properties: std::collections::HashMap<String, serde_json::Value> = 
                serde_json::from_str(&properties_str).unwrap_or_default();
            
            let created_at: DateTime<Utc> = row.get(6)?;
            let updated_at: DateTime<Utc> = row.get(7)?;
            let block_type_str: String = row.get(10)?;
            let block_type = block_type_str.parse().unwrap_or(BlockType::Text);
            
            Ok(Block {
                id: row.get(0)?,
                text: row.get(2)?,
                page_id: row.get(1)?,
                parent_id: row.get(3)?,
                children: Vec::new(),
                order: row.get(4)?,
                properties,
                created_at,
                updated_at,
                created_by: row.get(8)?,
                last_edited_by: row.get(8)?,
                version: row.get(9)?,
                block_type,
                vector_clock: None,
                operation_id: None,
            })
        })?;
        
        let mut blocks = Vec::new();
        for block in block_iter {
            blocks.push(block?);
        }
        
        Ok(blocks)
    }
    
    /// Update a block
    pub fn update_block(&self, block_id: &str, request: UpdateBlockRequest) -> AppResult<Block> {
        let conn = self.connection.lock().unwrap();
        let now = Utc::now();
        
        // Pre-allocate strings to ensure they live long enough
        let mut text_str = String::new();
        let mut properties_json = String::new();
        let mut block_type_str = String::new();
        
        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        
        if let Some(text) = &request.text {
            updates.push("text = ?");
            text_str = text.clone();
            params.push(&text_str);
        }
        
        if let Some(properties) = &request.properties {
            updates.push("properties = ?");
            properties_json = serde_json::to_string(properties)?;
            params.push(&properties_json);
        }
        
        if let Some(block_type) = &request.block_type {
            updates.push("block_type = ?");
            block_type_str = block_type.to_string();
            params.push(&block_type_str);
        }
        
        if updates.is_empty() {
            return self.get_block(block_id); // No updates needed
        }
        
        // Always update the timestamp and version
        updates.push("updated_at = ?");
        updates.push("version = version + 1");
        params.push(&now);
        
        // Add block_id parameter
        params.push(&block_id);
        
        let query = format!(
            "UPDATE blocks SET {} WHERE id = ?",
            updates.join(", ")
        );
        
        let affected = conn.execute(&query, params.as_slice())?;
        
        if affected == 0 {
            return Err(AppError::FileNotFound(format!("Block with ID {} not found", block_id)));
        }
        
        self.get_block(block_id)
    }
    
    /// Delete a block
    pub fn delete_block(&self, block_id: &str) -> AppResult<()> {
        let conn = self.connection.lock().unwrap();
        
        let affected = conn.execute("DELETE FROM blocks WHERE id = ?1", params![block_id])?;
        
        if affected == 0 {
            return Err(AppError::FileNotFound(format!("Block with ID {} not found", block_id)));
        }
        
        Ok(())
    }
    
    /// Move a block to a new position
    pub fn move_block(&self, block_id: &str, request: MoveBlockRequest) -> AppResult<Block> {
        let conn = self.connection.lock().unwrap();
        let now = Utc::now();
        
        let affected = conn.execute(
            "UPDATE blocks SET parent_id = ?1, order_index = ?2, updated_at = ?3, version = version + 1 WHERE id = ?4",
            params![request.new_parent_id, request.new_order, now, block_id],
        )?;
        
        if affected == 0 {
            return Err(AppError::FileNotFound(format!("Block with ID {} not found", block_id)));
        }
        
        self.get_block(block_id)
    }
}

// Make the database service thread-safe and implement Clone
impl Clone for DatabaseService {
    fn clone(&self) -> Self {
        DatabaseService {
            connection: Arc::clone(&self.connection),
        }
    }
} 