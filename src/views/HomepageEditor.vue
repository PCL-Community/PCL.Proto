<script setup lang="ts">
import { onMounted, ref } from 'vue'
import hompageEditorContent from '/pcl-home-zx/index.html?url'

const iframeRef = ref<HTMLIFrameElement>()

onMounted(() => {
  iframeRef.value!.src = hompageEditorContent
})
</script>

<template>
  <iframe ref="iframeRef" width="100%" height="100%" frameborder="0" style="border: none" />
</template>
