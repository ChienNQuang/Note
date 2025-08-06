use crate::errors::{AppResult, AppError};
use super::connection::DatabaseService;
use crate::models::{Node, NodeWithChildren};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use sqlx::Row;

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
    pub async fn export_to_json(&self, path: &Path) -> AppResult<()> {
        let nodes = self.get_all_nodes().await?;
        let links = self.get_all_links().await?;
        
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
    pub async fn import_from_json(&self, path: &Path) -> AppResult<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| AppError::FileNotFound(format!("Failed to read import file: {}", e)))?;
        
        let export_data: ExportData = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse import data: {}", e)))?;
        
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::DatabaseConnectionFailed(e.to_string()))?;
        
        // Import nodes
        for node in &export_data.nodes {
            let properties_json = serde_json::to_string(&node.properties)
                .map_err(|e| AppError::Internal(format!("Failed to serialize properties: {}", e)))?;
            let tags_json = serde_json::to_string(&node.tags)
                .map_err(|e| AppError::Internal(format!("Failed to serialize tags: {}", e)))?;
            
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO nodes (id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by, version)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&node.id)
            .bind(&node.content)
            .bind(&node.parent_id)
            .bind(node.order)
            .bind(&properties_json)
            .bind(&tags_json)
            .bind(&node.created_at)
            .bind(&node.updated_at)
            .bind(&node.created_by)
            .bind(node.version)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))?;
        }
        
        // Import links
        for link in &export_data.links {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO node_links (source_node_id, target_node_id)
                VALUES (?, ?)
                "#
            )
            .bind(&link.source_node_id)
            .bind(&link.target_node_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))?;
        }
        
        tx.commit().await
            .map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Export a specific node and its descendants to markdown
    pub async fn export_node_to_markdown(&self, node_id: &str) -> AppResult<String> {
        let node_with_children = self.get_node_with_children(node_id).await?;
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
    pub async fn export_all_to_markdown(&self) -> AppResult<String> {
        let root_nodes = self.get_root_nodes().await?;
        let mut markdown = String::new();
        
        markdown.push_str("# Note Export\n\n");
        markdown.push_str(&format!("*Exported on: {}*\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        for root in root_nodes {
            let node_with_children = self.get_node_with_children(&root.id).await?;
            markdown.push_str(&self.node_to_markdown(&node_with_children, 0));
            markdown.push_str("\n");
        }
        
        Ok(markdown)
    }
    
    // Helper methods
    async fn get_all_links(&self) -> AppResult<Vec<NodeLink>> {
        let rows = sqlx::query(
            r#"
            SELECT source_node_id, target_node_id
            FROM node_links
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))?;
        
        let links = rows.into_iter().map(|row| NodeLink {
            source_node_id: row.get("source_node_id"),
            target_node_id: row.get("target_node_id"),
        }).collect();
        
        Ok(links)
    }
}