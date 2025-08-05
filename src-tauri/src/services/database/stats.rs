use crate::errors::AppResult;
use super::connection::DatabaseService;
use serde::{Deserialize, Serialize};
use rusqlite::params;

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
    pub fn get_database_stats(&self) -> AppResult<DatabaseStats> {
        self.with_connection(|conn| {
            let total_nodes: i64 = conn.query_row(
                "SELECT COUNT(*) FROM nodes",
                [],
                |row| row.get(0)
            )?;
            
            let total_links: i64 = conn.query_row(
                "SELECT COUNT(*) FROM node_links",
                [],
                |row| row.get(0)
            )?;
            
            let root_nodes: i64 = conn.query_row(
                "SELECT COUNT(*) FROM nodes WHERE parent_id IS NULL",
                [],
                |row| row.get(0)
            )?;
            
            let nodes_with_children: i64 = conn.query_row(
                "SELECT COUNT(DISTINCT parent_id) FROM nodes WHERE parent_id IS NOT NULL",
                [],
                |row| row.get(0)
            )?;
            
            let leaf_nodes = total_nodes - nodes_with_children;
            
            let journal_nodes: i64 = conn.query_row(
                "SELECT COUNT(*) FROM nodes WHERE tags LIKE '%\"#Journal\"%'",
                [],
                |row| row.get(0)
            )?;
            
            // Calculate average and max depth using recursive CTE
            let (avg_depth, max_depth): (f64, i32) = conn.query_row(
                "WITH RECURSIVE node_depth AS (
                    SELECT id, 0 as depth FROM nodes WHERE parent_id IS NULL
                    UNION ALL
                    SELECT n.id, nd.depth + 1
                    FROM nodes n
                    INNER JOIN node_depth nd ON n.parent_id = nd.id
                )
                SELECT AVG(CAST(depth AS REAL)), MAX(depth)
                FROM node_depth",
                [],
                |row| Ok((row.get(0)?, row.get(1)?))
            )?;
            
            Ok(DatabaseStats {
                total_nodes,
                total_links,
                root_nodes,
                leaf_nodes,
                nodes_with_children,
                avg_node_depth: avg_depth,
                max_node_depth: max_depth,
                journal_nodes,
            })
        })
    }
    
    /// Get statistics for a specific node
    pub fn get_node_stats(&self, node_id: &str) -> AppResult<NodeStats> {
        self.with_connection(|conn| {
            // Get direct children count
            let direct_children: i64 = conn.query_row(
                "SELECT COUNT(*) FROM nodes WHERE parent_id = ?",
                params![node_id],
                |row| row.get(0)
            )?;
            
            // Get descendant count using recursive CTE
            let descendant_count: i64 = conn.query_row(
                "WITH RECURSIVE descendants AS (
                    SELECT id FROM nodes WHERE parent_id = ?1
                    UNION ALL
                    SELECT n.id FROM nodes n
                    INNER JOIN descendants d ON n.parent_id = d.id
                )
                SELECT COUNT(*) FROM descendants",
                params![node_id],
                |row| row.get(0)
            )?;
            
            // Get depth of this node
            let depth: i32 = conn.query_row(
                "WITH RECURSIVE node_path AS (
                    SELECT id, parent_id, 0 as depth FROM nodes WHERE id = ?1
                    UNION ALL
                    SELECT n.id, n.parent_id, np.depth + 1
                    FROM nodes n
                    INNER JOIN node_path np ON n.id = np.parent_id
                )
                SELECT MAX(depth) FROM node_path",
                params![node_id],
                |row| row.get(0)
            )?;
            
            Ok(NodeStats {
                node_id: node_id.to_string(),
                descendant_count,
                depth,
                direct_children,
            })
        })
    }
    
    /// Get link statistics
    pub fn get_link_stats(&self) -> AppResult<Vec<(String, i64)>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("
                SELECT target_node_id, COUNT(*) as reference_count
                FROM node_links
                GROUP BY target_node_id
                ORDER BY reference_count DESC
                LIMIT 20
            ")?;
            
            let stats = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(stats)
        })
    }
}