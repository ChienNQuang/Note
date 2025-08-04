import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './style.css'

// Import routes
import { routes } from './router'

// Create Vue app
const app = createApp(App)

// Create Pinia store
const pinia = createPinia()
app.use(pinia)

// Create Vue Router
const router = createRouter({
  history: createWebHistory(),
  routes,
})
app.use(router)

// Global error handler
app.config.errorHandler = (err, _vm, info) => {
  console.error('Vue error:', err, info)
  // TODO: Add error reporting service in production
}

// Global warning handler (development only)
if (import.meta.env.DEV) {
  app.config.warnHandler = (msg, _vm, trace) => {
    console.warn('Vue warning:', msg, trace)
  }
}

// Mount the app
app.mount('#app')
