import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import DownloadView from '@/views/DownloadView.vue'
import LinkView from '@/views/LinkView.vue'
import SetupView from '@/views/SetupView.vue'
import MoreView from '@/views/MoreView.vue'

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
      component: HomeView
    },
    {
      path: '/download',
      name: 'download',
      component: DownloadView
    },
    {
      path: '/link',
      name: 'link',
      component: LinkView,
    },
    {
      path: '/setup',
      name: 'setup',
      component: SetupView,
    },
    {
      path: '/more',
      name: 'more',
      component: MoreView
    }
  ],
})

export default router
