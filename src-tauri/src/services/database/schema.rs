use crate::errors::AppResult;
use super::connection::DatabaseService;

impl DatabaseService {
    pub(crate) fn initialize_schema(&self) -> AppResult<()> {
        let conn = self.get_connection()?;
        
        // Create nodes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                parent_id TEXT,
                order_index INTEGER NOT NULL,
                properties TEXT,
                tags TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT NOT NULL,
                version INTEGER DEFAULT 1,
                FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create node_links table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS node_links (
                source_node_id TEXT NOT NULL,
                target_node_id TEXT NOT NULL,
                PRIMARY KEY (source_node_id, target_node_id),
                FOREIGN KEY (source_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
                FOREIGN KEY (target_node_id) REFERENCES nodes(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create FTS table
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
                content,
                content=nodes,
                content_rowid=rowid
            )",
            [],
        )?;
        
        // Create FTS triggers
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS nodes_fts_insert AFTER INSERT ON nodes BEGIN
                INSERT INTO nodes_fts(rowid, content) VALUES (new.rowid, new.content);
            END",
            [],
        )?;
        
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS nodes_fts_delete AFTER DELETE ON nodes BEGIN
                INSERT INTO nodes_fts(nodes_fts, rowid, content) VALUES('delete', old.rowid, old.content);
            END",
            [],
        )?;
        
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS nodes_fts_update AFTER UPDATE ON nodes BEGIN
                INSERT INTO nodes_fts(nodes_fts, rowid, content) VALUES('delete', old.rowid, old.content);
                INSERT INTO nodes_fts(rowid, content) VALUES (new.rowid, new.content);
            END",
            [],
        )?;
        
        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nodes_parent_id ON nodes(parent_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nodes_order ON nodes(parent_id, order_index)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nodes_updated_at ON nodes(updated_at DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_target_id ON node_links(target_node_id)",
            [],
        )?;
        
        // Create users table
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
        
        self.return_connection(conn);
        Ok(())
    }

    pub(crate) fn ensure_default_user(&self) -> AppResult<()> {
        let conn = self.get_connection()?;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users",
            [],
            |row| row.get(0)
        )?;
        
        if count == 0 {
            conn.execute(
                "INSERT INTO users (id, name) VALUES (?1, ?2)",
                rusqlite::params!["default_user", "Local User"],
            )?;
        }
        
        self.return_connection(conn);
        Ok(())
    }
} 