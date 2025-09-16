<script setup lang="ts">
import { useFloatButton } from '@/composables/useFloatButton'
import { onMounted, onUnmounted } from 'vue'
const { floatButtonState } = useFloatButton()
onMounted(() => {
  floatButtonState.visible = false
})

onUnmounted(() => {
  floatButtonState.visible = true
})
</script>

<template>
  <div class="view-content">aaa</div>
</template>
