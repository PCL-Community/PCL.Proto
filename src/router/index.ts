import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import LinkView from '@/views/LinkView'
import SetupView from '@/views/SetupView'
import MoreView from '@/views/MoreView'
import GameDownload from '@/views/DownloadSubViews/GameDownload.vue'
import ManualDownload from '@/views/DownloadSubViews/ManualDownload.vue'
import HomeSubView from '@/views/HomeSubView.vue'
import DownloadView from '@/views/DownloadView'
import JavaManage from '@/views/SetupSubViews/JavaSetup.vue'
import LaunchSetup from '@/views/SetupSubViews/LaunchSetup.vue'
import AboutAndThanks from '@/views/MoreSubViews/AboutAndThanks.vue'
import VersionSelect from '@/views/InstanceSelect'
import VersionSelectSubView from '@/views/InstanceSelectSubView.vue'
import VersionSetting from '@/views/VersionSetting'
import Overview from '@/views/InstanceSettingSubViews/Overview.vue'
import { selectedGame } from '@/util/gameLaunch'
import HomeNew from '@/views/HomeNew'

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
          component: HomeNew,
        },
      ],
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
          component: ManualDownload,
        },
      ],
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
      redirect: '/setup/launch',
      children: [
        {
          path: 'java',
          component: JavaManage,
        },
        {
          path: 'launch',
          component: LaunchSetup,
        },
      ],
    },
    {
      path: '/more',
      name: 'more',
      component: MoreView,
      redirect: '/more/about_and_thanks',
      children: [
        {
          path: 'about_and_thanks',
          component: AboutAndThanks,
        },
      ],
    },
    {
      path: '/version_select',
      name: 'version_select',
      component: VersionSelect,
      redirect: '/version_select/version_select_sub',
      meta: { isSubPage: true, title: '实例选择' }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'version_select_sub',
          component: VersionSelectSubView,
        },
      ],
    },
    {
      path: '/version_setting',
      name: 'version_setting',
      component: VersionSetting,
      redirect: '/version_setting/overview',
      meta: { isSubPage: true, title: '实例设置 - ' + selectedGame.name }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'overview',
          component: Overview,
        },
      ],
    },
  ],
})

export default router
