# Frontend Component Implementation

## 1. Component Architecture

### 1.1 Component Hierarchy

```
App.vue
├── Layout/
│   ├── AppHeader.vue
│   ├── AppSidebar.vue
│   ├── MainContent.vue
│   └── StatusBar.vue
├── Editor/
│   ├── BlockEditor.vue
│   ├── Block.vue
│   ├── BlockToolbar.vue
│   └── BlockSelector.vue
├── Sidebar/
│   ├── PageTree.vue
│   ├── SearchPanel.vue
│   ├── RecentPages.vue
│   └── TagBrowser.vue
├── Modal/
│   ├── SettingsModal.vue
│   ├── ExportModal.vue
│   └── ImportModal.vue
└── UI/
    ├── Button.vue
    ├── Input.vue
    ├── Modal.vue
    └── Toast.vue
```

## 2. Core Components Implementation

### 2.1 App.vue - Main Application Container

#### Planning
- [ ] Define main application layout structure
- [ ] Plan global error handling
- [ ] Design loading state management
- [ ] Specify keyboard shortcut system

#### Design
- [ ] Create application wireframe
- [ ] Define responsive breakpoints
- [ ] Plan theme switching system
- [ ] Design global navigation flow

#### Implementation
```typescript
// App.vue
<template>
  <div id="app" :class="[themeClass, 'h-screen flex flex-col']">
    <AppHeader />
    <div class="flex flex-1 overflow-hidden">
      <AppSidebar v-if="showSidebar" />
      <MainContent class="flex-1" />
    </div>
    <StatusBar />
    
    <!-- Global Modals -->
    <SettingsModal v-if="showSettings" @close="closeSettings" />
    <CommandPalette v-if="showCommandPalette" @close="closeCommandPalette" />
    
    <!-- Toast Notifications -->
    <ToastContainer />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'
import { useKeyboardStore } from '@/stores/keyboard'
import AppHeader from '@/components/layout/AppHeader.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import MainContent from '@/components/layout/MainContent.vue'
import StatusBar from '@/components/layout/StatusBar.vue'

const appStore = useAppStore()
const themeStore = useThemeStore()
const keyboardStore = useKeyboardStore()

const themeClass = computed(() => themeStore.currentTheme)
const showSidebar = computed(() => appStore.sidebarVisible)
const showSettings = computed(() => appStore.settingsModalVisible)
const showCommandPalette = computed(() => appStore.commandPaletteVisible)

onMounted(() => {
  // Initialize app
  appStore.initialize()
  keyboardStore.registerGlobalShortcuts()
})

function closeSettings() {
  appStore.hideSettingsModal()
}

function closeCommandPalette() {
  appStore.hideCommandPalette()
}
</script>
```

#### Tasks
- [ ] Set up Vue 3 composition API structure
- [ ] Implement global error boundary
- [ ] Add keyboard shortcut system
- [ ] Create theme switching logic
- [ ] Add loading states and transitions
- [ ] Implement toast notification system
- [ ] Add responsive layout handling

#### Testing
- [ ] Test theme switching functionality
- [ ] Validate keyboard shortcuts work
- [ ] Test responsive behavior
- [ ] Verify error handling

#### Validation
- [ ] App loads without errors
- [ ] Theme switching works correctly
- [ ] Keyboard shortcuts are responsive
- [ ] Layout is responsive on all screen sizes

### 2.2 BlockEditor.vue - Core Block Editing Component

#### Planning
- [ ] Define block editing interface
- [ ] Plan text editing functionality
- [ ] Design block manipulation system
- [ ] Specify undo/redo system

#### Design
- [ ] Create block editor wireframes
- [ ] Design text editing UX
- [ ] Plan block selection interface
- [ ] Design drag-and-drop system

