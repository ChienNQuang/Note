<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useNodeStore } from '@/stores/nodeStore'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Link } from 'lucide-vue-next'

interface Props {
  nodeId: string
}

const props = defineProps<Props>()
const router = useRouter()
const nodeStore = useNodeStore()
const references = ref<any[]>([])
const isLoading = ref(false)
const isExpanded = ref(true)

onMounted(async () => {
  await loadReferences()
})

async function loadReferences() {
  isLoading.value = true
  try {
    references.value = await nodeStore.getLinkedReferences(props.nodeId)
  } catch (error) {
    console.error('Failed to load linked references:', error)
  } finally {
    isLoading.value = false
  }
}

function navigateToNode(nodeId: string) {
  router.push(`/node/${nodeId}`)
}

function toggleExpanded() {
  isExpanded.value = !isExpanded.value
}
</script>

<template>
  <Card v-if="references.length > 0 || isLoading">
    <CardHeader class="cursor-pointer" @click="toggleExpanded">
      <CardTitle class="text-base flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <Link class="h-4 w-4" />
          <span>Linked References</span>
          <span class="text-muted-foreground">({{ references.length }})</span>
        </div>
        <Button variant="ghost" size="sm" class="h-6 px-2">
          {{ isExpanded ? '−' : '+' }}
        </Button>
      </CardTitle>
    </CardHeader>
    <CardContent v-if="isExpanded">
      <div v-if="isLoading" class="text-muted-foreground text-sm">
        Loading references...
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