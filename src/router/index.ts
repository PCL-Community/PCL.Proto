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
          component: HomeNew,
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
      meta: { isSubPage: true, title: '实例设置' /* + selectedInstance.name */ }, // 用于标识当前处于特殊子页面
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
      meta: { isSubPage: true, title: '资源下载', fullPage: true },
    },
    {
      path: '/homepageeditor',
      name: 'homepageeditor',
      component: () => import('@/views/HomepageEditor.vue'),
      meta: { isSubPage: true, title: ' PCL 主页制作器', fullPage: true },
    }
  ],
})

export default router
