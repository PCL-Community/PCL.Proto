<script setup lang="ts">
import { useId } from 'vue'

defineProps<{ label?: string }>()
const controlId = useId()
</script>

<template>
  <div class="setup-item">
    <label v-if="label" class="setup-label" :for="controlId">{{ label }}</label>
    <div v-else class="setup-label"><slot name="label" :id="controlId" /></div>
    <div class="setup-control"><slot :id="controlId" /></div>
  </div>
</template>

<style scoped>
.setup-item {
  display: contents;
}
.setup-label {
  margin-inline-end: 25px;
  align-self: center;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.setup-control {
  display: grid;
  align-content: center;
  min-width: 0;
  min-height: 28px;
}
.setup-control:has(> input[type='range']) {
  min-height: 19px;
}
</style>
