# Data Models & API Interfaces

This document defines the core data structures, database schema, and API interfaces for the Note application.

## Core Data Structures

### TypeScript Interfaces

#### Block Interface
```typescript
interface Block {
  // Unique identifier (UUID v4)
  id: string;
  
  // Block content (markdown text)
  text: string;
  
  // Page relationship (REQUIRED - fixed from review analysis)
  page_id: string;
  
  // Hierarchical relationships
  parent_id?: string;
  children: string[];
  
  // Block ordering within parent
  order: number;
  
  // Metadata
  properties: Record<string, any>;
  
  // Timestamps
  created_at: Date;
  updated_at: Date;
  
  // Collaboration
  created_by: string;
  last_edited_by: string;
  
  // Version control
  version: number;
  
  // Block type (for future extensibility)
  block_type: BlockType;
  
  // CRDT fields (prepared for Phase 3)
  vector_clock?: VectorClock;
  operation_id?: string;
}

enum BlockType {
  Text = 'text',
  Heading = 'heading',
  ListItem = 'list_item', 
  Code = 'code',
  Quote = 'quote',
  Image = 'image',
  Link = 'link',
  Reference = 'reference'
}
```

#### Page Interface
```typescript
interface Page {
  // Page identifier (UUID)
  id: string;
  
  // Page title
  title: string;
  
  // Root blocks of the page (block IDs)
  root_blocks: string[];
  
  // Page metadata and properties
  properties: Record<string, any>;
  
  // File system mapping (for Phase 2)
  file_path?: string;
  
  // Timestamps
  created_at: Date;
  updated_at: Date;
  
  // Collaboration info
  created_by: string;
  last_edited_by: string;
  
  // Tags and categories
  tags: string[];
  
  // Page status
  status: PageStatus;
}

enum PageStatus {
  Draft = 'draft',
  Published = 'published', 
  Archived = 'archived',
  Deleted = 'deleted'
}
```

#### User Interface
```typescript
interface User {
  // User identifier
  id: string;
  
  // Display information
  name: string;
  email?: string;
  avatar_url?: string;
  
  // User preferences
  preferences: UserPreferences;
  
  // Collaboration status (for future phases)
  is_online: boolean;
  last_seen: Date;
  
  // Authentication (for future phases)
  auth_token?: string;
  
  // Timestamps
  created_at: Date;
  updated_at: Date;
}

interface UserPreferences {
  theme: string; // "light", "dark", "system"
  font_size: number;
  auto_save_interval: number; // seconds
  git_auto_commit: boolean;
  collaboration_enabled: boolean;
  shortcuts: Record<string, string>;
}
```

#### Event Interface (for CRDT - Phase 3)
```typescript
interface BlockEvent {
  type: 'block_created' | 'block_updated' | 'block_deleted' | 'block_moved';
  block_id: string;
  data: any;
  user_id: string;
  timestamp: number;
  sequence: number;
}

interface VectorClock {
  clocks: Record<string, number>;
}
```

## Database Schema (SQLite)

### Phase 1 Schema (Simplified)
```sql
-- Core tables for Phase 1
CREATE TABLE pages (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    properties TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    status TEXT DEFAULT 'draft'
);

CREATE TABLE blocks (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL,  -- Required field
    text TEXT NOT NULL,
    parent_id TEXT,
    order_index INTEGER NOT NULL,
    properties TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    block_type TEXT DEFAULT 'text',
    
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES blocks(id) ON DELETE CASCADE
);

CREATE TABLE page_blocks (
    page_id TEXT NOT NULL,
    block_id TEXT NOT NULL,
    is_root BOOLEAN DEFAULT FALSE,
    
    PRIMARY KEY (page_id, block_id),
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE,
    FOREIGN KEY (block_id) REFERENCES blocks(id) ON DELETE CASCADE
);

-- Users table (for Phase 1 - single local user)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    preferences TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_blocks_page_id ON blocks(page_id);
CREATE INDEX idx_blocks_parent_id ON blocks(parent_id);
CREATE INDEX idx_blocks_order ON blocks(page_id, order_index);
CREATE INDEX idx_page_blocks_page ON page_blocks(page_id);
```

### Phase 3+ Schema Additions
```sql
-- Events table for CRDT (Phase 3)
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    block_id TEXT NOT NULL,
    data TEXT, -- JSON
    user_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    sequence INTEGER NOT NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Full-text search (Phase 4) 
CREATE VIRTUAL TABLE search_index USING fts5(
    block_id,
    content,
    page_title
);
```

## API Interfaces

### Tauri Commands

#### Page Commands
```rust
#[tauri::command]
async fn create_page(title: String) -> Result<Page, AppError>;

#[tauri::command] 
async fn get_page(page_id: String) -> Result<Page, AppError>;

#[tauri::command]
async fn update_page(page_id: String, data: UpdatePageRequest) -> Result<Page, AppError>;

#[tauri::command]
async fn delete_page(page_id: String) -> Result<(), AppError>;

#[tauri::command]
async fn list_pages() -> Result<Vec<Page>, AppError>;
```

