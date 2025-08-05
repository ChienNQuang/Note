<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useNodeStore } from '@/stores/nodeStore'
import type { Node, NodeWithChildren } from '@/services/nodeService'
import { Button } from '@/components/ui/button'
import { ChevronRight, ChevronDown, Plus, Trash2, GripVertical, Hash, CheckSquare, List } from 'lucide-vue-next'
import SlashMenu from './SlashMenu.vue'

interface Props {
  node: Node | NodeWithChildren
  isRoot?: boolean
  level?: number
  isNew?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isRoot: false,
  level: 0,
  isNew: false
})

const emit = defineEmits<{
  'node-updated': [node: Node]
  'node-created': [parentId: string, afterId?: string]
  'node-deleted': [nodeId: string]
  'focus-next': []
  'focus-prev': []
}>()

const nodeStore = useNodeStore()
const isEditing = ref(false)
const editContent = ref('')
const contentEl = ref<HTMLDivElement>()
const isExpanded = ref(true)
const showActions = ref(false)
const showSlashMenu = ref(false)
const slashMenuPosition = ref({ x: 0, y: 0 })
const nodeType = ref(props.node.properties?.type || 'text')
const isChecked = ref(props.node.properties?.checked || false)

const hasChildren = computed(() => {
  if ('child_nodes' in props.node) {
    return props.node.child_nodes.length > 0
  }
  return props.node.children.length > 0
})

const childNodes = computed(() => {
  if ('child_nodes' in props.node) {
    return props.node.child_nodes
  }
  // Load children if we have IDs but not the full nodes
  if (props.node.children.length > 0) {
    return props.node.children.map(id => nodeStore.getNodeById(id)).filter(Boolean)
  }
  return []
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

watch(() => props.node.content, (newContent) => {
  editContent.value = newContent
}, { immediate: true })

onMounted(() => {
  if (props.isNew || (props.node.content === '' && !props.isRoot)) {
    startEdit()
  }
})

function startEdit() {
  isEditing.value = true
  editContent.value = props.node.content
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
  if (newContent !== props.node.content) {
    try {
      const updated = await nodeStore.updateNode(props.node.id, {
        content: newContent,
        properties: {
          ...props.node.properties,
          type: nodeType.value,
          checked: isChecked.value
        }
      })
      emit('node-updated', updated)
    } catch (error) {
      console.error('Failed to update node:', error)
      editContent.value = props.node.content
    }
  }
  isEditing.value = false
  showSlashMenu.value = false
}

function cancelEdit() {
  editContent.value = props.node.content
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
    if (showSlashMenu.value) return // Let slash menu handle it
    
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
      await deleteNode()
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
    const order = hasChildren.value ? props.node.children.length : 0
    const newNode = await nodeStore.createNode({
      content: '',
      parent_id: props.node.id,
      order: order
    })
    
    // Expand this node to show the new child
    isExpanded.value = true
    
    // Reload to get updated children
    await nodeStore.loadNodeWithChildren(props.node.id)
    
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
  if (!props.isRoot && props.node.parent_id) {
    try {
      const newNode = await nodeStore.createNode({
        content: '',
        parent_id: props.node.parent_id,
        order: props.node.order + 1
      })
      
      // Reload parent to show new sibling
      await nodeStore.loadNodeWithChildren(props.node.parent_id)
      
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
    // For root node, create a child instead
    createChildNode()
  }
}

async function deleteNode() {
  if (props.isRoot) return // Can't delete root
  
  try {
    await nodeStore.deleteNode(props.node.id)
    emit('node-deleted', props.node.id)
  } catch (error) {
    console.error('Failed to delete node:', error)
  }
}

async function indentNode() {
  if (props.isRoot) return
  
  // Find previous sibling to make it the parent
  const siblings = await getSiblings()
  const currentIndex = siblings.findIndex(n => n.id === props.node.id)
  
  if (currentIndex > 0) {
    const newParent = siblings[currentIndex - 1]
    try {
      await nodeStore.moveNode(props.node.id, newParent.id, newParent.children.length)
      // Reload the previous parent
      if (props.node.parent_id) {
        await nodeStore.loadNodeWithChildren(props.node.parent_id)
      }
    } catch (error) {
      console.error('Failed to indent node:', error)
    }
  }
}

async function outdentNode() {
  if (props.isRoot || !props.node.parent_id) return
  
  // Get parent's parent
  const parent = await nodeStore.loadNode(props.node.parent_id)
  if (!parent.parent_id) return // Can't outdent from root level
  
  try {
    // Move after parent
    await nodeStore.moveNode(props.node.id, parent.parent_id, parent.order + 1)
    // Reload both old and new parents
    await nodeStore.loadNodeWithChildren(props.node.parent_id)
    await nodeStore.loadNodeWithChildren(parent.parent_id)
  } catch (error) {
    console.error('Failed to outdent node:', error)
  }
}

async function getSiblings(): Promise<Node[]> {
  if (!props.node.parent_id) return []
  
  const parent = await nodeStore.loadNode(props.node.parent_id)
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
    await nodeStore.updateNode(props.node.id, {
      properties: {
        ...props.node.properties,
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
  let content = props.node.content
  
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
    :data-node-id="node.id"
    class="group relative node-editor"
    @mouseenter="showActions = true"
    @mouseleave="showActions = false"
  >
    <div class="flex items-start space-x-1 py-0.5">
      <!-- Drag handle -->
      <div 
        v-if="!isRoot && showActions" 
        class="opacity-0 group-hover:opacity-30 hover:!opacity-60 cursor-move p-1"
      >
        <GripVertical class="h-4 w-4" />
      </div>
      
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
          <span v-if="node.content" v-html="formattedContent" />
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
      
      <!-- Actions -->
      <div 
        v-if="showActions && !isEditing" 
        class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        <Button
          variant="ghost"
          size="icon"
          class="h-6 w-6 p-0"
          @click="createChildNode"
          title="Add child node"
        >
          <Plus class="h-4 w-4" />
        </Button>
        <Button
          v-if="!isRoot"
          variant="ghost"
          size="icon"
          class="h-6 w-6 p-0 hover:text-destructive"
          @click="deleteNode"
          title="Delete node"
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    </div>
    
    <!-- Children -->
    <div v-if="isExpanded && hasChildren" class="ml-6 border-l-2 border-accent pl-2">
      <template v-for="child in childNodes" :key="child?.id || 'unknown'">
        <Node
          v-if="child"
          :node="child"
          :level="level + 1"
          @node-updated="emit('node-updated', $event)"
          @node-created="emit('node-created', $event)"
          @node-deleted="emit('node-deleted', $event)"
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