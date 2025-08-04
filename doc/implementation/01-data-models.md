# Data Models & Schemas

## 1. Core Data Structures

### 1.1 Block Interface

```typescript
interface Block {
  // Unique identifier (UUID v4)
  id: string;
  
  // Block content (markdown text)
  text: string;
  
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
}

enum BlockType {
  TEXT = "text",
  HEADING = "heading", 
  LIST_ITEM = "list_item",
  CODE = "code",
  QUOTE = "quote",
  IMAGE = "image",
  LINK = "link",
  REFERENCE = "reference"
}
```

### 1.2 Page Interface

```typescript
interface Page {
  // Page identifier (derived from title or explicit)
  id: string;
  
  // Page title
  title: string;
  
  // Root blocks of the page
  root_blocks: string[];
  
  // Page metadata
  properties: Record<string, any>;
  
  // File system mapping
  file_path?: string;
  
  // Timestamps
  created_at: Date;
  updated_at: Date;
  
  // Collaboration
  created_by: string;
  last_edited_by: string;
  
  // Tags and categories
  tags: string[];
  
  // Page status
  status: PageStatus;
}

enum PageStatus {
  DRAFT = "draft",
  PUBLISHED = "published",
  ARCHIVED = "archived",
  DELETED = "deleted"
}
```

### 1.3 User Interface

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
  
  // Collaboration status
  is_online: boolean;
  last_seen: Date;
  
  // Authentication
  auth_token?: string;
  
  // Timestamps
  created_at: Date;
  updated_at: Date;
}

interface UserPreferences {
  theme: "light" | "dark" | "system";
  font_size: number;
  auto_save_interval: number;
  git_auto_commit: boolean;
  collaboration_enabled: boolean;
  shortcuts: Record<string, string>;
}
```

## 2. Event System

### 2.1 Block Events

```typescript
interface BlockEvent {
  // Event identification
  id: string;
  type: BlockEventType;
  
  // Target block
  block_id: string;
  
  // Event data
  data: BlockEventData;
  
  // Event metadata
  user_id: string;
  timestamp: number;
  sequence: number;
  
  // CRDT information
  vector_clock: VectorClock;
  causal_context: string[];
}

enum BlockEventType {
  BLOCK_CREATED = "block_created",
  BLOCK_UPDATED = "block_updated", 
  BLOCK_DELETED = "block_deleted",
  BLOCK_MOVED = "block_moved",
  BLOCK_LINKED = "block_linked",
  BLOCK_UNLINKED = "block_unlinked"
}

type BlockEventData = 
  | BlockCreatedData
  | BlockUpdatedData
  | BlockDeletedData
  | BlockMovedData
  | BlockLinkedData;

interface BlockCreatedData {
  block: Block;
  parent_id?: string;
  position: number;
}

interface BlockUpdatedData {
  field: keyof Block;
  old_value: any;
  new_value: any;
  partial_update?: boolean;
}

interface BlockDeletedData {
  block_snapshot: Block;
  preserve_children: boolean;
}

interface BlockMovedData {
  old_parent_id?: string;
  new_parent_id?: string;
  old_position: number;
  new_position: number;
}

interface BlockLinkedData {
  target_block_id: string;
  link_type: LinkType;
}

enum LinkType {
  REFERENCE = "reference",
  EMBED = "embed",
  ALIAS = "alias"
}
```

### 2.2 Vector Clock for CRDT

```typescript
interface VectorClock {
  // User ID -> logical timestamp mapping
  clocks: Record<string, number>;
  
  // Create new vector clock
  increment(user_id: string): VectorClock;
  
  // Compare vector clocks
  compare(other: VectorClock): ClockComparison;
  
  // Merge vector clocks
  merge(other: VectorClock): VectorClock;
}

