<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useNodeStore } from '@/stores/nodeStore'
import type { Node, NodeId } from '@/services/nodeService'
import { Button } from '@/components/ui/button'
import { ChevronRight, ChevronDown, GripVertical, Hash, CheckSquare, List } from 'lucide-vue-next'
import SlashMenu from './SlashMenu.vue'

interface Props {
  nodeId: NodeId
  isRoot?: boolean
  level?: number
  isNew?: boolean
  defaultExpanded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isRoot: false,
  level: 0,
  isNew: false,
  defaultExpanded: true
})

const emit = defineEmits<{
  'focus-next': []
  'focus-prev': []
}>()

const nodeStore = useNodeStore()

// Get the most up-to-date node data from the store, falling back to props
const currentNode = nodeStore.getNodeRef(props.nodeId)

const isEditing = ref(false)
const editContent = ref('')
const contentEl = ref<HTMLDivElement>()
const isExpanded = ref(props.defaultExpanded)
const showActions = ref(true)
const showSlashMenu = ref(false)
const slashMenuPosition = ref({ x: 0, y: 0 })
const nodeType = ref(currentNode.value.properties?.type || 'text')
const isChecked = ref(currentNode.value.properties?.checked || false)

const hasChildren = computed(() => {
  return currentNode.value.children.length > 0
})

const placeholder = computed(() => {
  if (props.isRoot) return "Today's thoughts..."
  if (nodeType.value === 'heading') return 'Heading'
  if (nodeType.value === 'todo') return 'To-do'
  return 'Type \'/\' for commands'
})

const nodeIcon = computed(() => {
  switch (nodeType.value) {
    case 'heading': return Hash
    case 'todo': return CheckSquare
    case 'list': return List
    default: return null
  }
})

watch(() => currentNode.value.content, (newContent) => {
  editContent.value = newContent
}, { immediate: true })

onMounted(() => {
  if (props.isNew || (currentNode.value.content === '' && !props.isRoot)) {
    startEdit()
  }
})

function startEdit() {
  isEditing.value = true
  editContent.value = currentNode.value.content
  nextTick(() => {
    if (contentEl.value) {
      contentEl.value.textContent = editContent.value
      contentEl.value.focus()
      // Place cursor at end
      const range = document.createRange()
      const sel = window.getSelection()
      if (contentEl.value.childNodes.length > 0) {
        range.selectNodeContents(contentEl.value)
        range.collapse(false)
      } else {
        range.setStart(contentEl.value, 0)
        range.setEnd(contentEl.value, 0)
      }
      sel?.removeAllRanges()
      sel?.addRange(range)
    }
  })
}

async function saveEdit() {
  const newContent = contentEl.value?.textContent || ''
  
  if (newContent !== currentNode.value.content) {
    console.log('saveEdit', newContent)
    try {
      const updated = await nodeStore.updateNode(currentNode.value.id, {
        content: newContent,
        properties: {
          ...currentNode.value.properties,
          type: nodeType.value,
          checked: isChecked.value
        }
      })
      console.log('updated', updated)
    } catch (error) {
      console.error('Failed to update node:', error)
      editContent.value = currentNode.value.content
    }
  }
  isEditing.value = false
  showSlashMenu.value = false
}

function cancelEdit() {
  editContent.value = currentNode.value.content
  isEditing.value = false
  showSlashMenu.value = false
}

async function handleKeydown(event: KeyboardEvent) {
  const content = contentEl.value?.textContent || ''
  
  // Handle slash commands
  if (event.key === '/' && content === '') {
    event.preventDefault()
    showSlashCommand()
    return
  }
  
  // Hide slash menu on any other key if it's showing
  if (showSlashMenu.value && event.key !== 'ArrowDown' && event.key !== 'ArrowUp' && event.key !== 'Enter') {
    showSlashMenu.value = false
  }
  
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    
    await saveEdit()
    createSiblingNode()
  } else if (event.key === 'Escape') {
    cancelEdit()
  } else if (event.key === 'Tab') {
    event.preventDefault()
    await saveEdit()
    if (event.shiftKey) {
      outdentNode()
    } else {
      indentNode()
    }
  } else if (event.key === 'Backspace' && content === '') {
    event.preventDefault()
    const prevNode = getPreviousNode()
    if (prevNode) {
      await nodeStore.deleteNode(currentNode.value.id)
      // Focus previous node
      nextTick(() => {
        const prevEditor = document.querySelector(`[data-node-id="${prevNode.id}"]`)
        if (prevEditor) {
          (prevEditor as any).click()
        }
      })
    }
  } else if (event.key === 'ArrowUp' && getCursorPosition() === 0) {
    event.preventDefault()
    focusPreviousNode()
  } else if (event.key === 'ArrowDown' && getCursorPosition() === content.length) {
    event.preventDefault()
    focusNextNode()
  }
}

