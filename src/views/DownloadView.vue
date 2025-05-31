<script setup lang="ts">
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { downloadSubViewManifest } from '@/router/naviOptions';
</script>

<template>
  <SideNavLayout :sideNavGroups="downloadSubViewManifest">
    <RouterView />
  </SideNavLayout>
</template>