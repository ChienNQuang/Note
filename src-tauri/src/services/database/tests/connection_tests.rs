#[cfg(test)]
mod tests {
    use crate::services::DatabaseService;
    use crate::errors::AppResult;
    use tempfile::tempdir;

    /// Create a test database service with temporary database
    fn create_test_db() -> AppResult<DatabaseService> {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        
        let service = DatabaseService::new()?;
        
        service.initialize_schema()?;
        service.ensure_default_user()?;
        
        Ok(service)
    }

    #[test]
    fn test_database_initialization() {
        let db = create_test_db().expect("Failed to create test database");
        
        // Test that default user was created
        let user_id = db.get_default_user_id().expect("Failed to get default user");
        assert!(!user_id.is_empty());
    }
} 