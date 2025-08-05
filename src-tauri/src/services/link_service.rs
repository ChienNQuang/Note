use crate::errors::AppResult;
use super::database::connection::DatabaseService;
use crate::models::Node;
use regex::Regex;

pub struct LinkService {
    db: DatabaseService,
}

impl LinkService {
    pub fn new(db: DatabaseService) -> Self {
        LinkService { db }
    }

    /// Update all links for a given node
    pub fn update_links_for_node(&self, node: &Node) -> AppResult<()> {
        self.db.with_transaction(|tx| {
            // Find all links in the node content
            let link_regex = Regex::new(r"\[\[(.*?)\]\]").unwrap();
            let link_texts: Vec<String> = link_regex
                .captures_iter(&node.content)
                .map(|cap| cap[1].to_string())
                .collect();
            
            // Remove existing links for this source node
            tx.execute(
                "DELETE FROM node_links WHERE source_node_id = ?",
                &[&node.id],
            )?;
            
            // For each link text, find the corresponding node by content
            let mut insert_stmt = tx.prepare(
                "INSERT OR IGNORE INTO node_links (source_node_id, target_node_id) VALUES (?, ?)"
            )?;
            
            for link_text in link_texts {
                // Try to find a node whose content matches the link text
                let mut find_stmt = tx.prepare(
                    "SELECT id FROM nodes WHERE content = ? OR content LIKE ? LIMIT 1"
                )?;
                
                // First try exact match, then try with wildcard to handle nodes with additional content
                if let Ok(target_id) = find_stmt.query_row(
                    rusqlite::params![&link_text, format!("{}%", link_text)],
                    |row| row.get::<_, String>(0)
                ) {
                    insert_stmt.execute(&[&node.id, &target_id])?;
                }
                // If no node found, we just skip this link (unlinked reference)
            }
            
            Ok(())
        })
    }

    /// Get all nodes that link to the given node
    pub fn get_linked_references(&self, node_id: &str) -> AppResult<Vec<Node>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT n.id, n.content, n.parent_id, n.order_index, n.properties, n.tags, n.created_at, n.updated_at, n.created_by, n.version FROM nodes n
                 JOIN node_links l ON n.id = l.source_node_id
                 WHERE l.target_node_id = ?"
            )?;
            
            let nodes = stmt.query_map(&[node_id], |row| {
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
            })?.collect::<Result<Vec<_>, _>>()?;
            
            Ok(nodes)
        })
    }

    /// Get unlinked references (basic implementation)
    pub fn get_unlinked_references(&self, _node_id: &str, node_content: &str) -> AppResult<Vec<Node>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by, version FROM nodes WHERE content LIKE ?"
            )?;
            
            let nodes = stmt.query_map([format!("%{}%", node_content)], |row| {
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
            })?.collect::<Result<Vec<_>, _>>()?;
            
            Ok(nodes)
        })
    }
} 