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
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/DownloadView.vue'),
      children: [
        {
          path: '',
          redirect: 'download/game',
        },
        {
          path: 'game',
          name: 'game-download',
          component: GameDownload,
        },
        {
          path: 'mod',
          name: 'mod-download',
          component: ModDownload
        }],
    },
  ],
})

export default router
