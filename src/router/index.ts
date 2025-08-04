import type { RouteRecordRaw } from 'vue-router'

// Lazy load components for better performance
const Home = () => import('@/views/Home.vue')
const Editor = () => import('@/views/Editor.vue')
const Settings = () => import('@/views/Settings.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    meta: {
      title: 'Note - Home',
    },
  },
  {
    path: '/page/:pageId',
    name: 'Editor',
    component: Editor,
    props: true,
    meta: {
      title: 'Note - Editor',
    },
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings,
    meta: {
      title: 'Note - Settings',
    },
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    redirect: '/',
  },
] 