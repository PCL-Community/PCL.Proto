<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { ref } from 'vue'
import PSvgIcon from './PSvgIcon.vue'

export type Severity = 'info' | 'warning' | 'error'
withDefaults(defineProps<{ severity: Severity; closable?: boolean }>(), {
  severity: 'info',
  closable: false,
})

const visibility = ref(true)
const emit = defineEmits<{ close: [] }>()
function close() {
  visibility.value = false
  emit('close')
}
</script>

<template>
  <div class="hint-container" :class="severity" v-if="visibility">
    <div><slot></slot></div>
    <button
      type="button"
      :aria-label="pclText('Proto.Action.CloseHint')"
      class="button-animated"
      v-if="closable"
      @click="close"
    >
      <PSvgIcon name="lucide/x" :size="14" />
    </button>
  </div>
</template>

<style lang="css" scoped>
button.button-animated {
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  cursor: pointer;
  padding: 3px;
  scale: 0.8;
  color: inherit;
  transition: opacity 150ms linear;
}

.hint-container {
  border-radius: 3px;
  border-left: 3px solid;
  padding: 6px 8px;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 2px;
  margin-bottom: 4px;
}

.theme-blue .hint-container.error {
  background-color: #fee6e8;
  border-left-color: #f42b3c;
  color: #f42b3c;
}

.theme-blue .hint-container.warning {
  background-color: #fdf1d8;
  border-left-color: #c1840a;
  color: #c1840a;
}

.theme-blue .hint-container.info {
  background-color: hsl(from #dcedfd calc(h + var(--hint-hue-offset, 0)) s l);
  color: hsl(from #0b6ed0 calc(h + var(--hint-hue-offset, 0)) s l);
}

.theme-blue.dark .hint-container.error {
  background-color: #7f0711;
  border-left-color: #f9959d;
  color: #f9959d;
}
.theme-blue.dark .hint-container.warning {
  background-color: #573c05;
  border-left-color: #f6c35d;
  color: #f6c35d;
}
.theme-blue.dark .hint-container.info {
  background-color: hsl(from #05315e calc(h + var(--hint-hue-offset, 0)) s l);
  color: hsl(from #6eb3f7 calc(h + var(--hint-hue-offset, 0)) s l);
}
.theme-orange .hint-container.info {
  background-color: #ed931d3d;
  border-left-color: #ff9626;
  color: #ff9626;
}

button.button-animated:hover {
  opacity: 0.7;
  transition-duration: 120ms;
}

.pcl-page .hint-container {
  margin-top: 0;
  margin-bottom: 0;
}
</style>
