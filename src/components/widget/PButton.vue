<script setup lang="ts">
import { motion } from 'motion-v'

export type ButtonType = 'default' | 'tint' | 'warn'

defineProps<{
  type?: ButtonType
  inline?: boolean
  compact?: boolean
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
    :class="['mybutton', inline && 'inline', compact && 'compact', type ?? 'default']"
    :title="tooltip"
    :disabled="disabled"
    :style="{
      color: inline
        ? undefined
        : disabled
          ? 'var(--color-text-grey)'
          : baseColor[type ?? 'default'],
      cursor: disabled ? 'not-allowed' : 'pointer',
    }"
    @click="!disabled && emit('click', $event)"
    :while-press="
      !disabled && !inline
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
  transition: background-color 0.2s linear, border-color 0.2s linear;
  border-radius: 4px;
  border: 1px solid var(--color-text-black);
  outline: none;
}

.mybutton.tint {
  border-color: var(--ce-color-2);
}

/* 警告按钮静止态也用警告色、仅 alpha 0，保证过渡只改 alpha。 */
.mybutton.warn {
  border-color: var(--color-warn);
  background-color: rgb(from var(--color-warn) r g b / 0);
}

.mybutton:not(:disabled):is(:hover, :active) {
  background-color: var(--color-tint-lighter);
  transition-duration: 0.1s;
}

.mybutton.default:not(:disabled):is(:hover, :active) {
  border-color: var(--ce-color-2);
}

.mybutton.warn:not(:disabled):is(:hover, :active) {
  background-color: var(--color-warn-lighter);
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

.mybutton.compact {
  height: 28px;
  min-height: 28px;
  padding: 0 10px;
}

.mybutton.inline,
.pcl-page .mybutton.inline {
  display: inline;
  min-height: 0;
  padding: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: var(--ce-color-1);
  font: inherit;
  line-height: inherit;
  transition: color 200ms linear;
}

.mybutton.inline.warn {
  color: var(--color-warn);
}

.mybutton.inline:not(:disabled):is(:hover, :active) {
  background: transparent;
  color: var(--ce-color-3);
  transition-duration: 100ms;
}

.mybutton.inline:not(:disabled):active {
  color: var(--ce-color-4);
  transition-duration: 30ms;
}

.mybutton.inline:focus-visible {
  outline: 2px solid var(--color-tint);
  outline-offset: 2px;
}

.mybutton.inline:disabled {
  color: var(--color-text-grey);
}
</style>
