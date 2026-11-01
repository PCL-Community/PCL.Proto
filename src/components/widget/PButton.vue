<script setup lang="ts">
import { motion } from 'motion-v'

export type ButtonType = 'default' | 'tint' | 'warn'

defineProps<{
  type?: ButtonType
  inline?: boolean
  tooltip?: string
  disabled?: boolean
}>()

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const baseColor = {
  default: 'var(--color-text-black)',
  tint: 'var(--ce-color-2)',
  warn: 'var(--color-warn)',
}
</script>

<template>
  <motion.button
    type="button"
    :class="['mybutton', inline && 'inline', type ?? 'default']"
    :title="tooltip"
    :disabled="disabled"
    :style="{
      color: disabled ? 'var(--color-text-grey)' : baseColor[type ?? 'default'],
      borderColor: disabled ? 'var(--color-text-grey)' : baseColor[type ?? 'default'],
      cursor: disabled ? 'not-allowed' : 'pointer',
    }"
    @click="!disabled && emit('click', $event)"
    :while-press="
      !disabled
        ? {
            scale: 0.955,
            transition: { scale: { type: 'tween', duration: 0.08, ease: [0.22, 1, 0.36, 1] } },
          }
        : undefined
    "
    :transition="{
      scale: { type: 'tween', duration: 0.3, delay: 0.01, ease: [0.33, 1, 0.68, 1] },
    }"
  >
    <slot />
  </motion.button>
</template>

<style scoped>
.mybutton {
  font-size: 13px;
  padding: 0.5rem;
  /* 背景过渡由 CSS 处理，Motion 只负责缩放。 */
  background-color: var(--color-tint-rest);
  transition: background-color 0.2s linear;
  border-radius: 4px;
  border: 1px solid rgba(52, 61, 74, 1);
  outline: none;
}

/* 警告按钮静止态也用警告色、仅 alpha 0，保证过渡只改 alpha。 */
.mybutton.warn {
  background-color: rgb(from var(--color-warn) r g b / 0);
}

.mybutton:not(:disabled):is(:hover, :active) {
  background-color: var(--color-tint-lighter);
  transition-duration: 0.1s;
}

.mybutton.warn:not(:disabled):is(:hover, :active) {
  background-color: var(--color-warn-lighter);
}

.inline {
  border: 0;
  padding-block: 5px;
  padding-inline: 10px;
}

.mybutton:disabled {
  color: var(--color-text-grey);
  border-color: var(--color-text-grey);
  background: transparent;
}

.pcl-page .mybutton {
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  font: inherit;
}
</style>
