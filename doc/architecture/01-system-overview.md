# System Architecture Overview

## 1. Vision & Objectives

### Primary Goal
Build a local-first, Git-native note-taking application that addresses Logseq's limitations with real-time collaboration, extensibility, and cross-platform performance.

### Core Principles
- **Local-First**: All data lives locally, sync is optional
- **Git-Native**: Automatic versioning and change tracking
- **Block-Centric**: Everything is a referenceable block with UUID
- **Real-Time Collaboration**: WebSocket + CRDT for multi-user editing
- **Cross-Platform**: Desktop-first with mobile support
- **Performance**: Handle 10,000+ blocks without lag
- **Extensibility**: Plugin architecture for future features

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (Vue 3 + TS)                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Block Editor  │  │   Sidebar Nav   │  │  UI/UX      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │ (Tauri API)
┌─────────────────────────────────────────────────────────────┐
│                   Backend (Rust + Tauri)                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   CRDT Engine   │  │   Git Manager   │  │  WebSocket  │ │
│  │    (Yjs/Auto)   │  │     (libgit2)   │  │   Server    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │     SQLite      │  │   Markdown      │  │ Git Repo    │ │
│  │   (Local DB)    │  │    Files        │  │  (Version)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 3. Data Flow Architecture

### Core Data Flow
1. **User Interaction** → Frontend emits block events
2. **Event Processing** → CRDT engine applies operations
3. **State Update** → Local SQLite database updated
4. **File Generation** → Markdown files derived from database
5. **Version Control** → Git commits generated automatically
6. **Collaboration** → WebSocket broadcasts changes to peers

### Event-Driven Architecture
```
User Edit → Block Event → CRDT Operation → Database Update
                                ↓
                         Markdown Export ← File System
                                ↓
                         Git Commit ← Version Control
                                ↓
                     WebSocket Broadcast ← Collaboration
```

## 4. Core Components

### 4.1 Frontend Components
- **Block Editor**: Rich text editing with block structure
- **Sidebar Navigation**: File/page browser and search
- **Graph View**: Visual representation of block relationships
- **Command Palette**: Quick actions and navigation
- **Settings Panel**: Configuration and preferences

### 4.2 Backend Services
- **CRDT Engine**: Conflict-free replicated data types for collaboration
- **Git Manager**: Automatic versioning and repository management
- **WebSocket Server**: Real-time communication
- **Search Engine**: Full-text search across all blocks
- **File System Manager**: Markdown file I/O operations

### 4.3 Storage Components
- **SQLite Database**: Local structured data storage
- **Markdown Files**: Human-readable text format
- **Git Repository**: Version control and history
- **Cache Layer**: Performance optimization

## 5. Technology Stack

### Frontend
- **Framework**: Vue 3 with Composition API
- **Language**: TypeScript (strict mode)
- **Build Tool**: Vite
- **UI Framework**: ShadCN-UI + TailwindCSS
- **State Management**: Pinia
- **Desktop Framework**: Tauri

### Backend
- **Language**: Rust (latest stable)
- **Web Framework**: Tauri (for IPC)
- **CRDT Library**: Yjs or Automerge
- **Git Integration**: libgit2
- **WebSocket**: tokio-tungstenite
- **Database**: rusqlite

### Storage
- **Local Database**: SQLite
- **File Format**: Markdown
- **Version Control**: Git
- **Optional Sync**: PostgreSQL (server mode)

## 6. Security & Performance

### Security Considerations
- Local file encryption option
- Secure WebSocket connections (WSS)
- Input sanitization for markdown
- Git credential management
- Sandboxed plugin execution

### Performance Targets
- App startup: <2 seconds
- Block editing: <50ms response
- Search: <100ms for 10k blocks
- Sync: <1 second for small changes
- Memory usage: <500MB for 10k blocks

## 7. Deployment & Distribution

### Desktop Distribution
- Tauri bundled executables
- Auto-updater for releases
- Cross-platform support (Windows, macOS, Linux)

### Optional Server Components
- Docker container for self-hosted sync
- PostgreSQL for server-side storage
- WebSocket server for real-time collaboration

## 8. Scalability Considerations

### Local Scalability
- Efficient block indexing for large documents
- Lazy loading for performance
- Background sync operations
- Incremental Git operations

### Collaboration Scalability
- CRDT operations for conflict resolution
- Efficient delta synchronization
- Peer-to-peer sync options
- Server-based sync for teams

## 9. Extensibility Architecture

### Plugin System
- TypeScript/JavaScript plugins
- Sandboxed execution environment
- Plugin API for block operations
- Theme and UI customization

### API Design
- RESTful API for external integrations
- GraphQL for complex queries
- WebSocket for real-time updates
- CLI tools for automation 