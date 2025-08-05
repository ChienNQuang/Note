use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;
use tracing::{info, debug};
use dirs;

use crate::errors::{AppError, AppResult};

pub struct DatabaseService {
    pub(crate) db_path: PathBuf,
    connection_pool: Arc<Mutex<Vec<Connection>>>,
    max_connections: usize,
    pool_timeout: Duration,
}

impl DatabaseService {
    pub fn new() -> AppResult<Self> {
        let app_dir = dirs::data_dir()
            .ok_or_else(|| AppError::Internal("Could not find data directory".to_string()))?
            .join("note-app");
        
        std::fs::create_dir_all(&app_dir)?;
        
        let db_path = app_dir.join("note.db");
        info!("Initializing database at: {:?}", db_path);
        
        let service = DatabaseService {
            db_path,
            connection_pool: Arc::new(Mutex::new(Vec::new())),
            max_connections: 5,
            pool_timeout: Duration::from_secs(10),
        };
        
        service.initialize_schema()?;
        service.ensure_default_user()?;
        
        info!("Database service initialized successfully");
        Ok(service)
    }
    
    #[cfg(test)]
    pub fn new_test(db_path: &str) -> Self {
        let service = DatabaseService {
            db_path: PathBuf::from(db_path),
            connection_pool: Arc::new(Mutex::new(Vec::new())),
            max_connections: 5,
            pool_timeout: Duration::from_secs(10),
        };
        service
    }
    
    #[cfg(test)]
    pub fn init_database(&self) -> AppResult<()> {
        self.initialize_schema()?;
        self.ensure_default_user()?;
        Ok(())
    }
    
    pub(crate) fn get_connection(&self) -> AppResult<Connection> {
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed() > self.pool_timeout {
                return Err(AppError::DatabaseConnectionFailed(
                    "Connection pool timeout".to_string()
                ));
            }
            
            if let Some(conn) = self.connection_pool.lock().unwrap().pop() {
                debug!("Reusing existing connection from pool");
                return Ok(conn);
            }
            
            if self.connection_pool.lock().unwrap().len() < self.max_connections {
                let conn = self.create_connection()?;
                debug!("Created new database connection");
                return Ok(conn);
            }
            
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    
    pub(crate) fn return_connection(&self, conn: Connection) {
        if let Ok(mut pool) = self.connection_pool.lock() {
            pool.push(conn);
            debug!("Returned connection to pool");
        }
    }
    
    fn create_connection(&self) -> AppResult<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(conn)
    }

    /// Execute a database operation with proper connection management
    pub fn with_connection<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T>,
    {
        let conn = self.get_connection()?;
        let result = operation(&conn);
        self.return_connection(conn);
        
        result.map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))
    }
    
    /// Execute a database operation with transaction
    pub fn with_transaction<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: FnOnce(&mut rusqlite::Transaction) -> rusqlite::Result<T>,
    {
        let mut conn = self.get_connection()?;
        
        let mut tx = conn.transaction()?;
        let result = operation(&mut tx);
        
        if result.is_ok() {
            tx.commit()?;
        } else {
            let _ = tx.rollback();
        }
        
        self.return_connection(conn);
        
        result.map_err(|e| AppError::DatabaseQueryFailed(e.to_string()))
    }

    /// Get the default user ID for Phase 1
    pub fn get_default_user_id(&self) -> AppResult<String> {
        self.with_connection(|conn| {
            let user_id: String = conn.query_row(
                "SELECT id FROM users LIMIT 1",
                [],
                |row| row.get(0)
            )?;
            Ok(user_id)
        })
    }
}

impl Clone for DatabaseService {
    fn clone(&self) -> Self {
        DatabaseService {
            db_path: self.db_path.clone(),
            connection_pool: self.connection_pool.clone(),
            max_connections: self.max_connections,
            pool_timeout: self.pool_timeout,
        }
    }
} 