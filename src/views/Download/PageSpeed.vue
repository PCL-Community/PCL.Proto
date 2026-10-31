<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import PageSpeedLeft from './PageSpeedLeft.vue'
import PageSpeedRight from './PageSpeedRight.vue'
import { useFloatButton } from '@/composables/useFloatButton'
import { TaskStatus, useTaskManager } from '@/stores/task'
const { floatButtonState } = useFloatButton()
const tasks = useTaskManager()
const router = useRouter()
onMounted(() => (floatButtonState.visible = false))
onUnmounted(() => (floatButtonState.visible = tasks.activeTaskCount > 0))
onMounted(() => {
  watch(
    () => tasks.activeTaskCount,
    (count) => {
      if (count || tasks.tasks.some((task) => task.status === TaskStatus.Failed)) return
      if (router.currentRoute.value.path !== '/downloading') return
      const back = router.options.history.state.back
      if (typeof back === 'string' && back && back !== '/downloading') router.back()
      else void router.replace('/download/game')
    },
    { immediate: true },
  )
})
</script>
<template>
  <div class="view-content"><PageSpeedLeft /><PageSpeedRight /></div>
</template>
