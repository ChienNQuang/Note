# Backend Services Implementation (Refactored)

## 1. Rust/Tauri Backend Architecture (Node-Centric)

### 1.1 Project Structure
```
src-tauri/src/
├── main.rs                 # Entry point and Tauri setup
├── lib.rs                  # Library root and module declarations
├── commands/               # Tauri command handlers
│   ├── mod.rs
│   └── nodes.rs           # All node, journal, and link commands
├── services/              # Business logic services
│   ├── mod.rs
│   ├── database/          # Database connection and operations (now a module)
│   │   ├── mod.rs
│   │   ├── connection.rs
│   │   └── nodes.rs
│   └── link_service.rs    # Service for bi-directional linking
├── models/                # Data models and structures
│   ├── mod.rs
│   └── node.rs            # Unified Node model
└── ... (utils, errors)
```

## 2. Core Services Implementation

### 2.1 Database Service (`services/database/`)
- **Responsibility**: All direct interactions with the SQLite database.
- **Implementation**:
    - `nodes.rs`: Contains all SQL queries for CRUD operations on the `nodes` table.
    - The service will be refactored to use a single `nodes` table.

### 2.2 Link Service (`services/link_service.rs`)
- **Responsibility**: Manages bi-directional links.
- **Implementation**:
    - `update_links_for_node(node)`: A function that is called after a node is created or updated. It parses the `node.content` for `[[wikilinks]]`, resolves the target node IDs, and updates the `node_links` table.
    - `get_linked_references(node_id)`: Queries the `node_links` table to find all nodes that reference the given `node_id`.

## 3. Tauri Command Handlers (`commands/nodes.rs`)

All commands will be consolidated into a single `nodes.rs` file.

### Node Commands
- `create_node`, `get_node`, `update_node`, `delete_node`, `move_node`. These will call the corresponding methods in the `DatabaseService`. `create_node` and `update_node` will also trigger the `LinkService`.

### Journal Commands
- `get_or_create_daily_note`: This command will:
    1.  Check if a node with the tag `#Journal` and the property `date:: YYYY-MM-DD` already exists.
    2.  If it exists, return that node.
    3.  If not, create a new root node with the content "YYYY-MM-DD", add the `#Journal` tag, and set the `date` property, then return it.

### Linking Commands
- `get_linked_references`: Calls the `LinkService`.
- `get_unlinked_references`: Performs a full-text search for the node's content to find potential unlinked references. 