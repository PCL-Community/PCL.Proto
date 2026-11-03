import { createRouter, createWebHashHistory } from 'vue-router'
import { lastSubTabOf, rememberSubTab } from '@/layout/navMemory'
import HomeView from '@/views/Home/PageLaunch.vue'
import SetupView from '@/views/Setup/PageSetupLeft'
import HomeSubView from '@/views/Home/PageLaunchRight.vue'
import DownloadView from '@/views/Download/PageDownloadLeft'
import JavaManage from '@/views/Setup/PageSetupJava.vue'
import InstanceSelect from '@/views/InstanceSelect/PageSelectLeft.vue'
import InstanceSelectSubView from '@/views/InstanceSelect/PageSelectRight.vue'
import InstanceSetting from '@/views/InstanceSetting/PageInstanceLeft'
// import { useSelectedInstance } from '@/stores/gameLaunch'
import ResouceVersions from '@/views/Download/PageDownloadCompDetail.vue'
import Dowloading from '@/views/Download/PageSpeed.vue'
import GameDownloadInner from '@/views/Download/PageDownloadInstall.vue'
import GameDownload from '@/views/Download/PageDownloadInstall.vue'

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
      redirect: () => lastSubTabOf('/download') ?? '/download/game',
      children: [
        {
          path: 'favorites',
          component: () => import('@/views/Download/PageDownloadCompFavorites.vue'),
        },
        {
          path: 'client',
          component: () => import('@/views/Download/PageDownloadInstall.vue'),
          props: { manual: true },
        },
        {
          path: 'optifine',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'OptiFine', website: 'https://www.optifine.net/' },
        },
        {
          path: 'forge',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'Forge', website: 'https://files.minecraftforge.net', supported: true },
        },
        {
          path: 'neoforge',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'NeoForge', website: 'https://neoforged.net' },
        },
        {
          path: 'cleanroom',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'Cleanroom', website: 'https://cleanroommc.com/zh/' },
        },
        {
          path: 'fabric',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'Fabric', website: 'https://www.fabricmc.net', supported: true },
        },
        {
          path: 'legacyfabric',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'LegacyFabric', website: 'https://legacyfabric.net/', supported: true },
        },
        {
          path: 'labymod',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'LabyMod', website: 'https://labymod.net' },
        },
        {
          path: 'liteloader',
          component: () => import('@/views/Download/PageDownloadVersionList.vue'),
          props: { loader: 'LiteLoader', website: 'https://www.liteloader.com' },
        },
        {
          path: 'datapack',
          component: () => import('@/views/Download/PageComp.vue'),
          props: { projectType: 'datapack' },
        },

        {
          path: 'game',
          children: [
            {
              path: '',
              component: GameDownload,
            },
            {
              path: 'inner',
              component: GameDownloadInner,
            },
          ],
        },
        {
          path: 'mod',
          component: () => import('@/views/Download/PageComp.vue'),
          props: { projectType: 'mod' },
        },
        {
          path: 'modpack',
          component: () => import('@/views/Download/PageComp.vue'),
          props: { projectType: 'modpack' },
        },
        {
          path: 'resourcepack',
          component: () => import('@/views/Download/PageComp.vue'),
          props: { projectType: 'resourcepack' },
        },
        {
          path: 'shader',
          component: () => import('@/views/Download/PageComp.vue'),
          props: { projectType: 'shader' },
        },
        {
          path: 'world',
          component: () => import('@/views/Download/PageComp.vue'),
          props: { projectType: 'world' },
        },
      ],
    },
    {
      path: '/setup',
      name: 'setup',
      component: SetupView,
      redirect: () => lastSubTabOf('/setup') ?? '/setup/launch',
      children: [
        { path: 'manage', component: () => import('@/views/Setup/PageSetupGameManage.vue') },
        {
          path: 'language',
          component: () => import('@/views/Setup/PageSetupLauncherLanguage.vue'),
        },
        { path: 'misc', component: () => import('@/views/Setup/PageSetupLauncherMisc.vue') },
        { path: 'link', component: () => import('@/views/Setup/PageSetupGameLink.vue') },
        { path: 'update', component: () => import('@/views/Setup/PageSetupUpdate.vue') },
        { path: 'feedback', component: () => import('@/views/Setup/PageSetupFeedback.vue') },
        { path: 'log', component: () => import('@/views/Setup/PageSetupLog.vue') },

        {
          path: 'java',
          component: JavaManage,
        },
        {
          path: 'launch',
          component: () => import('@/views/Setup/PageSetupLaunch.vue'),
        },
        {
          path: 'personalization',
          component: () => import('@/views/Setup/PageSetupUI.vue'),
        },
        {
          path: 'about',
          component: () => import('@/views/Setup/PageSetupAbout.vue'),
        },
      ],
    },
    {
      path: '/tools',
      name: 'tools',
      component: () => import('@/views/Tools/PageToolsLeft.ts'),
      redirect: () => lastSubTabOf('/tools') ?? '/tools/lobby',
      children: [
        {
          path: 'lobby',
          children: [
            {
              path: '',
              component: () => import('@/views/Tools/PageToolsGameLink.vue'),
            },
            {
              path: 'inner',
              component: () => import('@/views/Tools/PageToolsGameLinkConnected.vue'),
            },
          ],
        },
        {
          path: 'wonderbox',
          component: () => import('@/views/Tools/PageToolsTest.vue'),
        },
        {
          path: 'help',
          component: () => import('@/views/Tools/PageHelpDetail.vue'),
        },
      ],
    },
    {
      path: '/instance_select',
      component: InstanceSelect,
      meta: { isSubPage: true, title: 'Main.Title.InstanceSelect' }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'instance_select_sub/:repository',
          component: InstanceSelectSubView,
        },
        {
          path: '',
          component: InstanceSelectSubView,
        },
      ],
    },
    {
      path: '/instance_setting',
      name: 'instance_setting',
      component: InstanceSetting,
      redirect: '/instance_setting/overview',
      meta: { isSubPage: true, title: 'Proto.Title.InstanceSetup' /* + selectedInstance.name */ }, // 用于标识当前处于特殊子页面
      children: [
        {
          path: 'mod/disabled',
          component: () => import('@/views/InstanceSetting/PageInstanceModDisabled.vue'),
        },
        {
          path: 'mod/jarinjar',
          component: () => import('@/views/InstanceSetting/PageInstanceCompJarInJar.vue'),
        },
        {
          path: 'screenshot',
          component: () => import('@/views/InstanceSetting/PageInstanceScreenshot.vue'),
          meta: { resourceType: 'screenshot', downloadType: 'screenshot' },
        },
        {
          path: 'mod',
          component: () => import('@/views/InstanceSetting/PageInstanceCompResource.vue'),
          meta: { resourceType: 'Mod', downloadType: 'mod' },
        },
        {
          path: 'resourcepack',
          component: () => import('@/views/InstanceSetting/PageInstanceCompResource.vue'),
          meta: { resourceType: 'ResourcePack', downloadType: 'resourcepack' },
        },
        {
          path: 'shader',
          component: () => import('@/views/InstanceSetting/PageInstanceCompResource.vue'),
          meta: { resourceType: 'Shader', downloadType: 'shader' },
        },
        {
          path: 'schematic',
          component: () => import('@/views/InstanceSetting/PageInstanceCompResource.vue'),
          meta: { resourceType: 'Schematic', downloadType: 'schematic' },
        },
        {
          path: 'server',
          component: () => import('@/views/InstanceSetting/PageInstanceServer.vue'),
          meta: { resourceType: 'server', downloadType: 'server' },
        },

        {
          path: 'overview',
          component: () => import('@/views/InstanceSetting/PageInstanceOverall.vue'),
        },
        {
          path: 'setting',
          component: () => import('@/views/InstanceSetting/PageInstanceSetup.vue'),
        },
        {
          path: 'modify',
          component: () => import('@/views/InstanceSetting/PageInstanceInstall.vue'),
        },
        {
          path: 'export',
          component: () => import('@/views/InstanceSetting/PageInstanceExport.vue'),
        },
        {
          path: 'save',
          component: () => import('@/views/InstanceSetting/PageInstanceSaves.vue'),
        },
      ],
    },
    {
      path: '/instance_save',
      component: () => import('@/views/InstanceSetting/PageInstanceSavesLeft.vue'),
      redirect: '/instance_save/info',
      meta: { isSubPage: true, title: 'Proto.Title.Saves' },
      children: [
        {
          path: 'info',
          component: () => import('@/views/InstanceSetting/PageInstanceSavesInfo.vue'),
        },
        {
          path: 'datapack',
          component: () => import('@/views/InstanceSetting/PageInstanceSavesDatapack.vue'),
          meta: { downloadType: 'datapack' },
        },
      ],
    },
    {
      path: '/logs',
      component: () => import('@/views/Tools/PageLogLeft.vue'),
      redirect: '/logs/view',
      meta: { isSubPage: true, title: 'Proto.Title.Log' },
      children: [{ path: 'view', component: () => import('@/views/Tools/PageLogRight.vue') }],
    },
    {
      path: '/news',
      component: () => import('@/views/Home/PageHomepageNewsView.vue'),
      meta: { fullPage: true, isSubPage: true, title: 'Proto.Title.News' },
    },
    {
      path: '/resource',
      name: 'resource',
      component: ResouceVersions,
      meta: { isSubPage: true, title: 'Proto.Title.ResourceDownload', fullPage: true },
    },
    {
      path: '/homepageeditor',
      name: 'homepageeditor',
      component: () => import('@/views/Tools/HomepageEditor.vue'),
      meta: { isSubPage: true, title: 'Proto.Title.HomepageEditor', fullPage: true },
    },
    {
      path: '/downloading',
      name: 'downloading',
      component: Dowloading,
      meta: { fullPage: false, isSubPage: true, title: 'Main.Title.TaskManager' },
    },
  ],
})

// 记录每个一级标签页最后停留的二级标签页，便于切走再切回时恢复
router.afterEach((to) => {
  rememberSubTab(to.path)
})

export default router
