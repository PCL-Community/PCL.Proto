<script lang="ts" setup>
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TitleLogo from '@/assets/icons/TitleLogo.svg'
import useSideNavState from '@/stores/windowState'
import navItems from '@/layout/navItems'
import { lastSubTabOf } from '@/layout/navMemory'
import currentPlatform from '@/util/platform'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { debug } from '@tauri-apps/plugin-log'
import { isTauri } from '@tauri-apps/api/core'
import PSvgIcon from './widget/PSvgIcon.vue'
import { pclText } from '@/api/pclLocale'
import { appearance } from '@/api/appearance'
import { invoke } from '@tauri-apps/api/core'

const sideNavState = useSideNavState()
const appWindow = isTauri() ? getCurrentWindow() : null

const isSubPage = ref(false)
const route = useRoute()
const router = useRouter()

const fromPage = ref<string>() // 在进入特殊页面时记录的来源页面
const title = ref<string>() // 页面标题
const isFullscreen = ref(false)
const logoImage = ref<string>()
const logoType = computed(() => appearance.values.UiLogoType)
const logoLeft = computed(() => logoType.value === '0' && appearance.values.CheckLogoLeft)

if (isTauri()) {
  void invoke<string | null>('get_title_logo')
    .then((image) => {
      logoImage.value = image ?? undefined
    })
    .catch((error) => console.error('Failed to load title logo:', error))
  window.addEventListener('title-logo-updated', (event) => {
    logoImage.value = (event as CustomEvent<string | null>).detail ?? undefined
  })
}

// 监听路由变化，判断是否为特殊页面，并记录来源页面
watch(
  () => route.fullPath,
  (newPath, oldPath) => {
    if (isTauri()) void debug(`route change: ${oldPath} -> ${newPath}`)
    if (route.meta.isSubPage) {
      // 进入特殊页面时记录来源
      if (!isSubPage.value) {
        fromPage.value = oldPath
      }
      title.value = route.meta.title as string
      isSubPage.value = true
    } else {
      isSubPage.value = false
      fromPage.value = undefined
    }
    if (route.meta.fullPage) {
      sideNavState.setWidth(0)
    }
  },
  { immediate: true },
)

const backClicked = () => {
  if (fromPage.value) {
    router.push(fromPage.value)
  } else {
    router.back()
  }
}

if (appWindow && currentPlatform === 'macos') {
  const updateFullscreen = async () => {
    isFullscreen.value = await appWindow.isFullscreen()
  }
  void updateFullscreen()
  void appWindow.onResized(updateFullscreen)
}
</script>

<template lang="pug">
  header(data-tauri-drag-region :class="{ 'windows-drag-region': currentPlatform === 'windows', 'mac-header': currentPlatform === 'macos' && !isFullscreen }")
    Transition(name="nav")
      .left.title(v-if="!isSubPage")
        title-logo(v-if="logoType === '1'")
        .title-tag(v-if="logoType === '1'") Proto
        .custom-title(v-else-if="logoType === '2'") {{ appearance.values.TextLogoText }}
        img.custom-logo(v-else-if="logoType === '3' && logoImage" :src="logoImage" alt="")
      .left(v-else)
        i.button-animated(@click="backClicked")
          PSvgIcon(name="lucide/arrow-left" :size="18")
        span.title-text(v-if="title" data-tauri-drag-region) {{ pclText(title) }}


    Transition(name="nav")
      .center(v-if="!isSubPage" :class="{ 'logo-left': logoLeft }" data-tauri-drag-region)
        nav#main-nav
          RouterLink(v-for="item in navItems" :key="item.to" :to="lastSubTabOf(item.to) ?? item.to")
            component(:is="item.icon")
            | {{ pclText(item.label) }}

    //- web 预览下 appWindow 为空，按钮仅作展示，点击为 no-op
    .right(v-if="currentPlatform != 'macos'")
      i.button-animated(@click="appWindow?.minimize()"): PSvgIcon(name="lucide/minus" :size="18")
      i.button-animated(@click="appWindow?.close()"): PSvgIcon(name="lucide/x" :size="18")

</template>

<style scoped>
.left > .title-text {
  color: white;
}

.nav-enter-active,
.nav-leave-active {
  transition: all 0.5s ease;
}

.nav-enter-from,
.nav-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

header {
  --header-left: 18px;
  width: 100%;
  height: 48px;
  background: var(--color-titlebar);

  display: flex;
  align-items: center;
  padding: 0 18px 0 var(--header-left);
  flex-shrink: 0;
  z-index: 4;
  position: relative;
  color: white;
}

header.mac-header {
  --header-left: 82px;
}

/* WebView2 native drag regions also support touch input. */
header.windows-drag-region,
.windows-drag-region .center,
.windows-drag-region .title-text {
  app-region: drag;
}

.windows-drag-region #main-nav,
.windows-drag-region .left i,
.windows-drag-region .right {
  app-region: no-drag;
}

header > .left.title {
  /* 屏蔽标题栏左侧鼠标事件 */
  pointer-events: none;
}

header * {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}

header .left {
  display: flex;
  gap: 10px;
  position: absolute;
  left: var(--header-left);
  top: 0;
  height: 100%;
  align-items: center;
}

header .center.logo-left {
  justify-content: flex-start;
}

.custom-title {
  color: white;
}
.custom-logo {
  max-width: 180px;
  max-height: 32px;
  object-fit: contain;
}

header .center {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

header .right {
  display: flex;
  gap: 4px;
  position: absolute;
  right: 18px;
  top: 0;
  height: 100%;
  align-items: center;
}

/* 窗口控制按钮外面的圆形 */
.right i,
.left i {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  transition: background-color 0.4s;
}

.right i:hover,
.left i:hover {
  background-color: rgba(255, 255, 255, 0.25);
}

/* 按钮缩放已放入 main.css 中
.right i:active {
  transform: scale(0.9);
} */

/* 导航栏 */
header #main-nav {
  /* justify-self: center; */
  display: inline-flex;
  gap: 5px;
}

/* 导航栏元素 */
header #main-nav a {
  min-width: 72px;
  height: 25px;
  padding-inline: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  border-radius: 13px;
  color: white;
  gap: 7px;
  transition: 0.4s;
}

#main-nav a:hover {
  background-color: rgba(255, 255, 255, 0.25);
}

#main-nav a:active {
  background-color: rgba(255, 255, 255, 0.5);
}

#main-nav a.router-link-active {
  background-color: white;
  color: var(--color-titlebar);
}

.title-tag {
  height: 20px;
  border-radius: 5px;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 0px;
  text-align: center;
  vertical-align: top;
  padding-inline: 5px;
  background: rgba(255, 255, 255, 1);
  color: var(--color-titlebar);
  cursor: default;
}

.title-tag.dev {
  background: rgba(155, 240, 11, 1);
  color: var(--vt-c-black);
}
</style>
