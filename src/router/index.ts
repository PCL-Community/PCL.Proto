import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import LinkView from '@/views/LinkView.vue'
import SetupView from '@/views/SetupView.vue'
import MoreView from '@/views/MoreView.vue'
import GameDownload from '@/views/DownloadSubView/GameDownload.vue'
import ManualDownload from '@/views/DownloadSubView/ManualDownload.vue'
import HomeSubView from '@/views/HomeSubView.vue'
import DownloadView from '@/views/DownloadView.vue'
import JavaManage from '@/views/SetupSubView/JavaManage.vue'


const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/home',
    },
    {
      path: '/home',
      component: HomeView,
      children: [
        {
          path: '',
          component: HomeSubView
        }
      ]
    },
    {
      path: '/download',
      component: DownloadView,
      redirect: '/download/game',
      children: [
        {
          path: 'game',
          component: GameDownload,
        },
        {
          path: 'manual',
          component: ManualDownload
        }
      ]
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
      redirect: '/setup/javamanage',
      children: [
        {
          path: 'javamanage',
          component: JavaManage
        }
      ]
    },
    {
      path: '/more',
      name: 'more',
      component: MoreView
    }
  ],
})

export default router
