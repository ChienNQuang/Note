# Corrected Feature Development Roadmap

## 🔧 Key Changes from Original Plan

This corrected roadmap addresses critical dependency issues identified in the review analysis:

1. **Added Page Management System** to Phase 1 (Week 4)
2. **Restructured Phase 1** to ensure proper dependencies
3. **Simplified initial database schema** to avoid over-engineering
4. **Enhanced data models** for future CRDT support
5. **Moved file system integration** to Phase 2 where it belongs
6. **Aligned testing strategy** with incremental development

---

## Phase 1: Foundation (MVP) - Weeks 1-9 *(+1 week)*

### 1.1 Core Infrastructure
**Priority: CRITICAL | Estimated: 3 weeks**

#### Planning (Week 1, Days 1-2)
- [ ] Finalize technical architecture decisions
- [ ] Set up development environment standards
- [ ] Create detailed API specifications
- [ ] Define data models and schemas
- [ ] Establish coding standards and conventions

#### Design (Week 1, Days 3-5)
- [ ] Create wireframes for core UI components
- [ ] Design page and block editor interface mockups
- [ ] Define user interaction patterns
- [ ] Create component library design system
- [ ] Establish dark/light theme specifications

#### Implementation (Weeks 2-3)
- [ ] Set up Tauri project structure
- [ ] Configure Vue 3 + TypeScript frontend
- [ ] Implement basic Tauri commands for IPC
- [ ] Set up Pinia store structure
- [ ] Configure TailwindCSS and ShadCN-UI
- [ ] Create basic routing system
- [ ] Implement error handling framework
- [ ] Set up logging and debugging tools

#### Testing (Week 3, Days 4-5)
- [ ] Write unit tests for core utilities
- [ ] Set up E2E testing framework
- [ ] Create integration test structure
- [ ] Test Tauri IPC communication
- [ ] Validate cross-platform compatibility

#### Validation (Week 3, Day 5)
- [ ] Verify app starts on all platforms
- [ ] Confirm basic navigation works
- [ ] Validate build and packaging process

### 1.2 Page Management System *(NEW)*
**Priority: CRITICAL | Estimated: 1 week**

#### Planning (Week 4, Days 1-2)
- [ ] Define Page data structure and lifecycle
- [ ] Design page creation/deletion workflows
- [ ] Plan page metadata and properties system
- [ ] Specify page navigation requirements
- [ ] Design page-block relationship model

#### Design (Week 4, Day 3)
- [ ] Create page management UI wireframes
- [ ] Design page browser/sidebar interface
- [ ] Plan page creation and editing UX
- [ ] Define page templates system

#### Implementation (Week 4, Days 4-5)
- [ ] Create Page interface and types (with CRDT preparation)
- [ ] Implement Page CRUD operations (Tauri commands)
- [ ] Build page store (Pinia) with state management
- [ ] Create basic page navigation components
- [ ] Add page properties and metadata system
- [ ] Implement page creation/deletion UI

#### Testing (Week 4, Day 5)
- [ ] Unit tests for Page operations
- [ ] Test page store functionality
- [ ] Validate page creation/deletion flows
- [ ] Test page navigation components

#### Validation (Week 4, Day 5)
- [ ] Users can create and manage pages
- [ ] Page navigation works correctly
- [ ] Page metadata persists properly

### 1.3 Block System Core *(MODIFIED)*
**Priority: CRITICAL | Estimated: 2 weeks**

#### Planning (Week 5, Days 1-2)
- [ ] Define block data structure with page_id (required field)
- [ ] Design UUID generation strategy
- [ ] Plan parent-child relationship system within pages
- [ ] Specify block properties schema
- [ ] Design block event system (prepare for CRDT)

#### Design (Week 5, Day 3)
- [ ] Create block editor UI wireframes
- [ ] Design block selection and manipulation UX
- [ ] Plan keyboard shortcuts and accessibility
- [ ] Define block rendering specifications

