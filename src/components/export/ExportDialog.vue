<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'

interface Props {
  pageId: string
  pageTitle: string
  isOpen: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:isOpen': [value: boolean]
  'exported': [format: string, path: string]
}>()

// Component state
const isExporting = ref(false)
const exportError = ref<string | null>(null)
const lastExportedFormat = ref<string | null>(null)
const exportPreview = ref<string | null>(null)
const isGeneratingPreview = ref(false)

// Computed
const isVisible = computed({
  get: () => props.isOpen,
  set: (value) => emit('update:isOpen', value)
})

// Export functions
const exportToMarkdown = async () => {
  isExporting.value = true
  exportError.value = null

  try {
    // Get markdown content from backend
    const markdownContent = await invoke<string>('export_page_to_markdown', {
      pageId: props.pageId
    })

    // Show save dialog
    const filePath = await save({
      title: 'Export Page as Markdown',
      defaultPath: `${sanitizeFilename(props.pageTitle)}.md`,
      filters: [
        {
          name: 'Markdown',
          extensions: ['md']
        }
      ]
    })

    if (filePath) {
      // Write file
      await writeTextFile(filePath, markdownContent)
      
      lastExportedFormat.value = 'markdown'
      emit('exported', 'markdown', filePath)
      
      console.log('Exported to Markdown:', filePath)
    }
  } catch (error) {
    console.error('Markdown export failed:', error)
    exportError.value = 'Failed to export to Markdown: ' + (error as Error).message
  } finally {
    isExporting.value = false
  }
}

const exportToJSON = async () => {
  isExporting.value = true
  exportError.value = null

  try {
    // Get JSON content from backend
    const jsonContent = await invoke<string>('export_page_to_json', {
      pageId: props.pageId
    })

    // Show save dialog
    const filePath = await save({
      title: 'Export Page as JSON',
      defaultPath: `${sanitizeFilename(props.pageTitle)}.json`,
      filters: [
        {
          name: 'JSON',
          extensions: ['json']
        }
      ]
    })

    if (filePath) {
      // Write file
      await writeTextFile(filePath, jsonContent)
      
      lastExportedFormat.value = 'json'
      emit('exported', 'json', filePath)
      
      console.log('Exported to JSON:', filePath)
    }
  } catch (error) {
    console.error('JSON export failed:', error)
    exportError.value = 'Failed to export to JSON: ' + (error as Error).message
  } finally {
    isExporting.value = false
  }
}

const generatePreview = async (format: 'markdown' | 'json') => {
  isGeneratingPreview.value = true
  exportError.value = null

  try {
    let content: string
    
    if (format === 'markdown') {
      content = await invoke<string>('export_page_to_markdown', {
        pageId: props.pageId
      })
    } else {
      content = await invoke<string>('export_page_to_json', {
        pageId: props.pageId
      })
    }

    exportPreview.value = content
  } catch (error) {
    console.error('Preview generation failed:', error)
    exportError.value = 'Failed to generate preview: ' + (error as Error).message
  } finally {
    isGeneratingPreview.value = false
  }
}

// Helper functions
const sanitizeFilename = (filename: string): string => {
  return filename
    .replace(/[^a-z0-9]/gi, '_')
    .replace(/_{2,}/g, '_')
    .replace(/^_|_$/g, '')
    .toLowerCase()
}

const closeDialog = () => {
  isVisible.value = false
  exportPreview.value = null
  exportError.value = null
  lastExportedFormat.value = null
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    console.log('Copied to clipboard')
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
  }
}

// Format file size
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}
</script>

