// Services module - implementing in phases

// Phase 1: Database service
pub mod database;
pub mod link_service;

// Phase 2: Git manager (to be implemented)  
// pub mod git_manager;

// Phase 3: CRDT engine and WebSocket (to be implemented)
// pub mod crdt_engine;
// pub mod websocket_server;

// Re-exports for easier access
pub use database::connection::DatabaseService;
pub use link_service::LinkService; 