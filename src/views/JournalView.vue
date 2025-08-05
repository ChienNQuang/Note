<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useNodeStore } from '@/stores/nodeStore'
import Node from '@/components/node/Node.vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ChevronLeft, ChevronRight, Calendar, Settings, Plus } from 'lucide-vue-next'

const router = useRouter()
const nodeStore = useNodeStore()

const currentDate = ref(new Date())
const dateString = computed(() => currentDate.value.toISOString().split('T')[0])
const displayDate = computed(() => {
  const options: Intl.DateTimeFormatOptions = { 
    weekday: 'long', 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  }
  return currentDate.value.toLocaleDateString(undefined, options)
})

const isToday = computed(() => {
  const today = new Date()
  return currentDate.value.toDateString() === today.toDateString()
})

onMounted(async () => {
  await loadJournal()
})

async function loadJournal() {
  try {
    await nodeStore.loadDailyNote(dateString.value)
    if (nodeStore.currentDailyNote) {
      await nodeStore.loadNodeWithChildren(nodeStore.currentDailyNote.id)
    }
  } catch (error) {
    console.error('Failed to load journal:', error)
  }
}

async function navigateDate(days: number) {
  const newDate = new Date(currentDate.value)
  newDate.setDate(newDate.getDate() + days)
  currentDate.value = newDate
  await loadJournal()
}

function goToToday() {
  currentDate.value = new Date()
  loadJournal()
}

function openSettings() {
  router.push('/settings')
}

async function createFirstNode() {
  if (nodeStore.currentDailyNote) {
    try {
      await nodeStore.createNode({
        content: '',
        parent_id: nodeStore.currentDailyNote.id,
        order: 0
      })
      // Reload to show the new node
      await nodeStore.loadNodeWithChildren(nodeStore.currentDailyNote.id)
    } catch (error) {
      console.error('Failed to create first node:', error)
    }
  }
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <header class="border-b bg-card sticky top-0 z-10">
      <div class="container mx-auto px-4 py-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <h1 class="text-xl font-bold">Daily Note</h1>
            <div class="flex items-center space-x-2">
              <Button 
                variant="ghost" 
                size="icon"
                @click="navigateDate(-1)"
                :title="'Previous day'"
              >
                <ChevronLeft class="h-4 w-4" />
              </Button>
              
              <Button
                variant="ghost"
                size="sm"
                @click="goToToday"
                :disabled="isToday"
                class="flex items-center space-x-1"
              >
                <Calendar class="h-4 w-4" />
                <span>Today</span>
              </Button>
              
              <Button 
                variant="ghost" 
                size="icon"
                @click="navigateDate(1)"
                :title="'Next day'"
                :disabled="isToday"
              >
                <ChevronRight class="h-4 w-4" />
              </Button>
            </div>
          </div>
          
          <div class="flex items-center space-x-2">
            <span class="text-sm text-muted-foreground">{{ displayDate }}</span>
            <Button variant="ghost" size="icon" @click="openSettings">
              <Settings class="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <div class="container mx-auto px-4 py-6 max-w-4xl">
      <div v-if="nodeStore.isLoading" class="flex justify-center py-8">
        <div class="text-muted-foreground">Loading journal...</div>
      </div>
      
      <div v-else-if="nodeStore.error" class="text-center py-8">
        <Card>
          <CardContent class="pt-6">
            <p class="text-destructive mb-4">{{ nodeStore.error }}</p>
            <Button @click="loadJournal">Retry</Button>
          </CardContent>
        </Card>
      </div>
      
      <div v-else-if="nodeStore.currentDailyNote" class="space-y-4">
        <!-- Journal title/content -->
        <Node 
          :node="nodeStore.currentDailyNote" 
          :is-root="true"
          class="mb-4"
        />
        
        <!-- Children nodes -->
        <div class="space-y-1">
          <div v-if="nodeStore.currentDailyNote.children.length === 0" class="text-center py-12">
            <p class="text-muted-foreground mb-4">
              Start your day by pressing Enter or clicking below
            </p>
            <Button 
              variant="outline" 
              @click="createFirstNode"
              class="mx-auto"
            >
              <Plus class="h-4 w-4 mr-2" />
              Add first note
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>