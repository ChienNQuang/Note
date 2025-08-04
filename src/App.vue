<script setup lang="ts">
import { ref, onMounted, provide } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'

// App-level state
const isLoading = ref(true)
const isOnline = ref(true)

// Router
const router = useRouter()

// App initialization
onMounted(async () => {
  try {
    // Initialize the backend
    const version = await invoke('get_app_version')
    console.log('Note app initialized, version:', version)
    
    // Health check
    await invoke('health_check')
    console.log('Backend health check passed')
    
    isLoading.value = false
  } catch (error) {
    console.error('Failed to initialize app:', error)
    isLoading.value = false
  }
})

// Provide global app state
provide('isLoading', isLoading)
provide('isOnline', isOnline)

// Update document title based on route meta
router.afterEach((to) => {
  if (to.meta?.title) {
    document.title = to.meta.title as string
  }
})
</script>

<template>
  <div id="app" class="h-screen flex flex-col theme-transition">
    <!-- Loading Screen -->
    <div 
      v-if="isLoading"
      class="flex-center h-screen bg-gray-50 dark:bg-gray-900"
    >
      <div class="text-center">
        <div class="animate-spin w-8 h-8 border-2 border-primary-600 border-t-transparent rounded-full mx-auto mb-4"></div>
        <p class="text-gray-600 dark:text-gray-400">Loading Note app...</p>
      </div>
    </div>

    <!-- Main App -->
    <div 
      v-else
      class="flex-1 flex flex-col overflow-hidden"
    >
      <!-- Router View -->
      <router-view />
    </div>

    <!-- Development Info (only in dev mode) -->
    <div 
      v-if="$route.query.debug" 
      class="fixed bottom-4 right-4 bg-black bg-opacity-75 text-white text-xs p-2 rounded"
    >
      Route: {{ $route.path }} | Loading: {{ isLoading }}
    </div>
  </div>
</template>

<style>
/* Global styles are in style.css */

/* App-specific styles */
#app {
  min-height: 100vh;
  width: 100%;
}

/* Loading animation */
@keyframes spin {
  from {
    transform: rotate(0deg);
}
  to {
    transform: rotate(360deg);
}
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* Vue transition classes for route changes */
.router-enter-active,
.router-leave-active {
  transition: opacity 0.2s ease;
}

.router-enter-from,
.router-leave-to {
  opacity: 0;
  }
</style>