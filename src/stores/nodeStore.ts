import { defineStore } from 'pinia'
import { Ref, ref } from 'vue'
import { nodeService, type Node, type CreateNodeRequest, type UpdateNodeRequest, NodeId } from '@/services/nodeService'

export const useNodeStore = defineStore('node', () => {
  // State
  const nodes = ref<Map<NodeId, Ref<Node>>>(new Map())
  const currentDailyNote = ref<Node | null>(null)
  const error = ref<string | null>(null)

  // Getters
  const getNodeById = (id: string) => nodes.value.get(id)

  const getNodeRef = (id: string) => {
    const node = nodes.value.get(id)
    if (!node) {
      throw new Error(`Node with id ${id} not found`)
    }
    return node
  }

  // Actions
  async function loadDailyNote(date?: string) {
    error.value = null
    try {
      const dateStr = date || new Date().toISOString().split('T')[0]
      const node = await nodeService.getOrCreateDailyNote(dateStr)
      if (!nodes.value.has(node.id)) {
        nodes.value.set(node.id, ref(node))
      }
      else {
        nodes.value.get(node.id)!.value = node
      }
      currentDailyNote.value = node
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load daily note'
      throw e
    }
  }

  async function loadNode(nodeId: string) {
    error.value = null
    try {
      const node = await nodeService.getNode(nodeId)
      if (!nodes.value.has(node.id)) {
        nodes.value.set(node.id, ref(node))
      }
      else {
        nodes.value.get(node.id)!.value = node
      }
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load node'
      throw e
    }
  }

  async function createNode(data: CreateNodeRequest) {
    error.value = null
    try {
      const node = await nodeService.createNode(data)
      nodes.value.set(node.id, ref(node))
      
      // Update parent's children array if it exists
      if (data.parent_id) {
        const parent = nodes.value.get(data.parent_id)
        if (parent) {
          parent.value.children.push(node.id)
        }
      }
      
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create node'
      throw e
    }
  }

  async function updateNode(nodeId: string, data: UpdateNodeRequest) {
    error.value = null
    try {
      const node = await nodeService.updateNode(nodeId, data)
      if (!nodes.value.has(node.id)) {
        nodes.value.set(node.id, ref(node))
      }
      else {
        nodes.value.get(node.id)!.value = node
      }
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update node'
      throw e
    }
  }

  async function deleteNode(nodeId: string) {
    error.value = null
    try {
      await nodeService.deleteNode(nodeId)
      
      // Remove from parent's children array
      const node = nodes.value.get(nodeId)
      if (node?.value.parent_id) {
        const parent = nodes.value.get(node.value.parent_id)
        if (parent) {
          parent.value.children = parent.value.children.filter(id => id !== nodeId)
        }
      }
      
      nodes.value.delete(nodeId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete node'
      throw e
    }
  }

  async function moveNode(nodeId: string, newParentId: string | null, newOrder: number) {
    error.value = null
    try {
      await nodeService.moveNode(nodeId, newParentId, newOrder)
      
      // Update local state
      const node = nodes.value.get(nodeId)
      if (node) {
        // Remove from old parent
        if (node.value.parent_id) {
          const oldParent = nodes.value.get(node.value.parent_id)
          if (oldParent) {
            oldParent.value.children = oldParent.value.children.filter(id => id !== nodeId)
          }
        }
        
        // Add to new parent
        if (newParentId) {
          const newParent = nodes.value.get(newParentId)
          if (newParent) {
            newParent.value.children.push(nodeId)
          }
        }
        
        // Update node
        node.value.parent_id = newParentId || undefined
        node.value.order = newOrder
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to move node'
      throw e
    }
  }

  async function searchNodes(query: string) {
    error.value = null
    try {
      const results = await nodeService.searchNodes(query)
      results.forEach(node => {
        if (!nodes.value.has(node.id)) {
          nodes.value.set(node.id, ref(node))
        }
      })
      return results
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to search nodes'
      throw e
    }
  }

  async function getLinkedReferences(nodeId: string) {
    try {
      const refs = await nodeService.getLinkedReferences(nodeId)
      refs.forEach(node => {
        if (!nodes.value.has(node.id)) {
          nodes.value.set(node.id, ref(node))
        }
      })
      return refs
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to get linked references'
      throw e
    }
  }

  async function getUnlinkedReferences(nodeId: string) {
    try {
      const refs = await nodeService.getUnlinkedReferences(nodeId)
      refs.forEach(node => {
        if (!nodes.value.has(node.id)) {
          nodes.value.set(node.id, ref(node))
        }
      })
      return refs
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to get unlinked references'
      throw e
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    nodes,
    currentDailyNote,
    error,

    // Getters
    getNodeById,
    getNodeRef,
    
    // Actions

    loadDailyNote,
    loadNode,
    createNode,
    updateNode,
    deleteNode,
    moveNode,
    searchNodes,
    getLinkedReferences,
    getUnlinkedReferences,
    clearError
  }
})