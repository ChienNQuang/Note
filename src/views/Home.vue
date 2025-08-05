<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { usePageStore } from '@/stores/pageStore'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Skeleton } from '@/components/ui/skeleton'
import { Separator } from '@/components/ui/separator'

const router = useRouter()
const pageStore = usePageStore()
const greeting = ref('')
const newPageTitle = ref('')
const showCreateForm = ref(false)

// Initialize page store and greeting
onMounted(async () => {
  try {
    greeting.value = await invoke('greet', { name: 'Note User' })
    await pageStore.initialize()
  } catch (error) {
    console.error('Failed to initialize:', error)
    greeting.value = 'Welcome to Note!'
  }
})

// Computed
const hasPages = computed(() => pageStore.totalPages > 0)
const recentPagesDisplay = computed(() => pageStore.recentPages.slice(0, 5))

// Methods
async function createNewPage() {
  if (!newPageTitle.value.trim()) {
    newPageTitle.value = 'Untitled Page'
  }

  const page = await pageStore.createPage({
    title: newPageTitle.value.trim()
  })

  if (page) {
    newPageTitle.value = ''
    showCreateForm.value = false
    router.push(`/page/${page.id}`)
  }
}

function openPage(pageId: string) {
  router.push(`/page/${pageId}`)
}

function openSettings() {
  router.push('/settings')
}

function formatDate(dateString: string): string {
  const date = new Date(dateString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))
  
  if (diffDays === 0) {
    return 'Today'
  } else if (diffDays === 1) {
    return 'Yesterday'
  } else if (diffDays < 7) {
    return `${diffDays} days ago`
  } else {
    return date.toLocaleDateString()
  }
}

