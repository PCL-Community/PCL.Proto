<script lang="ts" setup>
import { useId } from 'vue'
import shapes from '@/assets/control-geometry.json'
defineProps<{ label: string; id?: string; disabled?: boolean }>()
const controlId = useId()
const model = defineModel<boolean>({ default: false })
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
    <span class="checkbox-shape" aria-hidden="true"
      ><svg viewBox="0 0 12 12">
        <path :d="shapes['MyCheckBox.ShapeCheck']" fill="currentColor" /></svg
    ></span>
    <label :for="id || controlId">{{ label }}</label>
  </div>
</template>
<style scoped>
.option {
  margin-block: 4px;
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
input:focus-visible + .checkbox-shape {
  outline: 2px solid var(--color-tint);
  outline-offset: 3px;
}
label {
  cursor: pointer;
}
.disabled {
  opacity: 0.5;
}
.disabled input,
.disabled label {
  cursor: not-allowed;
}
</style>
