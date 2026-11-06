<script setup lang="ts">
import { useAttrs, useId, type HTMLAttributes } from 'vue'
defineOptions({ inheritAttrs: false })
const attrs = useAttrs() as HTMLAttributes & Record<string, unknown>
const controlId = useId()
function controlAttrs() {
  const { class: _class, style: _style, ...inputAttrs } = attrs
  return inputAttrs
}
import IconUnfold from '../icons/control/IconUnfold.vue'
import { type ISetupOption } from '../../types/setupOption'

defineProps<{
  options: ISetupOption[]
  inputId?: string
  disabled?: boolean
}>()

const model = defineModel<string>()
</script>

<template>
  <div class="dropdown-wrapper" :class="attrs.class" :style="attrs.style">
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
  color: var(--ce-color-5);
  transition: transform 0.4s ease;
}

select:focus + i {
  transform: translateY(-50%) rotate(180deg);
}

select:disabled + i {
  color: var(--ce-gray-5);
}

.pcl-page .dropdown-wrapper {
  min-width: 0;
}
select:not(:disabled):hover + i {
  color: var(--ce-color-4);
}
select:not(:disabled):focus + i {
  color: var(--ce-color-3);
}
select:disabled {
  cursor: not-allowed;
}
select::-ms-expand {
  display: none;
}
</style>