#### Implementation
```typescript
// BlockEditor.vue
<template>
  <div 
    class="block-editor min-h-screen p-6 bg-white dark:bg-gray-900"
    @click="handleEditorClick"
  >
    <div class="max-w-4xl mx-auto">
      <!-- Page Title -->
      <input
        v-model="pageTitle"
        class="w-full text-3xl font-bold border-none outline-none bg-transparent mb-6"
        placeholder="Untitled"
        @input="updatePageTitle"
      />
      
      <!-- Block List -->
      <div class="space-y-2">
        <Block
          v-for="blockId in rootBlocks"
          :key="blockId"
          :block-id="blockId"
          :depth="0"
          @create-block="handleCreateBlock"
          @delete-block="handleDeleteBlock"
          @move-block="handleMoveBlock"
          @focus-block="handleFocusBlock"
        />
      </div>
      
      <!-- Add New Block Button -->
      <button
        v-if="rootBlocks.length === 0"
        class="w-full p-4 text-gray-400 border-2 border-dashed border-gray-300 rounded-lg hover:border-gray-400 transition-colors"
        @click="createFirstBlock"
      >
        Start writing...
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { usePageStore } from '@/stores/page'
import { useBlockStore } from '@/stores/block'
import { useSelectionStore } from '@/stores/selection'
import Block from './Block.vue'

interface Props {
  pageId: string
}

const props = defineProps<Props>()

const pageStore = usePageStore()
const blockStore = useBlockStore()
const selectionStore = useSelectionStore()

const pageTitle = ref('')

const currentPage = computed(() => pageStore.getPage(props.pageId))
const rootBlocks = computed(() => currentPage.value?.root_blocks || [])

onMounted(async () => {
  await loadPage()
})

async function loadPage() {
  await pageStore.loadPage(props.pageId)
  await blockStore.loadPageBlocks(props.pageId)
  pageTitle.value = currentPage.value?.title || ''
}

async function updatePageTitle() {
  if (currentPage.value) {
    await pageStore.updatePage(props.pageId, { title: pageTitle.value })
  }
}

async function handleCreateBlock(parentId?: string, position?: number) {
  const blockId = await blockStore.createBlock({
    text: '',
    parent_id: parentId,
    position: position || 0
  })
  
  await nextTick()
  // Focus the new block
  selectionStore.setFocus(blockId)
}

async function handleDeleteBlock(blockId: string) {
  await blockStore.deleteBlock(blockId)
}

async function handleMoveBlock(blockId: string, newParentId?: string, newPosition?: number) {
  await blockStore.moveBlock(blockId, newParentId, newPosition || 0)
}

function handleFocusBlock(blockId: string) {
  selectionStore.setFocus(blockId)
}

function handleEditorClick(event: MouseEvent) {
  // If clicking on empty space, clear selection
  if (event.target === event.currentTarget) {
    selectionStore.clearSelection()
  }
}

async function createFirstBlock() {
  await handleCreateBlock()
}
</script>
```

#### Tasks
- [ ] Implement block rendering system
- [ ] Add text editing functionality
- [ ] Create block selection system
- [ ] Implement drag-and-drop for blocks
- [ ] Add keyboard navigation
- [ ] Implement undo/redo system
- [ ] Add auto-save functionality
- [ ] Create block manipulation toolbar

#### Testing
- [ ] Test block creation and deletion
- [ ] Validate text editing functionality
- [ ] Test block selection and navigation
- [ ] Verify drag-and-drop works
- [ ] Test undo/redo functionality

#### Validation
- [ ] Users can create and edit blocks
- [ ] Block operations work smoothly
- [ ] Keyboard navigation is intuitive
- [ ] Performance is acceptable with many blocks

### 2.3 Block.vue - Individual Block Component

#### Planning
- [ ] Define block rendering logic
- [ ] Plan text editing interface
- [ ] Design block type handling
- [ ] Specify interaction behaviors

#### Design
- [ ] Create block component wireframes
- [ ] Design text editing interface
- [ ] Plan block type indicators
- [ ] Design hover and focus states

