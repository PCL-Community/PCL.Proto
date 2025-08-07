<script setup lang="ts">
import { ref } from 'vue'
import IconClose from '@/assets/icons/TitleClose.svg'

export type Severity = 'info' | 'warning' | 'error'
withDefaults(defineProps<{ severity: Severity; closable?: boolean }>(), {
  severity: 'info',
  closable: false,
})

const visibility = ref(true)
</script>

<template>
  <div class="hint-container" :class="severity" v-if="visibility">
    <slot></slot>
    <i class="button-animated" v-if="closable" @click="visibility = false"> <icon-close /></i>
  </div>
</template>

<style lang="css" scoped>
i.button-animated {
  display: flex;
  align-items: center;
  justify-content: center;
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

i.button-animated {
  scale: 0.8;
  color: var(--light-blue);
  transition: 0.4s;
}

i.button-animated:hover {
  color: var(--color-titlebar);
}
</style>
