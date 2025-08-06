import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/Home/HomeView.vue'
import LinkView from '@/views/Link/LinkView'
import SetupView from '@/views/Setup/SetupView'
import MoreView from '@/views/More/MoreView'
import GameDownload from '@/views/Download/GameDownload.vue'
import ManualDownload from '@/views/Download/ManualDownload.vue'
import HomeSubView from '@/views/Home/HomeSubView.vue'
import DownloadView from '@/views/Download/DownloadView'
import JavaManage from '@/views/Setup/JavaSetup.vue'
import LaunchSetup from '@/views/Setup/LaunchSetup.vue'
import AboutAndThanks from '@/views/More/AboutAndThanks.vue'
import InstanceSelect from '@/views/InstanceSelect/InstanceSelect'
import InstanceSelectSubView from '@/views/InstanceSelect/InstanceSelectSubView.vue'
import InstanceSetting from '@/views/InstanceSetting/InstanceSetting'
import Overview from '@/views/InstanceSetting/Overview.vue'
import { selectedInstance } from '@/util/gameLaunch'
import HomeNew from '@/views/Home/HomeNew'
import InstanceSettingSetting from '@/views/InstanceSetting/InstanceSettingSetting.vue'

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
      children: [
        {
          path: '',
          component: HomeSubView,
        },
      ],
    },
    {
      path: '/download',
      name: 'download',
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
      path: '/instance_select',
      name: 'instance_select',
      component: InstanceSelect,
      redirect: '/instance_select/instance_select_sub',
      meta: { isSubPage: true, title: '实例选择' }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'instance_select_sub',
          component: InstanceSelectSubView,
        },
      ],
    },
    {
      path: '/instance_setting',
      name: 'instance_setting',
      component: InstanceSetting,
      redirect: '/instance_setting/overview',
      meta: { isSubPage: true, title: '实例设置 - ' + selectedInstance.name }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'overview',
          component: Overview,
        },
        {
          path: 'setting',
          component: InstanceSettingSetting,
        }
      ],
    },
  ],
})

export default router
