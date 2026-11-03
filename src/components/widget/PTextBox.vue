<script lang="ts" setup>
import { useAttrs, useId, type HTMLAttributes } from 'vue'
defineOptions({ inheritAttrs: false })
const attrs = useAttrs() as HTMLAttributes & Record<string, unknown>
const controlId = useId()
function controlAttrs() {
  const { class: _class, style: _style, ...inputAttrs } = attrs
  return inputAttrs
}
const suggestionsId = useId()
const model = defineModel<string>()
defineProps<{
  placeholder?: string
  disabled?: boolean
  type?: 'text' | 'password'
  inputId?: string
  multiline?: boolean
  suggestions?: string[]
}>()
</script>

<template>
  <div class="input-wrapper" :class="attrs.class" :style="attrs.style">
    <textarea
      v-if="multiline"
      v-bind="controlAttrs()"
      :id="inputId || String(attrs.id || controlId)"
      :aria-label="attrs['aria-label'] || (attrs['aria-labelledby'] ? undefined : attrs.title)"
      :disabled="disabled"
      :placeholder
      v-model="model"
    />
    <input
      v-else
      v-bind="controlAttrs()"
      :id="inputId || String(attrs.id || controlId)"
      :aria-label="attrs['aria-label'] || (attrs['aria-labelledby'] ? undefined : attrs.title)"
      :disabled="disabled"
      :type="type || 'text'"
      :list="suggestions?.length ? suggestionsId : undefined"
      :placeholder
      v-model="model"
    />
    <datalist v-if="suggestions?.length" :id="suggestionsId">
      <option v-for="item in suggestions" :key="item" :value="item" />
    </datalist>
  </div>
</template>

<style scoped>
textarea {
  height: 100%;
  min-height: 80px;
  resize: vertical;
}
.pcl-page .input-wrapper {
  min-width: 0;
}
</style>