#### Implementation (Week 5-6)
- [ ] Create enhanced Block interface with page_id and CRDT fields
- [ ] Implement UUID generation system
- [ ] Build block creation/deletion operations (page-aware)
- [ ] Implement parent-child relationships within page context
- [ ] Create block properties system
- [ ] Build basic block editor component
- [ ] Implement text editing functionality
- [ ] Add block selection and manipulation
- [ ] Create block serialization/deserialization
- [ ] Connect blocks to pages (enforce page_id requirement)

#### Testing (Week 6, Days 4-5)
- [ ] Unit tests for block operations
- [ ] Test UUID uniqueness and generation
- [ ] Validate parent-child relationships within pages
- [ ] Test block editor functionality
- [ ] Performance tests for large block sets
- [ ] Test block-page relationship integrity

#### Validation (Week 6, Day 5)
- [ ] User can create and edit blocks within pages
- [ ] Block relationships work correctly
- [ ] Performance acceptable for 1000+ blocks per page
- [ ] Block-page associations are reliable

### 1.4 Local Storage (SQLite) *(SIMPLIFIED)*
**Priority: CRITICAL | Estimated: 1 week**

#### Planning (Week 7, Days 1-2)
- [ ] Design core database schema (pages, blocks, page_blocks only)
- [ ] Plan indexing strategy for performance
- [ ] Define migration system foundation
- [ ] Specify CRUD operations
- [ ] Plan backup and recovery procedures

#### Design (Week 7, Day 3)
- [ ] Create ER diagram for core schema
- [ ] Design query optimization strategy
- [ ] Plan transaction management
- [ ] Define data validation rules

#### Implementation (Week 7, Days 4-5)
- [ ] Set up rusqlite dependency
- [ ] Create simplified database schema (core tables only)
```sql
-- Core tables for Phase 1
CREATE TABLE pages (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    properties TEXT,
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
    properties TEXT,
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
```
- [ ] Implement Tauri commands for database operations
- [ ] Build page and block CRUD operations
- [ ] Add database indexing for performance
- [ ] Implement transaction management
- [ ] Create database connection pooling
- [ ] Add data validation and constraints

#### Testing (Week 7, Day 5)
- [ ] Unit tests for all database operations
- [ ] Test page-block relationship integrity
- [ ] Performance tests with realistic datasets
- [ ] Test concurrent access patterns
- [ ] Validate data integrity constraints

#### Validation (Week 7, Day 5)
- [ ] All page and block operations persist correctly
- [ ] Database performs well with 1000+ blocks
- [ ] Data integrity maintained under stress
- [ ] Page-block relationships are reliable

### 1.5 Basic Markdown Export *(SIMPLIFIED)*
**Priority: MEDIUM | Estimated: 1 week**

#### Planning (Week 8, Days 1-2)
- [ ] Define basic markdown export format for pages
- [ ] Plan block-to-markdown conversion rules
- [ ] Specify export file organization
- [ ] Design simple export/import API

#### Design (Week 8, Day 3)
- [ ] Create markdown template specifications
- [ ] Plan hierarchical block export format
- [ ] Design export progress UI

#### Implementation (Week 8, Days 4-5)
- [ ] Create markdown serialization service
- [ ] Implement page export to markdown functionality
- [ ] Build simple export UI
- [ ] Add basic import from markdown (optional)
- [ ] Create export file naming and organization

#### Testing (Week 8, Day 5)
- [ ] Test markdown export accuracy
- [ ] Validate export file format
- [ ] Test import functionality (if implemented)

#### Validation (Week 8, Day 5)
- [ ] Pages export to readable markdown
- [ ] Block hierarchy preserved in export
- [ ] Export/import round-trip works (if implemented)

### 1.6 Integration Testing & Polish *(NEW)*
**Priority: HIGH | Estimated: 1 week**