function getCursorPosition(): number {
  const sel = window.getSelection()
  if (!sel || !sel.rangeCount || !contentEl.value) return 0
  
  const range = sel.getRangeAt(0)
  const preRange = range.cloneRange()
  preRange.selectNodeContents(contentEl.value)
  preRange.setEnd(range.endContainer, range.endOffset)
  return preRange.toString().length
}

function showSlashCommand() {
  if (!contentEl.value) return
  
  const rect = contentEl.value.getBoundingClientRect()
  slashMenuPosition.value = {
    x: rect.left,
    y: rect.bottom + 5
  }
  showSlashMenu.value = true
}

function handleSlashCommand(command: any) {
  nodeType.value = command.type
  if (contentEl.value) {
    contentEl.value.textContent = ''
    contentEl.value.focus()
  }
  showSlashMenu.value = false
}

async function createChildNode() {
  try {
    const order = hasChildren.value ? currentNode.value.children.length : 0
    const newNode = await nodeStore.createNode({
      content: '',
      parent_id: currentNode.value.id,
      order: order
    })
    // Expand this node to show the new child
    isExpanded.value = true
    
    // Reload to get updated children
    await nodeStore.loadNode(currentNode.value.id)
    // Focus the new node
    nextTick(() => {
      const newEditor = document.querySelector(`[data-node-id="${newNode.id}"]`)
      if (newEditor) {
        (newEditor as any).click()
      }
    })
  } catch (error) {
    console.error('Failed to create child node:', error)
  }
}

async function createSiblingNode() {
  if (!props.isRoot && currentNode.value.parent_id) {
    try {
      const newNode = await nodeStore.createNode({
        content: '',
        parent_id: currentNode.value.parent_id,
        order: currentNode.value.order + 1
      })
      
      // Reload parent to show new sibling
      await nodeStore.loadNode(currentNode.value.parent_id)
      
      // Focus the new node
      nextTick(() => {
        const newEditor = document.querySelector(`[data-node-id="${newNode.id}"]`)
        if (newEditor) {
          (newEditor as any).click()
        }
      })
    } catch (error) {
      console.error('Failed to create sibling node:', error)
    }
  } else if (props.isRoot) {
    console.log('createChildNode')
    // For root node, create a child instead
    await createChildNode()
  }
}

async function indentNode() {
  if (props.isRoot) return
  
  // Find previous sibling to make it the parent
  const siblings = await getSiblings()
  const currentIndex = siblings.findIndex(n => n.id === currentNode.value.id)
  
  if (currentIndex > 0) {
    const newParent = siblings[currentIndex - 1]
    try {
      await nodeStore.moveNode(currentNode.value.id, newParent.id, newParent.children.length)
      // Reload the previous parent
      if (currentNode.value.parent_id) {
        await nodeStore.loadNode(currentNode.value.parent_id)
      }
    } catch (error) {
      console.error('Failed to indent node:', error)
    }
  }
}

async function outdentNode() {
  if (props.isRoot || !currentNode.value.parent_id) return
  
  // Get parent's parent
  const parent = await nodeStore.loadNode(currentNode.value.parent_id)
  if (!parent.parent_id) return // Can't outdent from root level
  
  try {
    // Move after parent
    await nodeStore.moveNode(currentNode.value.id, parent.parent_id, parent.order + 1)
    // Reload both old and new parents
    await nodeStore.loadNode(currentNode.value.parent_id)
    await nodeStore.loadNode(parent.parent_id)
  } catch (error) {
    console.error('Failed to outdent node:', error)
  }
}

async function getSiblings(): Promise<Node[]> {
  if (!currentNode.value.parent_id) return []
  
  const parent = await nodeStore.loadNode(currentNode.value.parent_id)
  const siblings = []
  for (const childId of parent.children) {
    const child = await nodeStore.loadNode(childId)
    siblings.push(child)
  }
  return siblings.sort((a, b) => a.order - b.order)
}

function getPreviousNode(): Node | null {
  // This is simplified - in a real implementation, you'd traverse the tree properly
  return null
}

function focusPreviousNode() {
  emit('focus-prev')
}

function focusNextNode() {
  emit('focus-next')
}

function toggleExpand() {
  isExpanded.value = !isExpanded.value
}

