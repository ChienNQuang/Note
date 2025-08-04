# Backend Services Implementation

## 1. Rust/Tauri Backend Architecture

### 1.1 Project Structure

```
src-tauri/src/
├── main.rs                 # Entry point and Tauri setup
├── lib.rs                  # Library root and module declarations
├── commands/               # Tauri command handlers
│   ├── mod.rs
│   ├── blocks.rs          # Block CRUD operations
│   ├── pages.rs           # Page operations
│   ├── git.rs             # Git operations
│   ├── search.rs          # Search operations
│   └── collaboration.rs   # Real-time collaboration
├── services/              # Business logic services
│   ├── mod.rs
│   ├── database.rs        # Database connection and operations
│   ├── git_manager.rs     # Git repository management
│   ├── file_manager.rs    # File system operations
│   ├── crdt_engine.rs     # CRDT implementation
│   └── websocket_server.rs # WebSocket server for collaboration
├── models/                # Data models and structures
│   ├── mod.rs
│   ├── block.rs
│   ├── page.rs
│   ├── user.rs
│   └── event.rs
├── utils/                 # Utility functions
│   ├── mod.rs
│   ├── uuid.rs
│   ├── crypto.rs
│   └── validation.rs
└── errors/                # Error handling
    ├── mod.rs
    └── app_error.rs
```

## 2. Core Services Implementation

### 2.1 Database Service

#### Planning
- [ ] Design SQLite connection management
- [ ] Plan migration system
- [ ] Define query optimization strategy
- [ ] Specify transaction handling

#### Design
- [ ] Create database schema
- [ ] Design query interface
- [ ] Plan connection pooling
- [ ] Design backup system

