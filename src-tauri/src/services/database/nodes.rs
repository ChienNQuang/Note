use crate::errors::AppResult;
use super::connection::DatabaseService;
use crate::models::{Node, CreateNodeRequest, UpdateNodeRequest, NodeWithChildren};
use rusqlite::params;
use chrono::Utc;
use crate::utils::generate_id;

impl DatabaseService {
    pub fn create_node(&self, request: CreateNodeRequest) -> AppResult<Node> {
        let node_id = generate_id();
        let user_id = "default_user"; // Placeholder
        let now = Utc::now();
        
        self.with_transaction(|conn| {
            let properties_json = serde_json::to_string(&request.properties.unwrap_or_default())
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let tags_json = serde_json::to_string(&request.tags.unwrap_or_default())
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            
            conn.execute(
                "INSERT INTO nodes (id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    node_id,
                    request.content,
                    request.parent_id,
                    request.order.unwrap_or(0),
                    properties_json,
                    tags_json,
                    now,
                    now,
                    user_id,
                ],
            )?;
            Ok(())
        })?;
        
        self.get_node(&node_id)
    }

    pub fn get_node(&self, node_id: &str) -> AppResult<Node> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by, version FROM nodes WHERE id = ?")?;
            let mut node = stmt.query_row(&[node_id], |row| {
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
                    children: Vec::new(), // Will be populated next
                })
            })?;
            
            // Populate children array
            let mut children_stmt = conn.prepare("SELECT id FROM nodes WHERE parent_id = ? ORDER BY order_index")?;
            let children_ids = children_stmt.query_map(&[node_id], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?;
            node.children = children_ids;
            
            Ok(node)
        })
    }

    pub fn get_node_with_children(&self, node_id: &str) -> AppResult<NodeWithChildren> {
        let node = self.get_node(node_id)?;
        let children = self.get_child_nodes(node_id)?;
        Ok(NodeWithChildren {
            node,
            child_nodes: children,
        })
    }

    fn get_child_nodes(&self, parent_id: &str) -> AppResult<Vec<NodeWithChildren>> {
        let children = self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM nodes WHERE parent_id = ? ORDER BY order_index")?;
            let ids = stmt.query_map(&[parent_id], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?;
            Ok(ids)
        })?;

        children.into_iter()
            .map(|id| self.get_node_with_children(&id))
            .collect()
    }

    pub fn update_node(&self, node_id: &str, request: UpdateNodeRequest) -> AppResult<Node> {
        let now = Utc::now();
        self.with_transaction(|conn| {
            let mut updates = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(content) = request.content {
                updates.push("content = ?");
                params.push(Box::new(content));
            }

            if let Some(properties) = request.properties {
                updates.push("properties = ?");
                params.push(Box::new(serde_json::to_string(&properties)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?));
            }

            if let Some(tags) = request.tags {
                updates.push("tags = ?");
                params.push(Box::new(serde_json::to_string(&tags)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?));
            }

            updates.push("updated_at = ?");
            params.push(Box::new(now));
            updates.push("version = version + 1");

            if !updates.is_empty() {
                let sql = format!("UPDATE nodes SET {} WHERE id = ?", updates.join(", "));
                params.push(Box::new(node_id.to_string()));
                conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
            }

            Ok(())
        })?;

        self.get_node(node_id)
    }

    pub fn delete_node(&self, node_id: &str) -> AppResult<()> {
        self.with_transaction(|conn| {
            conn.execute("DELETE FROM nodes WHERE id = ?", &[node_id])?;
            Ok(())
        })
    }

    pub fn move_node(&self, node_id: &str, new_parent_id: Option<String>, new_order: i32) -> AppResult<()> {
        self.with_transaction(|conn| {
            conn.execute(
                "UPDATE nodes SET parent_id = ?, order_index = ?, updated_at = ? WHERE id = ?",
                params![new_parent_id, new_order, Utc::now(), node_id],
            )?;
            Ok(())
        })
    }

    pub fn get_daily_note(&self, date: &str) -> AppResult<Node> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("
                SELECT id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by, version 
                FROM nodes 
                WHERE content = ? 
                AND tags LIKE '%\"#Journal\"%'
            ")?;
            let node = stmt.query_row(params![date], |row| {
                Ok(Node {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    parent_id: row.get(2)?,
                    children: Vec::new(),
                    order: row.get(3)?,
                    properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    created_by: row.get(8)?,
                    version: row.get(9)?,
                })
            })?;
            Ok(node)
        })
    }

    pub fn get_or_create_daily_note(&self, date: &str) -> AppResult<Node> {
        match self.get_daily_note(date) {
            Ok(node) => Ok(node),
            Err(_) => {
                let create_request = CreateNodeRequest {
                    content: date.to_string(),
                    parent_id: None,
                    order: None,
                    properties: None,
                    tags: Some(vec!["#Journal".to_string()]),
                };
                self.create_node(create_request)
            }
        }
    }
} 