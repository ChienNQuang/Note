<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useNodeStore } from '@/stores/nodeStore'
import { Dialog, DialogContent } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Search, FileText, Calendar, Hash, CheckSquare } from 'lucide-vue-next'

interface Props {
  open: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const router = useRouter()
const nodeStore = useNodeStore()

const searchQuery = ref('')
const selectedIndex = ref(0)
const searchResults = ref<any[]>([])
const isSearching = ref(false)

const recentNodes = computed(() => {
  const nodes = Array.from(nodeStore.nodes.values())
    .filter(node => node.content && !node.content.includes(new Date().toISOString().split('T')[0]))
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
    .slice(0, 5)
  return nodes
})

watch(() => props.open, (newVal) => {
  if (newVal) {
    searchQuery.value = ''
    selectedIndex.value = 0
    searchResults.value = []
    // Focus search input after dialog opens
    setTimeout(() => {
      const input = document.querySelector('[data-command-input]') as HTMLInputElement
      input?.focus()
    }, 100)
  }
})

watch(searchQuery, async (query) => {
  if (!query.trim()) {
    searchResults.value = []
    selectedIndex.value = 0
    return
  }
  
  await performSearch(query)
})

async function performSearch(query: string) {
  isSearching.value = true
  try {
    const results = await nodeStore.searchNodes(query)
    searchResults.value = results
    selectedIndex.value = 0
  } catch (error) {
    console.error('Search failed:', error)
    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}

function handleKeydown(event: KeyboardEvent) {
  const items = searchQuery.value ? searchResults.value : recentNodes.value
  
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % Math.max(items.length, 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      selectedIndex.value = selectedIndex.value === 0 
        ? Math.max(items.length - 1, 0)
        : selectedIndex.value - 1
      break
    case 'Enter':
      event.preventDefault()
      if (items[selectedIndex.value]) {
        selectNode(items[selectedIndex.value])
      }
      break
    case 'Escape':
      event.preventDefault()
      close()
      break
  }
}

function selectNode(node: any) {
  // Check if it's a daily note
  if (node.tags?.includes('#Journal')) {
    router.push('/')
  } else {
    router.push(`/node/${node.id}`)
  }
  close()
}

function close() {
  emit('update:open', false)
}

function getNodeIcon(node: any) {
  if (node.tags?.includes('#Journal')) return Calendar
  if (node.properties?.type === 'heading') return Hash
  if (node.properties?.type === 'todo') return CheckSquare
  return FileText
}

function formatDate(date: string) {
  const d = new Date(date)
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  
  if (days === 0) return 'Today'
  if (days === 1) return 'Yesterday'
  if (days < 7) return `${days} days ago`
  return d.toLocaleDateString()
}

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown)
})

function handleGlobalKeydown(event: KeyboardEvent) {
  // Cmd/Ctrl + K to open
  if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
    event.preventDefault()
    emit('update:open', true)
  }
}
</script>

<template>
  <Dialog :open="open" @update:open="close">
    <DialogContent class="max-w-2xl p-0 overflow-hidden">
      <div class="flex items-center border-b px-3">
        <Search class="h-4 w-4 text-muted-foreground mr-2" />
        <Input
          v-model="searchQuery"
          data-command-input
          placeholder="Search nodes..."
          class="border-0 focus:ring-0 h-12"
          @keydown="handleKeydown"
        />
      </div>
      
      <div class="max-h-[400px] overflow-y-auto">
        <!-- Search Results -->
        <div v-if="searchQuery && isSearching" class="px-2 py-8 text-center text-muted-foreground">
          Searching...
        </div>
        
        <div v-else-if="searchQuery && searchResults.length === 0" class="px-2 py-8 text-center text-muted-foreground">
          No results found
        </div>
        
        <div v-else-if="searchResults.length > 0" class="py-2">
          <div class="px-2 mb-2">
            <p class="text-xs text-muted-foreground">Search Results</p>
          </div>
          <button
            v-for="(node, index) in searchResults"
            :key="node.id"
            @click="selectNode(node)"
            @mouseenter="selectedIndex = index"
            class="w-full px-2 py-2 flex items-center space-x-3 hover:bg-accent"
            :class="{ 'bg-accent': selectedIndex === index }"
          >
            <component :is="getNodeIcon(node)" class="h-4 w-4 text-muted-foreground flex-shrink-0" />
            <div class="flex-1 text-left min-w-0">
              <div class="text-sm truncate">{{ node.content || 'Untitled' }}</div>
              <div class="text-xs text-muted-foreground">{{ formatDate(node.updated_at) }}</div>
            </div>
          </button>
        </div>
        
        <!-- Recent Nodes (when not searching) -->
        <div v-else-if="recentNodes.length > 0" class="py-2">
          <div class="px-2 mb-2">
            <p class="text-xs text-muted-foreground">Recent</p>
          </div>
          <button
            v-for="(node, index) in recentNodes"
            :key="node.id"
            @click="selectNode(node)"
            @mouseenter="selectedIndex = index"
            class="w-full px-2 py-2 flex items-center space-x-3 hover:bg-accent"
            :class="{ 'bg-accent': selectedIndex === index }"
          >
            <component :is="getNodeIcon(node)" class="h-4 w-4 text-muted-foreground flex-shrink-0" />
            <div class="flex-1 text-left min-w-0">
              <div class="text-sm truncate">{{ node.content || 'Untitled' }}</div>
              <div class="text-xs text-muted-foreground">{{ formatDate(node.updated_at) }}</div>
            </div>
          </button>
        </div>
        
        <div v-else class="px-2 py-8 text-center text-muted-foreground">
          No recent nodes
        </div>
      </div>
      
      <div class="border-t px-3 py-2 text-xs text-muted-foreground">
        <span class="mr-4"><kbd class="px-1.5 py-0.5 bg-muted rounded">↑↓</kbd> Navigate</span>
        <span class="mr-4"><kbd class="px-1.5 py-0.5 bg-muted rounded">↵</kbd> Open</span>
        <span><kbd class="px-1.5 py-0.5 bg-muted rounded">Esc</kbd> Close</span>
      </div>
    </DialogContent>
  </Dialog>
</template>