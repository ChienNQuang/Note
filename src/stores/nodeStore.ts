import { defineStore } from 'pinia'
import { ref } from 'vue'
import { nodeService, type Node, type NodeWithChildren, type CreateNodeRequest, type UpdateNodeRequest } from '@/services/nodeService'

export const useNodeStore = defineStore('node', () => {
  // State
  const nodes = ref<Map<string, Node>>(new Map())
  const currentDailyNote = ref<Node | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const getNodeById = (id: string) => nodes.value.get(id)

  // Actions
  async function loadDailyNote(date?: string) {
    isLoading.value = true
    error.value = null
    try {
      const dateStr = date || new Date().toISOString().split('T')[0]
      const node = await nodeService.getOrCreateDailyNote(dateStr)
      nodes.value.set(node.id, node)
      currentDailyNote.value = node
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load daily note'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function loadNode(nodeId: string) {
    isLoading.value = true
    error.value = null
    try {
      const node = await nodeService.getNode(nodeId)
      nodes.value.set(node.id, node)
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load node'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function loadNodeWithChildren(nodeId: string) {
    isLoading.value = true
    error.value = null
    try {
      const nodeWithChildren = await nodeService.getNodeWithChildren(nodeId)
      // Store the main node and all children in the map
      const storeNodeRecursive = (node: NodeWithChildren) => {
        nodes.value.set(node.id, node)
        node.child_nodes.forEach(child => storeNodeRecursive(child))
      }
      storeNodeRecursive(nodeWithChildren)
      return nodeWithChildren
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load node with children'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function createNode(data: CreateNodeRequest) {
    isLoading.value = true
    error.value = null
    try {
      const node = await nodeService.createNode(data)
      nodes.value.set(node.id, node)
      
      // Update parent's children array if it exists
      if (data.parent_id) {
        const parent = nodes.value.get(data.parent_id)
        if (parent) {
          parent.children.push(node.id)
        }
      }
      
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create node'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function updateNode(nodeId: string, data: UpdateNodeRequest) {
    isLoading.value = true
    error.value = null
    try {
      const node = await nodeService.updateNode(nodeId, data)
      nodes.value.set(node.id, node)
      return node
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update node'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function deleteNode(nodeId: string) {
    isLoading.value = true
    error.value = null
    try {
      await nodeService.deleteNode(nodeId)
      
      // Remove from parent's children array
      const node = nodes.value.get(nodeId)
      if (node?.parent_id) {
        const parent = nodes.value.get(node.parent_id)
        if (parent) {
          parent.children = parent.children.filter(id => id !== nodeId)
        }
      }
      
      nodes.value.delete(nodeId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete node'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function moveNode(nodeId: string, newParentId: string | null, newOrder: number) {
    isLoading.value = true
    error.value = null
    try {
      await nodeService.moveNode(nodeId, newParentId, newOrder)
      
      // Update local state
      const node = nodes.value.get(nodeId)
      if (node) {
        // Remove from old parent
        if (node.parent_id) {
          const oldParent = nodes.value.get(node.parent_id)
          if (oldParent) {
            oldParent.children = oldParent.children.filter(id => id !== nodeId)
          }
        }
        
        // Add to new parent
        if (newParentId) {
          const newParent = nodes.value.get(newParentId)
          if (newParent) {
            newParent.children.push(nodeId)
          }
        }
        
        // Update node
        node.parent_id = newParentId || undefined
        node.order = newOrder
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to move node'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function searchNodes(query: string) {
    isLoading.value = true
    error.value = null
    try {
      const results = await nodeService.searchNodes(query)
      results.forEach(node => nodes.value.set(node.id, node))
      return results
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to search nodes'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function getLinkedReferences(nodeId: string) {
    try {
      const refs = await nodeService.getLinkedReferences(nodeId)
      refs.forEach(node => nodes.value.set(node.id, node))
      return refs
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to get linked references'
      throw e
    }
  }

  async function getUnlinkedReferences(nodeId: string) {
    try {
      const refs = await nodeService.getUnlinkedReferences(nodeId)
      refs.forEach(node => nodes.value.set(node.id, node))
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
    isLoading,
    error,
    
    // Getters
    getNodeById,
    
    // Actions
    loadDailyNote,
    loadNode,
    loadNodeWithChildren,
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