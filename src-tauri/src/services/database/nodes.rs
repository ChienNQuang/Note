use crate::errors::AppResult;
use super::connection::DatabaseService;
use crate::models::{Node, CreateNodeRequest, UpdateNodeRequest, NodeWithChildren};
use sqlx::Row;
use chrono::Utc;
use crate::utils::generate_id;
use std::collections::HashMap;

impl DatabaseService {
    pub async fn create_node(&self, request: CreateNodeRequest) -> AppResult<Node> {
        let node_id = generate_id();
        let user_id = "default_user"; // Placeholder
        let now = Utc::now();
        
        let properties_json = serde_json::to_string(&request.properties.unwrap_or_default())
            .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
        let tags_json = serde_json::to_string(&request.tags.unwrap_or_default())
            .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO nodes (id, content, parent_id, order_index, properties, tags, created_at, updated_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&node_id)
        .bind(&request.content)
        .bind(&request.parent_id)
        .bind(request.order.unwrap_or(0))
        .bind(&properties_json)
        .bind(&tags_json)
        .bind(&now)
        .bind(&now)
        .bind(&user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        self.get_node(&node_id).await
    }

    pub async fn get_node(&self, node_id: &str) -> AppResult<Node> {
        let row = sqlx::query(
            r#"
            SELECT id, content, parent_id, order_index, properties, tags, 
                   created_at, updated_at, created_by, version 
            FROM nodes 
            WHERE id = ?
            "#
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
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
        
        // Populate children array
        let children = sqlx::query("SELECT id FROM nodes WHERE parent_id = ? ORDER BY order_index")
            .bind(node_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        node.children = children.into_iter().map(|r| r.get::<String, _>("id")).collect();
        
        Ok(node)
    }

    pub async fn get_node_with_children(&self, node_id: &str) -> AppResult<NodeWithChildren> {
        let node = self.get_node(node_id).await?;
        let children = self.get_child_nodes(node_id).await?;
        Ok(NodeWithChildren {
            node,
            child_nodes: children,
        })
    }

    async fn get_child_nodes(&self, parent_id: &str) -> AppResult<Vec<NodeWithChildren>> {
        let children = sqlx::query("SELECT id FROM nodes WHERE parent_id = ? ORDER BY order_index")
            .bind(parent_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        let mut result = Vec::new();
        for child in children {
            result.push(Box::pin(self.get_node_with_children(&child.get::<String, _>("id"))).await?);
        }
        Ok(result)
    }

    pub async fn update_node(&self, node_id: &str, request: UpdateNodeRequest) -> AppResult<Node> {
        let now = Utc::now();
        let mut tx = self.pool.begin().await
            .map_err(|e| crate::errors::AppError::DatabaseConnectionFailed(e.to_string()))?;

        // Build dynamic update query
        let mut query_builder = String::from("UPDATE nodes SET updated_at = ?");
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync>> = vec![
            Box::new(now)
        ];
        let mut param_count = 2;

        if let Some(content) = &request.content {
            query_builder.push_str(&format!(", content = ?{}", param_count));
            params.push(Box::new(content.clone()));
            param_count += 1;
        }

        if let Some(parent_id) = &request.parent_id {
            query_builder.push_str(&format!(", parent_id = ?{}", param_count));
            params.push(Box::new(parent_id.clone()));
            param_count += 1;
        }

        if let Some(order) = request.order {
            query_builder.push_str(&format!(", order_index = ?{}", param_count));
            params.push(Box::new(order));
            param_count += 1;
        }

        if let Some(properties) = &request.properties {
            let json = serde_json::to_string(properties)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
            query_builder.push_str(&format!(", properties = ?{}", param_count));
            params.push(Box::new(json));
            param_count += 1;
        }

        if let Some(tags) = &request.tags {
            let json = serde_json::to_string(tags)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
            query_builder.push_str(&format!(", tags = ?{}", param_count));
            params.push(Box::new(json));
            param_count += 1;
        }

        query_builder.push_str(&format!(", version = version + 1 WHERE id = ?{}", param_count));
        params.push(Box::new(node_id.to_string()));

        // For simplicity with dynamic queries, we'll use a simpler approach
        // SQLx doesn't support fully dynamic queries with the macro, so we'll use the query builder
        
        // Update with individual fields for clarity
        if request.content.is_some() || request.parent_id.is_some() || 
           request.order.is_some() || request.properties.is_some() || request.tags.is_some() {
            
            // For a cleaner implementation, let's update each field explicitly
            if let Some(content) = request.content {
                sqlx::query("UPDATE nodes SET content = ?, updated_at = ? WHERE id = ?")
                    .bind(&content)
                    .bind(&now)
                    .bind(&node_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            }
            
            if let Some(parent_id) = request.parent_id {
                sqlx::query("UPDATE nodes SET parent_id = ?, updated_at = ? WHERE id = ?")
                    .bind(&parent_id)
                    .bind(&now)
                    .bind(&node_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            }
            
            if let Some(order) = request.order {
                sqlx::query("UPDATE nodes SET order_index = ?, updated_at = ? WHERE id = ?")
                    .bind(order)
                    .bind(&now)
                    .bind(&node_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            }
            
            if let Some(properties) = request.properties {
                let json = serde_json::to_string(&properties)
                    .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
                sqlx::query("UPDATE nodes SET properties = ?, updated_at = ? WHERE id = ?")
                    .bind(&json)
                    .bind(&now)
                    .bind(&node_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            }
            
            if let Some(tags) = request.tags {
                let json = serde_json::to_string(&tags)
                    .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
                sqlx::query("UPDATE nodes SET tags = ?, updated_at = ? WHERE id = ?")
                    .bind(&json)
                    .bind(&now)
                    .bind(&node_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
            }
            
            // Update version
            sqlx::query("UPDATE nodes SET version = version + 1 WHERE id = ?")
                .bind(&node_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        self.get_node(node_id).await
    }

    pub async fn delete_node(&self, node_id: &str) -> AppResult<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| crate::errors::AppError::DatabaseConnectionFailed(e.to_string()))?;

        // Recursively delete children
        let children = sqlx::query("SELECT id FROM nodes WHERE parent_id = ?")
            .bind(&node_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        for child in children {
            Box::pin(self.delete_node(&child.get::<String, _>("id"))).await?;
        }

        // Delete the node itself
        sqlx::query("DELETE FROM nodes WHERE id = ?")
            .bind(&node_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        tx.commit().await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn move_node(&self, node_id: &str, new_parent_id: Option<String>, new_order: i32) -> AppResult<Node> {
        let now = Utc::now();
        
        sqlx::query(
            "UPDATE nodes SET parent_id = ?, order_index = ?, updated_at = ?, version = version + 1 WHERE id = ?"
        )
        .bind(&new_parent_id)
        .bind(new_order)
        .bind(&now)
        .bind(&node_id)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;

        self.get_node(node_id).await
    }

    pub async fn get_all_nodes(&self) -> AppResult<Vec<Node>> {
        let rows = sqlx::query(
            r#"
            SELECT id, content, parent_id, order_index, properties, tags, 
                   created_at, updated_at, created_by, version 
            FROM nodes 
            ORDER BY created_at DESC
            "#
        )
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

    pub async fn get_root_nodes(&self) -> AppResult<Vec<Node>> {
        let rows = sqlx::query(
            r#"
            SELECT id, content, parent_id, order_index, properties, tags, 
                   created_at, updated_at, created_by, version 
            FROM nodes 
            WHERE parent_id IS NULL 
            ORDER BY order_index
            "#
        )
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

    pub async fn get_daily_note(&self, date: &str) -> AppResult<Node> {
        let title = format!("Daily Note - {}", date);
        let row = sqlx::query(
            r#"
            SELECT id, content, parent_id, order_index, properties, tags, 
                   created_at, updated_at, created_by, version 
            FROM nodes 
            WHERE content LIKE ?
            LIMIT 1
            "#
        )
        .bind(format!("{}%", title))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
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
        
        Ok(node)
    }

    pub async fn get_or_create_daily_note(&self, date: &str) -> AppResult<Node> {
        let title = format!("Daily Note - {}", date);
        
        // Try to find existing daily note
        let existing = sqlx::query("SELECT id FROM nodes WHERE content LIKE ? LIMIT 1")
            .bind(format!("{}%", title))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::DatabaseQueryFailed(e.to_string()))?;
        
        if let Some(row) = existing {
            self.get_node(&row.get::<String, _>("id")).await
        } else {
            // Create new daily note
            let request = CreateNodeRequest {
                content: title,
                parent_id: None,
                order: Some(0),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("type".to_string(), serde_json::Value::String("daily_note".to_string()));
                    props.insert("date".to_string(), serde_json::Value::String(date.to_string()));
                    props
                }),
                tags: Some(vec!["daily".to_string(), "journal".to_string()]),
            };
            self.create_node(request).await
        }
    }
}