<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch } from 'vue'
import { useBlockStore, type Block, type BlockType } from '@/stores/blockStore'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface Props {
  pageId: string
  initialBlock?: Block
  showControls?: boolean
  readonly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showControls: true,
  readonly: false,
})

const emit = defineEmits<{
  'block-created': [block: Block]
  'block-updated': [block: Block]
  'block-deleted': [blockId: string]
  'block-focused': [blockId: string]
}>()

const blockStore = useBlockStore()
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const blockRef = ref<HTMLDivElement | null>(null)
const isCreating = ref(!props.initialBlock)
const blockText = ref(props.initialBlock?.text || '')
const blockType = ref<BlockType>(props.initialBlock?.block_type || 'text')
const isComposing = ref(false)

// Computed
const block = computed(() => props.initialBlock)
const isEditing = computed(() => 
  block.value ? blockStore.focusedBlockId === block.value.id : isCreating.value
)
const depth = computed(() => block.value ? blockStore.getBlockDepth(block.value.id) : 0)
const isDevelopment = computed(() => import.meta.env.DEV)
const placeholder = computed(() => {
  switch (blockType.value) {
    case 'heading': return 'Heading'
    case 'code': return '// Code block'
    case 'quote': return 'Quote'
    case 'list_item': return '• List item'
    default: return 'Type something...'
  }
})

// Auto-resize textarea
const autoResize = () => {
  if (textareaRef.value) {
    textareaRef.value.style.height = 'auto'
    textareaRef.value.style.height = textareaRef.value.scrollHeight + 'px'
  }
}

// Save block content
const saveBlock = async () => {
  if (props.readonly) return

  const text = blockText.value.trim()
  
  if (isCreating.value && block.value) {
    // Update existing block during creation flow
    const updated = await blockStore.updateBlock(block.value.id, {
      text,
      block_type: blockType.value,
    })
    if (updated) {
      emit('block-updated', updated)
      isCreating.value = false
    }
  } else if (block.value) {
    // Update existing block
    const updated = await blockStore.updateBlock(block.value.id, { 
      text,
      block_type: blockType.value,
    })
    if (updated) {
      emit('block-updated', updated)
    }
  }
}

// Create a new block
const createNewBlock = async (atOrder?: number, parentId?: string): Promise<Block | null> => {
  if (props.readonly) return null

  const newBlock = await blockStore.createBlock({
    page_id: props.pageId,
    text: '',
    parent_id: parentId,
    order: atOrder ?? 0,
    block_type: 'text',
  })

  if (newBlock) {
    emit('block-created', newBlock)
    return newBlock
  }
  return null
}

// Delete current block
const deleteCurrentBlock = async () => {
  if (props.readonly || !block.value) return

  const blockId = block.value.id
  const success = await blockStore.deleteBlock(blockId)
  if (success) {
    emit('block-deleted', blockId)
  }
}

// Focus management
const focusBlock = async () => {
  if (block.value) {
    blockStore.setFocusedBlock(block.value.id)
    emit('block-focused', block.value.id)
  }
  
  await nextTick()
  if (textareaRef.value) {
    textareaRef.value.focus()
    autoResize()
  }
}

