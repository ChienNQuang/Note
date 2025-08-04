# Testing Strategy

## 1. Testing Pyramid Overview

```
                  E2E Tests (10%)
                 ├─ User Workflows
                 ├─ Cross-browser
                 └─ Performance
                
            Integration Tests (20%)
           ├─ API Integration
           ├─ Database Integration  
           ├─ Git Integration
           └─ Real-time Collaboration
           
        Unit Tests (70%)
       ├─ Component Tests
       ├─ Service Tests
       ├─ Store Tests
       └─ Utility Tests
```

## 2. Unit Testing Strategy

### 2.1 Frontend Unit Tests (Vue/TypeScript)

#### Planning
- [ ] Set up Vitest testing framework
- [ ] Configure Vue Test Utils
- [ ] Set up TypeScript testing support
- [ ] Define component testing patterns

#### Test Categories

**Component Tests**
```typescript
// Block.spec.ts
import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import Block from '@/components/editor/Block.vue'
import { useBlockStore } from '@/stores/block'

describe('Block Component', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should render block content correctly', () => {
    const wrapper = mount(Block, {
      props: {
        blockId: 'test-block-1',
        depth: 0
      }
    })
    
    expect(wrapper.find('[contenteditable]').exists()).toBe(true)
  })

  it('should handle text input correctly', async () => {
    const wrapper = mount(Block, {
      props: {
        blockId: 'test-block-1', 
        depth: 0
      }
    })
    
    const contentDiv = wrapper.find('[contenteditable]')
    await contentDiv.setValue('New text content')
    
    // Verify store was updated
    const blockStore = useBlockStore()
    expect(blockStore.getBlock('test-block-1')?.text).toBe('New text content')
  })

  it('should handle Enter key to create new block', async () => {
    const wrapper = mount(Block, {
      props: {
        blockId: 'test-block-1',
        depth: 0
      }
    })
    
    const contentDiv = wrapper.find('[contenteditable]')
    await contentDiv.trigger('keydown', { key: 'Enter' })
    
    expect(wrapper.emitted('create-block')).toBeTruthy()
  })

  it('should handle Backspace key to delete empty block', async () => {
    const wrapper = mount(Block, {
      props: {
        blockId: 'test-block-1',
        depth: 0
      }
    })
    
    const contentDiv = wrapper.find('[contenteditable]')
    // Simulate empty block
    contentDiv.element.textContent = ''
    
    await contentDiv.trigger('keydown', { key: 'Backspace' })
    
    expect(wrapper.emitted('delete-block')).toBeTruthy()
  })

  it('should handle Tab indentation correctly', async () => {
    const wrapper = mount(Block, {
      props: {
        blockId: 'test-block-1',
        depth: 0
      }
    })
    
    const contentDiv = wrapper.find('[contenteditable]')
    await contentDiv.trigger('keydown', { key: 'Tab' })
    
    expect(wrapper.emitted('move-block')).toBeTruthy()
  })
})
```

**Store Tests**
```typescript
// blockStore.spec.ts
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useBlockStore } from '@/stores/block'
import * as blockApi from '@/api/blocks'

// Mock the API
vi.mock('@/api/blocks')

describe('Block Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('should create block successfully', async () => {
    const blockStore = useBlockStore()
    const mockBlock = {
      id: 'test-id',
      text: 'Test block',
      parent_id: null,
      children: [],
      order: 0,
      properties: {},
      created_at: new Date(),
      updated_at: new Date(),
      created_by: 'user-1',
      last_edited_by: 'user-1',
      version: 1,
      block_type: 'text'
    }

    vi.mocked(blockApi.createBlock).mockResolvedValue(mockBlock)

    const result = await blockStore.createBlock({
      text: 'Test block',
      parent_id: null,
      position: 0
    })

    expect(result).toEqual(mockBlock)
    expect(blockStore.getBlock('test-id')).toEqual(mockBlock)
  })

  it('should handle optimistic updates', async () => {
    const blockStore = useBlockStore()
    // Pre-populate with a block
    await blockStore.createBlock({ text: 'Original text' })
    
    const blockId = Array.from(blockStore.blocks.keys())[0]
    
    // Update should be immediate (optimistic)
    await blockStore.updateBlock(blockId, { text: 'Updated text' })
    
    expect(blockStore.getBlock(blockId)?.text).toBe('Updated text')
  })

  it('should handle API errors gracefully', async () => {
    const blockStore = useBlockStore()
    
    vi.mocked(blockApi.createBlock).mockRejectedValue(new Error('API Error'))
    
    await expect(blockStore.createBlock({ text: 'Test' })).rejects.toThrow('API Error')
  })
})
```

