<script setup lang="ts">
import { sideNavState, defaultWidths } from '@/router/windowState';
import { onMounted } from 'vue';

onMounted(() => {
  sideNavState.width = defaultWidths.link
})
</script>

<template>
  <main>
    <h1>Link</h1>
  </main>
</template>