#### Implementation
```typescript
// Block.vue
<template>
  <div
    :class="[
      'block group relative',
      { 'block-focused': isFocused },
      { 'block-selected': isSelected }
    ]"
    :style="{ paddingLeft: `${depth * 24}px` }"
  >
    <!-- Block Drag Handle -->
    <div
      class="absolute left-0 top-0 w-6 h-6 opacity-0 group-hover:opacity-100 cursor-move transition-opacity"
      @mousedown="startDrag"
    >
      <DragIcon class="w-4 h-4 text-gray-400" />
    </div>
    
    <!-- Block Content -->
    <div class="flex items-start space-x-2 pl-6">
      <!-- Block Type Indicator -->
      <div class="flex-shrink-0 mt-2">
        <BlockTypeIndicator :type="block.block_type" />
      </div>
      
      <!-- Editable Content -->
      <div
        ref="contentRef"
        class="flex-1 min-h-[24px] outline-none"
        contenteditable
        :data-placeholder="placeholder"
        @input="handleInput"
        @keydown="handleKeydown"
        @focus="handleFocus"
        @blur="handleBlur"
        v-html="formattedContent"
      />
      
      <!-- Block Actions -->
      <div class="flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <BlockActions
          :block="block"
          @delete="$emit('delete-block', block.id)"
          @duplicate="duplicateBlock"
          @convert="convertBlock"
        />
      </div>
    </div>
    
    <!-- Child Blocks -->
    <div v-if="block.children.length > 0" class="mt-1">
      <Block
        v-for="childId in block.children"
        :key="childId"
        :block-id="childId"
        :depth="depth + 1"
        @create-block="$emit('create-block', $event)"
        @delete-block="$emit('delete-block', $event)"
        @move-block="$emit('move-block', $event)"
        @focus-block="$emit('focus-block', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useBlockStore } from '@/stores/block'
import { useSelectionStore } from '@/stores/selection'
import { parseMarkdown, formatText } from '@/utils/markdown'
import BlockTypeIndicator from './BlockTypeIndicator.vue'
import BlockActions from './BlockActions.vue'
import DragIcon from '@/components/ui/icons/DragIcon.vue'

interface Props {
  blockId: string
  depth: number
}

interface Emits {
  'create-block': [parentId?: string, position?: number]
  'delete-block': [blockId: string]
  'move-block': [blockId: string, newParentId?: string, newPosition?: number]
  'focus-block': [blockId: string]
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const blockStore = useBlockStore()
const selectionStore = useSelectionStore()

const contentRef = ref<HTMLElement>()

const block = computed(() => blockStore.getBlock(props.blockId))
const isFocused = computed(() => selectionStore.focusedBlock === props.blockId)
const isSelected = computed(() => selectionStore.selectedBlocks.includes(props.blockId))
const formattedContent = computed(() => formatText(block.value?.text || ''))
const placeholder = computed(() => getPlaceholderText(block.value?.block_type))

watch(isFocused, (focused) => {
  if (focused && contentRef.value) {
    nextTick(() => {
      contentRef.value?.focus()
      // Restore cursor position if needed
    })
  }
})

function handleInput(event: Event) {
  const target = event.target as HTMLElement
  const text = target.textContent || ''
  
  if (block.value) {
    blockStore.updateBlock(props.blockId, { text })
  }
}

function handleKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case 'Enter':
      if (event.shiftKey) {
        // Insert line break
        return
      }
      event.preventDefault()
      createNewBlock()
      break
      
    case 'Backspace':
      if (isAtStart() && block.value?.text === '') {
        event.preventDefault()
        deleteCurrentBlock()
      }
      break
      
    case 'ArrowUp':
      if (isAtStart()) {
        event.preventDefault()
        focusPreviousBlock()
      }
      break
      
    case 'ArrowDown':
      if (isAtEnd()) {
        event.preventDefault()
        focusNextBlock()
      }
      break
      
    case 'Tab':
      event.preventDefault()
      if (event.shiftKey) {
        outdentBlock()
      } else {
        indentBlock()
      }
      break
  }
}

function handleFocus() {
  emit('focus-block', props.blockId)
}

function handleBlur() {
  // Save any pending changes
  if (block.value) {
    blockStore.saveBlock(props.blockId)
  }
}

function createNewBlock() {
  const position = getNextPosition()
  emit('create-block', block.value?.parent_id, position)
}

function deleteCurrentBlock() {
  emit('delete-block', props.blockId)
}

function duplicateBlock() {
  if (block.value) {
    blockStore.duplicateBlock(props.blockId)
  }
}

function convertBlock(newType: string) {
  if (block.value) {
    blockStore.updateBlock(props.blockId, { block_type: newType })
  }
}

function indentBlock() {
  // Move block to be child of previous sibling
  const previousSibling = getPreviousSibling()
  if (previousSibling) {
    emit('move-block', props.blockId, previousSibling.id)
  }
}

function outdentBlock() {
  // Move block to be sibling of parent
  if (block.value?.parent_id) {
    const parent = blockStore.getBlock(block.value.parent_id)
    if (parent) {
      emit('move-block', props.blockId, parent.parent_id)
    }
  }
}

// Helper functions
function isAtStart(): boolean {
  const selection = window.getSelection()
  return selection?.focusOffset === 0
}

function isAtEnd(): boolean {
  const selection = window.getSelection()
  const content = contentRef.value?.textContent || ''
  return selection?.focusOffset === content.length
}

function getNextPosition(): number {
  // Calculate position for new block
  return (block.value?.order || 0) + 1
}

function getPreviousSibling() {
  // Find previous sibling block
  const parent = block.value?.parent_id ? blockStore.getBlock(block.value.parent_id) : null
  const siblings = parent?.children || []
  const currentIndex = siblings.indexOf(props.blockId)
  return currentIndex > 0 ? blockStore.getBlock(siblings[currentIndex - 1]) : null
}

function focusPreviousBlock() {
  // Focus the previous block in document order
  const prevBlock = blockStore.getPreviousBlock(props.blockId)
  if (prevBlock) {
    emit('focus-block', prevBlock.id)
  }
}

function focusNextBlock() {
  // Focus the next block in document order
  const nextBlock = blockStore.getNextBlock(props.blockId)
  if (nextBlock) {
    emit('focus-block', nextBlock.id)
  }
}

function getPlaceholderText(blockType?: string): string {
  switch (blockType) {
    case 'heading': return 'Heading'
    case 'code': return 'Code'
    case 'quote': return 'Quote'
    default: return 'Type something...'
  }
}

function startDrag(event: MouseEvent) {
  // Implement drag functionality
  event.preventDefault()
  // TODO: Implement drag and drop
}
</script>

<style scoped>
.block-focused {
  @apply ring-2 ring-blue-500 ring-opacity-50;
}

.block-selected {
  @apply bg-blue-50 dark:bg-blue-900;
}

[contenteditable]:empty:before {
  content: attr(data-placeholder);
  @apply text-gray-400 pointer-events-none;
}
</style>
```

