<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { usePageStore } from '@/stores/pageStore'
import { useBlockStore, type Block } from '@/stores/blockStore'
import BlockEditor from './BlockEditor.vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

interface Props {
  pageId: string
  readonly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  readonly: false,
})

const pageStore = usePageStore()
const blockStore = useBlockStore()

// Component state
const isLoading = ref(true)
const showEmptyState = ref(false)
const draggedBlock = ref<Block | null>(null)

// Computed
const currentPage = computed(() => pageStore.findPageById(props.pageId))
const rootBlocks = computed(() => blockStore.getRootBlocks(props.pageId))
const hasBlocks = computed(() => rootBlocks.value.length > 0)
const isEditable = computed(() => !props.readonly)

// Create first block for empty pages
const createFirstBlock = async () => {
  if (props.readonly) return

  const firstBlock = await blockStore.createBlock({
    page_id: props.pageId,
    text: '',
    block_type: 'text',
    order: 0,
  })

  if (firstBlock) {
    blockStore.setFocusedBlock(firstBlock.id)
  }
}

// Recursively render block and its children
const renderBlock = (block: Block): Block[] => {
  const children = blockStore.getChildBlocks(block.id)
  const result = [block]
  
  for (const child of children) {
    result.push(...renderBlock(child))
  }
  
  return result
}

// Get all blocks in hierarchical order
const allBlocksInOrder = computed(() => {
  const result: Block[] = []
  
  for (const rootBlock of rootBlocks.value) {
    result.push(...renderBlock(rootBlock))
  }
  
  return result
})

// Handle block events
const handleBlockCreated = (block: Block) => {
  console.log('Block created:', block.id)
  // Block is already added to store, just ensure it's focused if needed
}

const handleBlockUpdated = (block: Block) => {
  console.log('Block updated:', block.id)
  // Block is already updated in store
}

const handleBlockDeleted = (blockId: string) => {
  console.log('Block deleted:', blockId)
  
  // If no blocks remain, show empty state
  if (allBlocksInOrder.value.length === 0) {
    showEmptyState.value = true
  }
}

const handleBlockFocused = (blockId: string) => {
  console.log('Block focused:', blockId)
  // Focus is already handled by the store
}

// Keyboard shortcuts at page level
const handleGlobalKeyDown = (event: KeyboardEvent) => {
  if (props.readonly) return

  const { key, ctrlKey, metaKey } = event
  const cmdKey = ctrlKey || metaKey

  // Global shortcuts
  if (cmdKey) {
    switch (key) {
      case 'a':
        // Select all blocks
        event.preventDefault()
        blockStore.clearSelection()
        allBlocksInOrder.value.forEach(block => {
          blockStore.addToSelection(block.id)
        })
        break

      case '/':
        // Quick block creation
        event.preventDefault()
        if (!blockStore.focusedBlockId) {
          createFirstBlock()
        }
        break
    }
  }
}

// Initialize the page
const initializePage = async () => {
  isLoading.value = true
  showEmptyState.value = false

  try {
    // Load page data if not already loaded
    if (!currentPage.value && props.pageId !== 'new') {
      await pageStore.getPage(props.pageId)
    }

    // Initialize blocks for this page
    await blockStore.initializePage(props.pageId)

    // Show empty state if no blocks exist
    if (!hasBlocks.value) {
      showEmptyState.value = true
    }
  } catch (error) {
    console.error('Failed to initialize page:', error)
  } finally {
    isLoading.value = false
  }
}

// Lifecycle
onMounted(async () => {
  await initializePage()
  
  // Add global keyboard listener
  document.addEventListener('keydown', handleGlobalKeyDown)
})

onUnmounted(() => {
  // Clean up global keyboard listener
  document.removeEventListener('keydown', handleGlobalKeyDown)
})

// Watch for page changes
watch(() => props.pageId, async (newPageId) => {
  if (newPageId) {
    await initializePage()
  }
})

// Watch for empty state changes
watch(hasBlocks, (newHasBlocks) => {
  if (newHasBlocks) {
    showEmptyState.value = false
  }
})

// Drag and drop functionality (basic implementation)
const handleDragStart = (event: DragEvent, block: Block) => {
  if (props.readonly) return
  
  draggedBlock.value = block
  event.dataTransfer?.setData('text/plain', block.id)
}

const handleDragOver = (event: DragEvent) => {
  event.preventDefault()
}

