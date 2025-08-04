// Tauri commands module - implementing in phases

// Phase 1: Page and Block commands
pub mod pages;

// Phase 2: Git commands (to be implemented)
// pub mod git;

// Phase 3: Collaboration commands (to be implemented) 
// pub mod collaboration;

// Phase 4: Search commands (to be implemented)
// pub mod search;

// Re-export all commands for easy registration
pub use pages::*; 