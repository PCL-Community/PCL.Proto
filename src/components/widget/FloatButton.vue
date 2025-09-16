<script setup lang="ts">
import { FloatButtonType, type FloatButtonState } from '@/composables/useFloatButton'
import { emit, listen } from '@tauri-apps/api/event'
import { ref, watchEffect } from 'vue'
import { useRouter } from 'vue-router'

const props = defineProps<FloatButtonState>()
const backVisible = ref(false)
const buttonWidth = ref(36)
const router = useRouter()

watchEffect(() => {
  if (props.type === FloatButtonType.TaskManage) {
    backVisible.value = true
    setTimeout(() => {
      backVisible.value = false
    }, 1000)
  }
  buttonWidth.value = props.type === FloatButtonType.DownloadGame ? 120 : 36
})

listen('float-button-click', (event) => {
  if (event.payload === FloatButtonType.TaskManage) {
    router.push({ name: 'downloading' })
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
      <span v-if="text">{{ text }}</span>
      <div :class="{ back: backVisible }" v-if="backVisible" />
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
  transition: all 1s;
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
  justify-content: space-around;
}

.float-button > .back {
  position: absolute;
  height: 40vi;
  width: 40vw;
  z-index: -1;
  border-radius: 50%;
  animation: back-scale 0.8s;
}

@keyframes back-scale {
  0% {
    background-color: transparent;
    transform: scale(0);
  }
  50% {
    background-color: var(--half-transparent-blue);
  }
  100% {
    background-color: transparent;
    transform: scale(1.1);
  }
}

.float-button > span {
  white-space: nowrap;
  overflow: hidden;
}

.float-button:hover {
  background-color: var(--light-blue);
}

.float-button:active {
  transform: scale(0.95);
}
</style>