enum ClockComparison {
  BEFORE = -1,
  CONCURRENT = 0,
  AFTER = 1
}
```

## 3. Database Schema (SQLite)

### 3.1 Blocks Table

```sql
CREATE TABLE blocks (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    parent_id TEXT,
    order_index INTEGER NOT NULL,
    properties TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    last_edited_by TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    block_type TEXT DEFAULT 'text',
    is_deleted BOOLEAN DEFAULT FALSE,
    
    FOREIGN KEY (parent_id) REFERENCES blocks(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (last_edited_by) REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX idx_blocks_parent_id ON blocks(parent_id);
CREATE INDEX idx_blocks_created_by ON blocks(created_by);
CREATE INDEX idx_blocks_updated_at ON blocks(updated_at);
CREATE INDEX idx_blocks_order ON blocks(parent_id, order_index);
CREATE INDEX idx_blocks_type ON blocks(block_type);
```

### 3.2 Pages Table

```sql
CREATE TABLE pages (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    file_path TEXT UNIQUE,
    properties TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    last_edited_by TEXT NOT NULL,
    status TEXT DEFAULT 'draft',
    is_deleted BOOLEAN DEFAULT FALSE,
    
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (last_edited_by) REFERENCES users(id)
);

-- Indexes
CREATE INDEX idx_pages_title ON pages(title);
CREATE INDEX idx_pages_created_by ON pages(created_by);
CREATE INDEX idx_pages_updated_at ON pages(updated_at);
CREATE INDEX idx_pages_status ON pages(status);
CREATE UNIQUE INDEX idx_pages_file_path ON pages(file_path) WHERE file_path IS NOT NULL;
```

### 3.3 Page Blocks Junction Table

```sql
CREATE TABLE page_blocks (
    page_id TEXT NOT NULL,
    block_id TEXT NOT NULL,
    is_root BOOLEAN DEFAULT FALSE,
    
    PRIMARY KEY (page_id, block_id),
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE,
    FOREIGN KEY (block_id) REFERENCES blocks(id) ON DELETE CASCADE
);

CREATE INDEX idx_page_blocks_page ON page_blocks(page_id);
CREATE INDEX idx_page_blocks_block ON page_blocks(block_id);
CREATE INDEX idx_page_blocks_root ON page_blocks(page_id, is_root);
```

### 3.4 Events Table

```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    block_id TEXT,
    page_id TEXT,
    event_data TEXT NOT NULL, -- JSON
    user_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    sequence INTEGER NOT NULL,
    vector_clock TEXT NOT NULL, -- JSON
    causal_context TEXT, -- JSON array
    is_applied BOOLEAN DEFAULT FALSE,
    
    FOREIGN KEY (block_id) REFERENCES blocks(id),
    FOREIGN KEY (page_id) REFERENCES pages(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Indexes for event ordering and querying
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_sequence ON events(sequence);
CREATE INDEX idx_events_block_id ON events(block_id);
CREATE INDEX idx_events_user_id ON events(user_id);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_applied ON events(is_applied);
```

### 3.5 Users Table

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    avatar_url TEXT,
    preferences TEXT, -- JSON
    is_online BOOLEAN DEFAULT FALSE,
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
    auth_token TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_online ON users(is_online);
CREATE INDEX idx_users_last_seen ON users(last_seen);
```

### 3.6 Search Index Table

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
    block_id UNINDEXED,
    content,
    title,
    tags,
    content='blocks',
    content_rowid='rowid'
);

-- Triggers to maintain search index
CREATE TRIGGER search_index_insert AFTER INSERT ON blocks BEGIN
    INSERT INTO search_index(block_id, content) VALUES (new.id, new.text);
END;

CREATE TRIGGER search_index_update AFTER UPDATE ON blocks BEGIN
    UPDATE search_index SET content = new.text WHERE block_id = new.id;
END;

CREATE TRIGGER search_index_delete AFTER DELETE ON blocks BEGIN
    DELETE FROM search_index WHERE block_id = old.id;
END;
```

## 4. API Interfaces

### 4.1 Tauri Commands

```typescript
// Block operations
interface BlockCommands {
  create_block(data: CreateBlockData): Promise<Block>;
  update_block(id: string, data: UpdateBlockData): Promise<Block>;
  delete_block(id: string): Promise<void>;
  get_block(id: string): Promise<Block | null>;
  get_blocks_by_page(page_id: string): Promise<Block[]>;
  move_block(id: string, new_parent: string, position: number): Promise<void>;
  search_blocks(query: string): Promise<SearchResult[]>;
}

// Page operations  
interface PageCommands {
  create_page(data: CreatePageData): Promise<Page>;
  update_page(id: string, data: UpdatePageData): Promise<Page>;
  delete_page(id: string): Promise<void>;
  get_page(id: string): Promise<Page | null>;
  list_pages(): Promise<Page[]>;
  export_page_markdown(id: string): Promise<string>;
  import_page_markdown(content: string): Promise<Page>;
}

// Git operations
interface GitCommands {
  init_repository(path: string): Promise<void>;
  commit_changes(message: string): Promise<string>;
  get_commit_history(): Promise<GitCommit[]>;
  get_file_diff(commit: string, file: string): Promise<string>;
  restore_version(commit: string): Promise<void>;
  get_repository_status(): Promise<GitStatus>;
}

// Collaboration operations
interface CollaborationCommands {
  start_collaboration_server(port: number): Promise<void>;
  stop_collaboration_server(): Promise<void>;
  connect_to_server(url: string): Promise<void>;
  disconnect_from_server(): Promise<void>;
  get_online_users(): Promise<User[]>;
  send_event(event: BlockEvent): Promise<void>;
}
```

### 4.2 WebSocket Protocol

```typescript
interface WebSocketMessage {
  type: MessageType;
  data: any;
  user_id: string;
  timestamp: number;
  message_id: string;
}

enum MessageType {
  // Connection management
  CONNECT = "connect",
  DISCONNECT = "disconnect",
  HEARTBEAT = "heartbeat",
  
  // Authentication
  AUTH = "auth",
  AUTH_SUCCESS = "auth_success",
  AUTH_FAILURE = "auth_failure",
  
  // Block events
  BLOCK_EVENT = "block_event",
  BLOCK_EVENT_ACK = "block_event_ack",
  
  // Sync events
  SYNC_REQUEST = "sync_request",
  SYNC_RESPONSE = "sync_response",
  SYNC_COMPLETE = "sync_complete",
  
  // User presence
  USER_JOINED = "user_joined",
  USER_LEFT = "user_left",
  USER_STATUS = "user_status",
  
  // Error handling
  ERROR = "error"
}
```

## 5. Configuration Schemas

### 5.1 Application Configuration

```typescript
interface AppConfig {
  // Database settings
  database: {
    path: string;
    backup_interval: number;
    max_backup_count: number;
  };
  
  // Git settings
  git: {
    auto_commit: boolean;
    commit_interval: number;
    auto_push: boolean;
    remote_url?: string;
  };
  
  // Collaboration settings
  collaboration: {
    enabled: boolean;
    server_port: number;
    max_connections: number;
    heartbeat_interval: number;
  };
  
  // Performance settings
  performance: {
    max_blocks_per_page: number;
    search_debounce_ms: number;
    auto_save_interval: number;
    virtual_scroll_threshold: number;
  };
  
  // Security settings
  security: {
    encrypt_local_data: boolean;
    require_auth: boolean;
    session_timeout: number;
  };
}
```

### 5.2 Export/Import Formats

```typescript
interface ExportFormat {
  version: string;
  exported_at: Date;
  exported_by: string;
  data: {
    pages: Page[];
    blocks: Block[];
    users: User[];
    metadata: ExportMetadata;
  };
}

interface ExportMetadata {
  total_pages: number;
  total_blocks: number;
  export_type: "full" | "partial" | "page";
  filters_applied?: ExportFilter[];
}

interface ExportFilter {
  field: string;
  operator: "eq" | "neq" | "gt" | "lt" | "contains";
  value: any;
}
```

## 6. Error Types

```typescript
interface AppError {
  code: ErrorCode;
  message: string;
  context?: Record<string, any>;
  timestamp: Date;
  user_id?: string;
  stack_trace?: string;
}

enum ErrorCode {
  // Database errors
  DB_CONNECTION_FAILED = "DB_CONNECTION_FAILED",
  DB_QUERY_FAILED = "DB_QUERY_FAILED",
  DB_CONSTRAINT_VIOLATION = "DB_CONSTRAINT_VIOLATION",
  
  // Git errors
  GIT_INIT_FAILED = "GIT_INIT_FAILED",
  GIT_COMMIT_FAILED = "GIT_COMMIT_FAILED",
  GIT_PUSH_FAILED = "GIT_PUSH_FAILED",
  
  // Collaboration errors
  WEBSOCKET_CONNECTION_FAILED = "WEBSOCKET_CONNECTION_FAILED",
  SYNC_CONFLICT = "SYNC_CONFLICT",
  USER_UNAUTHORIZED = "USER_UNAUTHORIZED",
  
  // Validation errors
  INVALID_BLOCK_DATA = "INVALID_BLOCK_DATA",
  INVALID_PAGE_DATA = "INVALID_PAGE_DATA",
  MISSING_REQUIRED_FIELD = "MISSING_REQUIRED_FIELD",
  
  // File system errors
  FILE_NOT_FOUND = "FILE_NOT_FOUND",
  FILE_PERMISSION_DENIED = "FILE_PERMISSION_DENIED",
  EXPORT_FAILED = "EXPORT_FAILED"
}
```

## 7. Migration Schema

```typescript
interface Migration {
  version: number;
  name: string;
  up: string[];  // SQL statements to apply
  down: string[]; // SQL statements to rollback
  applied_at?: Date;
  checksum: string;
}

interface MigrationHistory {
  current_version: number;
  migrations: Migration[];
  last_migration_at: Date;
}
``` 