**Service Tests**
```typescript
// markdownService.spec.ts
import { describe, it, expect } from 'vitest'
import { markdownService } from '@/services/markdown'

describe('Markdown Service', () => {
  it('should parse basic markdown correctly', () => {
    const input = '# Heading\n\nSome **bold** text'
    const result = markdownService.parse(input)
    
    expect(result).toContain('<h1>Heading</h1>')
    expect(result).toContain('<strong>bold</strong>')
  })

  it('should extract block references', () => {
    const input = 'See [[block-123]] and [[block-456]] for details'
    const references = markdownService.extractReferences(input)
    
    expect(references).toEqual(['block-123', 'block-456'])
  })

  it('should convert blocks to markdown', () => {
    const blocks = [
      {
        id: '1',
        text: 'Heading',
        block_type: 'heading',
        properties: { level: 1 },
        depth: 0
      },
      {
        id: '2', 
        text: 'List item',
        block_type: 'list_item',
        depth: 1
      }
    ]
    
    const result = markdownService.convertToMarkdown(blocks)
    
    expect(result).toContain('# Heading')
    expect(result).toContain('  - List item')
  })
})
```

#### Testing Tasks
- [ ] Set up testing environment with Vitest
- [ ] Create component test templates
- [ ] Write tests for all major components
- [ ] Test all Pinia stores
- [ ] Test utility functions
- [ ] Test service classes
- [ ] Add snapshot testing for UI components
- [ ] Set up coverage reporting

### 2.2 Backend Unit Tests (Rust)

#### Planning
- [ ] Set up cargo test framework
- [ ] Configure test database
- [ ] Set up mocking for external dependencies
- [ ] Define testing patterns for async code

#### Test Categories

**Database Service Tests**
```rust
// tests/database_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::services::DatabaseService;
    use crate::models::Block;

    fn setup_test_db() -> (DatabaseService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseService::new(&db_path).unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_create_block() {
        let (db, _temp_dir) = setup_test_db();
        
        let block = Block {
            id: "test-block-1".to_string(),
            text: "Test content".to_string(),
            parent_id: None,
            children: vec![],
            order: 0,
            properties: HashMap::new(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            created_by: "test-user".to_string(),
            last_edited_by: "test-user".to_string(),
            version: 1,
            block_type: "text".to_string(),
        };

        let result = db.create_block(&block).await;
        assert!(result.is_ok());

        let retrieved = db.get_block("test-block-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().text, "Test content");
    }

    #[tokio::test]
    async fn test_update_block() {
        let (db, _temp_dir) = setup_test_db();
        
        // First create a block
        let block = create_test_block();
        db.create_block(&block).await.unwrap();

        // Update it
        let updates = serde_json::json!({
            "text": "Updated content"
        });
        
        let result = db.update_block("test-block-1", &updates).await;
        assert!(result.is_ok());

        let updated = db.get_block("test-block-1").await.unwrap().unwrap();
        assert_eq!(updated.text, "Updated content");
    }

    #[tokio::test]
    async fn test_search_blocks() {
        let (db, _temp_dir) = setup_test_db();
        
        // Create test blocks
        for i in 0..5 {
            let block = Block {
                id: format!("block-{}", i),
                text: format!("Content {} with searchable text", i),
                // ... other fields
            };
            db.create_block(&block).await.unwrap();
        }

        let results = db.search_blocks("searchable").await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let (db, _temp_dir) = setup_test_db();
        
        let result = db.execute_transaction(|tx| {
            // Create block
            let block = create_test_block();
            db.create_block_in_transaction(tx, &block)?;
            
            // Force an error
            Err(AppError::DatabaseError("Forced error".to_string()))
        }).await;

        assert!(result.is_err());
        
        // Verify block was not created
        let block = db.get_block("test-block-1").await.unwrap();
        assert!(block.is_none());
    }
}
```

