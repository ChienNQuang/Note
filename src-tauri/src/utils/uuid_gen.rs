use uuid::Uuid;

/// Generate a new UUID v4 as a string
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a block-specific ID with prefix
pub fn generate_block_id() -> String {
    format!("block-{}", generate_id())
}

/// Generate a page-specific ID with prefix
pub fn generate_page_id() -> String {
    format!("page-{}", generate_id())
}

/// Validate that a string is a valid UUID
pub fn is_valid_uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok()
}

/// Extract UUID from prefixed ID (e.g., "block-123" -> "123")  
pub fn extract_uuid(prefixed_id: &str) -> Option<String> {
    if let Some((_, uuid_part)) = prefixed_id.split_once('-') {
        if is_valid_uuid(uuid_part) {
            return Some(uuid_part.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert!(is_valid_uuid(&id));
    }

    #[test]
    fn test_generate_block_id() {
        let id = generate_block_id();
        assert!(id.starts_with("block-"));
        let uuid_part = id.strip_prefix("block-").unwrap();
        assert!(is_valid_uuid(uuid_part));
    }

    #[test]
    fn test_generate_page_id() {
        let id = generate_page_id();
        assert!(id.starts_with("page-"));
        let uuid_part = id.strip_prefix("page-").unwrap();
        assert!(is_valid_uuid(uuid_part));
    }

    #[test]
    fn test_extract_uuid() {
        let block_id = generate_block_id();
        let extracted = extract_uuid(&block_id);
        assert!(extracted.is_some());
        assert!(is_valid_uuid(&extracted.unwrap()));
    }
} 