#### Tasks
- [ ] Implement contenteditable text editing
- [ ] Add keyboard navigation (arrow keys, tab)
- [ ] Create block type conversion system
- [ ] Implement drag-and-drop functionality
- [ ] Add block selection system
- [ ] Create block toolbar with actions
- [ ] Implement markdown parsing and formatting
- [ ] Add auto-save on changes

#### Testing
- [ ] Test text editing functionality
- [ ] Validate keyboard navigation
- [ ] Test block type conversions
- [ ] Verify drag-and-drop works
- [ ] Test selection system

#### Validation
- [ ] Text editing is smooth and responsive
- [ ] Keyboard navigation works intuitively
- [ ] Block operations don't cause data loss
- [ ] Performance is good with nested blocks

### 2.4 AppSidebar.vue - Navigation Sidebar

#### Planning
- [ ] Define sidebar layout structure
- [ ] Plan page tree navigation
- [ ] Design search functionality
- [ ] Specify recent pages system

#### Design
- [ ] Create sidebar wireframes
- [ ] Design page tree interface
- [ ] Plan search UI
- [ ] Design collapsible sections

#### Implementation
```typescript
// AppSidebar.vue
<template>
  <div class="w-64 bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col h-full">
    <!-- Sidebar Header -->
    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
          Notes
        </h2>
        <button
          class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
          @click="createNewPage"
        >
          <PlusIcon class="w-5 h-5" />
        </button>
      </div>
    </div>
    
    <!-- Search -->
    <div class="p-4">
      <SearchInput
        v-model="searchQuery"
        placeholder="Search notes..."
        @search="handleSearch"
      />
    </div>
    
    <!-- Sidebar Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- Recent Pages -->
      <SidebarSection title="Recent" :collapsible="true" v-model:collapsed="recentCollapsed">
        <RecentPages :pages="recentPages" @select="selectPage" />
      </SidebarSection>
      
      <!-- All Pages -->
      <SidebarSection title="All Pages" :collapsible="true" v-model:collapsed="pagesCollapsed">
        <PageTree :pages="allPages" @select="selectPage" @rename="renamePage" />
      </SidebarSection>
      
      <!-- Tags -->
      <SidebarSection title="Tags" :collapsible="true" v-model:collapsed="tagsCollapsed">
        <TagBrowser :tags="allTags" @select="selectTag" />
      </SidebarSection>
    </div>
    
    <!-- Sidebar Footer -->
    <div class="p-4 border-t border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between text-sm text-gray-500">
        <span>{{ pageCount }} notes</span>
        <button @click="openSettings" class="hover:text-gray-700">
          Settings
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePageStore } from '@/stores/page'
import { useSearchStore } from '@/stores/search'
import { useTagStore } from '@/stores/tag'
import { useAppStore } from '@/stores/app'
import SearchInput from '@/components/ui/SearchInput.vue'
import SidebarSection from './SidebarSection.vue'
import RecentPages from './RecentPages.vue'
import PageTree from './PageTree.vue'
import TagBrowser from './TagBrowser.vue'
import PlusIcon from '@/components/ui/icons/PlusIcon.vue'

const pageStore = usePageStore()
const searchStore = useSearchStore()
const tagStore = useTagStore()
const appStore = useAppStore()

const searchQuery = ref('')
const recentCollapsed = ref(false)
const pagesCollapsed = ref(false)
const tagsCollapsed = ref(false)

const allPages = computed(() => pageStore.getAllPages())
const recentPages = computed(() => pageStore.getRecentPages())
const allTags = computed(() => tagStore.getAllTags())
const pageCount = computed(() => allPages.value.length)

async function createNewPage() {
  const page = await pageStore.createPage({
    title: 'Untitled',
    status: 'draft'
  })
  selectPage(page.id)
}

function selectPage(pageId: string) {
  appStore.setCurrentPage(pageId)
}

function selectTag(tag: string) {
  searchStore.setTagFilter(tag)
}

async function renamePage(pageId: string, newTitle: string) {
  await pageStore.updatePage(pageId, { title: newTitle })
}

async function handleSearch(query: string) {
  if (query.trim()) {
    const results = await searchStore.searchBlocks(query)
    appStore.showSearchResults(results)
  } else {
    appStore.hideSearchResults()
  }
}

function openSettings() {
  appStore.showSettingsModal()
}
</script>
```

