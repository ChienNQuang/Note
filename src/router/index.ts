import type { RouteRecordRaw } from 'vue-router'

// Lazy load components for better performance
const JournalView = () => import('@/views/JournalView.vue')
const NodeView = () => import('@/views/NodeView.vue')
const Settings = () => import('@/views/Settings.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Journal',
    component: JournalView,
    meta: {
      title: 'Daily Note',
    },
  },
  {
    path: '/node/:nodeId',
    name: 'Node',
    component: NodeView,
    props: true,
    meta: {
      title: 'Note',
    },
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings,
    meta: {
      title: 'Settings',
    },
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    redirect: '/',
  },
] 