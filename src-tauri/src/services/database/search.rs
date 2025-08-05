use crate::errors::AppResult;
use super::connection::DatabaseService;
use crate::models::Node;
use rusqlite::params;

impl DatabaseService {
    /// Search nodes by content using full-text search
    pub fn search_nodes(&self, query: &str, limit: Option<usize>) -> AppResult<Vec<Node>> {
        self.with_connection(|conn| {
            let limit = limit.unwrap_or(50);
            let mut stmt = conn.prepare("
                SELECT DISTINCT n.id, n.content, n.parent_id, n.order_index, n.properties, 
                       n.tags, n.created_at, n.updated_at, n.created_by, n.version
                FROM nodes n
                INNER JOIN nodes_fts ON nodes_fts.node_id = n.id
                WHERE nodes_fts MATCH ?
                ORDER BY rank
                LIMIT ?
            ")?;
            
            let nodes = stmt.query_map(params![query, limit], |row| {
                Ok(Node {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    parent_id: row.get(2)?,
                    order: row.get(3)?,
                    properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    created_by: row.get(8)?,
                    version: row.get(9)?,
                    children: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(nodes)
        })
    }
    
    /// Find nodes by tag
    pub fn find_nodes_by_tag(&self, tag: &str) -> AppResult<Vec<Node>> {
        self.with_connection(|conn| {
            let tag_search = format!("%\"{}%", tag);
            let mut stmt = conn.prepare("
                SELECT id, content, parent_id, order_index, properties, tags, 
                       created_at, updated_at, created_by, version
                FROM nodes
                WHERE tags LIKE ?
                ORDER BY updated_at DESC
            ")?;
            
            let nodes = stmt.query_map(params![tag_search], |row| {
                Ok(Node {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    parent_id: row.get(2)?,
                    order: row.get(3)?,
                    properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    created_by: row.get(8)?,
                    version: row.get(9)?,
                    children: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(nodes)
        })
    }
    
    /// Find all root nodes (nodes without parent)
    pub fn get_root_nodes(&self) -> AppResult<Vec<Node>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("
                SELECT id, content, parent_id, order_index, properties, tags, 
                       created_at, updated_at, created_by, version
                FROM nodes
                WHERE parent_id IS NULL
                ORDER BY order_index, created_at
            ")?;
            
            let nodes = stmt.query_map([], |row| {
                Ok(Node {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    parent_id: row.get(2)?,
                    order: row.get(3)?,
                    properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    created_by: row.get(8)?,
                    version: row.get(9)?,
                    children: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(nodes)
        })
    }
    
    /// Get recent nodes
    pub fn get_recent_nodes(&self, limit: usize) -> AppResult<Vec<Node>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("
                SELECT id, content, parent_id, order_index, properties, tags, 
                       created_at, updated_at, created_by, version
                FROM nodes
                ORDER BY updated_at DESC
                LIMIT ?
            ")?;
            
            let nodes = stmt.query_map(params![limit], |row| {
                Ok(Node {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    parent_id: row.get(2)?,
                    order: row.get(3)?,
                    properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    created_by: row.get(8)?,
                    version: row.get(9)?,
                    children: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(nodes)
        })
    }
}