#### Tasks
- [ ] Implement page tree navigation
- [ ] Add search functionality
- [ ] Create recent pages list
- [ ] Implement tag browser
- [ ] Add page creation and management
- [ ] Create collapsible sections
- [ ] Add context menus for pages
- [ ] Implement drag-and-drop for page organization

#### Testing
- [ ] Test page navigation
- [ ] Validate search functionality
- [ ] Test page creation and deletion
- [ ] Verify tag filtering works

#### Validation
- [ ] Navigation is intuitive and fast
- [ ] Search returns relevant results
- [ ] Page management operations work correctly
- [ ] UI is responsive and accessible

## 3. Pinia Stores Implementation

### 3.1 Block Store

#### Planning
- [ ] Define block state management
- [ ] Plan CRUD operations
- [ ] Design caching strategy
- [ ] Specify sync operations

#### Implementation
```typescript
// stores/block.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Block, CreateBlockData, UpdateBlockData } from '@/types'
import * as blockApi from '@/api/blocks'

export const useBlockStore = defineStore('blocks', () => {
  // State
  const blocks = ref(new Map<string, Block>())
  const loadedPages = ref(new Set<string>())
  const pendingChanges = ref(new Map<string, UpdateBlockData>())
  
  // Getters
  const getBlock = computed(() => (id: string): Block | undefined => {
    return blocks.value.get(id)
  })
  
  const getBlocksByPage = computed(() => (pageId: string): Block[] => {
    return Array.from(blocks.value.values())
      .filter(block => block.page_id === pageId)
      .sort((a, b) => a.order - b.order)
  })
  
  const getRootBlocks = computed(() => (pageId: string): Block[] => {
    return Array.from(blocks.value.values())
      .filter(block => block.page_id === pageId && !block.parent_id)
      .sort((a, b) => a.order - b.order)
  })
  
  // Actions
  async function loadPageBlocks(pageId: string) {
    if (loadedPages.value.has(pageId)) return
    
    try {
      const pageBlocks = await blockApi.getBlocksByPage(pageId)
      pageBlocks.forEach(block => {
        blocks.value.set(block.id, block)
      })
      loadedPages.value.add(pageId)
    } catch (error) {
      console.error('Failed to load page blocks:', error)
      throw error
    }
  }
  
  async function createBlock(data: CreateBlockData): Promise<Block> {
    try {
      const block = await blockApi.createBlock(data)
      blocks.value.set(block.id, block)
      return block
    } catch (error) {
      console.error('Failed to create block:', error)
      throw error
    }
  }
  
  async function updateBlock(id: string, data: UpdateBlockData): Promise<Block> {
    // Optimistic update
    const currentBlock = blocks.value.get(id)
    if (currentBlock) {
      const updatedBlock = { ...currentBlock, ...data, updated_at: new Date() }
      blocks.value.set(id, updatedBlock)
      
      // Queue for backend sync
      pendingChanges.value.set(id, data)
      await syncPendingChanges()
      
      return updatedBlock
    }
    throw new Error(`Block ${id} not found`)
  }
  
  async function deleteBlock(id: string): Promise<void> {
    try {
      await blockApi.deleteBlock(id)
      blocks.value.delete(id)
      pendingChanges.value.delete(id)
    } catch (error) {
      console.error('Failed to delete block:', error)
      throw error
    }
  }
  
  async function moveBlock(id: string, newParentId?: string, newPosition?: number): Promise<void> {
    try {
      await blockApi.moveBlock(id, newParentId, newPosition)
      // Reload affected blocks
      const block = blocks.value.get(id)
      if (block?.page_id) {
        await loadPageBlocks(block.page_id)
      }
    } catch (error) {
      console.error('Failed to move block:', error)
      throw error
    }
  }
  
  async function syncPendingChanges() {
    const changes = Array.from(pendingChanges.value.entries())
    pendingChanges.value.clear()
    
    for (const [blockId, data] of changes) {
      try {
        const updatedBlock = await blockApi.updateBlock(blockId, data)
        blocks.value.set(blockId, updatedBlock)
      } catch (error) {
        console.error('Failed to sync block changes:', error)
        // Re-queue the change for retry
        pendingChanges.value.set(blockId, data)
      }
    }
  }
  
  // Collaboration methods
  function applyRemoteEvent(event: BlockEvent) {
    switch (event.type) {
      case 'block_created':
        blocks.value.set(event.block_id, event.data.block)
        break
      case 'block_updated':
        const block = blocks.value.get(event.block_id)
        if (block) {
          blocks.value.set(event.block_id, { ...block, ...event.data })
        }
        break
      case 'block_deleted':
        blocks.value.delete(event.block_id)
        break
    }
  }
  
  return {
    // State
    blocks: computed(() => blocks.value),
    
    // Getters
    getBlock,
    getBlocksByPage,
    getRootBlocks,
    
    // Actions
    loadPageBlocks,
    createBlock,
    updateBlock,
    deleteBlock,
    moveBlock,
    syncPendingChanges,
    applyRemoteEvent
  }
})
```

