<script setup lang="ts">
import { motion } from 'motion-v'

export type ButtonType = 'default' | 'tint' | 'warn'

defineProps<{
  type?: ButtonType
  inline?: boolean
  tooltip?: string
  click?: () => void
  disabled?: boolean
}>()

const baseColor = {
  default: 'var(--color-text-black)',
  tint: 'var(--color-tint)',
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
    :class="['mybutton', inline && 'inline', type ?? 'default']"
    :title="tooltip"
    :disabled="disabled"
    :style="{
      color: disabled ? 'var(--color-text-grey)' : baseColor[type ?? 'default'],
      borderColor: disabled ? 'var(--color-text-grey)' : baseColor[type ?? 'default'],
      cursor: disabled ? 'not-allowed' : 'pointer',
    }"
    @click="!disabled && click?.()"
    :while-hover="!disabled ? { backgroundColor: hoverBg[type ?? 'default'] } : undefined"
    :while-press="!disabled ? { scale: 0.95 } : undefined"
    :transition="{
      default: { type: 'tween', duration: 0.4 },
      scale: { ease: 'easeOut', duration: 0.2 },
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
  padding-block: 5px;
  padding-inline: 10px;
}

.mybutton:disabled {
  color: var(--color-text-grey);
  border-color: var(--color-text-grey);
  background: transparent;
}
</style>