// Keyboard navigation and shortcuts
const handleKeyDown = async (event: KeyboardEvent) => {
  if (isComposing.value || props.readonly) return

  const { key, shiftKey, ctrlKey, metaKey } = event
  const cmdKey = ctrlKey || metaKey

  // Save on Ctrl+S
  if (cmdKey && key === 's') {
    event.preventDefault()
    await saveBlock()
    return
  }

  // Block type shortcuts
  if (cmdKey && !shiftKey) {
    switch (key) {
      case '1':
      case '2':
      case '3':
        event.preventDefault()
        blockType.value = 'heading'
        await saveBlock()
        return
      case '`':
        event.preventDefault()
        blockType.value = 'code'
        await saveBlock()
        return
      case 'q':
        event.preventDefault()
        blockType.value = 'quote'
        await saveBlock()
        return
    }
  }

  // Handle Enter key
  if (key === 'Enter' && !shiftKey) {
    event.preventDefault()
    
    const textarea = textareaRef.value!
    const cursorPos = textarea.selectionStart
    const text = blockText.value
    
    // Split text at cursor position
    const beforeCursor = text.slice(0, cursorPos)
    const afterCursor = text.slice(cursorPos)
    
    // Update current block with text before cursor
    blockText.value = beforeCursor
    await saveBlock()
    
    // Create new block with text after cursor
    const currentOrder = block.value?.order ?? 0
    const newBlock = await createNewBlock(currentOrder + 1, block.value?.parent_id)
    
    if (newBlock && afterCursor) {
      await blockStore.updateBlock(newBlock.id, { text: afterCursor })
    }
    
    // Focus the new block
    if (newBlock) {
      await nextTick()
      blockStore.setFocusedBlock(newBlock.id)
    }
    return
  }

  // Handle Tab for indentation
  if (key === 'Tab') {
    event.preventDefault()
    
    if (!block.value) return
    
    if (shiftKey) {
      // Shift+Tab: Unindent (move to parent's level)
      const parent = blockStore.getParentBlock(block.value.id)
      if (parent) {
        const grandParent = blockStore.getParentBlock(parent.id)
        await blockStore.moveBlock(block.value.id, {
          new_parent_id: grandParent?.id,
          new_order: parent.order + 1,
        })
      }
    } else {
      // Tab: Indent (make child of previous sibling)
      const prevSibling = blockStore.getPreviousSibling(block.value.id)
      if (prevSibling) {
        const newOrder = blockStore.getChildBlocks(prevSibling.id).length
        await blockStore.moveBlock(block.value.id, {
          new_parent_id: prevSibling.id,
          new_order: newOrder,
        })
      }
    }
    return
  }

  // Handle Backspace at beginning
  if (key === 'Backspace' && blockText.value === '' && block.value) {
    event.preventDefault()
    
    const prevSibling = blockStore.getPreviousSibling(block.value.id)
    const parent = blockStore.getParentBlock(block.value.id)
    
    // Focus previous block or parent
    const targetBlock = prevSibling || parent
    if (targetBlock) {
      blockStore.setFocusedBlock(targetBlock.id)
    }
    
    // Delete current empty block
    await deleteCurrentBlock()
    return
  }

  // Arrow key navigation
  if (key === 'ArrowUp' && textareaRef.value?.selectionStart === 0) {
    event.preventDefault()
    const prevSibling = block.value && blockStore.getPreviousSibling(block.value.id)
    if (prevSibling) {
      blockStore.setFocusedBlock(prevSibling.id)
    }
    return
  }

  if (key === 'ArrowDown' && textareaRef.value?.selectionStart === blockText.value.length) {
    event.preventDefault()
    const nextSibling = block.value && blockStore.getNextSibling(block.value.id)
    if (nextSibling) {
      blockStore.setFocusedBlock(nextSibling.id)
    }
    return
  }
}

// Handle input changes
const handleInput = () => {
  autoResize()
}

// Handle blur (save changes)
const handleBlur = async () => {
  if (!props.readonly) {
    await saveBlock()
  }
}

// Block type styling
const getBlockTypeClasses = () => {
  const baseClasses = "w-full bg-transparent border-none outline-none resize-none font-sans placeholder:text-muted-foreground"
  
  switch (blockType.value) {
    case 'heading':
      return cn(baseClasses, "text-2xl font-bold text-foreground")
    case 'code':
      return cn(baseClasses, "font-mono text-sm bg-muted rounded p-2 border")
    case 'quote':
      return cn(baseClasses, "italic text-muted-foreground border-l-4 border-border pl-4")
    case 'list_item':
      return cn(baseClasses, "text-foreground")
    default:
      return cn(baseClasses, "text-foreground")
  }
}

const getBlockContainerClasses = () => {
  const baseClasses = "relative group"
  const isActive = isEditing.value
  const indentLevel = Math.min(depth.value, 6) // Max 6 levels
  
  return cn(
    baseClasses,
    `ml-${indentLevel * 4}`, // Indent based on depth
    isActive && "ring-2 ring-ring ring-offset-2 rounded-md",
    "hover:bg-accent/50 rounded-md transition-colors"
  )
}

// Handle composition events (for IME)
const handleCompositionStart = () => {
  isComposing.value = true
}

const handleCompositionEnd = () => {
  isComposing.value = false
}

// Handle click to focus
const handleClick = () => {
  if (!props.readonly) {
    focusBlock()
  }
}