**CRDT Engine Tests**
```rust
// tests/crdt_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::CRDTEngine;

    #[tokio::test]
    async fn test_concurrent_text_edits() {
        let mut engine1 = CRDTEngine::new("user1".to_string());
        let mut engine2 = CRDTEngine::new("user2".to_string());

        // User 1 creates a block
        let create_op = engine1.create_operation(
            OperationType::Insert,
            "block-1".to_string(),
            serde_json::json!({
                "text": "Hello World",
                "id": "block-1"
            }),
        );
        
        engine1.apply_local_operation(create_op.clone()).unwrap();
        engine2.apply_remote_operation(create_op).unwrap();

        // Both users edit the same block concurrently
        let edit1 = engine1.create_operation(
            OperationType::Update,
            "block-1".to_string(),
            serde_json::json!({
                "text": "Hello Beautiful World"
            }),
        );

        let edit2 = engine2.create_operation(
            OperationType::Update, 
            "block-1".to_string(),
            serde_json::json!({
                "text": "Hello Amazing World"
            }),
        );

        // Apply operations in different orders
        engine1.apply_local_operation(edit1.clone()).unwrap();
        engine1.apply_remote_operation(edit2.clone()).unwrap();
        
        engine2.apply_local_operation(edit2).unwrap();
        engine2.apply_remote_operation(edit1).unwrap();

        // Both engines should converge to the same state
        let state1 = engine1.get_current_state();
        let state2 = engine2.get_current_state();
        
        assert_eq!(state1, state2);
    }

    #[tokio::test]
    async fn test_causality_ordering() {
        let mut engine = CRDTEngine::new("user1".to_string());

        // Create operations with dependencies
        let op1 = engine.create_operation(
            OperationType::Insert,
            "block-1".to_string(),
            serde_json::json!({"text": "First"}),
        );

        let op2 = engine.create_operation(
            OperationType::Update,
            "block-1".to_string(), 
            serde_json::json!({"text": "Second"}),
        );

        // Apply them out of order
        engine.apply_remote_operation(op2).unwrap(); // Should be queued
        engine.apply_remote_operation(op1).unwrap(); // Should trigger op2

        let state = engine.get_current_state();
        assert_eq!(state.get("block-1").unwrap().text, "Second");
    }
}
```

#### Testing Tasks
- [ ] Set up Rust testing framework
- [ ] Create database test utilities
- [ ] Write comprehensive service tests
- [ ] Test error handling scenarios
- [ ] Test concurrent operations
- [ ] Add performance benchmarks
- [ ] Test memory safety
- [ ] Set up continuous integration

## 3. Integration Testing Strategy

### 3.1 API Integration Tests

#### Planning
- [ ] Set up test harness for Tauri commands
- [ ] Create mock external services
- [ ] Define integration test patterns
- [ ] Set up test data fixtures

#### Implementation
```typescript
// tests/integration/api.spec.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { invoke } from '@tauri-apps/api'
import { setupTestApp, cleanupTestApp } from './test-utils'

describe('API Integration Tests', () => {
  beforeEach(async () => {
    await setupTestApp()
  })

  afterEach(async () => {
    await cleanupTestApp()
  })

  it('should create and retrieve block via API', async () => {
    // Create block
    const createRequest = {
      text: 'Integration test block',
      parent_id: null,
      block_type: 'text'
    }

    const createdBlock = await invoke('create_block', { request: createRequest })
    expect(createdBlock).toBeDefined()
    expect(createdBlock.text).toBe('Integration test block')

    // Retrieve block
    const retrievedBlock = await invoke('get_block', { blockId: createdBlock.id })
    expect(retrievedBlock).toEqual(createdBlock)
  })

  it('should handle block hierarchy correctly', async () => {
    // Create parent block
    const parent = await invoke('create_block', {
      request: { text: 'Parent block', parent_id: null }
    })

    // Create child block
    const child = await invoke('create_block', {
      request: { text: 'Child block', parent_id: parent.id }
    })

    // Verify hierarchy
    const parentWithChildren = await invoke('get_block', { blockId: parent.id })
    expect(parentWithChildren.children).toContain(child.id)
  })

  it('should search blocks correctly', async () => {
    // Create test blocks
    await invoke('create_block', {
      request: { text: 'Unique searchable content' }
    })
    await invoke('create_block', {
      request: { text: 'Different content' }
    })

    // Search
    const results = await invoke('search_blocks', { query: 'searchable' })
    expect(results).toHaveLength(1)
    expect(results[0].text).toContain('searchable')
  })
})
```

### 3.2 Database Integration Tests

