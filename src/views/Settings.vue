<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

const router = useRouter()

// Settings state (will be replaced with Pinia store in later phases)
const theme = ref('system')
const fontSize = ref(14)
const autoSave = ref(true)
const gitAutoCommit = ref(true)

function goBack() {
  router.go(-1)
}

function saveSettings() {
  // TODO: Implement settings persistence in later phases
  console.log('Settings saved:', {
    theme: theme.value,
    fontSize: fontSize.value,
    autoSave: autoSave.value,
    gitAutoCommit: gitAutoCommit.value
  })
}

function resetSettings() {
  theme.value = 'system'
  fontSize.value = 14
  autoSave.value = true
  gitAutoCommit.value = true
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <header class="border-b bg-card">
      <div class="container mx-auto px-4 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <Button 
              variant="ghost" 
              size="sm" 
              @click="goBack"
            >
              ← Back
            </Button>
            <h1 class="text-xl font-semibold text-foreground">
              Settings
            </h1>
          </div>
          
          <div class="flex items-center space-x-2">
            <Button 
              variant="outline" 
              size="sm"
              @click="resetSettings"
            >
              Reset
            </Button>  
            <Button 
              size="sm"
              @click="saveSettings"
            >
              Save
            </Button>
          </div>
        </div>
      </div>
    </header>

    <div class="container mx-auto px-4 py-8">
      <div class="max-w-2xl mx-auto space-y-8">
        
        <!-- Appearance Settings -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center space-x-2">
              <span>🎨</span>
              <span>Appearance</span>
            </CardTitle>
            <CardDescription>
              Customize the look and feel of your note-taking experience
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-6">
            <!-- Theme -->
            <div class="space-y-2">
              <label class="text-sm font-medium text-foreground">
                Theme
              </label>
              <select 
                v-model="theme"
                class="flex h-10 w-full max-w-xs rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="system">System</option>
              </select>
              <p class="text-sm text-muted-foreground">
                Choose your preferred color theme
              </p>
            </div>

            <!-- Font Size -->
            <div class="space-y-2">
              <label class="text-sm font-medium text-foreground">
                Font Size: {{ fontSize }}px
              </label>
              <input
                v-model.number="fontSize"
                type="range"
                min="10"
                max="24"
                class="w-full max-w-xs h-2 bg-secondary rounded-lg appearance-none cursor-pointer"
              />
              <p class="text-sm text-muted-foreground">
                Adjust the font size for better readability
              </p>
            </div>
          </CardContent>
        </Card>

        <!-- Editor Settings -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center space-x-2">
              <span>✏️</span>
              <span>Editor</span>
            </CardTitle>
            <CardDescription>
              Configure your editing experience and preferences
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-6">
            <!-- Auto Save -->
            <div class="flex items-center justify-between">
              <div class="space-y-1">
                <label class="text-sm font-medium text-foreground">
                  Auto Save
                </label>
                <p class="text-sm text-muted-foreground">
                  Automatically save changes as you type
                </p>
              </div>
              <label class="relative inline-flex cursor-pointer">
                <input
                  v-model="autoSave"
                  type="checkbox"
                  class="sr-only peer"
                />
                <div class="w-11 h-6 bg-input peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-ring rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-background after:border-border after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
              </label>
            </div>

            <div class="text-sm text-muted-foreground bg-muted p-3 rounded-lg border">
              <strong>Coming in Phase 1.3:</strong> Block-based editing with advanced features
            </div>
          </CardContent>
        </Card>

        <!-- Git Settings -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center space-x-2">
              <span>🔄</span>
              <span>Git Integration</span>
            </CardTitle>
            <CardDescription>
              Version control and backup settings
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-6">
            <!-- Auto Commit -->
            <div class="flex items-center justify-between opacity-50">
              <div class="space-y-1">
                <label class="text-sm font-medium text-foreground">
                  Auto Commit
                </label>
                <p class="text-sm text-muted-foreground">
                  Automatically commit changes to Git
                </p>
              </div>
              <label class="relative inline-flex cursor-not-allowed">
                <input
                  v-model="gitAutoCommit"
                  type="checkbox"
                  class="sr-only peer"
                  disabled
                />
                <div class="w-11 h-6 bg-input rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-background after:border-border after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
              </label>
            </div>

            <div class="text-sm text-muted-foreground bg-muted p-3 rounded-lg border border-yellow-200 bg-yellow-50">
              <strong>Coming in Phase 2:</strong> Full Git integration with automatic versioning
            </div>
          </CardContent>
        </Card>

        <!-- Collaboration Settings -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center space-x-2">
              <span>👥</span>
              <span>Collaboration</span>
            </CardTitle>
            <CardDescription>
              Real-time collaboration and sharing settings
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div class="text-sm text-muted-foreground bg-muted p-4 rounded-lg border border-purple-200 bg-purple-50">
              <div class="flex items-center space-x-2 mb-2">
                <span>🚧</span>
                <strong>Coming in Phase 3</strong>
              </div>
              <ul class="space-y-1 ml-6 list-disc">
                <li>Real-time collaboration</li>
                <li>User management</li>
                <li>Conflict resolution</li>
                <li>WebSocket sync</li>
              </ul>
            </div>
          </CardContent>
        </Card>

        <!-- About Section -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center space-x-2">
              <span>ℹ️</span>
              <span>About</span>
            </CardTitle>
            <CardDescription>
              Application information and current status
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div class="space-y-3 text-sm">
              <div class="flex justify-between">
                <span class="text-muted-foreground">Version:</span>
                <span class="text-foreground font-medium">0.1.0</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Platform:</span>
                <span class="text-foreground font-medium">Tauri + Vue 3</span>
              </div>
              <div class="flex justify-between">  
                <span class="text-muted-foreground">Backend:</span>
                <span class="text-foreground font-medium">Rust</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Phase:</span>
                <span class="text-green-600 font-medium">1.2 - Page Management</span>
              </div>
            </div>

            <div class="mt-4 pt-4 border-t border-border">
              <p class="text-xs text-muted-foreground text-center">
                Built with ❤️ for local-first note-taking
              </p>
            </div>
          </CardContent>
        </Card>

      </div>
    </div>
  </div>
</template>

 