#### Block Commands
```rust
#[tauri::command]
async fn create_block(data: CreateBlockRequest) -> Result<Block, AppError>;

#[tauri::command]
async fn get_block(block_id: String) -> Result<Block, AppError>;

#[tauri::command]
async fn update_block(block_id: String, data: UpdateBlockRequest) -> Result<Block, AppError>;

#[tauri::command]
async fn delete_block(block_id: String) -> Result<(), AppError>;

#[tauri::command]
async fn move_block(block_id: String, data: MoveBlockRequest) -> Result<Block, AppError>;

#[tauri::command]
async fn get_page_blocks(page_id: String) -> Result<Vec<Block>, AppError>;
```

### WebSocket Protocol (Phase 3)

#### Message Types
```typescript
interface WebSocketMessage {
  type: 'connection' | 'auth' | 'event' | 'sync' | 'presence' | 'error';
  data: any;
  timestamp: number;
  user_id?: string;
}

// Connection messages
interface ConnectionMessage {
  type: 'connection';
  data: {
    action: 'connect' | 'disconnect';
    user_id: string;
  };
}

// Event messages (CRDT operations)
interface EventMessage {
  type: 'event';
  data: BlockEvent;
}

// Sync messages
interface SyncMessage {
  type: 'sync';
  data: {
    events: BlockEvent[];
    vector_clock: VectorClock;
  };
}
```

## Configuration Schema

### Application Configuration
```typescript
interface AppConfig {
  database: {
    path: string;
    backup_interval: number; // minutes
    max_backup_files: number;
  };
  git: {
    auto_commit: boolean;
    commit_interval: number; // seconds
    repository_path: string;
  };
  collaboration: {
    enabled: boolean;
    server_url?: string;
    port?: number;
  };
  performance: {
    max_blocks_per_page: number;
    search_index_update_interval: number;
  };
  security: {
    encrypt_local_data: boolean;
    session_timeout: number; // minutes
  };
}
```

## Error Types

### Error Handling
```typescript
interface AppError {
  code: ErrorCode;
  message: string;
  details?: any;
}

enum ErrorCode {
  // Database errors
  DatabaseConnectionFailed = 'database_connection_failed',
  DatabaseQueryFailed = 'database_query_failed',
  DatabaseConstraintViolation = 'database_constraint_violation',
  
  // Validation errors  
  InvalidBlockData = 'invalid_block_data',
  InvalidPageData = 'invalid_page_data',
  MissingRequiredField = 'missing_required_field',
  
  // File system errors
  FileNotFound = 'file_not_found',
  FilePermissionDenied = 'file_permission_denied',
  ExportFailed = 'export_failed',
  
  // Git errors (Phase 2)
  GitInitFailed = 'git_init_failed',
  GitCommitFailed = 'git_commit_failed',
  GitPushFailed = 'git_push_failed',
  
  // Collaboration errors (Phase 3)
  WebSocketConnectionFailed = 'websocket_connection_failed',
  SyncConflict = 'sync_conflict',
  UserUnauthorized = 'user_unauthorized',
  
  // Generic errors
  Internal = 'internal',
  SerializationError = 'serialization_error'
}
```

## Export/Import Formats

### Markdown Export Format
```markdown
---
title: "Page Title"
created_at: "2024-01-01T00:00:00Z"
updated_at: "2024-01-01T00:00:00Z"
tags: ["tag1", "tag2"]
properties:
  color: "blue"
  priority: 5
---

# Page Title

This is a root block.

- This is a child block
  - This is a nested child block
- Another child block

> This is a quote block

```javascript
// This is a code block
console.log("Hello, world!");
```
```

### JSON Export Format
```json
{
  "version": "1.0",
  "export_date": "2024-01-01T00:00:00Z",
  "pages": [
    {
      "id": "page-uuid",
      "title": "Page Title", 
      "blocks": [
        {
          "id": "block-uuid",
          "text": "Block content",
          "parent_id": null,
          "children": ["child-block-uuid"],
          "order": 0,
          "block_type": "text",
          "properties": {}
        }
      ],
      "properties": {},
      "tags": [],
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## Migration Schema

### Database Migration System
```sql
CREATE TABLE migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Example migration
INSERT INTO migrations (version, description) VALUES 
('001_initial_schema', 'Create initial tables for pages, blocks, and users'),
('002_add_search_index', 'Add full-text search capability'),
('003_add_crdt_support', 'Add CRDT fields and events table');
```

---

## Frontend UI Component Library

The Note application uses **Shadcn-Vue** as the primary UI component library:

### Shadcn-Vue Integration
- **Library**: [shadcn-vue](https://www.shadcn-vue.com/) - Vue port of the popular shadcn/ui
- **Installation**: Components are copied directly into the project (not as dependencies)
- **Styling**: Built with TailwindCSS for easy customization
- **Components Path**: `@/components/ui/`
- **Import Pattern**: `import { Button } from '@/components/ui/button'`

### Key Benefits
- **No Additional Dependencies**: Components are copied into your project
- **Full Customization**: Complete control over component code and styling
- **TailwindCSS Integration**: Seamless styling with our existing design system
- **TypeScript Support**: Full TypeScript support out of the box
- **Community Driven**: Active community with regular updates

### Usage Example
```vue
<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Create New Page</CardTitle>
    </CardHeader>
    <CardContent>
      <Input placeholder="Page title..." />
      <Button class="mt-4">Create Page</Button>
    </CardContent>
  </Card>
</template>
```

This approach ensures we have maximum flexibility and control over our UI components while maintaining consistency and quality. 