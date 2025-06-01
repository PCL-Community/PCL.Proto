<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import { sideNavState, defaultWidths, sideNavWidthStr } from '@/router/windowState';
import { onMounted } from 'vue';

onMounted(() => {
  sideNavState.width = defaultWidths.home
})

</script>

<template lang="pug">
  .view-content
    aside.left
      | 主页
      MyButton()

    article.subview
      RouterView()

</template>

<style scoped>
aside {
  height: 100%;
  flex: 0 0 auto;
  padding: 20px;
}

aside.left {
  width: v-bind('sideNavWidthStr');
}

article {
  flex: 1 1 0;
  overflow-y: auto;
  padding-bottom: 76px;
  /*我也不知道为什么要加这个padding，反正不加就会被遮挡 */
}

.view-content {
  display: flex;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
}
</style>