// Lifecycle
onMounted(async () => {
  if (isCreating.value || isEditing.value) {
    await focusBlock()
  }
  
  // Watch for focus changes
  watch(() => blockStore.focusedBlockId, (newFocusId) => {
    if (newFocusId === block.value?.id) {
      focusBlock()
    }
  })
})

// Auto-resize on text changes
watch(blockText, () => {
  nextTick(() => {
    autoResize()
  })
})

// Watch for external block updates
watch(() => block.value?.text, (newText) => {
  if (newText !== undefined && newText !== blockText.value) {
    blockText.value = newText
  }
})

watch(() => block.value?.block_type, (newType) => {
  if (newType && newType !== blockType.value) {
    blockType.value = newType
  }
})
</script>

<template>
  <div 
    ref="blockRef"
    :class="getBlockContainerClasses()"
    @click="handleClick"
  >
    <!-- Block content -->
    <div class="flex items-start space-x-2">
      <!-- Drag handle (visible on hover) -->
      <div class="opacity-0 group-hover:opacity-100 transition-opacity mt-1 flex items-center space-x-1">
        <div class="w-1 h-4 bg-muted-foreground rounded cursor-grab"></div>
        <div class="w-1 h-4 bg-muted-foreground rounded cursor-grab"></div>
      </div>

      <!-- Block type indicator -->
      <div class="mt-1 flex-shrink-0 w-4 text-xs text-muted-foreground">
        <span v-if="blockType === 'heading'">#</span>
        <span v-else-if="blockType === 'code'">&lt;/&gt;</span>
        <span v-else-if="blockType === 'quote'">❝</span>
        <span v-else-if="blockType === 'list_item'">•</span>
        <span v-else>•</span>
      </div>

      <!-- Main content area -->
      <div class="flex-1 min-w-0">
        <textarea
          ref="textareaRef"
          v-model="blockText"
          :class="getBlockTypeClasses()"
          :placeholder="placeholder"
          :readonly="readonly"
          rows="1"
          @input="handleInput"
          @blur="handleBlur"
          @keydown="handleKeyDown"
          @compositionstart="handleCompositionStart"
          @compositionend="handleCompositionEnd"
        />
      </div>
    </div>

    <!-- Block controls (visible when editing and showControls is true) -->
    <div 
      v-if="showControls && isEditing && !readonly"
      class="absolute right-0 top-0 opacity-0 group-hover:opacity-100 transition-opacity"
    >
      <div class="flex space-x-1 bg-background border rounded-md shadow-sm p-1">
        <!-- Block type selector -->
        <select 
          v-model="blockType"
          class="text-xs border-none bg-transparent cursor-pointer"
          @change="saveBlock"
        >
          <option value="text">Text</option>
          <option value="heading">Heading</option>
          <option value="code">Code</option>
          <option value="quote">Quote</option>
          <option value="list_item">List</option>
        </select>

        <!-- Delete button -->
        <Button 
          variant="ghost" 
          size="sm" 
          class="h-6 w-6 p-0 text-destructive hover:text-destructive"
          @click="deleteCurrentBlock"
        >
          ×
        </Button>
      </div>
    </div>

    <!-- Debug info (development only) -->
    <div 
      v-if="isDevelopment && block"
      class="text-xs text-muted-foreground mt-1 opacity-50"
    >
      ID: {{ block.id.slice(-8) }} | Order: {{ block.order }} | Depth: {{ depth }}
    </div>
  </div>
</template>

<style scoped>
/* Custom textarea styling to remove default behaviors */
textarea {
  min-height: 1.5rem;
  line-height: 1.5;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

textarea::-webkit-scrollbar {
  display: none;
}

/* Prevent default textarea resize handle */
textarea {
  resize: none;
}

/* Animation for focus state */
.group {
  transition: all 0.2s ease-in-out;
}

/* Responsive indentation */
@media (max-width: 768px) {
  .ml-4 { margin-left: 1rem; }
  .ml-8 { margin-left: 1.5rem; }
  .ml-12 { margin-left: 2rem; }
  .ml-16 { margin-left: 2.5rem; }
  .ml-20 { margin-left: 3rem; }
  .ml-24 { margin-left: 3.5rem; }
}
</style> 