#### Implementation
```rust
// services/database.rs
use rusqlite::{Connection, Result, Transaction, params};
use std::sync::{Arc, Mutex};
use std::path::Path;
use serde_json;

use crate::models::{Block, Page, User, Event};
use crate::errors::AppError;

pub struct DatabaseService {
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseService {
    pub fn new(db_path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)?;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        // Create initial schema
        Self::create_tables(&conn)?;
        
        Ok(DatabaseService {
            connection: Arc::new(Mutex::new(conn)),
        })
    }
    
    fn create_tables(conn: &Connection) -> Result<(), AppError> {
        // Create blocks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                parent_id TEXT,
                order_index INTEGER NOT NULL,
                properties TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT NOT NULL,
                last_edited_by TEXT NOT NULL,
                version INTEGER DEFAULT 1,
                block_type TEXT DEFAULT 'text',
                is_deleted BOOLEAN DEFAULT FALSE,
                
                FOREIGN KEY (parent_id) REFERENCES blocks(id) ON DELETE CASCADE,
                FOREIGN KEY (created_by) REFERENCES users(id),
                FOREIGN KEY (last_edited_by) REFERENCES users(id)
            )",
            [],
        )?;
        
        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_parent_id ON blocks(parent_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_order ON blocks(parent_id, order_index)", [])?;
        
        // Create pages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pages (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                file_path TEXT UNIQUE,
                properties TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT NOT NULL,
                last_edited_by TEXT NOT NULL,
                status TEXT DEFAULT 'draft',
                is_deleted BOOLEAN DEFAULT FALSE,
                
                FOREIGN KEY (created_by) REFERENCES users(id),
                FOREIGN KEY (last_edited_by) REFERENCES users(id)
            )",
            [],
        )?;
        
        // Create users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE,
                avatar_url TEXT,
                preferences TEXT,
                is_online BOOLEAN DEFAULT FALSE,
                last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Create events table for CRDT
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                block_id TEXT,
                page_id TEXT,
                event_data TEXT NOT NULL,
                user_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                sequence INTEGER NOT NULL,
                vector_clock TEXT NOT NULL,
                causal_context TEXT,
                is_applied BOOLEAN DEFAULT FALSE,
                
                FOREIGN KEY (block_id) REFERENCES blocks(id),
                FOREIGN KEY (page_id) REFERENCES pages(id),
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
            [],
        )?;
        
        Ok(())
    }
    
    // Block operations
    pub fn create_block(&self, block: &Block) -> Result<(), AppError> {
        let conn = self.connection.lock().unwrap();
        
        conn.execute(
            "INSERT INTO blocks (
                id, text, parent_id, order_index, properties, 
                created_by, last_edited_by, version, block_type
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                block.id,
                block.text,
                block.parent_id,
                block.order,
                serde_json::to_string(&block.properties)?,
                block.created_by,
                block.last_edited_by,
                block.version,
                block.block_type.to_string()
            ],
        )?;
        
        Ok(())
    }
    
    pub fn get_block(&self, block_id: &str) -> Result<Option<Block>, AppError> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, text, parent_id, order_index, properties, 
                    created_at, updated_at, created_by, last_edited_by, 
                    version, block_type 
             FROM blocks 
             WHERE id = ?1 AND is_deleted = FALSE"
        )?;
        
        let block_iter = stmt.query_map([block_id], |row| {
            Ok(Block {
                id: row.get(0)?,
                text: row.get(1)?,
                parent_id: row.get(2)?,
                children: vec![], // Will be populated separately
                order: row.get(3)?,
                properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                created_by: row.get(7)?,
                last_edited_by: row.get(8)?,
                version: row.get(9)?,
                block_type: row.get::<_, String>(10)?.parse().unwrap_or_default(),
            })
        })?;
        
        for block in block_iter {
            let mut block = block?;
            // Load children
            block.children = self.get_block_children(&block.id)?;
            return Ok(Some(block));
        }
        
        Ok(None)
    }
    
    pub fn get_blocks_by_page(&self, page_id: &str) -> Result<Vec<Block>, AppError> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT b.id, b.text, b.parent_id, b.order_index, b.properties,
                    b.created_at, b.updated_at, b.created_by, b.last_edited_by,
                    b.version, b.block_type
             FROM blocks b
             INNER JOIN page_blocks pb ON b.id = pb.block_id
             WHERE pb.page_id = ?1 AND b.is_deleted = FALSE
             ORDER BY b.parent_id, b.order_index"
        )?;
        
        let block_iter = stmt.query_map([page_id], |row| {
            Ok(Block {
                id: row.get(0)?,
                text: row.get(1)?,
                parent_id: row.get(2)?,
                children: vec![],
                order: row.get(3)?,
                properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                created_by: row.get(7)?,
                last_edited_by: row.get(8)?,
                version: row.get(9)?,
                block_type: row.get::<_, String>(10)?.parse().unwrap_or_default(),
            })
        })?;
        
        let mut blocks = Vec::new();
        for block in block_iter {
            blocks.push(block?);
        }
        
        // Build parent-child relationships
        self.build_block_hierarchy(&mut blocks)?;
        
        Ok(blocks)
    }
    
    pub fn update_block(&self, block_id: &str, updates: &serde_json::Value) -> Result<(), AppError> {
        let conn = self.connection.lock().unwrap();
        
        // Build dynamic UPDATE query based on provided fields
        let mut set_clauses = Vec::new();
        let mut params = Vec::new();
        
        if let Some(text) = updates.get("text").and_then(|v| v.as_str()) {
            set_clauses.push("text = ?");
            params.push(text.to_string());
        }
        
        if let Some(properties) = updates.get("properties") {
            set_clauses.push("properties = ?");
            params.push(properties.to_string());
        }
        
        if let Some(block_type) = updates.get("block_type").and_then(|v| v.as_str()) {
            set_clauses.push("block_type = ?");
            params.push(block_type.to_string());
        }
        
        if set_clauses.is_empty() {
            return Ok(()); // Nothing to update
        }
        
        set_clauses.push("updated_at = CURRENT_TIMESTAMP");
        set_clauses.push("version = version + 1");
        
        let query = format!(
            "UPDATE blocks SET {} WHERE id = ? AND is_deleted = FALSE",
            set_clauses.join(", ")
        );
        
        params.push(block_id.to_string());
        
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();
        
        conn.execute(&query, params_refs.as_slice())?;
        
        Ok(())
    }
    
    pub fn delete_block(&self, block_id: &str) -> Result<(), AppError> {
        let conn = self.connection.lock().unwrap();
        
        // Soft delete - mark as deleted instead of removing
        conn.execute(
            "UPDATE blocks SET is_deleted = TRUE, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ?1",
            [block_id],
        )?;
        
        Ok(())
    }
    
    // Helper methods
    fn get_block_children(&self, parent_id: &str) -> Result<Vec<String>, AppError> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id FROM blocks 
             WHERE parent_id = ?1 AND is_deleted = FALSE 
             ORDER BY order_index"
        )?;
        
        let children_iter = stmt.query_map([parent_id], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut children = Vec::new();
        for child in children_iter {
            children.push(child?);
        }
        
        Ok(children)
    }
    
    fn build_block_hierarchy(&self, blocks: &mut Vec<Block>) -> Result<(), AppError> {
        // Create a map for quick lookup
        let mut block_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for (i, block) in blocks.iter().enumerate() {
            block_map.insert(block.id.clone(), i);
        }
        
        // Build children relationships
        for i in 0..blocks.len() {
            let parent_id = blocks[i].parent_id.clone();
            if let Some(parent_id) = parent_id {
                if let Some(&parent_index) = block_map.get(&parent_id) {
                    let child_id = blocks[i].id.clone();
                    blocks[parent_index].children.push(child_id);
                }
            }
        }
        
        Ok(())
    }
    
    // Transaction support
    pub fn execute_transaction<F, R>(&self, f: F) -> Result<R, AppError>
    where
        F: FnOnce(&Transaction) -> Result<R, AppError>,
    {
        let mut conn = self.connection.lock().unwrap();
        let tx = conn.transaction()?;
        
        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback()?;
                Err(e)
            }
        }
    }
    
    // Search functionality
    pub fn search_blocks(&self, query: &str) -> Result<Vec<Block>, AppError> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT b.id, b.text, b.parent_id, b.order_index, b.properties,
                    b.created_at, b.updated_at, b.created_by, b.last_edited_by,
                    b.version, b.block_type,
                    rank
             FROM search_index s
             INNER JOIN blocks b ON s.block_id = b.id
             WHERE search_index MATCH ?1 AND b.is_deleted = FALSE
             ORDER BY rank"
        )?;
        
        let block_iter = stmt.query_map([query], |row| {
            Ok(Block {
                id: row.get(0)?,
                text: row.get(1)?,
                parent_id: row.get(2)?,
                children: vec![],
                order: row.get(3)?,
                properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                created_by: row.get(7)?,
                last_edited_by: row.get(8)?,
                version: row.get(9)?,
                block_type: row.get::<_, String>(10)?.parse().unwrap_or_default(),
            })
        })?;
        
        let mut blocks = Vec::new();
        for block in block_iter {
            blocks.push(block?);
        }
        
        Ok(blocks)
    }
}

// Database initialization and migration
pub fn initialize_database(db_path: &Path) -> Result<DatabaseService, AppError> {
    DatabaseService::new(db_path)
}
```

