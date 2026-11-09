<script lang="ts" setup>
import { ref, useId, watch } from 'vue'
import shapes from '@/assets/control-geometry.json'
defineProps<{ label: string; id?: string; disabled?: boolean; description?: string }>()
const controlId = useId()
const model = defineModel<boolean>({ default: false })
const changed = ref(false)
watch(model, () => {
  changed.value = true
})
</script>
<template>
  <div class="option" :class="{ disabled }">
    <input
      type="checkbox"
      :disabled="disabled"
      :id="id || controlId"
      class="checkable"
      v-model="model"
    />
    <span class="checkbox-shape" :class="{ changed }" aria-hidden="true"
      ><svg viewBox="0 0 12 12">
        <path :d="shapes['MyCheckBox.ShapeCheck']" fill="currentColor" /></svg
    ></span>
    <label :for="id || controlId">{{ label }}</label>
    <span v-if="description" class="description">{{ description }}</span>
  </div>
</template>
<style scoped>
.option {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  min-height: 28px;
}
.checkable {
  position: absolute;
  left: 1px;
  width: 18px;
  height: 18px;
  opacity: 0;
  cursor: pointer;
}
.checkbox-shape {
  margin-left: 1px;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border: 1.1px solid currentColor;
  border-radius: 3px;
  display: grid;
  place-items: center;
  pointer-events: none;
  transition:
    color 200ms linear,
    background-color 100ms linear,
    transform 150ms ease-out;
}
.checkbox-shape svg {
  width: 12px;
  height: 12px;
  transform: scale(0);
  /* CE MyCheckBox：勾选 300 ms（延迟 105 ms），取消 135 ms。 */
  transition: transform 135ms cubic-bezier(0.55, 0, 1, 0.45);
}
input:checked + .checkbox-shape {
  color: var(--color-tint);
}
input:checked + .checkbox-shape svg {
  transform: scale(1);
  transition: transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1) 105ms;
}
input:checked + .checkbox-shape.changed {
  animation: checkbox-check 405ms;
}
input:not(:checked) + .checkbox-shape.changed {
  animation: checkbox-uncheck 405ms;
}
.option:not(.disabled):hover {
  color: var(--ce-color-3, var(--color-tint));
  transition: color 100ms linear;
}
.option:not(.disabled):hover .checkbox-shape {
  color: var(--ce-color-3, var(--color-tint));
  transition-duration: 100ms;
}
.option:not(.disabled):active .checkbox-shape {
  background: var(--ce-color-7, var(--color-background));
  transform: scale(0.917);
  transition:
    transform 1000ms cubic-bezier(0.19, 1, 0.22, 1),
    background-color 100ms linear;
}
@keyframes checkbox-check {
  0% {
    transform: scale(1);
  }
  37% {
    transform: scale(0.667);
    animation-timing-function: cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  100% {
    transform: scale(1);
  }
}
@keyframes checkbox-uncheck {
  0% {
    transform: scale(1);
  }
  37% {
    transform: scale(0.667);
    animation-timing-function: cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  100% {
    transform: scale(1);
  }
}
@media (prefers-reduced-motion: reduce) {
  .checkbox-shape,
  .checkbox-shape svg {
    animation: none !important;
    transition: none !important;
  }
}
input:focus-visible + .checkbox-shape {
  outline: 2px solid var(--color-tint);
  outline-offset: 3px;
}
label {
  cursor: pointer;
}
.description {
  color: var(--ce-gray-5);
  overflow-wrap: anywhere;
}
.disabled {
  opacity: 0.5;
}
.disabled input,
.disabled label {
  cursor: not-allowed;
}
</style>
