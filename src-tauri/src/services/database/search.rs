use crate::errors::AppResult;
use super::connection::DatabaseService;
use crate::models::Node;
use sqlx::Row;

impl DatabaseService {
    /// Search nodes by content using FTS
    pub async fn search_nodes(&self, query: &str, limit: i64) -> AppResult<Vec<Node>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT n.id, n.content, n.parent_id, n.order_index, n.properties, 
                   n.tags, n.created_at, n.updated_at, n.created_by, n.version
            FROM nodes n
            JOIN nodes_fts fts ON n.rowid = fts.rowid
            WHERE fts.content MATCH ?
            ORDER BY rank
            LIMIT ?
            "#
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            let mut node = Node {
                id: row.get("id"),
                content: row.get("content"),
                parent_id: row.get("parent_id"),
                order: row.get("order_index"),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                version: row.get("version"),
                children: Vec::new(),
            };

            // Get children for this node
            let children = sqlx::query("SELECT id FROM nodes WHERE parent_id = ?")
                .bind(&node.id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            
            node.children = children.into_iter().map(|r| r.get::<String, _>("id")).collect();
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// Search nodes by tags
    pub async fn search_nodes_by_tags(&self, tags: &[String], limit: i64) -> AppResult<Vec<Node>> {
        let tag_search = tags.join(" ");
        let rows = sqlx::query(
            r#"
            SELECT id, content, parent_id, order_index, properties, tags, 
                   created_at, updated_at, created_by, version
            FROM nodes
            WHERE tags LIKE ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#
        )
        .bind(format!("%{}%", tag_search))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            let mut node = Node {
                id: row.get("id"),
                content: row.get("content"),
                parent_id: row.get("parent_id"),
                order: row.get("order_index"),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                version: row.get("version"),
                children: Vec::new(),
            };

            // Get children for this node
            let children = sqlx::query("SELECT id FROM nodes WHERE parent_id = ?")
                .bind(&node.id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            
            node.children = children.into_iter().map(|r| r.get::<String, _>("id")).collect();
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// Search nodes by properties
    pub async fn search_nodes_by_properties(&self, property_key: &str, property_value: &str, limit: i64) -> AppResult<Vec<Node>> {
        let rows = sqlx::query(
            r#"
            SELECT id, content, parent_id, order_index, properties, tags, 
                   created_at, updated_at, created_by, version
            FROM nodes
            WHERE properties LIKE ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#
        )
        .bind(format!("%\"{}\":\"{}\"%", property_key, property_value))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            let mut node = Node {
                id: row.get("id"),
                content: row.get("content"),
                parent_id: row.get("parent_id"),
                order: row.get("order_index"),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                version: row.get("version"),
                children: Vec::new(),
            };

            // Get children for this node
            let children = sqlx::query("SELECT id FROM nodes WHERE parent_id = ?")
                .bind(&node.id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            
            node.children = children.into_iter().map(|r| r.get::<String, _>("id")).collect();
            nodes.push(node);
        }

        Ok(nodes)
    }
}