<template>
  <div
    v-if="isVisible"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50"
    @click.self="closeDialog"
  >
    <Card class="w-full max-w-2xl max-h-[90vh] overflow-hidden">
      <CardHeader>
        <div class="flex items-center justify-between">
          <div>
            <CardTitle>Export Page</CardTitle>
            <CardDescription>
              Export "{{ pageTitle }}" in different formats
            </CardDescription>
          </div>
          <Button variant="ghost" size="sm" @click="closeDialog">
            ✕
          </Button>
        </div>
      </CardHeader>

      <CardContent class="space-y-6 max-h-[70vh] overflow-auto">
        <!-- Export Options -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- Markdown Export -->
          <Card class="border-2 hover:border-primary transition-colors">
            <CardHeader class="pb-3">
              <div class="flex items-center space-x-3">
                <div class="text-2xl">📝</div>
                <div>
                  <CardTitle class="text-lg">Markdown</CardTitle>
                  <CardDescription class="text-sm">
                    Human-readable format with formatting
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent class="space-y-3">
              <div class="text-sm text-muted-foreground space-y-1">
                <div>• Preserves block hierarchy</div>
                <div>• Includes YAML frontmatter</div>
                <div>• Compatible with most editors</div>
              </div>
              <div class="flex space-x-2">
                <Button
                  @click="exportToMarkdown"
                  :disabled="isExporting"
                  class="flex-1"
                >
                  <span v-if="isExporting && lastExportedFormat !== 'json'">Exporting...</span>
                  <span v-else>Export .md</span>
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  @click="generatePreview('markdown')"
                  :disabled="isGeneratingPreview"
                >
                  Preview
                </Button>
              </div>
            </CardContent>
          </Card>

          <!-- JSON Export -->
          <Card class="border-2 hover:border-primary transition-colors">
            <CardHeader class="pb-3">
              <div class="flex items-center space-x-3">
                <div class="text-2xl">📄</div>
                <div>
                  <CardTitle class="text-lg">JSON</CardTitle>
                  <CardDescription class="text-sm">
                    Structured data format for import
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent class="space-y-3">
              <div class="text-sm text-muted-foreground space-y-1">
                <div>• Complete data structure</div>
                <div>• Includes all metadata</div>  
                <div>• Re-importable format</div>
              </div>
              <div class="flex space-x-2">
                <Button
                  @click="exportToJSON"
                  :disabled="isExporting"
                  class="flex-1"
                >
                  <span v-if="isExporting && lastExportedFormat !== 'markdown'">Exporting...</span>
                  <span v-else>Export .json</span>
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  @click="generatePreview('json')"
                  :disabled="isGeneratingPreview"
                >
                  Preview
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>

        <!-- Error Display -->
        <Card v-if="exportError" class="border-destructive">
          <CardContent class="p-4">
            <div class="flex items-center justify-between">
              <div class="text-destructive text-sm">
                {{ exportError }}
              </div>
              <Button
                variant="ghost"
                size="sm"
                @click="exportError = null"
              >
                ✕
              </Button>
            </div>
          </CardContent>
        </Card>

        <!-- Preview Section -->
        <div v-if="exportPreview" class="space-y-3">
          <Separator />
          
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-medium">Preview</h3>
            <div class="flex space-x-2">
              <Badge variant="outline">
                {{ formatBytes(exportPreview.length * 2) }}
              </Badge>
              <Button
                variant="outline"
                size="sm"
                @click="copyToClipboard(exportPreview)"
              >
                Copy
              </Button>
              <Button
                variant="ghost"
                size="sm"
                @click="exportPreview = null"
              >
                Close
              </Button>
            </div>
          </div>

          <Card class="max-h-96 overflow-auto">
            <CardContent class="p-4">
              <pre class="text-sm text-muted-foreground whitespace-pre-wrap font-mono">{{ exportPreview }}</pre>
            </CardContent>
          </Card>
        </div>

        <!-- Loading Preview -->
        <div v-if="isGeneratingPreview" class="space-y-3">
          <Separator />
          <div class="space-y-2">
            <Skeleton class="h-4 w-32" />
            <Skeleton class="h-32 w-full" />
          </div>
        </div>

        <!-- Export Tips -->
        <div class="text-xs text-muted-foreground space-y-2 border-t pt-4">
          <div class="font-medium">💡 Export Tips:</div>
          <div>• Markdown files can be opened in any text editor or imported into other note apps</div>
          <div>• JSON exports preserve complete data structure and can be re-imported later</div>
          <div>• Large pages may take a moment to export</div>
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<style scoped>
/* Custom scrollbar for preview */
pre {
  scrollbar-width: thin;
  scrollbar-color: hsl(var(--border)) transparent;
}

pre::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

pre::-webkit-scrollbar-track {
  background: transparent;
}

pre::-webkit-scrollbar-thumb {
  background-color: hsl(var(--border));
  border-radius: 3px;
}

pre::-webkit-scrollbar-thumb:hover {
  background-color: hsl(var(--border));
}
</style> 