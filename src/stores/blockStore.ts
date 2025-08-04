import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Type definitions matching our Rust backend
export interface Block {
  id: string
  text: string
  page_id: string
  parent_id?: string
  children: string[]
  order: number
  properties: Record<string, any>
  created_at: string
  updated_at: string
  created_by: string
  last_edited_by: string
  version: number
  block_type: BlockType
  vector_clock?: any
  operation_id?: string
}

export type BlockType = 'text' | 'heading' | 'list_item' | 'code' | 'quote' | 'image' | 'link' | 'reference'

export interface CreateBlockData {
  page_id: string
  text: string
  parent_id?: string
  order?: number
  block_type?: BlockType
  properties?: Record<string, any>
}

export interface UpdateBlockData {
  text?: string
  properties?: Record<string, any>
  block_type?: BlockType
}

export interface MoveBlockData {
  new_parent_id?: string
  new_order: number
}

export const useBlockStore = defineStore('block', () => {
  // State
  const blocks = ref<Record<string, Block>>({}) // Map of block_id -> Block
  const pageBlocks = ref<Record<string, string[]>>({}) // Map of page_id -> block_ids[]
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const focusedBlockId = ref<string | null>(null)
  const selectedBlockIds = ref<Set<string>>(new Set())

  // Computed
  const getBlockById = computed(() => (blockId: string): Block | undefined => {
    return blocks.value[blockId]
  })

  const getPageBlocks = computed(() => (pageId: string): Block[] => {
    const blockIds = pageBlocks.value[pageId] || []
    return blockIds
      .map(id => blocks.value[id])
      .filter(Boolean)
      .sort((a, b) => a.order - b.order)
  })

  const getRootBlocks = computed(() => (pageId: string): Block[] => {
    return getPageBlocks.value(pageId).filter(block => !block.parent_id)
  })

  const getChildBlocks = computed(() => (parentId: string): Block[] => {
    const parentBlock = blocks.value[parentId]
    if (!parentBlock) return []
    
    return parentBlock.children
      .map(id => blocks.value[id])
      .filter(Boolean)
      .sort((a, b) => a.order - b.order)
  })

  const getBlockDepth = computed(() => (blockId: string): number => {
    const block = blocks.value[blockId]
    if (!block || !block.parent_id) return 0
    return getBlockDepth.value(block.parent_id) + 1
  })

  const getBlockPath = computed(() => (blockId: string): Block[] => {
    const block = blocks.value[blockId]
    if (!block) return []
    
    const path = [block]
    let currentBlock = block
    
    while (currentBlock.parent_id) {
      const parent = blocks.value[currentBlock.parent_id]
      if (!parent) break
      path.unshift(parent)
      currentBlock = parent
    }
    
    return path
  })

  // Helper function to handle errors
  const handleError = (err: any, context: string) => {
    console.error(`${context}:`, err)
    error.value = err.message || `An error occurred during ${context}`
    isLoading.value = false
  }

  // Actions
  const loadPageBlocks = async (pageId: string) => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Block[]>('get_page_blocks', { pageId })
      
      // Update blocks map
      const blockMap: Record<string, Block> = {}
      const blockIds: string[] = []
      
      result.forEach(block => {
        blockMap[block.id] = block
        blockIds.push(block.id)
      })
      
      // Merge with existing blocks
      blocks.value = { ...blocks.value, ...blockMap }
      pageBlocks.value[pageId] = blockIds
      
      console.log(`Loaded ${result.length} blocks for page ${pageId}`)
    } catch (err) {
      handleError(err, 'loading page blocks')
    } finally {
      isLoading.value = false
    }
  }

  const createBlock = async (data: CreateBlockData): Promise<Block | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Block>('create_block', {
        pageId: data.page_id,
        text: data.text,
        parentId: data.parent_id,
        order: data.order || 0,
        blockType: data.block_type || 'text',
        properties: data.properties || {},
      })

      // Add to local state
      blocks.value[result.id] = result
      
      // Update page blocks list
      if (!pageBlocks.value[data.page_id]) {
        pageBlocks.value[data.page_id] = []
      }
      pageBlocks.value[data.page_id].push(result.id)
      
      // Update parent's children if this is a child block
      if (result.parent_id && blocks.value[result.parent_id]) {
        if (!blocks.value[result.parent_id].children.includes(result.id)) {
          blocks.value[result.parent_id].children.push(result.id)
        }
      }
      
      console.log('Created block:', result)
      return result
    } catch (err) {
      handleError(err, 'creating block')
      return null
    } finally {
      isLoading.value = false
    }
  }

  const updateBlock = async (blockId: string, data: UpdateBlockData): Promise<Block | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Block>('update_block', {
        blockId,
        text: data.text,
        properties: data.properties,
        blockType: data.block_type,
      })

      // Update local state
      blocks.value[blockId] = result

      console.log('Updated block:', result)
      return result
    } catch (err) {
      handleError(err, 'updating block')
      return null
    } finally {
      isLoading.value = false
    }
  }

  const deleteBlock = async (blockId: string): Promise<boolean> => {
    isLoading.value = true
    error.value = null

    try {
      await invoke('delete_block', { blockId })
      
      const block = blocks.value[blockId]
      if (block) {
        // Remove from page blocks list
        if (pageBlocks.value[block.page_id]) {
          pageBlocks.value[block.page_id] = pageBlocks.value[block.page_id].filter(id => id !== blockId)
        }
        
        // Remove from parent's children
        if (block.parent_id && blocks.value[block.parent_id]) {
          blocks.value[block.parent_id].children = blocks.value[block.parent_id].children.filter(id => id !== blockId)
        }
        
        // Remove all children recursively
        const childrenToDelete = [...block.children]
        for (const childId of childrenToDelete) {
          await deleteBlock(childId)
        }
        
        // Remove from blocks map
        delete blocks.value[blockId]
      }

      // Clear focus if this block was focused
      if (focusedBlockId.value === blockId) {
        focusedBlockId.value = null
      }
      
      // Remove from selection
      selectedBlockIds.value.delete(blockId)

      console.log('Deleted block:', blockId)
      return true
    } catch (err) {
      handleError(err, 'deleting block')
      return false
    } finally {
      isLoading.value = false
    }
  }

  const moveBlock = async (blockId: string, data: MoveBlockData): Promise<Block | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Block>('move_block', {
        blockId,
        newParentId: data.new_parent_id,
        newOrder: data.new_order,
      })

      const oldBlock = blocks.value[blockId]
      
      // Update local state
      blocks.value[blockId] = result
      
      // Update parent-child relationships
      if (oldBlock.parent_id !== result.parent_id) {
        // Remove from old parent
        if (oldBlock.parent_id && blocks.value[oldBlock.parent_id]) {
          blocks.value[oldBlock.parent_id].children = blocks.value[oldBlock.parent_id].children.filter(id => id !== blockId)
        }
        
        // Add to new parent
        if (result.parent_id && blocks.value[result.parent_id]) {
          if (!blocks.value[result.parent_id].children.includes(blockId)) {
            blocks.value[result.parent_id].children.push(blockId)
          }
        }
      }

      console.log('Moved block:', result)
      return result
    } catch (err) {
      handleError(err, 'moving block')
      return null
    } finally {
      isLoading.value = false
    }
  }

  // Focus and selection management
  const setFocusedBlock = (blockId: string | null) => {
    focusedBlockId.value = blockId
  }

  const addToSelection = (blockId: string) => {
    selectedBlockIds.value.add(blockId)
  }

  const removeFromSelection = (blockId: string) => {
    selectedBlockIds.value.delete(blockId)
  }

  const clearSelection = () => {
    selectedBlockIds.value.clear()
  }

  const toggleSelection = (blockId: string) => {
    if (selectedBlockIds.value.has(blockId)) {
      selectedBlockIds.value.delete(blockId)
    } else {
      selectedBlockIds.value.add(blockId)
    }
  }

  // Keyboard navigation helpers
  const getNextSibling = (blockId: string): Block | null => {
    const block = blocks.value[blockId]
    if (!block) return null

    const siblings = block.parent_id 
      ? getChildBlocks.value(block.parent_id)
      : getRootBlocks.value(block.page_id)
    
    const currentIndex = siblings.findIndex(b => b.id === blockId)
    return currentIndex >= 0 && currentIndex < siblings.length - 1 
      ? siblings[currentIndex + 1] 
      : null
  }

  const getPreviousSibling = (blockId: string): Block | null => {
    const block = blocks.value[blockId]
    if (!block) return null

    const siblings = block.parent_id 
      ? getChildBlocks.value(block.parent_id)
      : getRootBlocks.value(block.page_id)
    
    const currentIndex = siblings.findIndex(b => b.id === blockId)
    return currentIndex > 0 ? siblings[currentIndex - 1] : null
  }

  const getParentBlock = (blockId: string): Block | null => {
    const block = blocks.value[blockId]
    return block?.parent_id ? blocks.value[block.parent_id] || null : null
  }

  const getFirstChild = (blockId: string): Block | null => {
    const children = getChildBlocks.value(blockId)
    return children.length > 0 ? children[0] : null
  }

  const clearError = () => {
    error.value = null
  }

  // Initialize blocks for a page
  const initializePage = async (pageId: string) => {
    await loadPageBlocks(pageId)
  }

  return {
    // State
    blocks,
    pageBlocks,
    isLoading,
    error,
    focusedBlockId,
    selectedBlockIds: computed(() => Array.from(selectedBlockIds.value)),

    // Computed
    getBlockById,
    getPageBlocks,
    getRootBlocks,
    getChildBlocks,
    getBlockDepth,
    getBlockPath,

    // Actions
    loadPageBlocks,
    createBlock,
    updateBlock,
    deleteBlock,
    moveBlock,
    initializePage,

    // Focus and selection
    setFocusedBlock,
    addToSelection,
    removeFromSelection,
    clearSelection,
    toggleSelection,

    // Navigation helpers
    getNextSibling,
    getPreviousSibling,
    getParentBlock,
    getFirstChild,

    // Utilities
    clearError,
  }
}) 