#### Implementation
```rust
// tests/integration/database_integration.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_concurrent_database_access() {
        let db = Arc::new(setup_test_database().await);
        let mut handles = vec![];

        // Spawn multiple tasks that write to database concurrently
        for i in 0..10 {
            let db_clone = db.clone();
            let handle = tokio::spawn(async move {
                let block = create_test_block(&format!("block-{}", i));
                db_clone.create_block(&block).await.unwrap();
                
                // Update the block
                let updates = serde_json::json!({
                    "text": format!("Updated block {}", i)
                });
                db_clone.update_block(&block.id, &updates).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all blocks were created and updated
        for i in 0..10 {
            let block = db.get_block(&format!("block-{}", i)).await.unwrap();
            assert!(block.is_some());
            assert_eq!(block.unwrap().text, format!("Updated block {}", i));
        }
    }

    #[tokio::test]
    async fn test_database_recovery_after_corruption() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        {
            let db = DatabaseService::new(&db_path).unwrap();
            let block = create_test_block("test-block");
            db.create_block(&block).await.unwrap();
        }

        // Simulate corruption by writing invalid data
        std::fs::write(&db_path, "corrupted data").unwrap();

        // Try to open database - should handle gracefully
        let result = DatabaseService::new(&db_path);
        
        // Depending on implementation, might recover or need manual intervention
        // This test verifies the error handling behavior
        assert!(result.is_err() || result.is_ok());
    }
}
```

### 3.3 Real-time Collaboration Tests

#### Implementation
```typescript
// tests/integration/collaboration.spec.ts
import { describe, it, expect, beforeEach } from 'vitest'
import { WebSocket } from 'ws'
import { setupCollaborationServer, createTestUser } from './collaboration-utils'

describe('Real-time Collaboration', () => {
  let server: any
  let user1: WebSocket
  let user2: WebSocket

  beforeEach(async () => {
    server = await setupCollaborationServer()
    user1 = await createTestUser('user1')
    user2 = await createTestUser('user2')
  })

  it('should sync block creation between users', async () => {
    const blockCreated = new Promise((resolve) => {
      user2.on('message', (data) => {
        const message = JSON.parse(data.toString())
        if (message.type === 'block_event' && message.data.type === 'block_created') {
          resolve(message.data)
        }
      })
    })

    // User 1 creates a block
    user1.send(JSON.stringify({
      type: 'block_event',
      data: {
        type: 'block_created',
        block_id: 'new-block-1',
        block: {
          id: 'new-block-1',
          text: 'New collaborative block',
          parent_id: null
        }
      }
    }))

    const createdEvent = await blockCreated
    expect(createdEvent.block.text).toBe('New collaborative block')
  })

  it('should handle concurrent edits with CRDT', async () => {
    // Both users edit the same block simultaneously
    const user1Edit = {
      type: 'block_event',
      data: {
        type: 'block_updated',
        block_id: 'shared-block',
        changes: { text: 'User 1 edit' }
      }
    }

    const user2Edit = {
      type: 'block_event', 
      data: {
        type: 'block_updated',
        block_id: 'shared-block',
        changes: { text: 'User 2 edit' }
      }
    }

    // Send edits simultaneously
    user1.send(JSON.stringify(user1Edit))
    user2.send(JSON.stringify(user2Edit))

    // Both users should receive both edits and converge
    await new Promise(resolve => setTimeout(resolve, 100))
    
    // Verify convergence by checking final state
    const finalState1 = await requestState(user1, 'shared-block')
    const finalState2 = await requestState(user2, 'shared-block')
    
    expect(finalState1).toEqual(finalState2)
  })
})
```

#### Testing Tasks
- [ ] Set up integration test environment
- [ ] Create test database fixtures
- [ ] Test API command flows
- [ ] Test real-time collaboration
- [ ] Test file system integration
- [ ] Test Git integration workflows
- [ ] Test error propagation
- [ ] Test performance under load

## 4. End-to-End Testing Strategy

### 4.1 User Workflow Tests

#### Planning
- [ ] Set up Playwright/Cypress testing
- [ ] Define critical user journeys
- [ ] Create page object models
- [ ] Set up visual regression testing

