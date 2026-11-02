import type { DownloadStats } from '@/api/downloadSettings'
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export type Launching = {
  name: string
  progress: number
  stage: string
  method: string
  downloading: boolean
}

export type LaunchFailure = {
  exit_code: number | null
  details: string | null
  solution: string | null
}

export type LaunchProgress = Pick<Launching, 'progress' | 'stage' | 'downloading'> & {
  failure?: LaunchFailure | null
}

function formatSpeed(speed: number) {
  if (speed < 1024) return `${speed.toFixed(0)} B/s`
  if (speed < 1048576) return `${(speed / 1024).toFixed(1)} KiB/s`
  return `${(speed / 1048576).toFixed(1)} MiB/s`
}

export const useLaunchState = defineStore('launch-state', () => {
  const preparing = ref(false)
  const launching = ref<Launching | null>(null)
  const downloadSpeed = ref<string | null>(null)
  const leavingLaunch = ref(false)
  const cancelRequested = ref(false)
  const attempt = ref(0)
  const pendingServer = ref<{ directory: string; address: string } | null>(null)
  let speedTimer: ReturnType<typeof setTimeout> | undefined
  let launchLeaveTimer: ReturnType<typeof setTimeout> | undefined
  let speedRefreshingFor: number | undefined

  function isCurrent(id: number) {
    return id === attempt.value
  }

  function isCancelled(id: number) {
    return !isCurrent(id) || cancelRequested.value
  }

  function clearSpeed() {
    clearTimeout(speedTimer)
    speedTimer = undefined
    downloadSpeed.value = null
  }

  function start(name: string, method: string) {
    clearTimeout(launchLeaveTimer)
    clearSpeed()
    attempt.value++
    cancelRequested.value = false
    leavingLaunch.value = false
    launching.value = {
      name,
      progress: 0,
      stage: 'Common.Action.Initialize',
      method,
      downloading: false,
    }
    return attempt.value
  }

  function setPreparing(id: number, value: boolean) {
    if (isCurrent(id)) preparing.value = value
  }

  function setMethod(id: number, method: string) {
    if (!isCancelled(id) && launching.value) launching.value = { ...launching.value, method }
  }

  function update(id: number, update: LaunchProgress) {
    if (isCancelled(id) || !launching.value) return false
    launching.value = {
      ...launching.value,
      progress: update.progress,
      stage: update.stage,
      downloading: update.downloading,
    }
    if (launching.value.downloading) ensureSpeed(id)
    else clearSpeed()
    return true
  }

  async function refreshSpeed(id: number) {
    if (isCancelled(id) || !launching.value?.downloading) return
    speedRefreshingFor = id
    try {
      const stats = await invoke<DownloadStats>('get_download_stats')
      if (!isCancelled(id) && launching.value?.downloading)
        downloadSpeed.value = formatSpeed(stats.speed)
    } catch {
      if (isCurrent(id)) downloadSpeed.value = null
    } finally {
      if (speedRefreshingFor === id) speedRefreshingFor = undefined
      if (!isCancelled(id) && launching.value?.downloading) {
        speedTimer = setTimeout(() => {
          speedTimer = undefined
          void refreshSpeed(id)
        }, 300)
      }
    }
  }

  function ensureSpeed(id: number) {
    if (!speedTimer && speedRefreshingFor !== id) void refreshSpeed(id)
  }

  function requestCancel() {
    if (!launching.value || leavingLaunch.value) return null
    cancelRequested.value = true
    clearSpeed()
    return attempt.value
  }

  function returnToHome(id: number) {
    if (!isCurrent(id) || !launching.value || leavingLaunch.value) return
    leavingLaunch.value = true
    clearSpeed()
    launchLeaveTimer = setTimeout(() => {
      if (!isCurrent(id)) return
      leavingLaunch.value = false
      launching.value = null
    }, 360)
  }

  return {
    pendingServer,
    preparing,
    launching,
    downloadSpeed,
    leavingLaunch,
    start,
    setPreparing,
    setMethod,
    update,
    isCurrent,
    isCancelled,
    requestCancel,
    returnToHome,
  }
})
