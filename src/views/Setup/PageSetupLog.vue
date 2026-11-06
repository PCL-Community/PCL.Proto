<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { pclFormatLocale, pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import {
  clearLogFiles,
  exportLogFiles,
  listLogFiles,
  openLogDirectory,
  openLogPath,
  type LogFileEntry,
} from '@/api/logs'
import { isTauri } from '@tauri-apps/api/core'

const text = pclText
const modal = useModal()

const loading = ref(false)
const loadError = ref<string | null>(null)
const logs = ref<LogFileEntry[]>([])

async function refresh() {
  loading.value = true
  loadError.value = null
  try {
    logs.value = isTauri() ? await listLogFiles() : []
  } catch (error) {
    loadError.value = String(error)
  } finally {
    loading.value = false
  }
}
onMounted(() => void refresh())

// CE 的启动日志命名 `Launch-yyyy-M-d-HHmmssfff.log`，列表中显示为本地化时间
const launchLogName = /^Launch-(\d{4})-(\d{1,2})-(\d{1,2})-(\d{2})(\d{2})(\d{2})(\d{3})$/

function logTitle(entry: LogFileEntry) {
  const stem = entry.name.replace(/\.log$/i, '')
  const launch = launchLogName.exec(stem)
  if (launch) {
    const [, year, month, day, hour, minute, second, milli] = launch
    const date = new Date(+year, +month - 1, +day, +hour, +minute, +second, +milli)
    const title = date.toLocaleString(pclFormatLocale.value)
    return entry.current ? text('Setup.Log.CurrentSuffix', [title]) : title
  }
  if (stem.startsWith('LastPending')) {
    const rest = stem.slice(11)
    return rest.length > 1
      ? text('Setup.Log.TempStored', [rest.slice(1)])
      : text('Setup.Log.TempUnoutput')
  }
  return entry.name
}

const currentLog = computed(() => logs.value.find((entry) => entry.current))

async function exportCurrent() {
  await exportLogFiles(currentLog.value ? [currentLog.value.path] : [])
}

async function exportAll() {
  await exportLogFiles(logs.value.map((entry) => entry.path))
}

async function openLog(entry: LogFileEntry) {
  try {
    await openLogPath(entry.path)
  } catch (error) {
    sideTip.show(String(error), 'warn')
  }
}

async function openDirectory() {
  try {
    await openLogDirectory()
  } catch (error) {
    sideTip.show(String(error), 'warn')
  }
}

async function clearHistory() {
  const answer = await modal.open({
    title: text('Setup.Log.Clear.Confirm.Title'),
    content: text('Setup.Log.Clear.Confirm.Message'),
    buttons: [
      { type: 'warn', content: text('Common.Action.Confirm'), operation: () => modal.close(true) },
      { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  if (!answer.confirmed) return
  try {
    await clearLogFiles()
    sideTip.show(text('Setup.Log.Clear.Success'), 'success')
    await refresh()
  } catch (error) {
    sideTip.show(String(error), 'warn')
  }
}
</script>
<template>
  <div class="pcl-page">
    <PCard :title="text('Setup.Log.Operations')">
      <div
        style="
          margin: 0px 20px 20px 25px;
          display: flex;
          flex-direction: row;
          align-items: center;
          flex-wrap: wrap;
        "
      >
        <PButton
          style="min-width: 100px; margin: 0px 15px 0px 0px"
          @click="exportCurrent"
          type="tint"
          >{{ text('Setup.Log.Export') }}</PButton
        >
        <PButton style="min-width: 120px; margin: 0px 15px 0px 0px" @click="exportAll">{{
          text('Setup.Log.ExportAll')
        }}</PButton>
        <PButton style="min-width: 120px; margin: 0px 15px 0px 0px" @click="openDirectory">{{
          text('Setup.Log.OpenDirectory')
        }}</PButton>
        <PButton style="min-width: 120px" @click="clearHistory" type="warn">{{
          text('Setup.Log.ClearHistory')
        }}</PButton>
      </div>
    </PCard>
    <PCard :title="text('Setup.Log.AllLogs')">
      <div style="margin: 0px 18px 18px 20px; display: flex; flex-direction: column">
        <PLoading v-if="loading" :card="false" :loading-text="text('Proto.Download.Loading')" />
        <PHint v-else-if="loadError" severity="error">{{ loadError }}</PHint>
        <template v-else-if="logs.length">
          <PListItem
            v-for="entry in logs"
            :key="entry.path"
            clickable
            :title="logTitle(entry)"
            :subtitle="entry.path"
            @click="openLog(entry)"
          />
        </template>
        <p v-else class="pcl-empty">{{ text('Proto.Empty.LogFiles') }}</p>
      </div>
    </PCard>
  </div>
</template>
