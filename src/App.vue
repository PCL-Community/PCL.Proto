<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { vPageEntrance } from './util/cardDropAnimate'
import { useTemplateRef } from 'vue'
import { usePageTransition } from './composables/usePageTransition'
import TitleMessage from './components/website/TitleMessage.vue'
import useSideNavState from '@/stores/windowState'
import WindowHeader from './components/WindowHeader.vue'
import Modal from './components/Modal.vue'
import SideTip from './components/SideTip.vue'
import { isTauri } from '@tauri-apps/api/core'
import FloatButton from './components/widget/FloatButton.vue'
import { useFloatButton } from './composables/useFloatButton'

const route = useRoute()
const sideNavState = useSideNavState()
const { floatButtonState } = useFloatButton()
const website = !isTauri()
usePageTransition(useTemplateRef<HTMLElement>('page'))
</script>

<template>
  <TitleMessage v-if="website" />
  <!-- 主体窗口 -->
  <div id="main-window" :class="{ 'website-preview': website }">
    <!-- 标题框 -->
    <WindowHeader />
    <!-- 主页面 -->
    <div id="page" ref="page" v-page-entrance="route.path">
      <!-- 页面 -->
      <RouterView />
      <!-- 用作动画 -->
      <div id="side-nav-background" />
    </div>
    <!-- 一些浮动内容 -->
    <Modal />
    <SideTip />
    <FloatButton v-bind="floatButtonState" />
  </div>
</template>

<style>
#page {
  position: relative;
  height: 100%;
  z-index: 2;
  width: 100%;
  overflow: hidden;
}

#side-nav-background {
  position: absolute;
  left: 0;
  top: 0;
  width: v-bind('sideNavState.sideNavWidthStr');
  height: 100%;
  background: var(--color-background-soft);
  box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.15);
  transition: width 0.4s cubic-bezier(0.4, 1.4, 0.6, 1);
  z-index: -1;
}

#main-window {
  width: 100lvw;
  height: 100lvh;
  /* width: 814px; */
  /* height: 464px; */
  /* border-radius: 8px; */
  background: var(--base-background);
  /* box-shadow: var(--box-shadow); */
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
}

#main-window.website-preview {
  width: 814px;
  height: 464px;
  margin: 0 auto;
  border-radius: 8px;
  box-shadow: var(--box-shadow);
}

html.website,
.website body {
  overflow: auto;
}

.website #app {
  width: max-content;
  min-width: 100%;
  height: auto;
  padding: 32px 32px 64px;
}

@supports selector(::-webkit-scrollbar) {
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  ::-webkit-scrollbar-track {
    background: transparent;
    margin: 2px;
  }

  ::-webkit-scrollbar-thumb {
    background-color: color-mix(in srgb, var(--ce-color-4) 50%, transparent);
    background-clip: padding-box;
    border: 2px solid transparent;
    border-radius: 999px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background-color: color-mix(in srgb, var(--ce-color-3) 90%, transparent);
  }

  ::-webkit-scrollbar-thumb:active {
    background-color: var(--ce-color-4);
  }
}

@supports (scrollbar-width: thin) and (not selector(::-webkit-scrollbar)) {
  * {
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--ce-color-4) 50%, transparent) transparent;
  }
}
</style>
