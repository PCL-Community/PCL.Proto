<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import TitleMessage from './components/TitleMessage.vue'
import { sideNavState, sideNavWidthStr } from '@/util/windowState';
import WindowHeader from './components/WindowHeader.vue';
import Modal from './components/Modal.vue';
</script>

<template lang="pug">
  //- 图文介绍的标题部分
  TitleMessage(v-if="true")
  //- 以下为主体部分
  #main-window
    WindowHeader()
    main
      //- 次级页面
      RouterView()
      //- 用作动画
      .side-nav-background
    
    Modal()

</template>

<style scoped>
main {
  position: relative;
  height: 100%;
  width: 100%;
  z-index: 2;
}

main>div.side-nav-background {
  position: absolute;
  left: 0;
  top: 0;
  width: v-bind('sideNavWidthStr');
  height: 100%;
  background: var(--color-background);
  box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.15);
  transition: width 0.4s cubic-bezier(.4, 2, .6, 1);
  z-index: -1;
}

#main-window {
  width: 814px;
  height: 464px;
  border-radius: 8px;
  background: linear-gradient(137.92deg, rgba(192, 196, 221, 1) 0%, rgba(182, 211, 220, 1) 100%);
  box-shadow: 0px 0px 18px rgba(0, 0, 0, 0.15);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
