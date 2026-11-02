<script setup lang="ts">
import { FloatButtonType, type FloatButtonState } from '@/composables/useFloatButton'
import { onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import useSideNavState from '@/stores/windowState'
import PPrimaryAction from './PPrimaryAction.vue'
import { useTaskManager } from '@/stores/task'

const props = defineProps<FloatButtonState>()
const router = useRouter()
const sideNavState = useSideNavState()
const rippleVisible = ref(false)
const tasks = useTaskManager()
const progress = ref(0)
const progressTimer = window.setInterval(() => {
  const target = Number(tasks.totalProgress) / 100
  progress.value = target <= 0 || target >= 1 || progress.value > target
    ? target
    : progress.value * 0.9 + target * 0.1
}, 50)
onUnmounted(() => window.clearInterval(progressTimer))

watch([() => props.type, () => props.visible], ([type, visible], [previousType]) => {
  rippleVisible.value =
    visible &&
    type === FloatButtonType.TaskManage &&
    previousType !== FloatButtonType.TaskManage &&
    !window.matchMedia('(prefers-reduced-motion: reduce)').matches
})
</script>

<template>
  <div
    v-if="rippleVisible"
    class="float-ripple"
    aria-hidden="true"
    @animationend="rippleVisible = false"
  />
  <Transition name="float-show">
    <PPrimaryAction
      v-if="visible"
      class="float-button"
      :class="{ 'task-button': type === FloatButtonType.TaskManage }"
      :title="title"
      :aria-label="title || text"
      @click="type === FloatButtonType.TaskManage ? router.push('/downloading') : action?.()"
    >
      <span v-if="type === FloatButtonType.TaskManage && progress > 0" class="progress-clip" aria-hidden="true">
        <span class="progress-fill" :style="{ height: `${progress * 100}%` }" />
      </span>
      <span class="float-icon"><component :is="icon" /></span>
      <Transition name="float-label">
        <span v-if="text" class="float-label">{{ text }}</span>
      </Transition>
    </PPrimaryAction>
  </Transition>
</template>

<style scoped>
/* Keep geometry and motion above the shared button's defaults, including during entrance. */
.float-button.pcl-primary-action {
  position: absolute;
  bottom: 20px;
  right: calc((100% - v-bind('sideNavState.sideNavWidthStr')) / 2 - 60px);
  z-index: 3;
  min-width: 0;
  width: 120px;
  height: 36px;
  padding: 0 12px;
  gap: 8px;
  border-radius: 999px;
  box-shadow:
    0 3px 10px var(--color-tint-lightist),
    0 2px 5px rgb(0 0 0 / 12%);
  transition:
    right 900ms cubic-bezier(0.22, 1.45, 0.36, 1),
    width 320ms ease,
    height 320ms ease,
    padding 320ms ease,
    gap 180ms ease,
    opacity 160ms ease,
    background-color 180ms ease,
    box-shadow 180ms ease,
    transform 160ms ease;
}

.float-button.task-button {
  right: 20px;
  width: 40px;
  height: 40px;
  padding: 0;
  gap: 0;
}

.float-button:hover {
  box-shadow:
    0 4px 14px var(--color-tint-lightist),
    0 3px 7px rgb(0 0 0 / 16%);
}

.float-button.pcl-primary-action:hover:not(:disabled),
.float-button.pcl-primary-action:active:not(:disabled) {
  transition-duration: 900ms, 320ms, 320ms, 320ms, 180ms, 160ms, 180ms, 180ms, 160ms;
}

.float-icon {
  position: relative;
  display: grid;
  place-items: center;
  flex: 0 0 16px;
  height: 16px;
}

.progress-clip {
  position: absolute;
  inset: 0;
  overflow: hidden;
  border-radius: inherit;
  pointer-events: none;
}

.progress-fill {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  background: rgb(255 255 255 / 19%);
}

.progress-fill::before {
  content: '';
  position: absolute;
  right: 0;
  bottom: 100%;
  left: 0;
  height: 4px;
  background: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='4' viewBox='0 0 40 4'%3E%3Cpath d='M0 2C10 2 10 0 20 0S30 2 40 2V4H0Z' fill='white' fill-opacity='.19'/%3E%3C/svg%3E") repeat-x;
  animation: progress-wave 3s linear infinite;
}

@keyframes progress-wave {
  to { background-position-x: 40px; }
}

.float-label {
  max-width: 96px;
  overflow: hidden;
  white-space: nowrap;
  transition:
    max-width 180ms ease,
    opacity 120ms ease;
}

.float-label-enter-from,
.float-label-leave-to {
  max-width: 0;
  opacity: 0;
}

.float-button.float-show-enter-from,
.float-button.float-show-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.9);
}

.float-button.task-button.float-show-enter-from,
.float-button.task-button.float-show-leave-to {
  transform: translateX(16px) scale(0.9);
}

.float-ripple {
  position: absolute;
  right: 40px;
  bottom: 40px;
  width: 420px;
  height: 420px;
  z-index: 3;
  pointer-events: none;
  border: 1px solid var(--color-tint-lightist);
  border-radius: 50%;
  background: var(--color-tint-lighter);
  opacity: 0;
  animation: float-ripple 800ms cubic-bezier(0.16, 1, 0.3, 1) 450ms both;
}

@keyframes float-ripple {
  0% {
    opacity: 0;
    transform: translate(50%, 50%) scale(0.08);
  }
  15% {
    opacity: 0.8;
  }
  100% {
    opacity: 0;
    transform: translate(50%, 50%) scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .float-button.pcl-primary-action,
  .float-button.pcl-primary-action:hover:not(:disabled),
  .float-button.pcl-primary-action:active:not(:disabled),
  .float-label {
    transition: none;
  }
  .float-ripple {
    animation: none;
  }
  .progress-fill::before {
    animation: none;
  }
}
</style>
