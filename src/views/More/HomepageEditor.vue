<script setup lang="ts">
import hompageEditorContent from '/pcl-home-zx/index.html?url'
</script>

<template>
  <iframe
    :src="hompageEditorContent"
    width="100%"
    height="100%"
    frameborder="0"
    style="border: none"
  />
</template>
