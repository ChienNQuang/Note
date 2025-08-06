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
    pub async fn update_links_for_node(&self, node: &Node) -> AppResult<()> {
        let mut tx = self.db.pool().begin().await?;
        
        // Find all links in the node content
        let link_regex = Regex::new(r"\[\[(.*?)\]\]").unwrap();
        let link_texts: Vec<String> = link_regex
            .captures_iter(&node.content)
            .map(|cap| cap[1].to_string())
            .collect();
        
        // Remove existing links for this source node
        sqlx::query("DELETE FROM node_links WHERE source_node_id = ?")
            .bind(&node.id)
            .execute(&mut *tx)
            .await?;
        
        // For each link text, find the corresponding node by content
        for link_text in link_texts {
            // Try to find a node whose content matches the link text
            let result = sqlx::query_scalar::<_, String>(
                "SELECT id FROM nodes WHERE content = ? OR content LIKE ? LIMIT 1"
            )
            .bind(&link_text)
            .bind(format!("{}%", link_text))
            .fetch_optional(&mut *tx)
            .await?;
            
            if let Some(target_id) = result {
                sqlx::query("INSERT OR IGNORE INTO node_links (source_node_id, target_node_id) VALUES (?, ?)")
                    .bind(&node.id)
                    .bind(&target_id)
                    .execute(&mut *tx)
                    .await?;
            }
            // If no node found, we just skip this link (unlinked reference)
        }
        
        tx.commit().await?;
        Ok(())
    }

    /// Get all nodes that link to a specific node
    pub async fn get_backlinks(&self, node_id: &str) -> AppResult<Vec<Node>> {
        let node_ids = sqlx::query_scalar::<_, String>(
            "SELECT source_node_id FROM node_links WHERE target_node_id = ?"
        )
        .bind(node_id)
        .fetch_all(self.db.pool())
        .await?;
        
        let mut nodes = Vec::new();
        for id in node_ids {
            if let Ok(node) = self.db.get_node(&id).await {
                nodes.push(node);
            }
        }
        
        Ok(nodes)
    }

    /// Get all nodes that are linked from a specific node
    pub async fn get_outgoing_links(&self, node_id: &str) -> AppResult<Vec<Node>> {
        let node_ids = sqlx::query_scalar::<_, String>(
            "SELECT target_node_id FROM node_links WHERE source_node_id = ?"
        )
        .bind(node_id)
        .fetch_all(self.db.pool())
        .await?;
        
        let mut nodes = Vec::new();
        for id in node_ids {
            if let Ok(node) = self.db.get_node(&id).await {
                nodes.push(node);
            }
        }
        
        Ok(nodes)
    }
}