function toggleCreateForm() {
  showCreateForm.value = !showCreateForm.value
  if (showCreateForm.value) {
    // Focus the input after the DOM updates
    setTimeout(() => {
      const input = document.querySelector('input[placeholder="Enter page title..."]') as HTMLInputElement
      input?.focus()
    }, 100)
  }
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <header class="border-b bg-card">
      <div class="container mx-auto px-4 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <h1 class="text-2xl font-bold text-foreground">
              📝 Note
            </h1>
            <Badge variant="secondary" class="text-xs">
              Phase 1.4
            </Badge>
          </div>
          <Button variant="ghost" size="sm" @click="openSettings">
            ⚙️ Settings
          </Button>
        </div>
      </div>
    </header>

    <div class="container mx-auto px-4 py-8">
      <!-- Welcome Section -->
      <div class="mb-8">
        <h2 class="text-3xl font-bold text-foreground mb-2">
          Welcome to Note
        </h2>
        <p class="text-muted-foreground">
          {{ greeting }}
        </p>
      </div>

      <!-- Stats Cards -->
      <div v-if="pageStore.stats" class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">Total Pages</CardTitle>
            <span class="text-2xl">📄</span>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{{ pageStore.stats.total_pages }}</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">Draft Pages</CardTitle>
            <span class="text-2xl">✏️</span>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{{ pageStore.stats.draft_pages }}</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">Published</CardTitle>
            <span class="text-2xl">✅</span>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{{ pageStore.stats.published_pages }}</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium">Recent Activity</CardTitle>
            <span class="text-2xl">⏰</span>
          </CardHeader>
          <CardContent>
            <div class="text-sm text-muted-foreground">
              {{ pageStore.stats.recent_page || 'No pages yet' }}
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Loading Stats -->
      <div v-else class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <Card v-for="i in 4" :key="i">
          <CardHeader>
            <Skeleton class="h-4 w-24" />
          </CardHeader>
          <CardContent>
            <Skeleton class="h-8 w-16" />
          </CardContent>
        </Card>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Create New Page -->
        <div class="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle>Create New Page</CardTitle>
              <CardDescription>
                Start writing a new note or document
              </CardDescription>
            </CardHeader>
            <CardContent class="space-y-4">
              <div v-if="!showCreateForm">
                <Button 
                  class="w-full" 
                  @click="toggleCreateForm"
                  :disabled="pageStore.isLoading"
                >
                  ✨ New Page
                </Button>
              </div>
              
              <div v-else class="space-y-3">
                <Input
                  v-model="newPageTitle"
                  placeholder="Enter page title..."
                  @keydown.enter="createNewPage"
                  @keydown.escape="toggleCreateForm"
                />
                <div class="flex space-x-2">
                  <Button 
                    @click="createNewPage"
                    :disabled="pageStore.isLoading"
                    size="sm"
                  >
                    Create
                  </Button>
                  <Button 
                    variant="outline" 
                    @click="toggleCreateForm"
                    size="sm"
                  >
                    Cancel
                  </Button>
                </div>
              </div>
              
              <!-- Error Display -->
              <div v-if="pageStore.error" class="text-sm text-destructive">
                {{ pageStore.error }}
                <Button 
                  variant="ghost" 
                  size="sm" 
                  @click="pageStore.clearError"
                  class="ml-2"
                >
                  ✕
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>

        <!-- Recent Pages -->
        <div class="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Recent Pages</CardTitle>
              <CardDescription>
                Your recently updated notes
              </CardDescription>
            </CardHeader>
            <CardContent>
              <!-- Loading State -->
              <div v-if="pageStore.isLoading" class="space-y-3">
                <div v-for="i in 3" :key="i" class="flex items-center space-x-3">
                  <Skeleton class="h-4 w-4" />
                  <Skeleton class="h-4 flex-1" />
                  <Skeleton class="h-4 w-20" />
                </div>
              </div>

              <!-- No Pages State -->
              <div v-else-if="!hasPages" class="text-center py-8">
                <p class="text-muted-foreground mb-4">
                  No pages yet. Create your first page to get started!
                </p>
                <Button variant="outline" @click="toggleCreateForm">
                  Create First Page
                </Button>
              </div>

              <!-- Pages List -->
              <div v-else class="space-y-3">
                <div 
                  v-for="page in recentPagesDisplay" 
                  :key="page.id"
                  class="flex items-center justify-between p-3 rounded-lg border hover:bg-accent cursor-pointer transition-colors"
                  @click="openPage(page.id)"
                >
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center space-x-2">
                      <h3 class="font-medium text-foreground truncate">
                        {{ page.title }}
                      </h3>
                      <Badge 
                        :variant="page.status === 'published' ? 'default' : 'secondary'"
                        class="text-xs"
                      >
                        {{ page.status }}
                      </Badge>
                    </div>
                    <p class="text-sm text-muted-foreground">
                      Updated {{ formatDate(page.updated_at) }}
                    </p>
                  </div>
                  <Button variant="ghost" size="sm">
                    →
                  </Button>
                </div>
                
                <Separator v-if="pageStore.totalPages > 5" />
                
                <div v-if="pageStore.totalPages > 5" class="text-center">
                  <Button variant="outline" size="sm" disabled>
                    View All Pages (Coming Soon)
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Development Status -->
      <div class="mt-8">
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center space-x-2">
              <span>🚧</span>
              <span>Development Status</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <h4 class="font-medium text-foreground mb-2">✅ Completed</h4>
                <ul class="space-y-1 text-muted-foreground">
                  <li>• Core Infrastructure</li>
                  <li>• Database Service</li>
                  <li>• Page Management</li>
                  <li>• Block System Core</li>
                  <li>• Enhanced Storage & FTS5 Search</li>
                  <li>• Markdown/JSON Export</li>
                  <li>• Shadcn-Vue Components</li>
                </ul>
              </div>
              <div>
                <h4 class="font-medium text-foreground mb-2">🔄 Coming Next</h4>
                <ul class="space-y-1 text-muted-foreground">
                  <li>• Page Templates (Phase 1.5)</li>
                  <li>• Git Integration (Phase 2)</li>
                  <li>• Real-time Collaboration (Phase 3)</li>
                  <li>• AI Integration (Phase 4)</li>
                </ul>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  </div>
</template>

 