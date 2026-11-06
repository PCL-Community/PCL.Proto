<script setup lang="ts">
import { SkinViewer, WalkingAnimation } from 'skinview3d'
import { onMounted, onUnmounted, useTemplateRef, watch } from 'vue'
const container = useTemplateRef<HTMLCanvasElement>('skin_container')
const props = defineProps<{ skinUrl: string }>()

let skinViewer: SkinViewer | undefined
onMounted(() => {
  skinViewer = new SkinViewer({
    canvas: container.value!,
    skin: props.skinUrl,
    height: 300,
    width: 200,
    animation: new WalkingAnimation(),
  })
})
watch(
  () => props.skinUrl,
  (url) => {
    void skinViewer?.loadSkin(url)
  },
)
onUnmounted(() => skinViewer?.dispose())
</script>
<template>
  <canvas ref="skin_container"></canvas>
</template>
