import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import GameDownload from '@/views/DownloadSubView/GameDownload.vue'
import ModDownload from '@/views/DownloadSubView/ModDownload.vue'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/home',
    },
    {
      path: '/home',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/download',
      name: 'download',
      component: () => import('@/views/DownloadView.vue'),
    },
    {
      path: '/link',
      name: 'link',
      component: () => import('@/views/LinkView.vue'),
    },
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
    },
    {
      path: '/more',
      name: 'more',
      component: () => import('@/views/MoreView.vue')
    }
  ],
})

export default router
