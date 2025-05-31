<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import { sideNavState, defaultWidths } from '@/windowState';
import { onMounted } from 'vue';

onMounted(() => {
  sideNavState.width = defaultWidths.home
})

</script>

<template>
  <div class="view-content">
    <aside class="left" :style="{ width: sideNavState.width + 'px' }">
      主页
      <MyButton></MyButton>
    </aside>
    <article>
      <RouterView />
    </article>
  </div>
</template>

<style scoped>
aside {
  height: 100%;
  flex: 0 0 auto;
  padding: 20px;
}

article {
  flex: 1 1 0;
  overflow: auto;
}

.view-content {
  display: flex;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
}
</style>