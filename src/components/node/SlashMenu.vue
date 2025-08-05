<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { 
  Type, Hash, CheckSquare, List, Quote, Code, Archive
} from 'lucide-vue-next'

interface Props {
  position: { x: number; y: number }
}

defineProps<Props>()

const emit = defineEmits<{
  select: [command: any]
  close: []
}>()

const searchQuery = ref('')
const selectedIndex = ref(0)
const menuEl = ref<HTMLDivElement>()

const commands = [
  { 
    type: 'text', 
    label: 'Text', 
    icon: Type, 
    description: 'Just start typing with plain text',
    shortcut: ''
  },
  { 
    type: 'heading', 
    label: 'Heading', 
    icon: Hash, 
    description: 'Big section heading',
    shortcut: 'h'
  },
  { 
    type: 'todo', 
    label: 'To-do', 
    icon: CheckSquare, 
    description: 'Track tasks with a checkbox',
    shortcut: 't'
  },
  { 
    type: 'list', 
    label: 'Bulleted List', 
    icon: List, 
    description: 'Create a simple bulleted list',
    shortcut: 'l'
  },
  { 
    type: 'quote', 
    label: 'Quote', 
    icon: Quote, 
    description: 'Capture a quote',
    shortcut: 'q'
  },
  { 
    type: 'code', 
    label: 'Code', 
    icon: Code, 
    description: 'Capture a code snippet',
    shortcut: 'c'
  },
  { 
    type: 'divider', 
    label: 'Divider', 
    icon: Archive, 
    description: 'Visually divide blocks',
    shortcut: 'd'
  },
]

const filteredCommands = computed(() => {
  if (!searchQuery.value) return commands
  
  const query = searchQuery.value.toLowerCase()
  return commands.filter(cmd => 
    cmd.label.toLowerCase().includes(query) ||
    cmd.description.toLowerCase().includes(query) ||
    cmd.shortcut.toLowerCase().includes(query)
  )
})

watch(filteredCommands, () => {
  selectedIndex.value = 0
})

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  // Focus the menu for keyboard navigation
  menuEl.value?.focus()
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

function handleKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % filteredCommands.value.length
      break
    case 'ArrowUp':
      event.preventDefault()
      selectedIndex.value = selectedIndex.value === 0 
        ? filteredCommands.value.length - 1 
        : selectedIndex.value - 1
      break
    case 'Enter':
      event.preventDefault()
      selectCommand(filteredCommands.value[selectedIndex.value])
      break
    case 'Escape':
      event.preventDefault()
      emit('close')
      break
    default:
      // Handle shortcuts
      if (event.key.length === 1) {
        const command = commands.find(cmd => cmd.shortcut === event.key)
        if (command) {
          event.preventDefault()
          selectCommand(command)
        }
      }
  }
}

function selectCommand(command: any) {
  emit('select', command)
}

function handleCommandClick(command: any, index: number) {
  selectedIndex.value = index
  selectCommand(command)
}
</script>

<template>
  <div 
    ref="menuEl"
    class="fixed z-50 w-64 bg-popover border rounded-lg shadow-lg p-1 max-h-80 overflow-y-auto"
    :style="{ left: `${position.x}px`, top: `${position.y}px` }"
    tabindex="-1"
  >
    <div v-if="filteredCommands.length === 0" class="px-2 py-1.5 text-sm text-muted-foreground">
      No commands found
    </div>
    
    <button
      v-for="(command, index) in filteredCommands"
      :key="command.type"
      @click="handleCommandClick(command, index)"
      @mouseenter="selectedIndex = index"
      class="w-full flex items-start space-x-3 px-2 py-1.5 rounded hover:bg-accent text-left transition-colors"
      :class="{ 'bg-accent': selectedIndex === index }"
    >
      <div class="flex-shrink-0 w-8 h-8 flex items-center justify-center rounded border bg-background">
        <component :is="command.icon" class="h-4 w-4" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center justify-between">
          <div class="text-sm font-medium">{{ command.label }}</div>
          <div v-if="command.shortcut" class="text-xs text-muted-foreground">
            {{ command.shortcut }}
          </div>
        </div>
        <div class="text-xs text-muted-foreground">{{ command.description }}</div>
      </div>
    </button>
  </div>
</template>