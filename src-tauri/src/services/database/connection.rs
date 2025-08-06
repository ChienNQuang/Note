use sqlx::{SqlitePool, SqliteConnection, Sqlite, Transaction, Row};
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;
use tracing::info;
use dirs;

use crate::errors::{AppError, AppResult};

pub struct DatabaseService {
    pub(crate) db_path: PathBuf,
    pub(crate) pool: SqlitePool,
}

impl DatabaseService {
    pub async fn new() -> AppResult<Self> {
        let app_dir = dirs::data_dir()
            .ok_or_else(|| AppError::Internal("Could not find data directory".to_string()))?
            .join("note-app");
        
        std::fs::create_dir_all(&app_dir)?;
        
        let db_path = app_dir.join("note.db");
        info!("Initializing database at: {:?}", db_path);
        
        let database_url = format!("sqlite:{}", db_path.display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&database_url)
            .await
            .map_err(|e| AppError::DatabaseConnectionFailed(e.to_string()))?;
        
        let service = DatabaseService {
            db_path,
            pool,
        };
        
        service.initialize_schema().await?;
        service.ensure_default_user().await?;
        
        info!("Database service initialized successfully");
        Ok(service)
    }
    
    #[cfg(test)]
    pub async fn new_test(db_path: &str) -> AppResult<Self> {
        let database_url = format!("sqlite:{}", db_path);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .map_err(|e| AppError::DatabaseConnectionFailed(e.to_string()))?;
        
        let service = DatabaseService {
            db_path: PathBuf::from(db_path),
            pool,
        };
        
        service.initialize_schema().await?;
        service.ensure_default_user().await?;
        Ok(service)
    }
    
    #[cfg(test)]
    pub async fn init_database(&self) -> AppResult<()> {
        self.initialize_schema().await?;
        self.ensure_default_user().await?;
        Ok(())
    }
    
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Execute a database operation with proper connection management
    pub async fn with_connection<F, T, Fut>(&self, operation: F) -> AppResult<T>
    where
        F: FnOnce(&mut SqliteConnection) -> Fut,
        Fut: std::future::Future<Output = sqlx::Result<T>>,
    {
        let mut conn = self.pool.acquire().await
            .map_err(|e| AppError::DatabaseConnectionFailed(e.to_string()))?;
        
        let result = operation(&mut *conn).await;
        
        result.map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))
    }
    
    /// Execute a database operation with transaction
    pub async fn with_transaction<F, T, Fut>(&self, operation: F) -> AppResult<T>
    where
        F: FnOnce(Transaction<'_, Sqlite>) -> Fut,
        Fut: std::future::Future<Output = sqlx::Result<T>>,
    {
        let tx = self.pool.begin().await
            .map_err(|e| AppError::DatabaseConnectionFailed(e.to_string()))?;
        
        let result = operation(tx).await;
        
        result.map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))
    }

    /// Get the default user ID for Phase 1
    pub async fn get_default_user_id(&self) -> AppResult<String> {
        let row = sqlx::query("SELECT id FROM users LIMIT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))?;
        
        Ok(row.get("id"))
    }
}

impl Clone for DatabaseService {
    fn clone(&self) -> Self {
        DatabaseService {
            db_path: self.db_path.clone(),
            pool: self.pool.clone(),
        }
    }
}