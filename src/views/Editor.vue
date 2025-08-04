<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import PageEditor from '@/components/editor/PageEditor.vue'

const router = useRouter()
const route = useRoute()

const pageId = ref(route.params.pageId as string)
const pageTitle = ref('Untitled Page')
const isEditing = ref(false)

onMounted(() => {
  console.log('Editor loaded for page:', pageId.value)
  
  // If it's a new page, start in editing mode
  if (pageId.value === 'new') {
    isEditing.value = true
    pageTitle.value = 'New Page'
  }
})

function goHome() {
  router.push('/')
}

function toggleEdit() {
  isEditing.value = !isEditing.value
}

function savePage() {
  // TODO: Implement page saving in later phases
  console.log('Saving page:', pageTitle.value)
  isEditing.value = false
}
</script>

<template>
  <div class="min-h-screen flex flex-col bg-background">
    <!-- Header -->
    <header class="border-b bg-card">
      <div class="container mx-auto px-4 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <Button
              variant="ghost"
              size="sm"
              @click="goHome"
            >
              ← Back
            </Button>
            
            <div class="flex items-center space-x-2">
              <input
                v-if="isEditing"
                v-model="pageTitle"
                class="text-xl font-semibold bg-transparent border-none outline-none text-foreground"
                placeholder="Page title..."
                @keydown.enter="savePage"
                @keydown.escape="isEditing = false"
              />
              <h1
                v-else
                class="text-xl font-semibold text-foreground cursor-pointer"
                @click="toggleEdit"
              >
                {{ pageTitle }}
              </h1>
              
              <Button
                variant="ghost"
                size="sm"
                @click="toggleEdit"
              >
                ✏️
              </Button>
            </div>
          </div>

          <div class="flex items-center space-x-2">
            <Button
              v-if="isEditing"
              size="sm"
              @click="savePage"
            >
              Save
            </Button>
            
            <Button
              variant="ghost"
              size="sm"
              disabled
            >
              ⚙️
            </Button>
          </div>
        </div>
      </div>
    </header>

    <!-- Editor Area -->
    <main class="flex-1 flex overflow-hidden">
      <!-- Sidebar -->
      <aside class="w-64 bg-muted border-r p-4 space-y-4">
        <Card>
          <CardHeader class="pb-3">
            <CardTitle class="text-sm">Block Editor</CardTitle>
          </CardHeader>
          <CardContent class="text-sm text-muted-foreground space-y-2">
            <div class="text-green-600 font-medium">✅ Phase 1.3 Complete!</div>
            <ul class="space-y-1 text-xs">
              <li>• Block-based editing</li>
              <li>• Hierarchical structure</li>
              <li>• Keyboard navigation</li>
              <li>• Multiple block types</li>
              <li>• Drag & drop (basic)</li>
            </ul>
          </CardContent>
        </Card>

        <Card>
          <CardHeader class="pb-3">
            <CardTitle class="text-sm">Block Types</CardTitle>
          </CardHeader>
          <CardContent class="space-y-2 text-xs text-muted-foreground">
            <div><span class="font-mono bg-background px-1 rounded">Cmd+1</span> Heading</div>
            <div><span class="font-mono bg-background px-1 rounded">Cmd+`</span> Code</div>
            <div><span class="font-mono bg-background px-1 rounded">Cmd+Q</span> Quote</div>
            <div><span class="font-mono bg-background px-1 rounded">Tab</span> Indent</div>
            <div><span class="font-mono bg-background px-1 rounded">S+Tab</span> Unindent</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader class="pb-3">
            <CardTitle class="text-sm">Coming Next</CardTitle>
          </CardHeader>
          <CardContent class="space-y-1 text-xs text-muted-foreground">
            <div>🗃️ SQLite Storage (Phase 1.4)</div>
            <div>📤 Markdown Export (Phase 1.4)</div>
            <div>🔄 Git Integration (Phase 2)</div>
            <div>👥 Collaboration (Phase 3)</div>
          </CardContent>
        </Card>
      </aside>

      <!-- Main Editor -->
      <div class="flex-1 flex flex-col">
        <!-- Editor Toolbar -->
        <div class="bg-background border-b p-2">
          <div class="flex items-center space-x-2">
            <Button variant="ghost" size="sm" disabled>
              <strong>B</strong>
            </Button>
            <Button variant="ghost" size="sm" disabled>
              <em>I</em>
            </Button>
            <Button variant="ghost" size="sm" disabled>
              🔗
            </Button>
            <div class="border-l border-border h-6 mx-2"></div>
            <Button variant="ghost" size="sm" disabled>
              &lt;/&gt;
            </Button>
            <Button variant="ghost" size="sm" disabled>
              💬
            </Button>
          </div>
        </div>

        <!-- Editor Content -->
        <div class="flex-1 overflow-hidden">
          <PageEditor :page-id="pageId" />
        </div>
      </div>
    </main>
  </div>
</template>

 