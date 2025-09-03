<!-- 一个自定义的按钮：传入参数tint，若为true则为高亮 -->

<script setup lang="ts">
export type ButtonType = 'default' | 'tint' | 'warn'
defineProps<{
  type?: ButtonType
  inline?: boolean
  tooltip?: string
  click?: () => void
  disabled?: boolean
}>()
</script>

<template lang="pug">
    button.mybutton(:class="[type ?? 'default', inline ? 'inline' : '']" :title="tooltip" @click="disabled? null :click?.()" :disabled="disabled")
        slot 我的按钮
</template>

<style scoped>
* {
  font-size: 13px;
}

button.mybutton {
  padding: 0.5rem;
  background-color: transparent;
  border-radius: 4px;
  border: 1px solid rgba(52, 61, 74, 1);
  transition: all 0.2s;
}

button.inline {
  padding-block: 5px;
  padding-inline: 10px;
}

button.tint {
  color: var(--color-tint);
  border-color: var(--color-tint);
}

button.warn {
  color: var(--color-warn);
  border-color: var(--color-warn);
}

button.mybutton:not(:disabled):hover {
  background-color: var(--color-tint-lighter);
  color: var(--color-tint);
  border-color: var(--color-tint);
}

button.warn:not(:disabled):hover {
  background-color: var(--color-warn-lighter);
  color: var(--color-warn);
  border-color: var(--color-warn);
}

button.mybutton:not(:disabled):active {
  transform: scale(0.95);
}

button.mybutton:disabled {
  color: var(--color-text-grey);
  border-color: var(--color-text-grey);
}
</style>
