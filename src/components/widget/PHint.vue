<script setup lang="ts">
export type Severity = 'info' | 'warning' | 'error'
withDefaults(defineProps<{ severity: Severity }>(), {
  severity: 'info',
})
</script>

<template>
  <div class="hint-container" :class="severity">
    <slot></slot>
  </div>
</template>

<style lang="css" scoped>
.hint-container {
  border-radius: 3px;
  border-left: 3px solid;
  padding: 6px 8px;
}

.hint-container.error {
  background-color: #feeaeb;
  border-left-color: #f31628;
  color: #f31628;
}

.hint-container.warning {
  background-color: #fff4e6;
  border-left-color: #f39c16;
  color: #f39c16;
}

.hint-container.info {
  background-color: var(--lighter-blue);
  border-left-color: var(--tint-blue);
  color: var(--tint-blue);
}
</style>