#### Tasks
- [ ] Implement SQLite connection management
- [ ] Create database schema and migrations
- [ ] Add block CRUD operations
- [ ] Implement full-text search
- [ ] Add transaction support
- [ ] Create backup and restore functionality
- [ ] Optimize queries for performance
- [ ] Add connection pooling

#### Testing
- [ ] Unit tests for all database operations
- [ ] Test transaction rollback scenarios
- [ ] Performance tests with large datasets
- [ ] Test search functionality
- [ ] Validate data integrity constraints

#### Validation
- [ ] All CRUD operations work correctly
- [ ] Transactions maintain data consistency
- [ ] Search returns relevant results
- [ ] Performance meets targets

### 2.2 Git Manager Service

#### Planning
- [ ] Define Git integration strategy
- [ ] Plan automatic commit system
- [ ] Design conflict resolution
- [ ] Specify branch management

#### Design
- [ ] Create Git workflow architecture
- [ ] Design commit message generation
- [ ] Plan diff visualization
- [ ] Design merge strategies

#### Implementation
```rust
// services/git_manager.rs
use git2::{Repository, Signature, Oid, DiffOptions, Time};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};

use crate::errors::AppError;
use crate::models::{GitCommit, GitStatus, GitDiff};

pub struct GitManager {
    repo_path: PathBuf,
    repo: Option<Repository>,
}

impl GitManager {
    pub fn new(repo_path: PathBuf) -> Self {
        GitManager {
            repo_path,
            repo: None,
        }
    }
    
    pub fn initialize(&mut self) -> Result<(), AppError> {
        if !self.repo_path.exists() {
            fs::create_dir_all(&self.repo_path)?;
        }
        
        let repo = if self.repo_path.join(".git").exists() {
            Repository::open(&self.repo_path)?
        } else {
            Repository::init(&self.repo_path)?
        };
        
        self.repo = Some(repo);
        
        // Create initial commit if repository is empty
        if self.get_commit_count()? == 0 {
            self.create_initial_commit()?;
        }
        
        Ok(())
    }
    
    fn get_repo(&self) -> Result<&Repository, AppError> {
        self.repo.as_ref().ok_or_else(|| AppError::GitError("Repository not initialized".to_string()))
    }
    
    pub fn commit_changes(&self, message: &str) -> Result<String, AppError> {
        let repo = self.get_repo()?;
        
        // Add all files to index
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        
        // Check if there are any changes to commit
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        let head = repo.head()?;
        let parent_commit = head.peel_to_commit()?;
        
        // Check if tree has changed
        if parent_commit.tree()?.id() == tree.id() {
            return Ok("No changes to commit".to_string());
        }
        
        // Create signature
        let signature = self.create_signature()?;
        
        // Create commit
        let commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;
        
        Ok(commit_id.to_string())
    }
    
    pub fn auto_commit(&self, files_changed: &[String]) -> Result<Option<String>, AppError> {
        if files_changed.is_empty() {
            return Ok(None);
        }
        
        let message = if files_changed.len() == 1 {
            format!("Auto-save: Updated {}", files_changed[0])
        } else {
            format!("Auto-save: Updated {} files", files_changed.len())
        };
        
        match self.commit_changes(&message) {
            Ok(commit_id) => Ok(Some(commit_id)),
            Err(_) => Ok(None), // Ignore errors for auto-commits
        }
    }
    
    pub fn get_commit_history(&self, limit: Option<usize>) -> Result<Vec<GitCommit>, AppError> {
        let repo = self.get_repo()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        
        let mut commits = Vec::new();
        let limit = limit.unwrap_or(100);
        
        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }
            
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            
            let author = commit.author();
            let committer = commit.committer();
            
            commits.push(GitCommit {
                id: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author_name: author.name().unwrap_or("").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                committer_name: committer.name().unwrap_or("").to_string(),
                committer_email: committer.email().unwrap_or("").to_string(),
                timestamp: DateTime::<Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(author.when().seconds(), 0)
                        .unwrap_or_default(),
                    Utc
                ),
                parent_ids: commit.parent_ids().map(|id| id.to_string()).collect(),
            });
        }
        
        Ok(commits)
    }
    
    pub fn get_file_diff(&self, commit_id: &str, file_path: &str) -> Result<GitDiff, AppError> {
        let repo = self.get_repo()?;
        let oid = Oid::from_str(commit_id)?;
        let commit = repo.find_commit(oid)?;
        
        let current_tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };
        
        let mut diff_options = DiffOptions::new();
        diff_options.pathspec(file_path);
        
        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&current_tree),
            Some(&mut diff_options),
        )?;
        
        let mut diff_content = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_content.push_str(&String::from_utf8_lossy(line.content()));
            true
        })?;
        
        Ok(GitDiff {
            file_path: file_path.to_string(),
            commit_id: commit_id.to_string(),
            content: diff_content,
            additions: 0, // TODO: Calculate from diff
            deletions: 0, // TODO: Calculate from diff
        })
    }
    
    pub fn restore_file(&self, commit_id: &str, file_path: &str) -> Result<(), AppError> {
        let repo = self.get_repo()?;
        let oid = Oid::from_str(commit_id)?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        
        let entry = tree.get_path(Path::new(file_path))?;
        let blob = repo.find_blob(entry.id())?;
        
        let full_path = self.repo_path.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(full_path, blob.content())?;
        
        Ok(())
    }
    
    pub fn get_status(&self) -> Result<GitStatus, AppError> {
        let repo = self.get_repo()?;
        let statuses = repo.statuses(None)?;
        
        let mut modified_files = Vec::new();
        let mut new_files = Vec::new();
        let mut deleted_files = Vec::new();
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();
            
            if status.contains(git2::Status::WT_MODIFIED) || status.contains(git2::Status::INDEX_MODIFIED) {
                modified_files.push(path);
            } else if status.contains(git2::Status::WT_NEW) || status.contains(git2::Status::INDEX_NEW) {
                new_files.push(path);
            } else if status.contains(git2::Status::WT_DELETED) || status.contains(git2::Status::INDEX_DELETED) {
                deleted_files.push(path);
            }
        }
        
        Ok(GitStatus {
            modified_files,
            new_files,
            deleted_files,
            is_clean: statuses.is_empty(),
            current_branch: self.get_current_branch()?,
        })
    }
    
    pub fn create_branch(&self, branch_name: &str) -> Result<(), AppError> {
        let repo = self.get_repo()?;
        let head = repo.head()?;
        let target = head.target().ok_or_else(|| AppError::GitError("No HEAD target".to_string()))?;
        let commit = repo.find_commit(target)?;
        
        repo.branch(branch_name, &commit, false)?;
        
        Ok(())
    }
    
    pub fn switch_branch(&self, branch_name: &str) -> Result<(), AppError> {
        let repo = self.get_repo()?;
        let branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
        let reference_name = branch.get().name().ok_or_else(|| AppError::GitError("Invalid branch reference".to_string()))?;
        
        repo.set_head(reference_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        
        Ok(())
    }
    
    // Private helper methods
    fn create_signature(&self) -> Result<Signature, AppError> {
        // TODO: Get user info from config
        let name = "Note App";
        let email = "user@noteapp.local";
        
        Ok(Signature::now(name, email)?)
    }
    
    fn create_initial_commit(&self) -> Result<(), AppError> {
        let repo = self.get_repo()?;
        
        // Create initial README
        let readme_path = self.repo_path.join("README.md");
        fs::write(&readme_path, "# My Notes\n\nThis is a local note repository managed by Note App.\n")?;
        
        // Add to index
        let mut index = repo.index()?;
        index.add_path(Path::new("README.md"))?;
        index.write()?;
        
        // Create tree
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        // Create signature
        let signature = self.create_signature()?;
        
        // Create initial commit
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
        
        Ok(())
    }
    
    fn get_commit_count(&self) -> Result<usize, AppError> {
        let repo = self.get_repo()?;
        let mut revwalk = repo.revwalk()?;
        
        match revwalk.push_head() {
            Ok(_) => Ok(revwalk.count()),
            Err(_) => Ok(0), // No commits yet
        }
    }
    
    fn get_current_branch(&self) -> Result<String, AppError> {
        let repo = self.get_repo()?;
        let head = repo.head()?;
        
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("detached".to_string())
        }
    }
}

// Auto-commit scheduler
pub struct AutoCommitScheduler {
    git_manager: GitManager,
    interval_seconds: u64,
    pending_files: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl AutoCommitScheduler {
    pub fn new(git_manager: GitManager, interval_seconds: u64) -> Self {
        AutoCommitScheduler {
            git_manager,
            interval_seconds,
            pending_files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    
    pub fn add_pending_file(&self, file_path: String) {
        let mut files = self.pending_files.lock().unwrap();
        if !files.contains(&file_path) {
            files.push(file_path);
        }
    }
    
    pub async fn start(&self) -> Result<(), AppError> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(self.interval_seconds));
        let files = self.pending_files.clone();
        
        loop {
            interval.tick().await;
            
            let pending_files = {
                let mut files_guard = files.lock().unwrap();
                if files_guard.is_empty() {
                    continue;
                }
                let files = files_guard.clone();
                files_guard.clear();
                files
            };
            
            if let Err(e) = self.git_manager.auto_commit(&pending_files) {
                eprintln!("Auto-commit failed: {:?}", e);
            }
        }
    }
}
```

