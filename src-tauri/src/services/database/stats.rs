use crate::errors::AppResult;
use super::connection::DatabaseService;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_nodes: i64,
    pub total_links: i64,
    pub root_nodes: i64,
    pub leaf_nodes: i64,
    pub nodes_with_children: i64,
    pub avg_node_depth: f64,
    pub max_node_depth: i32,
    pub journal_nodes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeStats {
    pub node_id: String,
    pub descendant_count: i64,
    pub depth: i32,
    pub direct_children: i64,
}

impl DatabaseService {
    /// Get overall database statistics
    pub async fn get_database_stats(&self) -> AppResult<DatabaseStats> {
        let total_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let total_links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_links")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let root_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE parent_id IS NULL")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let nodes_with_children: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT parent_id) FROM nodes WHERE parent_id IS NOT NULL")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let leaf_nodes = total_nodes - nodes_with_children;
        
        let journal_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE tags LIKE '%\"#Journal\"%'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        // Calculate average and max depth using recursive CTE
        let depth_stats = sqlx::query(
            r#"
            WITH RECURSIVE node_depth AS (
                SELECT id, 0 as depth FROM nodes WHERE parent_id IS NULL
                UNION ALL
                SELECT n.id, nd.depth + 1
                FROM nodes n
                INNER JOIN node_depth nd ON n.parent_id = nd.id
            )
            SELECT AVG(CAST(depth AS REAL)) as avg_depth, MAX(depth) as max_depth
            FROM node_depth
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        Ok(DatabaseStats {
            total_nodes,
            total_links,
            root_nodes,
            leaf_nodes,
            nodes_with_children,
            avg_node_depth: depth_stats.get::<Option<f64>, _>("avg_depth").unwrap_or(0.0),
            max_node_depth: depth_stats.get::<Option<i32>, _>("max_depth").unwrap_or(0),
            journal_nodes,
        })
    }
    
    /// Get statistics for a specific node
    pub async fn get_node_stats(&self, node_id: &str) -> AppResult<NodeStats> {
        // Get direct children count
        let direct_children: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM nodes WHERE parent_id = ?"
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        // Get descendant count using recursive CTE
        let descendant_count: i64 = sqlx::query_scalar(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM nodes WHERE parent_id = ?
                UNION ALL
                SELECT n.id FROM nodes n
                INNER JOIN descendants d ON n.parent_id = d.id
            )
            SELECT COUNT(*) FROM descendants
            "#
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        // Get depth of this node
        let depth: i32 = sqlx::query_scalar(
            r#"
            WITH RECURSIVE node_path AS (
                SELECT id, parent_id, 0 as depth FROM nodes WHERE id = ?
                UNION ALL
                SELECT n.id, n.parent_id, np.depth + 1
                FROM nodes n
                INNER JOIN node_path np ON n.id = np.parent_id
            )
            SELECT MAX(depth) FROM node_path
            "#
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        Ok(NodeStats {
            node_id: node_id.to_string(),
            descendant_count,
            depth,
            direct_children,
        })
    }
    
    /// Get link statistics
    pub async fn get_link_stats(&self) -> AppResult<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"
            SELECT target_node_id, COUNT(*) as reference_count
            FROM node_links
            GROUP BY target_node_id
            ORDER BY reference_count DESC
            LIMIT 20
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let stats = rows.into_iter()
            .map(|row| (row.get("target_node_id"), row.get("reference_count")))
            .collect();
        
        Ok(stats)
    }
}