#### Tasks
- [ ] Implement reactive block state management
- [ ] Add optimistic updates for better UX
- [ ] Create block CRUD operations
- [ ] Implement caching and lazy loading
- [ ] Add collaboration event handling
- [ ] Create pending changes queue
- [ ] Add error handling and retry logic
- [ ] Implement undo/redo system

#### Testing
- [ ] Test block CRUD operations
- [ ] Validate optimistic updates
- [ ] Test collaboration events
- [ ] Verify error handling

#### Validation
- [ ] Block operations are fast and responsive
- [ ] Data consistency is maintained
- [ ] Collaboration works seamlessly
- [ ] Error states are handled gracefully

## 4. Service Layer Implementation

### 4.1 Markdown Service

#### Planning
- [ ] Define markdown parsing requirements
- [ ] Plan syntax highlighting
- [ ] Design link detection system
- [ ] Specify export formatting

#### Implementation
```typescript
// services/markdown.ts
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'

class MarkdownService {
  private md: MarkdownIt
  
  constructor() {
    this.md = new MarkdownIt({
      html: true,
      linkify: true,
      typographer: true,
      highlight: (str, lang) => {
        if (lang && hljs.getLanguage(lang)) {
          try {
            return hljs.highlight(str, { language: lang }).value
          } catch (err) {
            console.error('Syntax highlighting error:', err)
          }
        }
        return ''
      }
    })
    
    this.setupCustomRules()
  }
  
  private setupCustomRules() {
    // Block references: [[block-id]]
    this.md.renderer.rules.block_reference = (tokens, idx) => {
      const token = tokens[idx]
      const blockId = token.content
      return `<span class="block-reference" data-block-id="${blockId}">[[${blockId}]]</span>`
    }
    
    // Page links: [[Page Title]]
    this.md.renderer.rules.page_link = (tokens, idx) => {
      const token = tokens[idx]
      const pageTitle = token.content
      return `<a class="page-link" href="#/page/${encodeURIComponent(pageTitle)}">${pageTitle}</a>`
    }
  }
  
  parse(content: string): string {
    return this.md.render(content)
  }
  
  parseInline(content: string): string {
    return this.md.renderInline(content)
  }
  
  extractReferences(content: string): string[] {
    const references: string[] = []
    const referenceRegex = /\[\[([^\]]+)\]\]/g
    let match
    
    while ((match = referenceRegex.exec(content)) !== null) {
      references.push(match[1])
    }
    
    return references
  }
  
  convertToMarkdown(blocks: Block[]): string {
    let markdown = ''
    
    for (const block of blocks) {
      const indent = '  '.repeat(block.depth || 0)
      
      switch (block.block_type) {
        case 'heading':
          const level = block.properties?.level || 1
          markdown += `${'#'.repeat(level)} ${block.text}\n\n`
          break
        case 'list_item':
          markdown += `${indent}- ${block.text}\n`
          break
        case 'code':
          const lang = block.properties?.language || ''
          markdown += `\`\`\`${lang}\n${block.text}\n\`\`\`\n\n`
          break
        case 'quote':
          markdown += `> ${block.text}\n\n`
          break
        default:
          markdown += `${indent}${block.text}\n\n`
      }
    }
    
    return markdown.trim()
  }
}

