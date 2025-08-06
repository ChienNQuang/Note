use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AppError {
    // Database errors
    #[error("Database connection failed: {0}")]
    DatabaseConnectionFailed(String),
    
    #[error("Database query failed: {0}")]
    DatabaseQueryFailed(String),
    
    #[error("Database constraint violation: {0}")]
    DatabaseConstraintViolation(String),
    
    // Validation errors
    #[error("Invalid block data: {0}")]
    InvalidBlockData(String),
    
    #[error("Invalid page data: {0}")]
    InvalidPageData(String),
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    
    // File system errors
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("File permission denied: {0}")]
    FilePermissionDenied(String),
    
    #[error("Export failed: {0}")]
    ExportFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    // Git errors (for future phases)
    #[error("Git initialization failed: {0}")]
    GitInitFailed(String),
    
    #[error("Git commit failed: {0}")]
    GitCommitFailed(String),
    
    #[error("Git push failed: {0}")]
    GitPushFailed(String),
    
    // Collaboration errors (for future phases)
    #[error("WebSocket connection failed: {0}")]
    WebSocketConnectionFailed(String),
    
    #[error("Sync conflict: {0}")]
    SyncConflict(String),
    
    #[error("User unauthorized: {0}")]
    UserUnauthorized(String),
    
    // Generic errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// Implement From traits for common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    AppError::DatabaseQueryFailed(format!("Database error {}: {}", code, db_err.message()))
                } else {
                    AppError::DatabaseQueryFailed(db_err.message().to_string())
                }
            }
            sqlx::Error::RowNotFound => AppError::DatabaseQueryFailed("Row not found".to_string()),
            sqlx::Error::ColumnNotFound(_) => AppError::DatabaseQueryFailed("Column not found".to_string()),
            sqlx::Error::ColumnDecode { .. } => AppError::DatabaseQueryFailed("Column decode error".to_string()),
            sqlx::Error::Protocol(_) => AppError::DatabaseConnectionFailed("Database protocol error".to_string()),
            sqlx::Error::PoolClosed => AppError::DatabaseConnectionFailed("Database pool closed".to_string()),
            sqlx::Error::PoolTimedOut => AppError::DatabaseConnectionFailed("Database pool timeout".to_string()),
            sqlx::Error::WorkerCrashed => AppError::DatabaseConnectionFailed("Database worker crashed".to_string()),
            _ => AppError::DatabaseQueryFailed(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AppError::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::FilePermissionDenied(err.to_string()),
            _ => AppError::Internal(err.to_string()),
        }
    }
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>; 