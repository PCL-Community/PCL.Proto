<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import { pclText } from '@/api/pclLocale'
import { logSlider, logSaveError, saveLogSettings, updateLogSlider } from '@/api/logSettings'
import sideTip from '@/composables/sideTip'
import { maxLogLines } from '@/util/logLines'
const props = withDefaults(defineProps<{ log?: string; stack?: string }>(), { log: '', stack: '' })
const text = pclText
const cleared = ref(false)
const autoScroll = ref(true)
const logElement = ref<HTMLElement>()
interface GameSession {
  id: string
  instance_directory: string
  pid: number
  running: boolean
  exit_code: number | null
  error: string | null
  lines: string[]
}
const sessions = ref<GameSession[]>([])
const selectedId = ref('')
const selected = computed(() => sessions.value.find((session) => session.id === selectedId.value))
const options = computed(() =>
  sessions.value.map((session) => ({
    key: session.id,
    text: `${session.instance_directory.split(/[\\/]/).pop()} (${session.pid}) - ${session.running ? text('Proto.Session.Running') : session.error || text('Proto.Session.Exited', [session.exit_code ?? '?'])}`,
  })),
)
let timer: ReturnType<typeof setTimeout> | undefined
let request = 0
let mounted = false
async function refresh() {
  const current = ++request
  try {
    const result = await invoke<GameSession[]>('get_game_sessions', {
      id: selectedId.value || null,
    })
    if (!mounted || current !== request) return
    sessions.value = result
    if (!result.some((session) => session.id === selectedId.value))
      selectedId.value = result[0]?.id || ''
  } catch (error) {
    if (mounted && current === request) console.warn('Cannot read game sessions', error)
  }
}
onMounted(() => {
  mounted = true
  async function poll() {
    await refresh()
    if (mounted) timer = setTimeout(poll, 500)
  }
  if (isTauri()) void poll()
})
onUnmounted(() => {
  mounted = false
  request++
  clearTimeout(timer)
})
const maxLines = computed(() => maxLogLines(logSlider.value))
const rawLog = computed(() =>
  selected.value ? selected.value.lines.join('\n') : cleared.value ? '' : props.log,
)
const content = computed(() => rawLog.value.split('\n').slice(-maxLines.value).join('\n'))
async function clearLog() {
  if (!selected.value) {
    cleared.value = true
    return
  }
  try {
    await invoke('clear_game_session_log', { id: selected.value.id })
    await refresh()
  } catch (error) {
    sideTip.show(text('Proto.Session.Error', [String(error)]), 'warn')
  }
}
async function stopGame() {
  if (!selected.value?.running) return
  try {
    await invoke('stop_game_session', { id: selected.value.id })
    await refresh()
  } catch (error) {
    sideTip.show(text('Proto.Session.Error', [String(error)]), 'warn')
    await refresh()
  }
}
watch(selectedId, () => {
  if (mounted) void refresh()
})
watch(
  [content, autoScroll],
  () => {
    if (autoScroll.value && logElement.value)
      logElement.value.scrollTop = logElement.value.scrollHeight
  },
  { flush: 'post' },
)
function exportText(value: string, filename: string) {
  if (!value) {
    sideTip.show(pclText('Proto.Empty.ExportLog'))
    return
  }
  const url = URL.createObjectURL(new Blob([value], { type: 'text/plain;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}
</script>
<template>
  <article class="subview pcl-page log-page">
    <PComboBox
      v-if="sessions.length"
      v-model="selectedId"
      :options="options"
      :title="text('Proto.Session.Select')"
    />
    <PCard hide-title class="log-content">
      <pre ref="logElement">{{ content || pclText('Proto.Empty.Log') }}</pre>
    </PCard>
    <PCard hide-title
      ><div class="log-actions">
        <PCheckBox
          id="CheckAutoScroll"
          v-model="autoScroll"
          :label="text('LogPage.Action.Scroll')"
        />
        <PButton inline @click="() => exportText(rawLog, 'PCL.Proto.log')"
          ><PSvgIcon name="lucide/save" />{{ text('LogPage.Action.Export') }}</PButton
        >
        <PButton inline @click="clearLog"
          ><PSvgIcon name="lucide/trash-2" />{{ text('LogPage.Action.Clear') }}</PButton
        >
        <PButton inline :disabled="!selected?.running" @click="stopGame"
          ><PSvgIcon name="lucide/x" />{{ text('LogPage.Action.KillProcess') }}</PButton
        >
        <PButton inline :disabled="!stack" @click="() => exportText(stack, 'PCL.Proto-stack.log')"
          ><PSvgIcon name="lucide/package" />{{ text('LogPage.Action.ExportStack') }}</PButton
        >
      </div>
      <p v-if="logSaveError" role="alert">
        {{ logSaveError }}
        <button @click="saveLogSettings">{{ text('Launch.Homepage.LoadFailed.Retry') }}</button>
      </p>
      <div class="log-limit">
        <label for="SliderMaxLog">{{ text('LogPage.MaxLines.Title') }}</label
        ><input
          id="SliderMaxLog"
          type="range"
          min="0"
          max="29"
          :value="logSlider"
          @input="updateLogSlider(Number(($event.target as HTMLInputElement).value))"
          :title="text('LogPage.MaxLines.ToolTip')"
        /><output>{{
          Number.isFinite(maxLines) ? maxLines : text('LogPage.MaxLines.Unlimited')
        }}</output>
      </div></PCard
    >
    <small>{{ text('Proto.Session.BufferLimit') }}</small>
  </article>
</template>
<style scoped>
.log-page {
  padding: 25px;
  display: flex;
  flex-direction: column;
  gap: 15px;
  height: 100%;
  overflow-y: auto;
}
.log-content {
  flex: 1;
  min-height: 120px;
}
.log-content pre {
  padding: 15px;
  white-space: pre-wrap;
  font: 12px/1.6 monospace;
  user-select: text;
  overflow: auto;
  max-height: calc(100vh - 220px);
}
.log-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 12px;
}
.log-limit {
  display: flex;
  gap: 25px;
  align-items: center;
  padding: 0 12px 12px;
}
.log-limit input {
  flex: 1;
}
</style>