#### Implementation (Week 9)
- [ ] End-to-end testing of complete page/block workflow
- [ ] Performance optimization for Phase 1 features
- [ ] UI/UX polish for core functionality
- [ ] Bug fixes and stability improvements
- [ ] Documentation updates for Phase 1

#### Validation (Week 9, Day 5)
- [ ] Complete page management workflow functional
- [ ] Block editing within pages works smoothly
- [ ] Data persists reliably
- [ ] Export functionality works correctly
- [ ] Ready for Phase 2 development

---

## Phase 2: Git Integration & File System - Weeks 10-13

### 2.1 Git Integration *(ENHANCED)*
**Priority: HIGH | Estimated: 2 weeks**

#### Planning (Week 10, Days 1-2)
- [ ] Define Git workflow and automation
- [ ] Plan commit message generation
- [ ] Specify repository initialization
- [ ] Design conflict resolution strategy
- [ ] Plan integration with markdown export

#### Design (Week 10, Day 3)
- [ ] Create Git integration architecture
- [ ] Design automatic commit triggers
- [ ] Plan branch management strategy
- [ ] Design file organization structure

#### Implementation (Week 10-11)
- [ ] Add libgit2 dependency
- [ ] Implement repository initialization
- [ ] Create automatic commit system
- [ ] Build commit message generation
- [ ] Add branch creation and switching
- [ ] Implement status checking
- [ ] Create diff generation
- [ ] Add file staging operations
- [ ] Integrate with markdown export system

#### Testing (Week 11, Days 4-5)
- [ ] Test repository operations
- [ ] Validate automatic commits
- [ ] Test branch operations
- [ ] Performance test with large repos

#### Validation (Week 11, Day 5)
- [ ] Repository initializes correctly
- [ ] Automatic commits work reliably
- [ ] Git history is meaningful and useful

### 2.2 Enhanced File System Integration *(MOVED FROM PHASE 1)*
**Priority: MEDIUM | Estimated: 1 week**

#### Planning (Week 12, Days 1-2)
- [ ] Design file watching system
- [ ] Plan incremental file updates
- [ ] Specify file sync with database
- [ ] Design import from existing markdown files

#### Implementation (Week 12, Days 3-5)
- [ ] Build file system watchers
- [ ] Add incremental file updates
- [ ] Create enhanced file import functionality
- [ ] Add file change detection and sync
- [ ] Implement file organization management

#### Testing & Validation (Week 12, Day 5)
- [ ] Test file watching system
- [ ] Validate file system permissions
- [ ] Test import functionality with real markdown files

### 2.3 Version History UI
**Priority: MEDIUM | Estimated: 1 week**

*(Same as original plan - Week 13)*

---

## Phase 3: Real-Time Collaboration - Weeks 14-19 *(+1 week)*

### 3.1 Enhanced CRDT Implementation *(MODIFIED)*
**Priority: HIGH | Estimated: 3 weeks**

#### Planning (Week 14, Days 1-3)
- [ ] Research CRDT libraries (Yjs vs Automerge)
- [ ] Design operational transformation system
- [ ] Plan conflict resolution strategies
- [ ] Specify merge semantics for blocks
- [ ] **NEW**: Design integration with existing Block model

#### Design (Week 14, Days 4-5)
- [ ] Create CRDT integration architecture
- [ ] Design event synchronization protocol
- [ ] Plan state reconciliation process
- [ ] **NEW**: Design migration from Phase 1 data model

#### Implementation (Week 15-16)
- [ ] Integrate chosen CRDT library
- [ ] **NEW**: Enhance existing Block model with CRDT fields
- [ ] Implement block operation types (compatible with existing API)
- [ ] Create merge conflict resolution
- [ ] Build state synchronization
- [ ] Add operational transformation
- [ ] Implement causal ordering
- [ ] Create vector clock system
- [ ] Add state persistence for CRDT
- [ ] **NEW**: Migrate existing data to CRDT-aware format

