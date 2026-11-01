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

const hoverBg = {
  default: 'var(--color-tint-lighter)',
  tint: 'var(--color-tint-lighter)',
  warn: 'var(--color-warn-lighter)',
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
    :while-hover="
      !disabled
        ? {
            backgroundColor: hoverBg[type ?? 'default'],
            transition: {
              backgroundColor: { type: 'tween', duration: 0.1, ease: 'linear' },
              scale: { type: 'tween', duration: 0.3, delay: 0.01, ease: [0.33, 1, 0.68, 1] },
            },
          }
        : undefined
    "
    :while-press="
      !disabled
        ? {
            scale: 0.955,
            transition: {
              scale: { type: 'tween', duration: 0.08, ease: [0.22, 1, 0.36, 1] },
              backgroundColor: { type: 'tween', duration: 0.1, ease: 'linear' },
            },
          }
        : undefined
    "
    :transition="{
      backgroundColor: { type: 'tween', duration: 0.2, ease: 'linear' },
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
  background-color: transparent;
  border-radius: 4px;
  border: 1px solid rgba(52, 61, 74, 1);
  outline: none;
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
