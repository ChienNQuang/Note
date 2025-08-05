<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { usePageStore } from '@/stores/pageStore'
import { type Block } from '@/stores/blockStore'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'

interface SearchResult {
  type: 'page' | 'block'
  id: string
  title: string
  snippet: string
  page_id?: string
  page_title?: string
  block_type?: string
  relevance: number
}

const router = useRouter()
const pageStore = usePageStore()

// Component state
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const isSearching = ref(false)
const searchError = ref<string | null>(null)
const selectedScope = ref<'all' | 'pages' | 'blocks'>('all')
const inputRef = ref<HTMLInputElement | null>(null)

// Debounced search
let searchTimeout: NodeJS.Timeout | null = null

// Computed
const hasQuery = computed(() => searchQuery.value.trim().length > 0)
const hasResults = computed(() => searchResults.value.length > 0)
const groupedResults = computed(() => {
  const groups = {
    pages: searchResults.value.filter(r => r.type === 'page'),
    blocks: searchResults.value.filter(r => r.type === 'block')
  }
  return groups
})

// Search functions
const performSearch = async () => {
  const query = searchQuery.value.trim()
  if (!query) {
    searchResults.value = []
    return
  }

  isSearching.value = true
  searchError.value = null

  try {
    const results: SearchResult[] = []

    // Search pages if scope includes them
    if (selectedScope.value === 'all' || selectedScope.value === 'pages') {
      const pages = await invoke<any[]>('search_pages', { query })
      for (const page of pages) {
        results.push({
          type: 'page',
          id: page.id,
          title: page.title,
          snippet: createSnippet(page.title, query),
          relevance: calculateRelevance(page.title, query),
        })
      }
    }

    // Search blocks if scope includes them
    if (selectedScope.value === 'all' || selectedScope.value === 'blocks') {
      const blocks = await invoke<Block[]>('search_all_blocks', { query })
      for (const block of blocks) {
        // Get page title for context
        const page = pageStore.findPageById(block.page_id)
        results.push({
          type: 'block',
          id: block.id,
          title: truncateText(block.text, 60),
          snippet: createSnippet(block.text, query),
          page_id: block.page_id,
          page_title: page?.title || 'Unknown Page',
          block_type: block.block_type,
          relevance: calculateRelevance(block.text, query),
        })
      }
    }

    // Sort by relevance
    results.sort((a, b) => b.relevance - a.relevance)
    searchResults.value = results

  } catch (error) {
    console.error('Search failed:', error)
    searchError.value = 'Search failed: ' + (error as Error).message
  } finally {
    isSearching.value = false
  }
}

// Helper functions
const createSnippet = (text: string, query: string, maxLength = 150): string => {
  const lowerText = text.toLowerCase()
  const lowerQuery = query.toLowerCase()
  const index = lowerText.indexOf(lowerQuery)
  
  if (index === -1) {
    return truncateText(text, maxLength)
  }
  
  const start = Math.max(0, index - 50)
  const end = Math.min(text.length, index + query.length + 50)
  let snippet = text.slice(start, end)
  
  if (start > 0) snippet = '...' + snippet
  if (end < text.length) snippet = snippet + '...'
  
  // Highlight the query term
  const regex = new RegExp(`(${escapeRegex(query)})`, 'gi')
  snippet = snippet.replace(regex, '<mark>$1</mark>')
  
  return snippet
}

const truncateText = (text: string, maxLength: number): string => {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength - 3) + '...'
}

const calculateRelevance = (text: string, query: string): number => {
  const lowerText = text.toLowerCase()
  const lowerQuery = query.toLowerCase()
  
  let score = 0
  
  // Exact match bonus
  if (lowerText.includes(lowerQuery)) {
    score += 10
  }
  
  // Word boundary bonus
  const words = lowerQuery.split(' ')
  for (const word of words) {
    if (lowerText.includes(word)) {
      score += 5
    }
  }
  
  // Length penalty (shorter results are more relevant)
  score -= text.length / 100
  
  return score
}

const escapeRegex = (string: string): string => {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

// Event handlers
const handleSearch = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
  
  searchTimeout = setTimeout(() => {
    performSearch()
  }, 300) // 300ms debounce
}

const handleResultClick = (result: SearchResult) => {
  if (result.type === 'page') {
    router.push(`/page/${result.id}`)
  } else if (result.type === 'block' && result.page_id) {
    // Navigate to page and highlight block
    router.push(`/page/${result.page_id}?highlight=${result.id}`)
  }
}

const clearSearch = () => {
  searchQuery.value = ''
  searchResults.value = []
  searchError.value = null
}

const focusInput = () => {
  nextTick(() => {
    inputRef.value?.focus()
  })
}

// Watch for search query changes
watch(searchQuery, handleSearch)

// Watch for scope changes
watch(selectedScope, () => {
  if (hasQuery.value) {
    handleSearch()
  }
})

// Lifecycle
onMounted(() => {
  focusInput()
})

// Keyboard shortcuts
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    clearSearch()
  }
}

// Format block type
const formatBlockType = (type: string): string => {
  return type.charAt(0).toUpperCase() + type.slice(1).replace('_', ' ')
}
</script>

