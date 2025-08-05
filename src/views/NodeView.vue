<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useNodeStore } from '@/stores/nodeStore'
import Node from '@/components/node/Node.vue'
import LinkedReferences from '@/components/references/LinkedReferences.vue'
import UnlinkedReferences from '@/components/references/UnlinkedReferences.vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ArrowLeft, Home } from 'lucide-vue-next'

interface Props {
  nodeId: string
}

const props = defineProps<Props>()
const router = useRouter()
const nodeStore = useNodeStore()

const currentNode = ref<any>(null)

watch(() => props.nodeId, async (newId) => {
  if (newId) {
    await loadNode(newId)
  }
})

onMounted(async () => {
  await loadNode(props.nodeId)
})

async function loadNode(id: string) {
  try {
    const nodeWithChildren = await nodeStore.loadNodeWithChildren(id)
    currentNode.value = nodeWithChildren
  } catch (error) {
    console.error('Failed to load node:', error)
  }
}

function goBack() {
  router.back()
}

function goToJournal() {
  router.push('/')
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <header class="border-b bg-card sticky top-0 z-10">
      <div class="container mx-auto px-4 py-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <Button 
              variant="ghost" 
              size="icon"
              @click="goBack"
              title="Go back"
            >
              <ArrowLeft class="h-4 w-4" />
            </Button>
            
            <h1 class="text-xl font-bold">{{ currentNode?.content || 'Loading...' }}</h1>
          </div>
          
          <Button 
            variant="ghost" 
            size="icon"
            @click="goToJournal"
            title="Go to journal"
          >
            <Home class="h-4 w-4" />
          </Button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <div class="container mx-auto px-4 py-6 max-w-4xl">
      <div v-if="nodeStore.isLoading" class="flex justify-center py-8">
        <div class="text-muted-foreground">Loading node...</div>
      </div>
      
      <div v-else-if="nodeStore.error" class="text-center py-8">
        <Card>
          <CardContent class="pt-6">
            <p class="text-destructive mb-4">{{ nodeStore.error }}</p>
            <Button @click="loadNode(nodeId)">Retry</Button>
          </CardContent>
        </Card>
      </div>
      
      <div v-else-if="currentNode" class="space-y-6">
        <!-- Node content as title -->
        <div class="mb-6">
          <h1 class="text-3xl font-bold">{{ currentNode.content }}</h1>
        </div>
        
        <!-- Children as content -->
        <div class="space-y-2">
          <Node
            v-for="child in currentNode.child_nodes"
            :key="child.id"
            :node="child"
            :level="0"
          />
        </div>
        
        <!-- References -->
        <div class="mt-8 space-y-4">
          <LinkedReferences :node-id="nodeId" />
          <UnlinkedReferences :node-id="nodeId" />
        </div>
      </div>
    </div>
  </div>
</template>