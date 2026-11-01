<script setup lang="ts">
import { useAttrs, useId, type HTMLAttributes } from 'vue'
defineOptions({ inheritAttrs: false })
const attrs = useAttrs() as HTMLAttributes & Record<string, unknown>
const controlId = useId()
function controlAttrs() {
  const { class: _class, style: _style, 'data-control': _control, ...inputAttrs } = attrs
  return inputAttrs
}
import IconUnfold from '../icons/control/IconUnfold.vue'
import { type ISetupOption } from '@/stores/setup'

defineProps<{
  options: ISetupOption[]
  inputId?: string
  disabled?: boolean
}>()

const model = defineModel<string>()
</script>

<template>
  <div
    class="dropdown-wrapper"
    :class="attrs.class"
    :style="attrs.style"
    :data-control="attrs['data-control']"
  >
    <select
      v-bind="controlAttrs()"
      :id="inputId || String(attrs.id || controlId)"
      v-model="model"
      :disabled="disabled"
      :aria-label="attrs['aria-label'] || (attrs['aria-labelledby'] ? undefined : attrs.title)"
    >
      <option
        v-for="option in options"
        :key="option.key"
        :title="option.title"
        :value="option.key"
        :disabled="option.disabled"
      >
        {{ option.text }}
      </option>
    </select>
    <i><IconUnfold /></i>
  </div>
</template>

<style lang="scss" scoped>
.dropdown-wrapper {
  position: relative;
  --pcl-unfold-icon-width: 8px;
  --pcl-unfold-icon-height: 5px;
}

i {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%) rotate(0deg);
  pointer-events: none;
  display: flex;
  place-items: center;
  color: var(--color-tint-lightist);
  transition: transform 0.4s ease;
}

select:focus + i {
  transform: translateY(-50%) rotate(180deg);
}

select:disabled + i {
  color: var(--ce-gray-5);
}

select {
  background-color: color-mix(in srgb, var(--color-background) 33.3%, transparent);
  font-size: 12px;
  padding: 0.3rem 0.8rem;
  border-radius: 4px;
  border: 1px solid var(--ce-color-5);
  /* CE MyComboBox：常规 100 ms、展开 10 ms、不可用 200 ms。 */
  transition: background-color 100ms linear;
  color: var(--ce-color-1);
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  width: 100%;
  &:hover {
    border-color: var(--ce-color-4);
    background-color: var(--color-tint-lighter);
  }
  &:focus {
    border-color: var(--ce-color-3);
    transition-duration: 10ms;
  }
  &:disabled {
    color: var(--ce-gray-4);
    -webkit-text-fill-color: var(--ce-gray-4);
    border-color: var(--ce-gray-5);
    background-color: var(--ce-gray-6);
    opacity: 1;
    transition-duration: 200ms;
  }

  &::-ms-expand {
    display: none;
  }
}

.pcl-page .dropdown-wrapper {
  min-width: 0;
}
.pcl-page .dropdown-wrapper select {
  height: 28px;
  padding: 0 8px;
}
</style>
