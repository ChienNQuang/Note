# Data Models & API Interfaces

This document defines the core data structures, database schema, and API interfaces for the Note application, based on a **unified node model**.

## Core Data Structures

### The Unified Node Model
In this architecture, there is no distinction between "pages" and "blocks". Everything is a **Node**. A node can be viewed as a page by "zooming in" on it, where its content becomes the title and its children become the content of the new view.

### TypeScript Interfaces

#### Node Interface
```typescript
interface Node {
  // Unique identifier (UUID v4)
  id: string;
  
  // Node content (markdown text)
  content: string;
  
  // Hierarchical relationships
  parent_id?: string;
  children: string[]; // Ordered list of child Node IDs
  
  // Metadata & Properties
  properties: Record<string, any>;
  
  // Timestamps
  created_at: Date;
  updated_at: Date;
  
  // Collaboration & Ownership
  created_by: string;
  last_edited_by: string;
  
  // Version control
  version: number;
  
  // For special nodes like Journals or Templates
  tags?: string[];
}
```

#### User Interface
```typescript
interface User {
  id: string;
  name: string;
  email?: string;
  preferences: UserPreferences;
  // ... other fields
}

interface UserPreferences {
  theme: 'light' | 'dark' | 'system'; // Default will be 'dark'
  // ... other preferences
}
```

## Database Schema (SQLite)

The database will be simplified to a single primary table for `nodes` and a linking table for references.

### `nodes` Table
This table stores all nodes, regardless of their level in the hierarchy.
```sql
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    parent_id TEXT,
    order_index INTEGER NOT NULL, -- Defines order among siblings
    properties TEXT, -- JSON blob for metadata
    tags TEXT, -- JSON array for special tags like '#Journal'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    
    FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_nodes_parent_id ON nodes(parent_id);
CREATE INDEX idx_nodes_order ON nodes(parent_id, order_index);
CREATE INDEX idx_nodes_updated_at ON nodes(updated_at DESC);
```

### `node_links` Table
This table powers bi-directional linking by storing all `[[wikilink]]` style references.
```sql
CREATE TABLE node_links (
    source_node_id TEXT NOT NULL,
    target_node_id TEXT NOT NULL,
    
    PRIMARY KEY (source_node_id, target_node_id),
    FOREIGN KEY (source_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Indexes for fast back-reference lookups
CREATE INDEX idx_links_target_id ON node_links(target_node_id);
```

### Full-Text Search
```sql
CREATE VIRTUAL TABLE nodes_fts USING fts5(
    node_id UNINDEXED,
    content,
    content='nodes',
    content_rowid='rowid'
);

-- Triggers to keep FTS table in sync with the nodes table
-- (INSERT, UPDATE, DELETE triggers)
```

## API Interfaces

### Tauri Commands (Refactored)

The API will be simplified to operate on nodes.

#### Node Commands
```rust
#[tauri::command]
async fn create_node(data: CreateNodeRequest) -> Result<Node, AppError>;

#[tauri::command] 
async fn get_node(node_id: String) -> Result<Node, AppError>;

#[tauri::command]
async fn get_node_with_children(node_id: String) -> Result<NodeWithChildren, AppError>;

#[tauri::command]
async fn update_node(node_id: String, data: UpdateNodeRequest) -> Result<Node, AppError>;

#[tauri::command]
async fn delete_node(node_id: String) -> Result<(), AppError>;

#[tauri::command]
async fn move_node(node_id: String, new_parent_id: Option<String>, new_order: i32) -> Result<(), AppError>;
```

#### Journal Commands
```rust
#[tauri::command]
async fn get_daily_note(date: String) -> Result<Node, AppError>; // YYYY-MM-DD format

#[tauri::command]
async fn get_or_create_daily_note(date: String) -> Result<Node, AppError>;
```

#### Linking Commands
```rust
#[tauri::command]
async fn get_linked_references(node_id: String) -> Result<Vec<Node>, AppError>;

#[tauri::command]
async fn get_unlinked_references(node_id: String) -> Result<Vec<Node>, AppError>;
```

## Frontend UI Component Library

The Note application uses **Shadcn-Vue** as the primary UI component library.

### Shadcn-Vue Integration
- **Library**: [shadcn-vue](https://www.shadcn-vue.com/)
- **Installation**: Components are copied directly into the project.
- **Styling**: Built with TailwindCSS.
- **Components Path**: `@/components/ui/`

This approach provides maximum flexibility and control over UI components. 