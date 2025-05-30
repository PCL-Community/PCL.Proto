<script setup lang="ts">
import SideNavItem from '@/components/SideNavItem.vue';
defineEmits(['navButton'])
</script>

<template>

  <div class="side-wrapper">
    <div class="side-group">
      <SideNavItem @click="$emit('navButton', 0)" item-name="游戏下载" :is-selected=true></SideNavItem>
      <SideNavItem @click="$emit('navButton', 1)" item-name="手动安装包" :is-selected="false">手</SideNavItem>
    </div>
  </div>
</template>

<style scoped>
.side-group {
  display: flex;
  flex-direction: column;
}

.side-wrapper {
  margin: 12px 0 0 0;
}
</style>