export const markdownService = new MarkdownService()
```

#### Tasks
- [ ] Implement markdown parsing with MarkdownIt
- [ ] Add syntax highlighting support
- [ ] Create custom rules for block references
- [ ] Implement page link detection
- [ ] Add export to markdown functionality
- [ ] Create import from markdown
- [ ] Add support for custom block types
- [ ] Implement real-time preview

#### Testing
- [ ] Test markdown parsing accuracy
- [ ] Validate syntax highlighting
- [ ] Test reference extraction
- [ ] Verify export/import functionality

#### Validation
- [ ] Markdown renders correctly
- [ ] Syntax highlighting works properly
- [ ] References are detected and linked
- [ ] Export/import preserves formatting

## 5. Testing Strategy for Frontend

### 5.1 Unit Tests
- [ ] Component rendering tests
- [ ] Store action tests  
- [ ] Service function tests
- [ ] Utility function tests

### 5.2 Integration Tests
- [ ] Component interaction tests
- [ ] Store integration tests
- [ ] API integration tests
- [ ] Navigation flow tests

### 5.3 E2E Tests
- [ ] Complete user workflows
- [ ] Cross-browser compatibility
- [ ] Performance testing
- [ ] Accessibility testing

### 5.4 Performance Tests
- [ ] Block rendering performance
- [ ] Large document handling
- [ ] Memory usage monitoring
- [ ] Bundle size optimization 