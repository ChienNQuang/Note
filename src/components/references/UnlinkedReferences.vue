<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useNodeStore } from '@/stores/nodeStore'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { FileQuestion } from 'lucide-vue-next'

interface Props {
  nodeId: string
}

const props = defineProps<Props>()
const router = useRouter()
const nodeStore = useNodeStore()
const references = ref<any[]>([])
const isLoading = ref(false)
const isExpanded = ref(false)

onMounted(async () => {
  // Load unlinked references only when expanded
})

async function loadReferences() {
  if (references.value.length > 0) return // Already loaded
  
  isLoading.value = true
  try {
    references.value = await nodeStore.getUnlinkedReferences(props.nodeId)
  } catch (error) {
    console.error('Failed to load unlinked references:', error)
  } finally {
    isLoading.value = false
  }
}

function navigateToNode(nodeId: string) {
  router.push(`/node/${nodeId}`)
}

async function toggleExpanded() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value && references.value.length === 0) {
    await loadReferences()
  }
}
</script>

<template>
  <Card>
    <CardHeader class="cursor-pointer" @click="toggleExpanded">
      <CardTitle class="text-base flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <FileQuestion class="h-4 w-4" />
          <span>Unlinked References</span>
        </div>
        <Button variant="ghost" size="sm" class="h-6 px-2">
          {{ isExpanded ? '−' : '+' }}
        </Button>
      </CardTitle>
    </CardHeader>
    <CardContent v-if="isExpanded">
      <div v-if="isLoading" class="text-muted-foreground text-sm">
        Searching for unlinked references...
      </div>
      <div v-else-if="references.length === 0" class="text-muted-foreground text-sm">
        No unlinked references found
      </div>
      <div v-else class="space-y-2">
        <div
          v-for="ref in references"
          :key="ref.id"
          @click="navigateToNode(ref.id)"
          class="p-2 rounded hover:bg-accent cursor-pointer text-sm"
        >
          <div class="font-medium">{{ ref.content }}</div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>