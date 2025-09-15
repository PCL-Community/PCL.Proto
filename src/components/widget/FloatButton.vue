<script setup lang="ts">
import type { Component } from 'vue'
defineProps<{ icon?: Component; text?: string; title?: string }>()
</script>

<template>
  <button class="float-button" :title>
    <component :is="icon" />
    {{ text }}
  </button>
</template>

<style lang="css" scoped>
.float-button {
  position: absolute;
  bottom: 20px;
  right: 20px;
  background-color: var(--tint-blue);
  color: var(--vt-c-white);
  height: 36px;
  min-width: 36px;
  padding-inline: 12px;
  border-radius: 30px;
  font-size: 14px;
  box-shadow: var(--box-shadow);
  transition: all 0.2s;
  border: none;
  outline: none;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.float-button:hover {
  background-color: var(--light-blue);
}

.float-button:active {
  transform: scale(0.95);
}
</style>
