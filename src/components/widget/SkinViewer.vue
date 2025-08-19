<script setup lang="ts">
import { useAccountInfo } from '@/stores/account'
import { SkinViewer, WalkingAnimation } from 'skinview3d'
import { onMounted, useTemplateRef } from 'vue'
const container = useTemplateRef<HTMLCanvasElement>('skin_container')
const props = defineProps<{ skinUrl: string }>()

onMounted(() => {
  const skinViewer = new SkinViewer({
    canvas: container.value!,
    skin: props.skinUrl,
    height: 300,
    width: 200,
    animation: new WalkingAnimation(),
  })
})
</script>
<template>
  <canvas ref="skin_container"></canvas>
</template>
