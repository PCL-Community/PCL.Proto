<script setup lang="ts">
import PSvgIcon from './PSvgIcon.vue'

withDefaults(defineProps<{ icon: string; disabled?: boolean; bordered?: boolean }>(), {
  bordered: false,
})
</script>

<template>
  <button type="button" :class="['icon-text-button', bordered && 'bordered']" :disabled="disabled">
    <PSvgIcon :name="icon" :size="16" /><slot />
  </button>
</template>

<style scoped>
.icon-text-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  height: 27px;
  padding: 0 12px;
  border: 0;
  border-radius: 13.5px;
  background: var(--color-tint-rest);
  color: var(--color-text-black);
  font: inherit;
  cursor: pointer;
  /* CE MyIconTextButton：移出 150 ms，指向 100 ms。 */
  transition:
    color 150ms linear,
    background-color 150ms linear;
}
.icon-text-button:not(:disabled):is(:hover, :focus-visible) {
  background: var(--color-tint-lighter);
  color: var(--color-tint);
  transition-duration: 100ms;
}
/* CE 资源详情顶部的按钮额外绘制了 1px 边框（ColorBrush6），按钮本身已经是胶囊圆角。 */
.icon-text-button.bordered {
  border: 1px solid var(--ce-color-6);
}
.icon-text-button:not(:disabled):active {
  background: var(--half-transparent-blue);
}
.icon-text-button:focus-visible {
  outline: 2px solid var(--color-tint);
  outline-offset: 2px;
}
.icon-text-button:disabled {
  color: var(--color-text-grey);
  cursor: not-allowed;
}
@media (prefers-reduced-motion: reduce) {
  .icon-text-button {
    transition: none;
  }
}
</style>
