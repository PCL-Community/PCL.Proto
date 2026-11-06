<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { DownloadStats } from '@/api/downloadSettings'
import { useTaskManager } from '@/stores/task'
import useSideNavState from '@/stores/windowState'
import { pclText } from '@/api/pclLocale'
const tasks = useTaskManager()
const network = ref<DownloadStats | null>(null)
const totalSpeed = computed(() => network.value?.speed ?? 0)
const speed = computed(() =>
  !network.value
    ? '—'
    : totalSpeed.value < 1024
      ? `${totalSpeed.value.toFixed(0)} B/s`
      : totalSpeed.value < 1048576
        ? `${(totalSpeed.value / 1024).toFixed(1)} KiB/s`
        : `${(totalSpeed.value / 1048576).toFixed(1)} MiB/s`,
)
const stats = computed(() => [
  { title: 'Speed.Progress.Total', value: `${tasks.totalProgress} %` },
  { title: 'Speed.Progress.Speed', value: speed.value },
  { title: 'Speed.Progress.RemainingFiles', value: String(tasks.totalRemaining) },
  {
    title: 'Speed.Progress.RemainingThreads',
    value: network.value
      ? `${network.value.active_connections} / ${network.value.max_connections}`
      : '—',
  },
])
let stopped = false
let timer: ReturnType<typeof setTimeout> | undefined
async function refresh() {
  try {
    const stats = await invoke<DownloadStats>('get_download_stats')
    if (!stopped) network.value = stats
  } catch (error) {
    if (!stopped) {
      if (network.value) console.warn('Download statistics unavailable', error)
      network.value = null
    }
  } finally {
    if (!stopped) timer = setTimeout(refresh, 300)
  }
}
onMounted(() => {
  useSideNavState().setWidth(200)
  if (isTauri()) void refresh()
})
onUnmounted(() => {
  stopped = true
  clearTimeout(timer)
})
</script>
<template>
  <aside class="speed-left">
    <div v-for="stat in stats" :key="stat.title" class="stat">
      <span>{{ pclText(stat.title) }}</span>
      <hr />
      <strong>{{ stat.value }}</strong>
    </div>
  </aside>
</template>
<style scoped>
.speed-left {
  width: 200px;
  flex: 0 0 200px;
  display: flex;
  flex-direction: column;
  justify-content: space-evenly;
  padding: 25px;
}
.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.stat > span {
  color: var(--color-tint);
  font-size: 14px;
}
.stat > strong {
  font-size: 20px;
  font-weight: normal;
}
.stat > hr {
  width: 100%;
  height: 2px;
  border: 0;
  background: var(--color-tint-light);
  margin: 7px 0;
}
</style>
