<script setup lang="ts">
import { FloatButtonType, type FloatButtonState } from '@/composables/useFloatButton'
import { emit, listen } from '@tauri-apps/api/event'
import { info } from '@tauri-apps/plugin-log'
import { ref } from 'vue'

defineProps<FloatButtonState>()
const buttonWidth = ref(36)

listen('float-button-click', (event) => {
  if (event.payload === FloatButtonType.TaskManage) {
    info('task manage clicked')
  }
})
</script>

<template>
  <Transition>
    <button
      class="float-button"
      :title="title"
      @click="emit('float-button-click', type)"
      v-if="visible"
    >
      <component :is="icon" />
      {{ text }}
    </button>
  </Transition>
</template>

<style lang="css" scoped>
.v-enter-from,
.v-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.v-enter-active,
.v-leave-active {
  transition: all 0.4s;
}

.float-button {
  position: absolute;
  bottom: 20px;
  right: 20px;
  background-color: var(--tint-blue);
  color: var(--vt-c-white);
  height: 36px;
  width: v-bind('buttonWidth + "px"');
  padding-inline: 12px;
  border-radius: 30px;
  font-size: 14px;
  box-shadow: var(--box-shadow);
  transition: all 0.2s;
  border: none;
  outline: none;
  z-index: 3;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  width: fit-content;
  min-width: 36px;
}

.float-button:hover {
  background-color: var(--light-blue);
}

.float-button:active {
  transform: scale(0.95);
}
</style>
