<script setup lang="ts">
import { RouterView } from 'vue-router'
// import TitleMessage from './components/website/TitleMessage.vue'
import useSideNavState from '@/stores/windowState'
import WindowHeader from './components/WindowHeader.vue'
import Modal from './components/Modal.vue'
import SideTip from './components/SideTip.vue'
import { listen } from '@tauri-apps/api/event'
import { useModal } from './composables/useModal'
// 引入 SvgDrawer 组件
// import SvgDrawer from './components/website/SVGDrawer.vue'
const sideNavState = useSideNavState()
const modal = useModal()

listen<string>('modal-open', (event) => {
  modal.open({
    title: '❌后端错误',
    content: event.payload,
    buttons: [{ type: 'tint', content: '确认' }],
  })
})
</script>

<template lang="pug">
  //- 图文介绍的标题部分
  //- TitleMessage(v-if="true")
  //- 以下为主体部分
  #main-window
    //- 标题框
    WindowHeader()
    //- 主页面
    #page
      //- 页面
      RouterView()
      //- 用作动画
      .side-nav-background
    //- 一些浮动内容
    Modal()
    SideTip()
  //- 添加 SvgDrawer 组件
  //- SvgDrawer()
</template>

<style scoped>
#page {
  position: relative;
  height: 100%;
  z-index: 2;
  width: 100%;
  overflow: hidden;
}

#page > div.side-nav-background {
  position: absolute;
  left: 0;
  top: 0;
  width: v-bind('sideNavState.sideNavWidthStr');
  height: 100%;
  background: var(--color-background-soft);
  box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.15);
  transition: width 0.4s cubic-bezier(0.4, 1.8, 0.6, 1);
  z-index: -1;
}

#main-window {
  width: 100vi;
  height: 100vh;
  /* width: 814px; */
  /* height: 464px; */
  /* border-radius: 8px; */
  background: linear-gradient(137.92deg, rgba(192, 196, 221, 1) 0%, rgba(182, 211, 220, 1) 100%);
  /* box-shadow: var(--box-shadow); */
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
}
</style>

<style>
@supports selector(::-webkit-scrollbar) {
  ::-webkit-scrollbar {
    width: 6px;
  }

  ::-webkit-scrollbar-track {
    background: transparent;
  }

  ::-webkit-scrollbar-thumb {
    background-color: var(--color-tint-lightist);
    border-radius: 3px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: var(--color-tint-light);
  }
}

@supports (scrollbar-width: thin) {
  html {
    scrollbar-width: thin;
    scrollbar-color: var(--color-tint-lightist) transparent;
  }
}
</style>
