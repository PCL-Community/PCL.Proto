<script setup lang="ts">
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { downloadSubViewManifest } from '@/options/naviOptions';
</script>

<template>
  <SideNavLayout :sideNavGroups="downloadSubViewManifest">
    <RouterView />
  </SideNavLayout>
</template>