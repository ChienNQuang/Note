use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>, // Ordered list of child Node IDs
    pub order: i32,
    pub properties: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWithChildren {
    #[serde(flatten)]
    pub node: Node,
    pub child_nodes: Vec<NodeWithChildren>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub content: String,
    pub parent_id: Option<String>,
    pub order: Option<i32>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub content: Option<String>,
    pub parent_id: Option<String>,
    pub order: Option<i32>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
} 