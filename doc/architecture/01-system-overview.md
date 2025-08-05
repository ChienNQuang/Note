# System Architecture Overview

## 1. Vision & Objectives

### Primary Goal
Build a local-first, Git-native note-taking application inspired by Logseq/Tana, focusing on a **unified node-based architecture**, bi-directional linking, and a journal-first workflow.

### Core Principles
- **Local-First**: All data lives locally, sync is optional
- **Git-Native**: Automatic versioning and change tracking  
- **Node-Centric**: Everything is a referenceable node with a UUID
- **Journal-First**: The primary interface is the daily journal
- **Bi-Directional Linking**: Automatic back-references for all nodes
- **Cross-Platform**: Desktop-first with mobile support
- **Performance**: Handle 10,000+ nodes without lag

## 2. High-Level Architecture (Refactored)

The architecture is now centered around a unified `Node` entity, a journal-first UI, and a bi-directional linking system.

```
┌─────────────────────────────────────────────────────────────┐
│                   Frontend (Vue 3 + TS)                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Journal View  │  │    Node View    │  │  UI/UX      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │ (Tauri API)
┌─────────────────────────────────────────────────────────────┐
│                   Backend (Rust + Tauri)                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Node Service  │  │   Link Service  │  │ Git Manager │ │
│  │  (CRUD on Nodes)│  │ (Bi-directional)│  │   (libgit2) │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   SQLite (Nodes)│  │ SQLite (Links)  │  │ Git Repo    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 3. Data Flow Architecture (Refactored)

### Core Data Flow
1. **User Interaction** → Frontend emits node events from Journal or Node view
2. **Event Processing** → Backend `NodeService` performs CRUD operations
3. **Link Analysis** → `LinkService` parses node content for `[[wikilinks]]`
4. **State Update** → `nodes` and `node_links` tables are updated
5. **UI Update** → Frontend receives updated node data and re-renders
6. **Version Control** → Git commits generated automatically for changes

### Bi-Directional Linking
```
User creates/updates a Node → `LinkService` parses `[[wikilink]]` references
                                  ↓
                  `node_links` table is updated (source_id → target_id)
                                  ↓
    When viewing a Node, `LinkService` queries for back-references
                                  ↓
             `Linked References` section is populated in the UI
```

## 4. Core Components (Refactored)

### 4.1 Frontend Components
- **Journal View**: The main interface, showing daily notes in an outliner format
- **Node View**: A "zoomed-in" view of a single node, where its content is the title
- **Node Component**: A recursive component for displaying and editing a node and its children
- **Sidebar**: For navigation between Journals, Library, and other views
- **Linked References**: A component to display bi-directional links

### 4.2 Backend Services
- **Node Service**: Handles all CRUD operations for nodes
- **Link Service**: Manages bi-directional links, parsing, and lookups
- **Git Manager**: Automatic versioning and repository management
- **Search Service**: Full-text search across all nodes

## 5. Technology Stack

(No changes to the technology stack)

## 6. Security & Performance

(No changes to security and performance targets, but now applied to nodes instead of pages/blocks)

## 7. Deployment & Distribution

(No changes to deployment and distribution)

## 8. Scalability Considerations

- **Node Indexing**: Efficient indexing on `parent_id` and `updated_at` for fast queries
- **Link Indexing**: Indexes on `source_node_id` and `target_node_id` for fast back-reference lookups
- **Lazy Loading**: Frontend will lazy-load node children as needed for performance

## 9. Extensibility Architecture

(No changes to extensibility architecture) 