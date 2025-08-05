import { invoke } from '@tauri-apps/api/core'

export interface Node {
  id: string
  content: string
  parent_id?: string
  children: string[]
  order: number
  properties: Record<string, any>
  tags: string[]
  created_at: string
  updated_at: string
  created_by: string
  version: number
}

export interface NodeWithChildren {
  id: string
  content: string
  parent_id?: string
  children: string[]
  order: number
  properties: Record<string, any>
  tags: string[]
  created_at: string
  updated_at: string
  created_by: string
  version: number
  child_nodes: NodeWithChildren[]
}

export interface CreateNodeRequest {
  content: string
  parent_id?: string
  order?: number
  properties?: Record<string, any>
  tags?: string[]
}

export interface UpdateNodeRequest {
  content?: string
  properties?: Record<string, any>
  tags?: string[]
}

class NodeService {
  async createNode(data: CreateNodeRequest): Promise<Node> {
    return await invoke('create_node', { data })
  }

  async getNode(nodeId: string): Promise<Node> {
    return await invoke('get_node', { nodeId })
  }

  async getNodeWithChildren(nodeId: string): Promise<NodeWithChildren> {
    return await invoke('get_node_with_children', { nodeId })
  }

  async updateNode(nodeId: string, data: UpdateNodeRequest): Promise<Node> {
    return await invoke('update_node', { nodeId, data })
  }

  async deleteNode(nodeId: string): Promise<void> {
    return await invoke('delete_node', { nodeId })
  }

  async moveNode(nodeId: string, newParentId: string | null, newOrder: number): Promise<void> {
    return await invoke('move_node', { nodeId, newParentId, newOrder })
  }

  async getDailyNote(date: string): Promise<Node> {
    return await invoke('get_daily_note', { date })
  }

  async getOrCreateDailyNote(date: string): Promise<Node> {
    return await invoke('get_or_create_daily_note', { date })
  }

  async getLinkedReferences(nodeId: string): Promise<Node[]> {
    return await invoke('get_linked_references', { nodeId })
  }

  async getUnlinkedReferences(nodeId: string): Promise<Node[]> {
    return await invoke('get_unlinked_references', { nodeId })
  }

  async searchNodes(query: string): Promise<Node[]> {
    return await invoke('search_nodes', { query })
  }

  async getRootNodes(): Promise<Node[]> {
    return await invoke('get_root_nodes', {})
  }

  async getRecentNodes(limit?: number): Promise<Node[]> {
    return await invoke('get_recent_nodes', { limit })
  }
}

export const nodeService = new NodeService()