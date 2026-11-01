<script lang="ts" setup>
import { useAttrs, useId, type HTMLAttributes } from 'vue'
defineOptions({ inheritAttrs: false })
const attrs = useAttrs() as HTMLAttributes & Record<string, unknown>
const controlId = useId()
function controlAttrs() {
  const { class: _class, style: _style, 'data-control': _control, ...inputAttrs } = attrs
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
  <div
    class="input-wrapper"
    :class="attrs.class"
    :style="attrs.style"
    :data-control="attrs['data-control']"
  >
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
input,
textarea {
  background-color: var(--color-tint-rest);
  font-size: 12px;
  padding: 0.3rem 0.8rem;
  border-radius: 4px;
  border: 1px solid var(--color-tint-lightist);
  /* CE MyTextBox：常规 100 ms、聚焦 10 ms、不可用 200 ms。 */
  transition: background-color 100ms linear;
  color: var(--color-text-black);
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  width: 100%;
}

textarea {
  height: 100%;
  min-height: 80px;
  resize: vertical;
}

input:not(:disabled):hover,
textarea:not(:disabled):hover {
  background-color: var(--color-tint-lighter);
}

input:focus,
textarea:focus {
  background-color: var(--color-tint-lighter);
  transition-duration: 10ms;
}

input:disabled,
textarea:disabled {
  color: var(--ce-gray-4);
  -webkit-text-fill-color: var(--ce-gray-4);
  border-color: var(--ce-gray-5);
  background-color: var(--ce-gray-6);
  opacity: 1;
  transition-duration: 200ms;
}

input:disabled::placeholder,
textarea:disabled::placeholder {
  color: var(--ce-gray-4);
  -webkit-text-fill-color: var(--ce-gray-4);
  opacity: 1;
}

.pcl-page .input-wrapper {
  min-width: 0;
}
.pcl-page .input-wrapper input {
  height: 28px;
  padding: 0 8px;
}
</style>
