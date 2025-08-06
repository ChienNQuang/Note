use crate::errors::AppResult;
use super::connection::DatabaseService;

impl DatabaseService {
    pub(crate) async fn initialize_schema(&self) -> AppResult<()> {
        // Create nodes table
        sqlx::query(
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
            )"
        )
        .execute(&self.pool)
        .await?;
        
        // Create node_links table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS node_links (
                source_node_id TEXT NOT NULL,
                target_node_id TEXT NOT NULL,
                PRIMARY KEY (source_node_id, target_node_id),
                FOREIGN KEY (source_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
                FOREIGN KEY (target_node_id) REFERENCES nodes(id) ON DELETE CASCADE
            )"
        )
        .execute(&self.pool)
        .await?;
        
        // Create FTS table
        sqlx::query(
            "CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
                content,
                content=nodes,
                content_rowid=rowid
            )"
        )
        .execute(&self.pool)
        .await?;
        
        // Create FTS triggers
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS nodes_fts_insert AFTER INSERT ON nodes BEGIN
                INSERT INTO nodes_fts(rowid, content) VALUES (new.rowid, new.content);
            END"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS nodes_fts_delete AFTER DELETE ON nodes BEGIN
                INSERT INTO nodes_fts(nodes_fts, rowid, content) VALUES('delete', old.rowid, old.content);
            END"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS nodes_fts_update AFTER UPDATE ON nodes BEGIN
                INSERT INTO nodes_fts(nodes_fts, rowid, content) VALUES('delete', old.rowid, old.content);
                INSERT INTO nodes_fts(rowid, content) VALUES (new.rowid, new.content);
            END"
        )
        .execute(&self.pool)
        .await?;
        
        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_parent_id ON nodes(parent_id)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_order ON nodes(parent_id, order_index)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_updated_at ON nodes(updated_at DESC)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_links_target_id ON node_links(target_node_id)")
            .execute(&self.pool)
            .await?;
        
        // Create users table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                preferences TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub(crate) async fn ensure_default_user(&self) -> AppResult<()> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        
        if count == 0 {
            sqlx::query("INSERT INTO users (id, name) VALUES (?1, ?2)")
                .bind("default_user")
                .bind("Local User")
                .execute(&self.pool)
                .await?;
        }
        
        Ok(())
    }
}