<template>
  <div class="w-full max-w-2xl mx-auto space-y-4">
    <!-- Search Input -->
    <div class="space-y-3">
      <div class="relative">
        <Input
          ref="inputRef"
          v-model="searchQuery"
          placeholder="Search pages and blocks..."
          class="pl-10 pr-10"
          @keydown="handleKeydown"
        />
        <div class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground">
          🔍
        </div>
        <Button
          v-if="hasQuery"
          variant="ghost"
          size="sm"
          class="absolute right-1 top-1/2 transform -translate-y-1/2 h-8 w-8 p-0"
          @click="clearSearch"
        >
          ✕
        </Button>
      </div>

      <!-- Search Scope -->
      <div class="flex space-x-2">
        <Button
          variant="ghost"
          size="sm"
          :class="selectedScope === 'all' ? 'bg-accent' : ''"
          @click="selectedScope = 'all'"
        >
          All
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :class="selectedScope === 'pages' ? 'bg-accent' : ''"
          @click="selectedScope = 'pages'"
        >
          Pages Only
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :class="selectedScope === 'blocks' ? 'bg-accent' : ''"
          @click="selectedScope = 'blocks'"
        >
          Blocks Only
        </Button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isSearching" class="space-y-3">
      <Skeleton class="h-20 w-full" />
      <Skeleton class="h-20 w-full" />
      <Skeleton class="h-20 w-full" />
    </div>

    <!-- Error State -->
    <Card v-else-if="searchError" class="border-destructive">
      <CardContent class="p-4">
        <div class="text-destructive text-sm">
          {{ searchError }}
        </div>
      </CardContent>
    </Card>

    <!-- Empty State -->
    <Card v-else-if="hasQuery && !hasResults" class="text-center">
      <CardContent class="p-8">
        <div class="text-4xl mb-4">🔍</div>
        <h3 class="text-lg font-medium text-foreground mb-2">
          No Results Found
        </h3>
        <p class="text-muted-foreground mb-4">
          No pages or blocks match "<strong>{{ searchQuery }}</strong>"
        </p>
        <Button variant="outline" @click="clearSearch">
          Clear Search
        </Button>
      </CardContent>
    </Card>

    <!-- Search Results -->
    <div v-else-if="hasResults" class="space-y-4">
      <!-- Results Summary -->
      <div class="text-sm text-muted-foreground">
        Found {{ searchResults.length }} result{{ searchResults.length !== 1 ? 's' : '' }}
        for "<strong>{{ searchQuery }}</strong>"
      </div>

      <!-- Page Results -->
      <div v-if="groupedResults.pages.length > 0">
        <h3 class="text-sm font-medium text-foreground mb-3 flex items-center space-x-2">
          <span>📄</span>
          <span>Pages ({{ groupedResults.pages.length }})</span>
        </h3>
        <div class="space-y-2">
          <Card
            v-for="result in groupedResults.pages"
            :key="result.id"
            class="cursor-pointer hover:bg-accent transition-colors"
            @click="handleResultClick(result)"
          >
            <CardContent class="p-4">
              <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                  <h4 class="font-medium text-foreground truncate">
                    {{ result.title }}
                  </h4>
                  <div class="text-sm text-muted-foreground mt-1" v-html="result.snippet"></div>
                </div>
                <Badge variant="secondary" class="ml-2">
                  Page
                </Badge>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Separator -->
      <Separator v-if="groupedResults.pages.length > 0 && groupedResults.blocks.length > 0" />

      <!-- Block Results -->
      <div v-if="groupedResults.blocks.length > 0">
        <h3 class="text-sm font-medium text-foreground mb-3 flex items-center space-x-2">
          <span>📝</span>
          <span>Blocks ({{ groupedResults.blocks.length }})</span>
        </h3>
        <div class="space-y-2">
          <Card
            v-for="result in groupedResults.blocks"
            :key="result.id"
            class="cursor-pointer hover:bg-accent transition-colors"
            @click="handleResultClick(result)"
          >
            <CardContent class="p-4">
              <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center space-x-2 mb-2">
                    <h4 class="font-medium text-foreground truncate">
                      {{ result.page_title }}
                    </h4>
                    <span class="text-muted-foreground">→</span>
                    <Badge variant="outline" class="text-xs">
                      {{ formatBlockType(result.block_type!) }}
                    </Badge>
                  </div>
                  <div class="text-sm text-muted-foreground" v-html="result.snippet"></div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>

    <!-- Help Text -->
    <div v-if="!hasQuery" class="text-center py-8">
      <div class="text-4xl mb-4">🔍</div>
      <h3 class="text-lg font-medium text-foreground mb-2">
        Search Your Notes
      </h3>
      <p class="text-muted-foreground mb-4">
        Find pages and blocks using full-text search.<br>
        Supports phrases, keywords, and fuzzy matching.
      </p>
      <div class="text-xs text-muted-foreground space-y-1">
        <div><kbd class="bg-muted px-1 rounded">Esc</kbd> Clear search</div>
        <div>Powered by SQLite FTS5</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Highlight search matches */
:deep(mark) {
  background-color: hsl(var(--accent));
  color: hsl(var(--accent-foreground));
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-weight: 500;
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
</style> 