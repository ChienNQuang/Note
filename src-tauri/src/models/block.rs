use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "heading")]
    Heading,
    #[serde(rename = "list_item")]
    ListItem,
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "quote")]
    Quote,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "reference")]
    Reference,
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::Text
    }
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::Text => write!(f, "text"),
            BlockType::Heading => write!(f, "heading"),
            BlockType::ListItem => write!(f, "list_item"),
            BlockType::Code => write!(f, "code"),
            BlockType::Quote => write!(f, "quote"),
            BlockType::Image => write!(f, "image"),
            BlockType::Link => write!(f, "link"),
            BlockType::Reference => write!(f, "reference"),
        }
    }
}

impl std::str::FromStr for BlockType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(BlockType::Text),
            "heading" => Ok(BlockType::Heading),
            "list_item" => Ok(BlockType::ListItem),
            "code" => Ok(BlockType::Code),
            "quote" => Ok(BlockType::Quote),
            "image" => Ok(BlockType::Image),
            "link" => Ok(BlockType::Link),  
            "reference" => Ok(BlockType::Reference),
            _ => Err(format!("Invalid block type: {}", s)),
        }
    }
}

/// Vector Clock for CRDT operations (prepared for Phase 3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorClock {
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        VectorClock {
            clocks: HashMap::new(),
        }
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Unique identifier (UUID)
    pub id: String,
    
    /// Block content (markdown text)
    pub text: String,
    
    /// Page relationship (REQUIRED - fixed from review analysis)
    pub page_id: String,
    
    /// Hierarchical relationships
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    
    /// Block ordering within parent
    pub order: i32,
    
    /// Metadata and properties
    pub properties: HashMap<String, serde_json::Value>,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    /// Collaboration info
    pub created_by: String,
    pub last_edited_by: String,
    
    /// Version control
    pub version: i32,
    
    /// Block type for rendering and behavior
    pub block_type: BlockType,
    
    /// CRDT fields (prepared for Phase 3)
    pub vector_clock: Option<VectorClock>,
    pub operation_id: Option<String>,
}

impl Block {
    /// Create a new block with required page_id
    pub fn new(text: String, page_id: String, created_by: String) -> Self {
        let now = Utc::now();
        let id = crate::utils::generate_block_id();

        Block {
            id,
            text,
            page_id, // Required field
            parent_id: None,
            children: Vec::new(),
            order: 0,
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by: created_by.clone(),
            last_edited_by: created_by,
            version: 1,
            block_type: BlockType::Text,
            // CRDT fields - None for Phase 1, will be used in Phase 3
            vector_clock: None,
            operation_id: None,
        }
    }

    /// Create a child block
    pub fn new_child(text: String, page_id: String, parent_id: String, order: i32, created_by: String) -> Self {
        let mut block = Self::new(text, page_id, created_by);
        block.parent_id = Some(parent_id);
        block.order = order;
        block
    }

    /// Update the block's last modified time and editor
    pub fn touch(&mut self, user_id: String) {
        self.updated_at = Utc::now();
        self.last_edited_by = user_id;
        self.version += 1;
    }

    /// Update block text content
    pub fn update_text(&mut self, text: String, user_id: String) {
        self.text = text;
        self.touch(user_id);
    }

    /// Add a child block
    pub fn add_child(&mut self, child_id: String) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child block
    pub fn remove_child(&mut self, child_id: &str) {
        self.children.retain(|id| id != child_id);
    }

    /// Set parent block
    pub fn set_parent(&mut self, parent_id: Option<String>) {
        self.parent_id = parent_id;
    }

    /// Set block order
    pub fn set_order(&mut self, order: i32) {
        self.order = order;
    }

    /// Set a property value
    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Remove a property
    pub fn remove_property(&mut self, key: &str) {
        self.properties.remove(key);
    }

    /// Check if this is a root block (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Check if this block has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Convert block type
    pub fn convert_type(&mut self, new_type: BlockType, user_id: String) {
        self.block_type = new_type;
        self.touch(user_id);
    }
}

/// Request to create a new block
#[derive(Debug, Deserialize)]
pub struct CreateBlockRequest {
    pub text: String,
    pub page_id: String, // Required field
    pub parent_id: Option<String>,
    pub block_type: Option<BlockType>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub order: Option<i32>,
}