#### Implementation
```typescript
// tests/e2e/block-editing.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Block Editing Workflows', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
  })

  test('should create and edit blocks', async ({ page }) => {
    // Create new page
    await page.click('[data-testid="new-page-button"]')
    await page.fill('[data-testid="page-title"]', 'Test Page')

    // Create first block
    await page.click('[data-testid="create-first-block"]')
    const firstBlock = page.locator('[data-testid="block"]').first()
    
    // Type content
    await firstBlock.click()
    await page.keyboard.type('This is my first block')
    
    // Create second block with Enter
    await page.keyboard.press('Enter')
    const secondBlock = page.locator('[data-testid="block"]').nth(1)
    await page.keyboard.type('This is my second block')

    // Verify blocks were created
    expect(await firstBlock.textContent()).toContain('first block')
    expect(await secondBlock.textContent()).toContain('second block')
  })

  test('should handle block indentation', async ({ page }) => {
    // Set up initial blocks
    await setupTestBlocks(page)

    const block = page.locator('[data-testid="block"]').first()
    await block.click()
    
    // Indent with Tab
    await page.keyboard.press('Tab')
    
    // Verify indentation changed
    await expect(block).toHaveClass(/indented/)
    
    // Outdent with Shift+Tab
    await page.keyboard.press('Shift+Tab')
    
    // Verify indentation removed
    await expect(block).not.toHaveClass(/indented/)
  })

  test('should support keyboard navigation', async ({ page }) => {
    await setupTestBlocks(page, 3)

    const firstBlock = page.locator('[data-testid="block"]').first()
    await firstBlock.click()

    // Navigate down with arrow key
    await page.keyboard.press('ArrowDown')
    
    // Verify focus moved to second block
    const secondBlock = page.locator('[data-testid="block"]').nth(1)
    await expect(secondBlock).toBeFocused()

    // Navigate back up
    await page.keyboard.press('ArrowUp')
    await expect(firstBlock).toBeFocused()
  })

  test('should auto-save changes', async ({ page }) => {
    await page.click('[data-testid="new-page-button"]')
    await page.fill('[data-testid="page-title"]', 'Auto-save Test')

    const block = await createBlock(page, 'Content to auto-save')
    
    // Wait for auto-save (assuming 2 second interval)
    await page.waitForTimeout(3000)
    
    // Refresh page to verify persistence
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    // Verify content was saved
    const savedBlock = page.locator('[data-testid="block"]').first()
    expect(await savedBlock.textContent()).toContain('Content to auto-save')
  })
})

// User collaboration workflow
test.describe('Real-time Collaboration', () => {
  test('should show other users edits in real-time', async ({ browser }) => {
    // Create two browser contexts (simulating two users)
    const context1 = await browser.newContext()
    const context2 = await browser.newContext()
    
    const page1 = await context1.newPage()
    const page2 = await context2.newPage()
    
    // Both users navigate to same page
    await page1.goto('/')
    await page2.goto('/')
    
    // User 1 creates a block
    await page1.click('[data-testid="new-page-button"]')
    const pageUrl = page1.url()
    
    // User 2 navigates to same page
    await page2.goto(pageUrl)
    
    // User 1 creates content
    await createBlock(page1, 'Collaborative content')
    
    // User 2 should see the content appear
    await expect(page2.locator('[data-testid="block"]').first())
      .toContainText('Collaborative content')
    
    // User 2 edits the block
    await page2.locator('[data-testid="block"]').first().click()
    await page2.keyboard.press('End')
    await page2.keyboard.type(' - edited by user 2')
    
    // User 1 should see the edit
    await expect(page1.locator('[data-testid="block"]').first())
      .toContainText('edited by user 2')
  })
})
```

### 4.2 Cross-Platform Tests

#### Implementation
```typescript
// tests/e2e/cross-platform.spec.ts
import { test, expect, devices } from '@playwright/test'

const platforms = ['Desktop Chrome', 'Desktop Firefox', 'Desktop Safari']

for (const platform of platforms) {
  test.describe(`${platform} Tests`, () => {
    test.use({ ...devices[platform] })

    test('should work consistently across browsers', async ({ page }) => {
      await page.goto('/')
      
      // Test core functionality works the same
      await page.click('[data-testid="new-page-button"]')
      await createBlock(page, 'Cross-platform test')
      
      expect(await page.locator('[data-testid="block"]').first().textContent())
        .toContain('Cross-platform test')
    })

    test('should handle keyboard shortcuts consistently', async ({ page }) => {
      await page.goto('/')
      await createBlock(page, 'Test content')
      
      // Test common shortcuts work
      await page.keyboard.press('Ctrl+Z') // Undo
      await page.keyboard.press('Ctrl+Y') // Redo
      await page.keyboard.press('Ctrl+S') // Save
      
      // Verify no errors occurred
      const errors = await page.evaluate(() => window.errors || [])
      expect(errors).toHaveLength(0)
    })
  })
}
```