#### Testing (Week 16, Days 4-5)
- [ ] Test concurrent edit scenarios
- [ ] Validate merge conflict resolution
- [ ] Test state synchronization
- [ ] Performance test with many operations
- [ ] **NEW**: Test data migration correctness

#### Validation (Week 16, Day 5)
- [ ] Concurrent edits merge correctly
- [ ] No data loss in conflict scenarios
- [ ] Performance acceptable for real-time use
- [ ] **NEW**: Existing data migrates successfully

### 3.2 WebSocket Server
**Priority: HIGH | Estimated: 2 weeks**

*(Same as original plan - Weeks 17-18)*

### 3.3 Real-Time Sync Integration *(ENHANCED)*
**Priority: HIGH | Estimated: 1 week**

#### Implementation (Week 19)
- [ ] Integrate WebSocket with enhanced CRDT engine
- [ ] Implement sync protocol with existing UI
- [ ] Add offline/online detection
- [ ] Create sync conflict UI
- [ ] **NEW**: Ensure compatibility with Git integration
- [ ] **NEW**: Handle file system sync during collaboration

*(Week 19 includes additional complexity for integration)*

---

## Phases 4-5: Advanced Features & Release - Weeks 20-32 *(+4 weeks)*

### Phase 4: Advanced Features - Weeks 20-28 *(+4 weeks)*

#### 4.1 Search & Indexing Enhancement
**Priority: HIGH | Estimated: 2 weeks**

#### Implementation (Week 20-21)
- [ ] **NEW**: Add search schema to database (deferred from Phase 1)
```sql
-- Search tables added in Phase 4
CREATE VIRTUAL TABLE search_index USING fts5(
    block_id UNINDEXED,
    content,
    title,
    tags
);

-- Add events table for CRDT (moved from Phase 1)
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    block_id TEXT,
    page_id TEXT,
    event_data TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    sequence INTEGER NOT NULL,
    vector_clock TEXT NOT NULL,
    is_applied BOOLEAN DEFAULT FALSE
);
```
- [ ] Build full-text search functionality
- [ ] Create search UI components
- [ ] Add search result highlighting
- [ ] Implement search filters
- [ ] Add search history

*(Continue with original Phase 4 plan...)*

### Phase 5: Polish & Release - Weeks 29-32 *(Same as original)*

---

## 📊 Summary of Changes

### Timeline Changes
- **Original**: 28 weeks
- **Corrected**: 32 weeks (+4 weeks)
- **Added**: Page Management System (Week 4)
- **Enhanced**: Integration complexity (+3 weeks distributed)

### Key Structural Changes
1. **Page Management First**: Added comprehensive page system before blocks
2. **Simplified Phase 1 Database**: Only core tables, defer advanced features
3. **Enhanced Data Models**: Prepared for CRDT from start
4. **Better Integration**: File system integration moved to Phase 2
5. **Progressive Testing**: Aligned with incremental development

### Dependencies Fixed
- ✅ Block system now properly depends on Page system
- ✅ Database schema matches implementation phases
- ✅ CRDT integration planned from data model level
- ✅ File system integration has proper dependencies
- ✅ Testing strategy aligns with development phases

## 🎯 Success Metrics (Updated)

### Phase 1 Completion Criteria
- [ ] Users can create and manage pages
- [ ] Users can create, edit, and organize blocks within pages
- [ ] All data persists reliably in SQLite
- [ ] Basic markdown export works
- [ ] Performance acceptable for 1000+ blocks per page
- [ ] No data loss under normal usage

### Overall Project Success
- ✅ All original success metrics still apply
- ✅ Timeline is more realistic and achievable
- ✅ Dependencies are properly managed
- ✅ Architecture supports all planned features

---

*This corrected roadmap addresses all critical dependency issues identified in the review analysis while maintaining the original project vision and scope.* 