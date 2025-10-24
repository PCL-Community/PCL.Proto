<script lang="ts" setup>
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TitleLogo from '@/assets/icons/TitleLogo.svg'
import TitleMinimize from '@/assets/icons/TitleMinimize.svg'
import TitleClose from '@/assets/icons/TitleClose.svg'
import ArrowLeft from '@/assets/icons/ArrowLeft.svg'
import useSideNavState from '@/stores/windowState'
import navItems from '@/layout/navItems'
import currentPlatform from '@/util/platform'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { debug } from '@tauri-apps/plugin-log'

const sideNavState = useSideNavState()
const appWindow = getCurrentWindow()

const isSubPage = ref(false)
const route = useRoute()
const router = useRouter()

const fromPage = ref<string>() // 在进入特殊页面时记录的来源页面
const title = ref<string>() // 页面标题
const isFullscreen = ref(false)

// 监听路由变化，判断是否为特殊页面，并记录来源页面
watch(
  () => route.fullPath,
  (newPath, oldPath) => {
    debug(`route change: ${oldPath} -> ${newPath}`)
    if (route.meta.isSubPage) {
      // 进入特殊页面时记录来源
      if (!isSubPage.value) {
        fromPage.value = oldPath
        title.value = route.meta.title as string
      }
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

appWindow.onResized(async () => {
  isFullscreen.value = await appWindow.isFullscreen()
})
</script>

<template lang="pug">
  header(data-tauri-drag-region)
    Transition(name="nav")
      .left.title(v-if="!isSubPage" :class="{ 'mac-margin-title': currentPlatform == 'macos' }")
        title-logo()
        .title-tag Proto
        .title-tag.dev dev
      .left(v-else :class="{ 'mac-margin-title': currentPlatform == 'macos' }")
        i.button-animated(@click="backClicked")
          arrow-left
        span.title-text(v-if="title" data-tauri-drag-region) {{ $t(title) }}


    Transition(name="nav")
      .center(v-if="!isSubPage" data-tauri-drag-region)
        nav#main-nav(:class="{ 'mac-margin-nav': currentPlatform == 'macos' }")
          RouterLink(v-for="item in navItems" :key="item.to" :to="item.to")
            component(:is="item.icon")
            | {{ $t(item.label) }}

    .right(v-if="currentPlatform != 'macos'")
      i.button-animated(@click="appWindow.minimize"): title-minimize
      i.button-animated(@click="appWindow.close"): title-close

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
  width: 100%;
  height: 48px;
  background: var(--color-titlebar);

  display: flex;
  align-items: center;
  padding: 0 18px;
  flex-shrink: 0;
  z-index: 4;
  position: relative;
  color: white;
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
  left: 18px;
  top: 0;
  height: 100%;
  align-items: center;
}

header .center {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

.mac-margin-title {
  margin-left: v-bind("isFullscreen ? '0px' : '64px'");
}

.mac-margin-nav {
  margin-left: v-bind("isFullscreen ? '0px' : '40px'");
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
  background-color: var(--color-background);
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
