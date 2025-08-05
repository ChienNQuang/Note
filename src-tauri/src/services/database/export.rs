use crate::errors::{AppResult, AppError};
use super::connection::DatabaseService;
use crate::models::{Node, NodeWithChildren};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub export_date: chrono::DateTime<chrono::Utc>,
    pub nodes: Vec<Node>,
    pub links: Vec<NodeLink>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeLink {
    pub source_node_id: String,
    pub target_node_id: String,
}

impl DatabaseService {
    /// Export all nodes and links to JSON
    pub fn export_to_json(&self, path: &Path) -> AppResult<()> {
        let nodes = self.get_all_nodes()?;
        let links = self.get_all_links()?;
        
        let export_data = ExportData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            export_date: chrono::Utc::now(),
            nodes,
            links,
        };
        
        let json = serde_json::to_string_pretty(&export_data)
            .map_err(|e| AppError::Internal(format!("Failed to serialize export data: {}", e)))?;
        
        fs::write(path, json)
            .map_err(|e| AppError::ExportFailed(format!("Failed to write export file: {}", e)))?;
        
        Ok(())
    }
    
    /// Import nodes and links from JSON
    pub fn import_from_json(&self, path: &Path) -> AppResult<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| AppError::FileNotFound(format!("Failed to read import file: {}", e)))?;
        
        let export_data: ExportData = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse import data: {}", e)))?;
        
        self.with_transaction(|conn| {
            // Import nodes
            for node in &export_data.nodes {
                conn.execute(
                    "INSERT OR REPLACE INTO nodes (id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by, version)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![
                        node.id,
                        node.content,
                        node.parent_id,
                        node.order,
                        serde_json::to_string(&node.properties).unwrap(),
                        serde_json::to_string(&node.tags).unwrap(),
                        node.created_at,
                        node.updated_at,
                        node.created_by,
                        node.version,
                    ],
                )?;
            }
            
            // Import links
            for link in &export_data.links {
                conn.execute(
                    "INSERT OR REPLACE INTO node_links (source_node_id, target_node_id)
                     VALUES (?1, ?2)",
                    rusqlite::params![link.source_node_id, link.target_node_id],
                )?;
            }
            
            Ok(())
        })
    }
    
    /// Export a specific node and its descendants to markdown
    pub fn export_node_to_markdown(&self, node_id: &str) -> AppResult<String> {
        let node_with_children = self.get_node_with_children(node_id)?;
        Ok(self.node_to_markdown(&node_with_children, 0))
    }
    
    fn node_to_markdown(&self, node: &NodeWithChildren, level: usize) -> String {
        let mut markdown = String::new();
        
        // Add indentation for nested nodes
        let indent = "  ".repeat(level);
        
        // Add the node content as a bullet point
        markdown.push_str(&format!("{}* {}\n", indent, node.node.content));
        
        // Recursively add children
        for child in &node.child_nodes {
            markdown.push_str(&self.node_to_markdown(child, level + 1));
        }
        
        markdown
    }
    
    /// Export all nodes as a flat markdown list
    pub fn export_all_to_markdown(&self) -> AppResult<String> {
        let root_nodes = self.get_root_nodes()?;
        let mut markdown = String::new();
        
        markdown.push_str("# Note Export\n\n");
        markdown.push_str(&format!("*Exported on: {}*\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        for root in root_nodes {
            let node_with_children = self.get_node_with_children(&root.id)?;
            markdown.push_str(&self.node_to_markdown(&node_with_children, 0));
            markdown.push_str("\n");
        }
        
        Ok(markdown)
    }
    
    // Helper methods
    fn get_all_nodes(&self) -> AppResult<Vec<Node>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("
                SELECT id, content, parent_id, order_index, properties, tags, 
                       created_at, updated_at, created_by, version
                FROM nodes
                ORDER BY parent_id, order_index
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
    
    fn get_all_links(&self) -> AppResult<Vec<NodeLink>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("
                SELECT source_node_id, target_node_id
                FROM node_links
            ")?;
            
            let links = stmt.query_map([], |row| {
                Ok(NodeLink {
                    source_node_id: row.get(0)?,
                    target_node_id: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(links)
        })
    }
}