#### Testing Tasks
- [ ] Set up E2E testing framework
- [ ] Create user journey tests
- [ ] Test cross-browser compatibility
- [ ] Add visual regression tests
- [ ] Test accessibility compliance
- [ ] Test mobile responsiveness
- [ ] Test offline functionality
- [ ] Set up automated test runs

## 5. Performance Testing Strategy

### 5.1 Load Testing

#### Planning
- [ ] Define performance benchmarks
- [ ] Set up load testing tools
- [ ] Create realistic test scenarios
- [ ] Define performance metrics

#### Implementation
```typescript
// tests/performance/load-test.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Performance Tests', () => {
  test('should handle large documents efficiently', async ({ page }) => {
    await page.goto('/')
    
    // Create page with many blocks
    await page.click('[data-testid="new-page-button"]')
    
    const startTime = Date.now()
    
    // Create 1000 blocks
    for (let i = 0; i < 1000; i++) {
      await createBlock(page, `Block ${i} with some content`)
      
      // Check performance every 100 blocks
      if (i % 100 === 0) {
        const currentTime = Date.now()
        const elapsed = currentTime - startTime
        
        // Should not take more than 30 seconds for 1000 blocks
        expect(elapsed).toBeLessThan(30000)
        
        // Verify app is still responsive
        const blockCount = await page.locator('[data-testid="block"]').count()
        expect(blockCount).toBe(i + 1)
      }
    }
    
    // Test scrolling performance
    const scrollStartTime = Date.now()
    await page.keyboard.press('End') // Scroll to bottom
    const scrollEndTime = Date.now()
    
    expect(scrollEndTime - scrollStartTime).toBeLessThan(1000) // < 1 second
  })

  test('should handle rapid text input without lag', async ({ page }) => {
    await page.goto('/')
    const block = await createBlock(page, '')
    
    const startTime = Date.now()
    
    // Type rapidly
    const longText = 'A'.repeat(1000)
    await page.keyboard.type(longText, { delay: 0 })
    
    const endTime = Date.now()
    const typingTime = endTime - startTime
    
    // Should handle fast typing (< 2 seconds for 1000 chars)
    expect(typingTime).toBeLessThan(2000)
    
    // Verify all text was captured
    expect(await block.textContent()).toContain('A'.repeat(1000))
  })

  test('should search large datasets quickly', async ({ page }) => {
    await page.goto('/')
    
    // Create many blocks with searchable content
    for (let i = 0; i < 500; i++) {
      await createBlock(page, `Block ${i} with searchable content and unique id ${i}`)
    }
    
    // Test search performance
    const searchStartTime = Date.now()
    
    await page.fill('[data-testid="search-input"]', 'searchable')
    await page.keyboard.press('Enter')
    
    // Wait for search results
    await page.waitForSelector('[data-testid="search-results"]')
    
    const searchEndTime = Date.now()
    const searchTime = searchEndTime - searchStartTime
    
    // Search should complete quickly (< 500ms)
    expect(searchTime).toBeLessThan(500)
    
    // Verify results are relevant
    const resultCount = await page.locator('[data-testid="search-result"]').count()
    expect(resultCount).toBeGreaterThan(0)
  })
})
```

### 5.2 Memory and Resource Testing

