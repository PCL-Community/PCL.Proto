<script setup lang="ts">
import { sideNavState, defaultWidths } from '@/windowState';
import { onMounted } from 'vue';

onMounted(() => {
  sideNavState.width = defaultWidths.home
})

</script>

<template>
  <div>Home page</div>
</template>