/// Request to update an existing block
#[derive(Debug, Deserialize)]
pub struct UpdateBlockRequest {
    pub text: Option<String>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub block_type: Option<BlockType>,
}

/// Request to move a block
#[derive(Debug, Deserialize)]
pub struct MoveBlockRequest {
    pub new_parent_id: Option<String>,
    pub new_order: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new(
            "Test content".to_string(),
            "page-123".to_string(),
            "user1".to_string()
        );
        
        assert_eq!(block.text, "Test content");
        assert_eq!(block.page_id, "page-123");
        assert_eq!(block.created_by, "user1");
        assert_eq!(block.last_edited_by, "user1");
        assert_eq!(block.version, 1);
        assert_eq!(block.block_type, BlockType::Text);
        assert!(block.is_root());
        assert!(!block.has_children());
        assert!(block.vector_clock.is_none()); // Phase 1 - not used yet
        assert!(block.operation_id.is_none()); // Phase 1 - not used yet
    }

    #[test]
    fn test_block_child_creation() {
        let child = Block::new_child(
            "Child content".to_string(),
            "page-123".to_string(),
            "block-parent".to_string(),
            1,
            "user1".to_string()
        );
        
        assert_eq!(child.parent_id, Some("block-parent".to_string()));
        assert_eq!(child.order, 1);
        assert!(!child.is_root());
    }

    #[test]
    fn test_block_text_update() {
        let mut block = Block::new(
            "Original".to_string(),
            "page-123".to_string(),
            "user1".to_string()
        );
        
        let original_version = block.version;
        let original_updated = block.updated_at;
        
        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        block.update_text("Updated".to_string(), "user2".to_string());
        
        assert_eq!(block.text, "Updated");
        assert_eq!(block.last_edited_by, "user2");
        assert_eq!(block.version, original_version + 1);
        assert!(block.updated_at > original_updated);
    }

    #[test]
    fn test_block_children() {
        let mut parent = Block::new(
            "Parent".to_string(),
            "page-123".to_string(),
            "user1".to_string()
        );
        
        parent.add_child("child-1".to_string());
        parent.add_child("child-2".to_string());
        parent.add_child("child-1".to_string()); // Duplicate should be ignored
        
        assert_eq!(parent.children.len(), 2);
        assert!(parent.has_children());
        assert!(parent.children.contains(&"child-1".to_string()));
        assert!(parent.children.contains(&"child-2".to_string()));
        
        parent.remove_child("child-1");
        assert_eq!(parent.children.len(), 1);
        assert!(!parent.children.contains(&"child-1".to_string()));
    }

    #[test]
    fn test_block_properties() {
        let mut block = Block::new(
            "Test".to_string(),
            "page-123".to_string(),
            "user1".to_string()
        );
        
        block.set_property("color".to_string(), serde_json::Value::String("blue".to_string()));
        block.set_property("level".to_string(), serde_json::Value::Number(serde_json::Number::from(2)));
        
        assert_eq!(block.get_property("color"), Some(&serde_json::Value::String("blue".to_string())));
        assert_eq!(block.get_property("level"), Some(&serde_json::Value::Number(serde_json::Number::from(2))));
        
        block.remove_property("color");
        assert_eq!(block.get_property("color"), None);
    }

    #[test]
    fn test_block_type_conversion() {
        use std::str::FromStr;
        
        assert_eq!(BlockType::from_str("text").unwrap(), BlockType::Text);
        assert_eq!(BlockType::from_str("heading").unwrap(), BlockType::Heading);
        assert_eq!(BlockType::from_str("code").unwrap(), BlockType::Code);
        assert!(BlockType::from_str("invalid").is_err());
        
        assert_eq!(BlockType::Text.to_string(), "text");
        assert_eq!(BlockType::Heading.to_string(), "heading");
        
        let mut block = Block::new(
            "Test".to_string(),
            "page-123".to_string(),
            "user1".to_string()
        );
        
        block.convert_type(BlockType::Heading, "user1".to_string());
        assert_eq!(block.block_type, BlockType::Heading);
    }
} 