#### Implementation
```rust
// tests/performance/memory_tests.rs
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_memory_usage_with_large_datasets() {
        let db = setup_test_database().await;
        
        // Measure initial memory usage
        let initial_memory = get_memory_usage();
        
        // Create 10,000 blocks
        for i in 0..10_000 {
            let block = create_test_block(&format!("block-{}", i));
            db.create_block(&block).await.unwrap();
        }
        
        // Measure memory after creating blocks
        let after_creation_memory = get_memory_usage();
        let memory_increase = after_creation_memory - initial_memory;
        
        // Memory increase should be reasonable (< 100MB for 10k blocks)
        assert!(memory_increase < 100 * 1024 * 1024);
        
        // Test retrieval performance
        let start_time = Instant::now();
        let blocks = db.get_blocks_by_page("test-page").await.unwrap();
        let retrieval_time = start_time.elapsed();
        
        assert_eq!(blocks.len(), 10_000);
        assert!(retrieval_time.as_millis() < 1000); // < 1 second
    }

    #[tokio::test]
    async fn test_crdt_performance_with_many_operations() {
        let mut engine = CRDTEngine::new("test-user".to_string());
        
        let start_time = Instant::now();
        
        // Apply 1000 operations
        for i in 0..1000 {
            let operation = engine.create_operation(
                OperationType::Insert,
                format!("block-{}", i),
                serde_json::json!({
                    "text": format!("Block {}", i)
                }),
            );
            
            engine.apply_local_operation(operation).unwrap();
        }
        
        let operations_time = start_time.elapsed();
        
        // Should handle 1000 operations quickly (< 100ms)
        assert!(operations_time.as_millis() < 100);
        
        // Test state retrieval performance
        let start_time = Instant::now();
        let state = engine.get_current_state();
        let state_time = start_time.elapsed();
        
        assert_eq!(state.len(), 1000);
        assert!(state_time.as_millis() < 50); // < 50ms
    }

    fn get_memory_usage() -> usize {
        // Platform-specific memory usage measurement
        #[cfg(target_os = "linux")]
        {
            let status = std::fs::read_to_string("/proc/self/status").unwrap();
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        
        0 // Fallback
    }
}
```

#### Testing Tasks
- [ ] Set up performance monitoring
- [ ] Create load testing scenarios
- [ ] Test memory usage patterns
- [ ] Benchmark database operations
- [ ] Test real-time sync performance
- [ ] Monitor resource usage
- [ ] Set up automated performance testing
- [ ] Create performance regression detection

## 6. Test Infrastructure

### 6.1 Continuous Integration

#### GitHub Actions Configuration
```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'pnpm'
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Run unit tests
        run: pnpm test:unit
      
      - name: Run component tests
        run: pnpm test:component
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Rust tests
        run: cd src-tauri && cargo test
      
      - name: Run integration tests
        run: cd src-tauri && cargo test --test integration_tests

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'pnpm'
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Install Playwright
        run: pnpm exec playwright install
      
      - name: Build app
        run: pnpm build
      
      - name: Run E2E tests
        run: pnpm test:e2e

  performance-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
      - uses: actions/checkout@v4
      - name: Run performance benchmarks
        run: pnpm test:performance
      
      - name: Comment performance results
        uses: actions/github-script@v6
        with:
          script: |
            // Post performance results as PR comment
```

### 6.2 Test Utilities and Helpers

#### Setup Functions
```typescript
// tests/utils/setup.ts
export async function setupTestApp() {
  // Initialize test database
  await invoke('init_test_database')
  
  // Clear any existing data
  await invoke('clear_test_data')
  
  // Set up test user
  await invoke('set_test_user', { userId: 'test-user-1' })
}

export async function createTestPage(title = 'Test Page') {
  return await invoke('create_page', {
    request: {
      title,
      status: 'draft'
    }
  })
}

export async function createTestBlock(text = 'Test block', parentId = null) {
  return await invoke('create_block', {
    request: {
      text,
      parent_id: parentId,
      block_type: 'text'
    }
  })
}

export async function setupTestBlocks(page: Page, count = 3) {
  await createTestPage()
  
  for (let i = 0; i < count; i++) {
    await createTestBlock(`Test block ${i + 1}`)
  }
}
```

#### Testing Tasks
- [ ] Set up CI/CD pipeline
- [ ] Create test data factories
- [ ] Set up test database utilities
- [ ] Create performance monitoring
- [ ] Set up test reporting
- [ ] Create test environment management
- [ ] Add test parallelization
- [ ] Set up test result notifications

## 7. Success Metrics

### 7.1 Coverage Targets
- [ ] Unit test coverage: >80%
- [ ] CRDT operations coverage: >95%
- [ ] Integration test coverage: >70%
- [ ] E2E critical path coverage: 100%

### 7.2 Performance Targets
- [ ] Unit tests run in <30 seconds
- [ ] Integration tests run in <5 minutes
- [ ] E2E tests run in <15 minutes
- [ ] Performance tests detect regressions >10%

### 7.3 Quality Metrics
- [ ] Test flakiness rate <5%
- [ ] Bug escape rate <1%
- [ ] Performance regression detection: 100%
- [ ] Critical bug detection time <24 hours 