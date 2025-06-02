<script lang="ts" setup>
import navItems from '@/components/navItems'
import currentPlatform from '@/util/platform';
import { onMounted } from 'vue';

// onMounted(() => {
//   if (currentPlatform.value == 'macos') {
//     console.log('h')
//   }
// })
</script>

<template lang="pug">
  header(data-tauri-drag-region)
    .left(:class="{'mac-margin-title': currentPlatform == 'macos'}")
      img(src="@/assets/icons/TitleLogo.svg" data-tauri-drag-region)
      .title-tag Proto
      .title-tag.dev dev

    nav#main-nav(:class="{'mac-margin-nav': currentPlatform == 'macos'}")
      RouterLink(v-for="item in navItems" :key="item.to" :to="item.to")
        component(:is="item.icon")
        | {{ item.label }}

    .right(v-if="currentPlatform != 'macos'")
      each icon in ['TitleMinimize', 'TitleClose']
        i.button-animated: img(src=`@/assets/icons/${icon}.svg`)

</template>

<style scoped>
header {
  width: 100%;
  height: 48px;
  background: var(--color-titlebar);

  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  padding: 0 18px;
  flex-shrink: 0;
}

header .left {
  justify-self: start;
  display: flex;
  gap: 10px;
}

.mac-margin-title {
  margin-left: 64px;
}

header .right {
  justify-self: end;
  display: flex;
  gap: 4px;
}

/* 窗口控制按钮外面的圆形 */
header .right i {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  transition: background-color 0.4s;
}

header .right i:hover {
  background-color: rgba(255, 255, 255, 0.25);
}

/* 按钮缩放已放入 main.css 中 
.right i:active {
  transform: scale(0.9);
} */

/* 导航栏 */
header #main-nav {
  justify-self: center;
  display: inline-flex;
  gap: 5px;
}

.mac-margin-nav {
  margin-left: 40px;
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
}

.title-tag.dev {
  background: rgba(155, 240, 11, 1);
  color: var(--color-text-black);
}
</style>