const handleDrop = async (event: DragEvent, targetBlock: Block) => {
  event.preventDefault()
  
  if (props.readonly || !draggedBlock.value) return

  const sourceBlock = draggedBlock.value
  draggedBlock.value = null

  // Simple reordering - move after target block
  if (sourceBlock.id !== targetBlock.id) {
    await blockStore.moveBlock(sourceBlock.id, {
      new_parent_id: targetBlock.parent_id,
      new_order: targetBlock.order + 1,
    })
  }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Loading state -->
    <div v-if="isLoading" class="flex-1 p-6 space-y-4">
      <Skeleton class="h-8 w-64" />
      <Skeleton class="h-4 w-full" />
      <Skeleton class="h-4 w-3/4" />
      <Skeleton class="h-4 w-1/2" />
    </div>

    <!-- Empty state -->
    <div v-else-if="showEmptyState" class="flex-1 flex items-center justify-center">
      <Card class="w-96">
        <CardContent class="text-center p-8">
          <div class="text-4xl mb-4">📝</div>
          <h3 class="text-lg font-medium text-foreground mb-2">
            Start Writing
          </h3>
          <p class="text-muted-foreground mb-6">
            Create your first block to begin taking notes. 
            <br>Press Enter to create new blocks.
          </p>
          <Button 
            v-if="isEditable"
            @click="createFirstBlock"
            class="w-full"
          >
            Create First Block
          </Button>
          <div v-else class="text-sm text-muted-foreground">
            This page is read-only
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Block editor content -->
    <div v-else class="flex-1 overflow-auto">
      <div class="max-w-4xl mx-auto p-6">
        <!-- Page title section -->
        <div class="mb-6">
          <h1 class="text-3xl font-bold text-foreground mb-2">
            {{ currentPage?.title || 'Untitled Page' }}
          </h1>
          <div class="text-sm text-muted-foreground">
            {{ allBlocksInOrder.length }} block{{ allBlocksInOrder.length !== 1 ? 's' : '' }}
            <span v-if="blockStore.selectedBlockIds.length > 0">
              • {{ blockStore.selectedBlockIds.length }} selected
            </span>
          </div>
        </div>

        <!-- Blocks container -->
        <div class="space-y-1">
          <div
            v-for="block in allBlocksInOrder"
            :key="block.id"
            :draggable="isEditable"
            class="block-container"
            @dragstart="handleDragStart($event, block)"
            @dragover="handleDragOver"
            @drop="handleDrop($event, block)"
          >
            <BlockEditor
              :page-id="pageId"
              :initial-block="block"
              :show-controls="true"
              :readonly="readonly"
              @block-created="handleBlockCreated"
              @block-updated="handleBlockUpdated"
              @block-deleted="handleBlockDeleted"
              @block-focused="handleBlockFocused"
            />
          </div>
        </div>

        <!-- Add block button (at bottom) -->
        <div v-if="isEditable" class="mt-6">
          <Button 
            variant="ghost" 
            class="w-full justify-start text-muted-foreground hover:text-foreground"
            @click="createFirstBlock"
          >
            <span class="mr-2">+</span>
            Add a block
          </Button>
        </div>

        <!-- Quick help -->
        <div v-if="isEditable && hasBlocks" class="mt-8 p-4 bg-muted rounded-lg">
          <h4 class="text-sm font-medium text-foreground mb-2">Keyboard Shortcuts</h4>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs text-muted-foreground">
            <div><kbd class="bg-background px-1 rounded">Enter</kbd> New block</div>
            <div><kbd class="bg-background px-1 rounded">Tab</kbd> Indent block</div>
            <div><kbd class="bg-background px-1 rounded">Shift+Tab</kbd> Unindent block</div>
            <div><kbd class="bg-background px-1 rounded">Backspace</kbd> Delete empty block</div>
            <div><kbd class="bg-background px-1 rounded">Cmd+1</kbd> Heading</div>
            <div><kbd class="bg-background px-1 rounded">Cmd+`</kbd> Code block</div>
            <div><kbd class="bg-background px-1 rounded">Cmd+Q</kbd> Quote block</div>
            <div><kbd class="bg-background px-1 rounded">↑↓</kbd> Navigate blocks</div>
          </div>
        </div>

        <!-- Error display -->
        <div v-if="blockStore.error" class="mt-4 p-4 bg-destructive/10 border border-destructive rounded-lg">
          <div class="flex items-center justify-between">
            <div class="text-sm text-destructive">
              {{ blockStore.error }}
            </div>
            <Button
              variant="ghost"
              size="sm"
              @click="blockStore.clearError"
            >
              ×
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.block-container {
  position: relative;
}

.block-container:hover {
  /* Smooth hover effects handled by BlockEditor */
}

kbd {
  font-family: inherit;
  font-size: 0.75rem;
  font-weight: 500;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  border: 1px solid hsl(var(--border));
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* Drag and drop visual feedback */
.block-container[draggable="true"]:hover {
  cursor: grab;
}

.block-container[draggable="true"]:active {
  cursor: grabbing;
}

/* Animation for block additions/removals */
.block-container {
  transition: opacity 0.2s ease-in-out, transform 0.2s ease-in-out;
}

/* Focus indicator for page level */
.page-editor:focus-within {
  /* Visual feedback when editor has focus */
}
</style> 