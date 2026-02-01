<script lang="ts" setup>
const model = defineModel<string>()
defineProps<{ placeholder?: string }>()
</script>

<template>
  <div class="input-wrapper">
    <input type="text" :placeholder v-model="model" />
  </div>
</template>

<style scoped>
input {
  background-color: transparent;
  font-size: 12px;
  padding: 0.3rem 0.8rem;
  border-radius: 4px;
  border: 1px solid var(--color-tint-lightist);
  transition: background-color 0.3s ease;
  color: var(--color-text-black);
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  width: 100%;
}

input:hover {
  background-color: var(--color-tint-lighter);
}

input:focus {
  background-color: var(--color-tint-lighter);
}
</style>