function handleContentInput() {
  // Check for slash command
  const content = contentEl.value?.textContent || ''
  if (content === '/') {
    showSlashCommand()
  }
}

async function toggleTodo() {
  isChecked.value = !isChecked.value
  try {
    await nodeStore.updateNode(currentNode.value.id, {
      properties: {
        ...currentNode.value.properties,
        checked: isChecked.value
      }
    })
  } catch (error) {
    console.error('Failed to toggle todo:', error)
    isChecked.value = !isChecked.value
  }
}

// Parse content for [[wikilinks]]
const formattedContent = computed(() => {
  let content = currentNode.value.content
  
  // Replace wikilinks
  content = content.replace(/\[\[(.*?)\]\]/g, (_match, linkText) => {
    return `<span class="text-primary underline cursor-pointer" data-link="${linkText}">${linkText}</span>`
  })
  
  // Add other formatting here (bold, italic, etc)
  
  return content
})

function handleContentClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.dataset.link) {
    // Navigate to linked node
    // TODO: Implement node navigation by content search
    console.log('Navigate to:', target.dataset.link)
  }
}
</script>

<template>
  <div 
    v-if="currentNode"
    :data-node-id="currentNode.id"
    class="group relative node-editor"
    @mouseenter="showActions = true"
    @mouseleave="showActions = false"
  >
    <div class="flex items-start space-x-1 py-0.5">
      <!-- Drag handle -->
      <!-- <div class="cursor-move p-1 flex items-center">
        <GripVertical class="h-4 w-4" />
      </div> -->
      
      <!-- Expand/Collapse button -->
      <Button
        v-if="hasChildren"
        variant="ghost"
        size="icon"
        class="h-6 w-6 p-0 hover:bg-accent"
        @click="toggleExpand"
      >
        <ChevronRight v-if="!isExpanded" class="h-4 w-4" />
        <ChevronDown v-else class="h-4 w-4" />
      </Button>
      <div v-else-if="!isRoot" class="w-6" />
      
      <!-- Node type icon or checkbox -->
      <div v-if="nodeType === 'todo'" class="flex items-center h-6">
        <input
          type="checkbox"
          :checked="isChecked"
          @change="toggleTodo"
          class="h-4 w-4 rounded border-gray-300 cursor-pointer"
        />
      </div>
      <div v-else-if="nodeIcon" class="flex items-center h-6 text-muted-foreground">
        <component :is="nodeIcon" class="h-4 w-4" />
      </div>
      
      <!-- Node content -->
      <div class="flex-1 min-w-0">
        <div
          v-if="!isEditing"
          @click="startEdit"
          @click.capture="handleContentClick"
          class="cursor-text hover:bg-accent/50 rounded px-2 py-1 -ml-2 min-h-[28px] flex items-center"
          :class="{ 
            'text-2xl font-bold': isRoot,
            'text-lg font-semibold': nodeType === 'heading',
            'line-through opacity-50': nodeType === 'todo' && isChecked,
            'text-base': nodeType === 'text' || nodeType === 'list'
          }"
        >
          <span v-if="currentNode.content" v-html="formattedContent" />
          <span v-else class="text-muted-foreground">{{ placeholder }}</span>
        </div>
        
        <div
          v-else
          ref="contentEl"
          contenteditable
          @blur="saveEdit"
          @keydown="handleKeydown"
          @input="handleContentInput"
          class="outline-none bg-accent rounded px-2 py-1 -ml-2 min-h-[28px]"
          :class="{ 
            'text-2xl font-bold': isRoot,
            'text-lg font-semibold': nodeType === 'heading',
            'text-base': nodeType === 'text' || nodeType === 'list'
          }"
          :data-placeholder="placeholder"
        />
      </div>
    </div>
    
    <!-- Children -->
    <div v-if="isExpanded && hasChildren" class="ml-6 border-l-2 border-accent pl-2">
      <template v-for="child in currentNode.children" :key="child">
        <Node
          :node-id="child"
          :level="level + 1"
        />
      </template>
    </div>
    
    <!-- Slash menu -->
    <SlashMenu
      v-if="showSlashMenu"
      :position="slashMenuPosition"
      @select="handleSlashCommand"
      @close="showSlashMenu = false"
    />
  </div>
</template>

<style scoped>
[contenteditable]:empty:before {
  content: attr(data-placeholder);
  color: #9ca3af;
  pointer-events: none;
  position: absolute;
}

[contenteditable] {
  position: relative;
}

.node-editor {
  transition: all 0.1s ease;
}
</style>