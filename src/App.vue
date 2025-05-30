<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import TitleMessage from './components/TitleMessage.vue'
import { ref, onMounted, watch, computed } from 'vue';
import IconLaunch from './components/icons/IconLaunch.vue';
import IconDownload from './components/icons/IconDownload.vue';
import IconLink from './components/icons/IconLink.vue';
import IconSetup from './components/icons/IconSetup.vue';
import IconMore from './components/icons/IconMore.vue';
import router from './router';
import viewDic from './SubViewDic';

const navItems = [
  { to: '/home', icon: IconLaunch, label: '启动' },
  { to: '/download', icon: IconDownload, label: '下载' },
  { to: '/link', icon: IconLink, label: '联机' },
  { to: '/setup', icon: IconSetup, label: '设置' },
  { to: '/more', icon: IconMore, label: '更多' },
]

const asideWidth = ref(120)
const asideRef = ref<HTMLElement>()
let observer: MutationObserver | null = null
const subNavSelect = ref<number>(0)

function updateAsideBackgroundWidth() {
  asideWidth.value = asideRef.value!.scrollWidth
}

onMounted(() => {
  observer = new MutationObserver(updateAsideBackgroundWidth)
  observer.observe(asideRef.value!, { childList: true, })
  updateAsideBackgroundWidth()
})

watch(subNavSelect, (to, from) => {
  console.log(`from ${from} to ${to}`)
})

const currentSubView = computed(() => {
  let currentView = router.currentRoute.value.name as keyof typeof viewDic;
  if (currentView && viewDic[currentView]) {
    return viewDic[currentView][subNavSelect.value]
  }
  else { return undefined }
})

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
        <div><img src="@/assets/icons/最小化.svg" /></div>
        <div><img src="@/assets/icons/关闭.svg" /></div>
      </div>
    </header>

    <main id="current">
      <div class="side-nav-background" :style="{ width: asideWidth + 'px' }" />
      <div ref="asideRef" class="side-nav-content">
        <RouterView @nav-button="(i: number) => { subNavSelect = i }" />
      </div>

      <article>
        <component :is="currentSubView"></component>
      </article>
    </main>
  </main>
</template>

<style scoped>
main#current {
  flex: 1 1 0;
  display: flex;
  height: 0;
}

main#current .side-nav-background {
  overflow: auto;
  background: rgba(255, 255, 255, 1);
  box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.15);
  transition: width 0.3s cubic-bezier(.4, 2, .6, 1);
}

main#current .side-nav-content {
  position: fixed;
  padding: 10px;
  /* min-width: 100px; */
}

main#current article {
  flex: 1 1 0;
  background: rgba(255, 255, 255, 0.1);
  overflow: auto;
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

.left {
  justify-self: start;
  display: flex;
  gap: 10px;
}

.right {
  justify-self: end;
  display: flex;
  gap: 4px;
}

/* 窗口控制按钮外面的圆形 */
.right div {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  transition: background-color 0.4s;
}

.right div:hover {
  background-color: rgba(255, 255, 255, 0.25);
}

.right div:active {
  transform: scale(0.9);
}

/* 导航栏 */
header #main-nav {
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
}

#main-nav a:hover {
  background-color: rgba(255, 255, 255, 0.25);
}

#main-nav a:active {
  background-color: rgba(255, 255, 255, 0.5);
}

#main-nav a.router-link-exact-active {
  background-color: white;
  color: var(--color-titlebar);
}
</style>
