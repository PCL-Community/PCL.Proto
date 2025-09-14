import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/Home/HomeView.vue'
import LinkView from '@/views/Link/LinkView'
import SetupView from '@/views/Setup/SetupView'
import MoreView from '@/views/More/MoreView'
import ManualDownload from '@/views/Download/ManualDownload.vue'
import HomeSubView from '@/views/Home/HomeSubView.vue'
import DownloadView from '@/views/Download/DownloadView'
import JavaManage from '@/views/Setup/JavaSetup.vue'
import LaunchSetup from '@/views/Setup/LaunchSetup.vue'
import AboutAndThanks from '@/views/More/AboutAndThanks.vue'
import InstanceSelect from '@/views/InstanceSelect/InstanceSelect.tsx'
import InstanceSelectSubView from '@/views/InstanceSelect/InstanceSelectSubView.vue'
import InstanceSetting from '@/views/InstanceSetting/InstanceSetting'
import InstanceOverview from '@/views/InstanceSetting/InstanceOverview.vue'
// import { useSelectedInstance } from '@/stores/gameLaunch'
import HomeNew from '@/views/Home/HomeNew'
import PageComp from '@/views/Download/PageComp.vue'
import ResouceVersions from '@/views/Download/ResourceVersions.vue'
import WonderBox from '@/views/More/WonderBox'
import Dowloading from '@/views/Download/Dowloading.vue'
import InstanceNotFound from '@/views/InstanceSelect/InstanceNotFound.vue'

// const selectedInstance = useSelectedInstance()
const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      redirect: '/home',
    },
    {
      path: '/home',
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
          component: () => import('@/views/Download/GameDownload.tsx'),
        },
        {
          path: 'manual',
          component: ManualDownload,
        },
        {
          path: 'mod',
          component: PageComp,
          meta: { project_type: 'mod' }
        },
        {
          path: 'modpack',
          component: PageComp,
          meta: { project_type: 'modpack' }
        },
        {
          path: 'resourcepack',
          component: PageComp,
          meta: { project_type: 'resourcepack' }
        },
        {
          path: 'shader',
          component: PageComp,
          meta: { project_type: 'shader' }
        }
      ],
    },
    {
      path: '/link',
      name: 'link',
      component: LinkView,
      redirect: '/link/lobby',
      children: [
        {
          path: 'lobby',
          component: () => import('@/views/Link/LinkLobby.vue'),
        },
        {
          path: 'help',
          component: () => import('@/views/Link/LinkHelp.vue')
        }
      ],
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
        {
          path: 'wonder_box',
          component: WonderBox,
        }
      ],
    },
    {
      path: '/instance_select',
      component: InstanceSelect,
      meta: { isSubPage: true, title: 'home.instance_select' }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'instance_select_sub/:repository',
          component: InstanceSelectSubView,
        },
        {
          path: '',
          component: InstanceNotFound,
        }
      ],
    },
    {
      path: '/instance_setting',
      name: 'instance_setting',
      component: InstanceSetting,
      redirect: '/instance_setting/overview',
      meta: { isSubPage: true, title: 'home.instance_setting' /* + selectedInstance.name */ }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'overview',
          component: InstanceOverview,
        },
        {
          path: 'setting',
          component: () => import('@/views/InstanceSetting/InstanceSettingSetting.vue'),
        },
        {
          path: 'modify',
          component: () => import('@/views/InstanceSetting/InstanceModify.vue'),
        },
        {
          path: 'export',
          component: () => import('@/views/InstanceSetting/InstanceExport.vue'),
        },
        {
          path: 'save',
          component: () => import('@/views/InstanceSetting/Saves.vue'),
        },
      ],
    },
    {
      path: '/resouce',
      name: 'resouce',
      component: ResouceVersions,
      meta: { isSubPage: true, title: 'download.resource_download', fullPage: true },
    },
    {
      path: '/homepageeditor',
      name: 'homepageeditor',
      component: () => import('@/views/More/HomepageEditor.vue'),
      meta: { isSubPage: true, title: ' PCL 主页制作器', fullPage: true },
    },
    {
      path: '/downloading',
      name: 'downloading',
      component: Dowloading,
      meta: { fullPage: false },
    }
  ],
})

export default router
