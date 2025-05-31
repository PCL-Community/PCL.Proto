<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import TitleMessage from './components/TitleMessage.vue'
import navItems from '@/router/navItems'
import { sideNavState } from '@/router/windowState';

</script>

<template>
  <!-- 图文介绍的标题部分 -->
  <TitleMessage v-if="true" />

  <!-- 以下为主体部分 -->
  <main id="main-window">
    <header>
      <div class="left">
        <img src="@/assets/icons/TitleLogo.svg" />
        <div class="title-tag">Proto</div>
        <div class="title-tag dev">dev</div>
      </div>
      <nav id="main-nav">
        <RouterLink v-for="item in navItems" :key="item.to" :to="item.to">
          <component :is="item.icon" />{{ item.label }}
        </RouterLink>
      </nav>
      <div class="right">
        <i class="button-animated"><img src="@/assets/icons/TitleMinimize.svg" /></i>
        <i class="button-animated"><img src="@/assets/icons/TitleClose.svg" /></i>
      </div>
    </header>
    <main id="current">
      <!-- 次级页面 -->
      <RouterView />
      <!-- 用作动画 -->
      <div class="side-nav-background" :style="{ width: sideNavState.width + 'px' }"></div>
    </main>
  </main>
</template>

<style scoped>
main#current {
  position: relative;
  height: 100%;
  width: 100%;
  z-index: 2;
}

main#current>.side-nav-background {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background: var(--color-background);
  box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.15);
  transition: width 0.4s cubic-bezier(.4, 2, .6, 1);
  z-index: -1;
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
}

.title-tag.dev {
  background: rgba(155, 240, 11, 1);
  color: rgba(52, 61, 74, 1);
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

#main-window header {
  top: 0;
  width: 100%;
  height: 48px;
  position: relative;
  background: var(--color-titlebar);

  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  padding: 0 18px;
  flex-shrink: 0;
}

#main-window header .left {
  justify-self: start;
  display: flex;
  gap: 10px;
}

#main-window header .right {
  justify-self: end;
  display: flex;
  gap: 4px;
}

/* 窗口控制按钮外面的圆形 */
#main-window header .right i {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  transition: background-color 0.4s;
}

#main-window header .right i:hover {
  background-color: rgba(255, 255, 255, 0.25);
}

/* 按钮缩放已放入 main.css 中 
.right i:active {
  transform: scale(0.9);
} */

/* 导航栏 */
#main-window header #main-nav {
  justify-self: center;
  display: inline-flex;
  gap: 5px;
}

/* 导航栏元素 */
header #main-nav a {
  width: 72px;
  height: 25px;
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
</style>