#### Tasks
- [ ] Implement Git repository initialization
- [ ] Add automatic commit functionality
- [ ] Create commit history retrieval
- [ ] Implement file diff generation
- [ ] Add branch management
- [ ] Create file restoration from commits
- [ ] Implement auto-commit scheduler
- [ ] Add conflict resolution handling

#### Testing
- [ ] Test repository initialization
- [ ] Validate commit operations
- [ ] Test branch operations
- [ ] Verify diff generation
- [ ] Test auto-commit functionality

#### Validation
- [ ] Git operations work correctly
- [ ] Auto-commits happen reliably
- [ ] Version history is preserved
- [ ] File restoration works properly

### 2.3 CRDT Engine Service

#### Planning
- [ ] Choose CRDT implementation (Yjs/Automerge)
- [ ] Design conflict resolution strategy
- [ ] Plan event sourcing system
- [ ] Specify merge semantics

#### Design
- [ ] Create CRDT architecture
- [ ] Design event processing pipeline
- [ ] Plan state synchronization
- [ ] Design conflict resolution UI

#### Implementation
```rust
// services/crdt_engine.rs
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Block, BlockEvent, VectorClock};
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTOperation {
    pub id: String,
    pub operation_type: OperationType,
    pub block_id: String,
    pub data: serde_json::Value,
    pub vector_clock: VectorClock,
    pub user_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Insert,
    Delete,
    Update,
    Move,
}

pub struct CRDTEngine {
    // Local state
    blocks: HashMap<String, Block>,
    
    // Operation log for synchronization
    operations: VecDeque<CRDTOperation>,
    
    // Vector clock for causality
    vector_clock: VectorClock,
    
    // User ID for this instance
    user_id: String,
    
    // Pending operations to be applied
    pending_ops: VecDeque<CRDTOperation>,
}

impl CRDTEngine {
    pub fn new(user_id: String) -> Self {
        CRDTEngine {
            blocks: HashMap::new(),
            operations: VecDeque::new(),
            vector_clock: VectorClock::new(),
            user_id,
            pending_ops: VecDeque::new(),
        }
    }
    
    pub fn apply_local_operation(&mut self, operation: CRDTOperation) -> Result<(), AppError> {
        // Increment vector clock
        self.vector_clock.increment(&self.user_id);
        
        // Apply operation locally
        self.apply_operation(&operation)?;
        
        // Add to operation log
        self.operations.push_back(operation);
        
        Ok(())
    }
    
    pub fn apply_remote_operation(&mut self, operation: CRDTOperation) -> Result<(), AppError> {
        // Check if we've already seen this operation
        if self.has_operation(&operation.id) {
            return Ok(());
        }
        
        // Check causality - ensure we have all prerequisite operations
        if !self.can_apply_operation(&operation) {
            self.pending_ops.push_back(operation);
            return Ok(());
        }
        
        // Apply the operation
        self.apply_operation(&operation)?;
        
        // Update vector clock
        self.vector_clock.merge(&operation.vector_clock);
        
        // Add to operation log
        self.operations.push_back(operation);
        
        // Try to apply any pending operations
        self.try_apply_pending_operations()?;
        
        Ok(())
    }
    
    fn apply_operation(&mut self, operation: &CRDTOperation) -> Result<(), AppError> {
        match operation.operation_type {
            OperationType::Insert => self.apply_insert(operation),
            OperationType::Delete => self.apply_delete(operation),
            OperationType::Update => self.apply_update(operation),
            OperationType::Move => self.apply_move(operation),
        }
    }
    
    fn apply_insert(&mut self, operation: &CRDTOperation) -> Result<(), AppError> {
        let block_data: Block = serde_json::from_value(operation.data.clone())?;
        
        // Check for conflicts - if block already exists, we need to resolve
        if let Some(existing_block) = self.blocks.get(&operation.block_id) {
            // Use timestamp as tie-breaker for concurrent inserts
            if operation.timestamp > existing_block.created_at.timestamp() as u64 {
                self.blocks.insert(operation.block_id.clone(), block_data);
            }
        } else {
            self.blocks.insert(operation.block_id.clone(), block_data);
        }
        
        Ok(())
    }
    
    fn apply_delete(&mut self, operation: &CRDTOperation) -> Result<(), AppError> {
        // For deletes, we keep a tombstone instead of actually removing
        if let Some(mut block) = self.blocks.get(&operation.block_id).cloned() {
            block.properties.insert("_deleted".to_string(), serde_json::Value::Bool(true));
            block.properties.insert("_deleted_at".to_string(), 
                serde_json::Value::Number(serde_json::Number::from(operation.timestamp)));
            self.blocks.insert(operation.block_id.clone(), block);
        }
        
        Ok(())
    }
    
    fn apply_update(&mut self, operation: &CRDTOperation) -> Result<(), AppError> {
        if let Some(mut block) = self.blocks.get(&operation.block_id).cloned() {
            // Merge the update data
            if let serde_json::Value::Object(update_map) = &operation.data {
                for (key, value) in update_map {
                    match key.as_str() {
                        "text" => {
                            if let Some(text) = value.as_str() {
                                // For text updates, we need to handle concurrent edits
                                block.text = self.merge_text_changes(&block.text, text, operation)?;
                            }
                        },
                        "properties" => {
                            if let serde_json::Value::Object(props) = value {
                                for (prop_key, prop_value) in props {
                                    block.properties.insert(prop_key.clone(), prop_value.clone());
                                }
                            }
                        },
                        _ => {} // Ignore unknown fields
                    }
                }
            }
            
            block.updated_at = chrono::Utc::now().naive_utc();
            block.last_edited_by = operation.user_id.clone();
            self.blocks.insert(operation.block_id.clone(), block);
        }
        
        Ok(())
    }
    
    fn apply_move(&mut self, operation: &CRDTOperation) -> Result<(), AppError> {
        if let Some(mut block) = self.blocks.get(&operation.block_id).cloned() {
            if let Some(new_parent_id) = operation.data.get("new_parent_id").and_then(|v| v.as_str()) {
                block.parent_id = Some(new_parent_id.to_string());
            }
            
            if let Some(new_order) = operation.data.get("new_order").and_then(|v| v.as_i64()) {
                block.order = new_order as i32;
            }
            
            self.blocks.insert(operation.block_id.clone(), block);
        }
        
        Ok(())
    }
    
    fn merge_text_changes(&self, current_text: &str, new_text: &str, operation: &CRDTOperation) -> Result<String, AppError> {
        // Simple text merging - in a real implementation, you'd use
        // more sophisticated algorithms like operational transformation
        
        // For now, just use the newest change based on timestamp
        // TODO: Implement proper text CRDT (like Yjs text type)
        Ok(new_text.to_string())
    }
    
    fn has_operation(&self, operation_id: &str) -> bool {
        self.operations.iter().any(|op| op.id == operation_id)
    }
    
    fn can_apply_operation(&self, operation: &CRDTOperation) -> bool {
        // Check if all causal dependencies are satisfied
        for (user_id, timestamp) in &operation.vector_clock.clocks {
            if let Some(local_timestamp) = self.vector_clock.clocks.get(user_id) {
                if timestamp > local_timestamp {
                    return false;
                }
            } else if *timestamp > 0 {
                return false;
            }
        }
        true
    }
    
    fn try_apply_pending_operations(&mut self) -> Result<(), AppError> {
        let mut applied_any = true;
        
        while applied_any {
            applied_any = false;
            let mut remaining_ops = VecDeque::new();
            
            while let Some(op) = self.pending_ops.pop_front() {
                if self.can_apply_operation(&op) {
                    self.apply_operation(&op)?;
                    self.vector_clock.merge(&op.vector_clock);
                    self.operations.push_back(op);
                    applied_any = true;
                } else {
                    remaining_ops.push_back(op);
                }
            }
            
            self.pending_ops = remaining_ops;
        }
        
        Ok(())
    }
    
    pub fn get_state_since(&self, vector_clock: &VectorClock) -> Vec<CRDTOperation> {
        // Return all operations that happened after the given vector clock
        self.operations.iter()
            .filter(|op| !vector_clock.happened_before(&op.vector_clock))
            .cloned()
            .collect()
    }
    
    pub fn get_current_state(&self) -> HashMap<String, Block> {
        // Filter out deleted blocks
        self.blocks.iter()
            .filter(|(_, block)| {
                !block.properties.get("_deleted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn create_operation(&mut self, operation_type: OperationType, block_id: String, data: serde_json::Value) -> CRDTOperation {
        self.vector_clock.increment(&self.user_id);
        
        CRDTOperation {
            id: Uuid::new_v4().to_string(),
            operation_type,
            block_id,
            data,
            vector_clock: self.vector_clock.clone(),
            user_id: self.user_id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

// Vector Clock implementation
impl VectorClock {
    pub fn new() -> Self {
        VectorClock {
            clocks: HashMap::new(),
        }
    }
    
    pub fn increment(&mut self, user_id: &str) {
        let current = self.clocks.get(user_id).unwrap_or(&0);
        self.clocks.insert(user_id.to_string(), current + 1);
    }
    
    pub fn merge(&mut self, other: &VectorClock) {
        for (user_id, timestamp) in &other.clocks {
            let current = self.clocks.get(user_id).unwrap_or(&0);
            self.clocks.insert(user_id.clone(), (*current).max(*timestamp));
        }
    }
    
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        // Check if this vector clock happened before the other
        for (user_id, other_timestamp) in &other.clocks {
            let self_timestamp = self.clocks.get(user_id).unwrap_or(&0);
            if self_timestamp > other_timestamp {
                return false;
            }
        }
        
        // Check if they're equal (concurrent)
        for (user_id, self_timestamp) in &self.clocks {
            let other_timestamp = other.clocks.get(user_id).unwrap_or(&0);
            if self_timestamp != other_timestamp {
                return true;
            }
        }
        
        false // They're equal
    }
}
```

