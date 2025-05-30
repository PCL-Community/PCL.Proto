import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import GameDownload from '@/views/DownloadSubView/GameDownload.vue'
import ModDownload from '@/views/DownloadSubView/ModDownload.vue'
import HomeSubView from '@/views/HomeSubView/HomeSubView.vue'
import DownloadView from '@/views/DownloadView.vue'

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
      components: {
        aside: HomeView,
        default: HomeSubView,
      }
    },
    {
      path: '/download',
      name: 'download',
      components: {
        aside: DownloadView,
        default: GameDownload
      },
    },
    {
      path: '/link',
      name: 'link',
      components: { aside: () => import('@/views/LinkView.vue') },
    },
    {
      path: '/setup',
      name: 'setup',
      components: { aside: () => import('@/views/SetupView.vue') },
    },
    {
      path: '/more',
      name: 'more',
      components: { aside: () => import('@/views/MoreView.vue') }
    }
  ],
})

export default router
