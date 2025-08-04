import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Type definitions matching our Rust backend
export interface Page {
  id: string
  title: string
  root_blocks: string[]
  properties: Record<string, any>
  file_path?: string
  created_at: string
  updated_at: string
  created_by: string
  last_edited_by: string
  tags: string[]
  status: 'draft' | 'published' | 'archived' | 'deleted'
}

export interface PageStats {
  total_pages: number
  draft_pages: number
  published_pages: number
  recent_page?: string
}

export interface CreatePageData {
  title: string
  properties?: Record<string, any>
  tags?: string[]
  status?: 'draft' | 'published' | 'archived'
}

export interface UpdatePageData {
  title?: string
  properties?: Record<string, any>
  tags?: string[]
  status?: 'draft' | 'published' | 'archived'
}

export const usePageStore = defineStore('page', () => {
  // State
  const pages = ref<Page[]>([])
  const currentPage = ref<Page | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const stats = ref<PageStats | null>(null)

  // Computed
  const totalPages = computed(() => pages.value.length)
  const draftPages = computed(() => pages.value.filter(p => p.status === 'draft'))
  const publishedPages = computed(() => pages.value.filter(p => p.status === 'published'))
  const recentPages = computed(() => 
    [...pages.value]
      .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
      .slice(0, 5)
  )

  // Helper function to handle errors
  const handleError = (err: any, context: string) => {
    console.error(`${context}:`, err)
    error.value = err.message || `An error occurred during ${context}`
    isLoading.value = false
  }

  // Actions
  const loadPages = async () => {
    isLoading.value = true
    error.value = null
    
    try {
      const result = await invoke<Page[]>('list_pages')
      pages.value = result
      console.log('Loaded pages:', result.length)
    } catch (err) {
      handleError(err, 'loading pages')
    } finally {
      isLoading.value = false
    }
  }

  const loadPageStats = async () => {
    try {
      const result = await invoke<PageStats>('get_page_stats')
      stats.value = result
      console.log('Loaded page stats:', result)
    } catch (err) {
      console.error('Failed to load page stats:', err)
    }
  }

  const createPage = async (data: CreatePageData): Promise<Page | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Page>('create_page', {
        title: data.title,
      })
      
      // Add to local state
      pages.value.unshift(result)
      
      console.log('Created page:', result)
      return result
    } catch (err) {
      handleError(err, 'creating page')
      return null
    } finally {
      isLoading.value = false
    }
  }

  const getPage = async (pageId: string): Promise<Page | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Page>('get_page', { pageId })
      currentPage.value = result
      
      // Update in local state if exists
      const index = pages.value.findIndex(p => p.id === pageId)
      if (index !== -1) {
        pages.value[index] = result
      }
      
      console.log('Loaded page:', result)
      return result
    } catch (err) {
      handleError(err, 'loading page')
      return null
    } finally {
      isLoading.value = false
    }
  }

  const updatePage = async (pageId: string, data: UpdatePageData): Promise<Page | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Page>('update_page', {
        pageId,
        title: data.title,
        properties: data.properties,
        tags: data.tags,
        status: data.status,
      })
      
      // Update in local state
      const index = pages.value.findIndex(p => p.id === pageId)
      if (index !== -1) {
        pages.value[index] = result
      }
      
      // Update current page if it matches
      if (currentPage.value?.id === pageId) {
        currentPage.value = result
      }
      
      console.log('Updated page:', result)
      return result
    } catch (err) {
      handleError(err, 'updating page')
      return null
    } finally {
      isLoading.value = false
    }
  }

  const deletePage = async (pageId: string): Promise<boolean> => {
    isLoading.value = true
    error.value = null

    try {
      await invoke('delete_page', { pageId })
      
      // Remove from local state
      pages.value = pages.value.filter(p => p.id !== pageId)
      
      // Clear current page if it was deleted
      if (currentPage.value?.id === pageId) {
        currentPage.value = null
      }
      
      console.log('Deleted page:', pageId)
      return true
    } catch (err) {
      handleError(err, 'deleting page')
      return false
    } finally {
      isLoading.value = false
    }
  }

  const clearError = () => {
    error.value = null
  }

  const setCurrentPage = (page: Page | null) => {
    currentPage.value = page
  }

  // Helper to get page by ID from local state
  const findPageById = (pageId: string): Page | undefined => {
    return pages.value.find(p => p.id === pageId)
  }

  // Initialize - load pages when store is created
  const initialize = async () => {
    await Promise.all([
      loadPages(),
      loadPageStats()
    ])
  }

  return {
    // State
    pages,
    currentPage,
    isLoading,
    error,
    stats,
    
    // Computed
    totalPages,
    draftPages,
    publishedPages,
    recentPages,
    
    // Actions
    loadPages,
    loadPageStats,
    createPage,
    getPage,
    updatePage,
    deletePage,
    clearError,
    setCurrentPage,
    findPageById,
    initialize,
  }
}) 