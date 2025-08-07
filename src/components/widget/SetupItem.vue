<script setup lang="ts">
import Dropdown from './Dropdown.vue'
import { type ISetupOption, type SetupItemType } from '@/util/setup'
import PInput from './PInput.vue'

withDefaults(
  defineProps<{
    label: string
    options?: ISetupOption[]
    type: SetupItemType
    ratio?: number
  }>(),
  {
    ratio: 4,
  },
)

const model = defineModel<string>()
</script>

<template lang="pug">
.setupitem-with-text-container
    p.text-label {{ label }}
    Dropdown.input(
        v-if="type == 'select' && options"
        :options="options"
        v-model="model"
    )
    PInput.input(
        v-else-if="type == 'input'"
        :placeholder="options?.find(v => v.key === 'placeholder')?.text"
        v-model="model"
    )
</template>

<style lang="css" scoped>
.setupitem-with-text-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-block: 4px;
}

.text-label {
  flex: 1;
  margin: 0;
}

.input {
  flex: v-bind(ratio);
}
</style>