#### Tasks
- [ ] Implement CRDT operations (insert, delete, update, move)
- [ ] Add vector clock for causality tracking
- [ ] Create conflict resolution algorithms
- [ ] Implement state synchronization
- [ ] Add operation queuing for dependencies
- [ ] Create text merging algorithms
- [ ] Add tombstone handling for deletes
- [ ] Implement state snapshots

#### Testing
- [ ] Test concurrent operation scenarios
- [ ] Validate conflict resolution
- [ ] Test causality ordering
- [ ] Verify state consistency
- [ ] Test merge operations

#### Validation
- [ ] Concurrent edits merge correctly
- [ ] No data loss in conflict scenarios
- [ ] Causality is preserved
- [ ] Performance is acceptable for real-time use

## 3. Tauri Command Handlers

### 3.1 Block Commands

#### Implementation
```rust
// commands/blocks.rs
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::services::{DatabaseService, CRDTEngine};
use crate::models::{Block, CreateBlockData, UpdateBlockData};
use crate::errors::AppError;

#[derive(Deserialize)]
pub struct CreateBlockRequest {
    pub text: String,
    pub parent_id: Option<String>,
    pub block_type: Option<String>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateBlockRequest {
    pub text: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub block_type: Option<String>,
}

#[tauri::command]
pub async fn create_block(
    request: CreateBlockRequest,
    db: State<'_, DatabaseService>,
    crdt: State<'_, std::sync::Mutex<CRDTEngine>>,
) -> Result<Block, String> {
    let block_id = uuid::Uuid::new_v4().to_string();
    
    let block = Block {
        id: block_id.clone(),
        text: request.text,
        parent_id: request.parent_id,
        children: vec![],
        order: 0, // TODO: Calculate proper order
        properties: request.properties.unwrap_or_default(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        created_by: "local_user".to_string(), // TODO: Get from user context
        last_edited_by: "local_user".to_string(),
        version: 1,
        block_type: request.block_type.unwrap_or_else(|| "text".to_string()),
    };
    
    // Apply to CRDT
    let operation = {
        let mut crdt_engine = crdt.lock().unwrap();
        crdt_engine.create_operation(
            crate::services::crdt_engine::OperationType::Insert,
            block_id.clone(),
            serde_json::to_value(&block).map_err(|e| e.to_string())?,
        )
    };
    
    // Apply locally
    {
        let mut crdt_engine = crdt.lock().unwrap();
        crdt_engine.apply_local_operation(operation).map_err(|e| e.to_string())?;
    }
    
    // Persist to database
    db.create_block(&block).map_err(|e| e.to_string())?;
    
    Ok(block)
}

#[tauri::command]
pub async fn update_block(
    block_id: String,
    request: UpdateBlockRequest,
    db: State<'_, DatabaseService>,
    crdt: State<'_, std::sync::Mutex<CRDTEngine>>,
) -> Result<Block, String> {
    // Create update data
    let mut update_data = serde_json::Map::new();
    
    if let Some(text) = request.text {
        update_data.insert("text".to_string(), serde_json::Value::String(text));
    }
    
    if let Some(properties) = request.properties {
        update_data.insert("properties".to_string(), properties);
    }
    
    if let Some(block_type) = request.block_type {
        update_data.insert("block_type".to_string(), serde_json::Value::String(block_type));
    }
    
    // Apply to CRDT
    let operation = {
        let mut crdt_engine = crdt.lock().unwrap();
        crdt_engine.create_operation(
            crate::services::crdt_engine::OperationType::Update,
            block_id.clone(),
            serde_json::Value::Object(update_data.clone()),
        )
    };
    
    // Apply locally
    {
        let mut crdt_engine = crdt.lock().unwrap();
        crdt_engine.apply_local_operation(operation).map_err(|e| e.to_string())?;
    }
    
    // Persist to database
    db.update_block(&block_id, &serde_json::Value::Object(update_data))
        .map_err(|e| e.to_string())?;
    
    // Return updated block
    db.get_block(&block_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Block not found".to_string())
}

#[tauri::command]
pub async fn delete_block(
    block_id: String,
    db: State<'_, DatabaseService>,
    crdt: State<'_, std::sync::Mutex<CRDTEngine>>,
) -> Result<(), String> {
    // Apply to CRDT
    let operation = {
        let mut crdt_engine = crdt.lock().unwrap();
        crdt_engine.create_operation(
            crate::services::crdt_engine::OperationType::Delete,
            block_id.clone(),
            serde_json::Value::Null,
        )
    };
    
    // Apply locally
    {
        let mut crdt_engine = crdt.lock().unwrap();
        crdt_engine.apply_local_operation(operation).map_err(|e| e.to_string())?;
    }
    
    // Persist to database
    db.delete_block(&block_id).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_block(
    block_id: String,
    db: State<'_, DatabaseService>,
) -> Result<Option<Block>, String> {
    db.get_block(&block_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_blocks_by_page(
    page_id: String,
    db: State<'_, DatabaseService>,
) -> Result<Vec<Block>, String> {
    db.get_blocks_by_page(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_blocks(
    query: String,
    db: State<'_, DatabaseService>,
) -> Result<Vec<Block>, String> {
    db.search_blocks(&query).map_err(|e| e.to_string())
}
```

#### Tasks
- [ ] Implement all block CRUD commands
- [ ] Add CRDT integration to commands
- [ ] Create error handling and validation
- [ ] Add user context and permissions
- [ ] Implement batch operations
- [ ] Add operation logging
- [ ] Create command result types
- [ ] Add performance monitoring

#### Testing
- [ ] Test all command endpoints
- [ ] Validate error scenarios
- [ ] Test CRDT integration
- [ ] Verify database persistence

#### Validation
- [ ] Commands work correctly from frontend
- [ ] Error handling provides useful feedback
- [ ] CRDT operations are applied correctly
- [ ] Database state remains consistent 