use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PageStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "published")]
    Published,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "deleted")]
    Deleted,
}

impl Default for PageStatus {
    fn default() -> Self {
        PageStatus::Draft
    }
}

impl std::fmt::Display for PageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageStatus::Draft => write!(f, "draft"),
            PageStatus::Published => write!(f, "published"),
            PageStatus::Archived => write!(f, "archived"),
            PageStatus::Deleted => write!(f, "deleted"),
        }
    }
}

impl std::str::FromStr for PageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(PageStatus::Draft),
            "published" => Ok(PageStatus::Published),
            "archived" => Ok(PageStatus::Archived),
            "deleted" => Ok(PageStatus::Deleted),
            _ => Err(format!("Invalid page status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Page identifier (UUID)
    pub id: String,
    
    /// Page title
    pub title: String,
    
    /// Root blocks of the page (block IDs)
    pub root_blocks: Vec<String>,
    
    /// Page metadata and properties
    pub properties: HashMap<String, serde_json::Value>,
    
    /// File system mapping (for Phase 2)
    pub file_path: Option<String>,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    /// Collaboration info
    pub created_by: String,
    pub last_edited_by: String,
    
    /// Tags and categories
    pub tags: Vec<String>,
    
    /// Page status
    pub status: PageStatus,
}

impl Page {
    /// Create a new page with default values
    pub fn new(title: String, created_by: String) -> Self {
        let now = Utc::now();
        let id = crate::utils::generate_page_id();

        Page {
            id,
            title,
            root_blocks: Vec::new(),
            properties: HashMap::new(),
            file_path: None,
            created_at: now,
            updated_at: now,
            created_by: created_by.clone(),
            last_edited_by: created_by,
            tags: Vec::new(),
            status: PageStatus::Draft,
        }
    }

    /// Update the page's last modified time and editor
    pub fn touch(&mut self, user_id: String) {
        self.updated_at = Utc::now();
        self.last_edited_by = user_id;
    }

    /// Add a root block to the page
    pub fn add_root_block(&mut self, block_id: String) {
        if !self.root_blocks.contains(&block_id) {
            self.root_blocks.push(block_id);
        }
    }

    /// Remove a root block from the page
    pub fn remove_root_block(&mut self, block_id: &str) {
        self.root_blocks.retain(|id| id != block_id);
    }

    /// Add a tag to the page
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from the page
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
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
}

/// Request to create a new page
#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub title: String,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
    pub status: Option<PageStatus>,
}

/// Request to update an existing page
#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub title: Option<String>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
    pub status: Option<PageStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_creation() {
        let page = Page::new("Test Page".to_string(), "user1".to_string());
        
        assert_eq!(page.title, "Test Page");
        assert_eq!(page.created_by, "user1");
        assert_eq!(page.last_edited_by, "user1");
        assert_eq!(page.status, PageStatus::Draft);
        assert!(page.root_blocks.is_empty());
        assert!(page.tags.is_empty());
    }

    #[test]
    fn test_page_root_blocks() {
        let mut page = Page::new("Test".to_string(), "user1".to_string());
        
        page.add_root_block("block-1".to_string());
        page.add_root_block("block-2".to_string());
        page.add_root_block("block-1".to_string()); // Duplicate should be ignored
        
        assert_eq!(page.root_blocks.len(), 2);
        assert!(page.root_blocks.contains(&"block-1".to_string()));
        assert!(page.root_blocks.contains(&"block-2".to_string()));
        
        page.remove_root_block("block-1");
        assert_eq!(page.root_blocks.len(), 1);
        assert!(!page.root_blocks.contains(&"block-1".to_string()));
    }

    #[test]
    fn test_page_tags() {
        let mut page = Page::new("Test".to_string(), "user1".to_string());
        
        page.add_tag("important".to_string());
        page.add_tag("work".to_string());
        page.add_tag("important".to_string()); // Duplicate should be ignored
        
        assert_eq!(page.tags.len(), 2);
        assert!(page.tags.contains(&"important".to_string()));
        assert!(page.tags.contains(&"work".to_string()));
        
        page.remove_tag("work");
        assert_eq!(page.tags.len(), 1);
        assert!(!page.tags.contains(&"work".to_string()));
    }

    #[test]
    fn test_page_properties() {
        let mut page = Page::new("Test".to_string(), "user1".to_string());
        
        page.set_property("color".to_string(), serde_json::Value::String("blue".to_string()));
        page.set_property("priority".to_string(), serde_json::Value::Number(serde_json::Number::from(5)));
        
        assert_eq!(page.get_property("color"), Some(&serde_json::Value::String("blue".to_string())));
        assert_eq!(page.get_property("priority"), Some(&serde_json::Value::Number(serde_json::Number::from(5))));
        
        page.remove_property("color");
        assert_eq!(page.get_property("color"), None);
    }

    #[test]
    fn test_page_status_conversion() {
        use std::str::FromStr;
        
        assert_eq!(PageStatus::from_str("draft").unwrap(), PageStatus::Draft);
        assert_eq!(PageStatus::from_str("published").unwrap(), PageStatus::Published);
        assert_eq!(PageStatus::from_str("archived").unwrap(), PageStatus::Archived);
        assert_eq!(PageStatus::from_str("deleted").unwrap(), PageStatus::Deleted);
        assert!(PageStatus::from_str("invalid").is_err());
        
        assert_eq!(PageStatus::Draft.to_string(), "draft");
        assert_eq!(PageStatus::Published.to_string(), "published");
    }
} 