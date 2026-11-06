<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { changeServer, listServers, type ServerChange, type ServerEntry } from '@/api/servers'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { useLaunchState } from '@/stores/launchState'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import type { MCPingResult, ExtraItem } from '@/types/mcPing'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import ServerCard from '@/components/widget/ServerCard.vue'

const instance = useSelectedInstance()
const launchState = useLaunchState()
const router = useRouter()
const modal = useModal()
type DisplayServer = ServerEntry & {
  motd?: string
  players?: string
  latency?: number
  queryId: number
}
const servers = ref<DisplayServer[]>([])
const revision = ref('')
const loading = ref(false)
const busy = ref(false)
const loadError = ref<string | null>(null)
let generation = 0
let disposed = false

function descriptionText(description: ExtraItem | ExtraItem[] | null): string {
  if (!description) return ''
  if (typeof description === 'string') return description.replace(/§[0-9a-fk-or]/gi, '')
  if (Array.isArray(description)) return description.map(descriptionText).join('')
  return `${description.text || ''}${(description.extra || []).map(descriptionText).join('')}`
}

function iconUrl(icon: string | null | undefined) {
  if (!icon) return undefined
  const data = icon.startsWith('data:image/png;base64,') ? icon.slice(22) : icon
  return /^[A-Za-z0-9+/=]+$/.test(data) ? `data:image/png;base64,${data}` : undefined
}

async function ping(server: DisplayServer, current: number) {
  const queryId = ++server.queryId
  server.motd = pclText('Instance.Server.Card.ConnectingDots')
  server.players = ''
  server.latency = undefined
  try {
    const [result, latency] = await invoke<[MCPingResult, number]>('server_query', {
      addrStr: server.address,
    })
    if (current !== generation || queryId !== server.queryId) return
    server.motd = descriptionText(result.description)
    server.players = `${result.players.online}/${result.players.max}`
    server.latency = latency
    if (result.favicon) server.icon = result.favicon
  } catch {
    if (current === generation && queryId === server.queryId)
      server.motd = pclText('Instance.Server.Card.Offline')
  }
}

async function pingAll(current: number) {
  let next = 0
  const snapshot = servers.value.slice()
  await Promise.all(
    Array.from({ length: Math.min(snapshot.length, 5) }, async () => {
      while (current === generation && next < snapshot.length) {
        const server = snapshot[next++]
        if (server) await ping(server, current)
      }
    }),
  )
}

async function refresh() {
  const current = ++generation
  const directory = instance.instance_info?.directory
  servers.value = []
  revision.value = ''
  loadError.value = null
  loading.value = false
  if (!directory || !isTauri()) return
  loading.value = true
  try {
    const list = await listServers(directory)
    if (current !== generation) return
    revision.value = list.revision
    servers.value = list.servers.map((server) => ({ ...server, queryId: 0 }))
    void pingAll(current)
  } catch (error) {
    if (current === generation)
      loadError.value = `${pclText('Instance.Server.ReadFileFailed')}: ${error}`
  } finally {
    if (current === generation) loading.value = false
  }
}

async function input(prompt: string, initial: string) {
  const answer = await modal.open({
    title: pclText('Instance.Server.EditTitle'),
    content: prompt,
    showInput: true,
    defaultInputText: initial,
    buttons: [
      { content: pclText('Common.Action.Confirm'), operation: () => modal.close(true) },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  return answer.confirmed ? answer.input?.trim() || '' : null
}

async function save(change: ServerChange, directory: string, expected: string) {
  if (
    busy.value ||
    !isTauri() ||
    disposed ||
    instance.instance_info?.directory !== directory ||
    revision.value !== expected
  )
    return
  busy.value = true
  try {
    await changeServer(directory, expected, change)
    if (!disposed && instance.instance_info?.directory === directory) await refresh()
  } catch (error) {
    sideTip.show(`${pclText('Instance.Server.WriteDataFailed')}: ${error}`, 'warn')
    if (!disposed && instance.instance_info?.directory === directory) await refresh()
  } finally {
    busy.value = false
  }
}

async function edit(index?: number) {
  const directory = instance.instance_info?.directory
  if (!directory || busy.value || !revision.value) return
  const expected = revision.value
  const server = index == null ? undefined : servers.value[index]
  if (index != null && !server) return
  const name = await input(
    pclText('Instance.Server.NamePrompt'),
    server?.name || pclText('Instance.Server.DefaultName'),
  )
  if (!name) return
  const address = await input(pclText('Instance.Server.AddressPrompt'), server?.address || '')
  if (!address) return
  await save(
    index == null ? { action: 'add', name, address } : { action: 'edit', index, name, address },
    directory,
    expected,
  )
}

async function remove(index: number) {
  const directory = instance.instance_info?.directory
  const server = servers.value[index]
  if (!directory || !server || busy.value) return
  const expected = revision.value
  const answer = await modal.open({
    title: pclText('Instance.Server.Card.RemoveConfirmTitle'),
    content: pclText('Instance.Server.Card.RemoveConfirmMessage', [server.name, server.address]),
    buttons: [
      {
        type: 'warn',
        content: pclText('Instance.Server.Card.Remove'),
        operation: () => modal.close(true),
      },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  if (answer.confirmed) await save({ action: 'remove', index }, directory, expected)
}

async function connect(server: DisplayServer) {
  const directory = instance.instance_info?.directory
  if (!directory || !isTauri() || launchState.preparing || launchState.launching) return
  try {
    if (await router.push('/home')) return
    launchState.pendingServer = { directory, address: server.address }
  } catch (error) {
    sideTip.show(pclText('Instance.Server.Card.LaunchFailedMsg', [String(error)]), 'warn')
  }
}

watch(
  () => instance.instance_info?.directory,
  () => void refresh(),
  { immediate: true },
)
onMounted(() => {
  if (!instance.instance_info && isTauri()) void instance.fetch().catch(() => {})
})
onUnmounted(() => {
  disposed = true
  generation++
})
</script>
<template>
  <div class="pcl-page">
    <PLoading v-if="loading" :loading-text="pclText('Tools.ServerQuery.State.Querying')" />
    <PCard v-else-if="loadError" hide-title role="alert">
      <div class="server-error">
        <p>{{ loadError }}</p>
        <PButton @click="refresh">{{ pclText('Common.Action.Refresh') }}</PButton>
      </div>
    </PCard>
    <template v-else-if="servers.length">
      <PCard :title="pclText('Instance.Server.QuickActions')">
        <div class="server-actions">
          <PButton type="tint" @click="refresh">{{
            pclText('Instance.Server.RefreshAll')
          }}</PButton>
          <PButton :disabled="busy" @click="() => edit()">{{
            pclText('Instance.Server.AddServer')
          }}</PButton>
        </div>
      </PCard>
      <ServerCard
        v-for="(server, index) in servers"
        :key="index"
        :name="server.name"
        :address="server.address"
        :icon="iconUrl(server.icon)"
        :motd="server.motd"
        :players="server.players"
        :latency="server.latency"
        @connect="connect(server)"
        @refresh="ping(server, generation)"
        @edit="edit(index)"
        @remove="remove(index)"
      />
    </template>
    <PCard v-else class="pcl-empty-state-card" hide-title>
      <div class="pcl-empty-state">
        <h2 class="pcl-empty-state-title pcl-text">{{ pclText('Instance.Server.Empty.Title') }}</h2>
        <div class="pcl-divider" />
        <p class="pcl-empty-state-message pcl-text">
          {{ pclText('Instance.Server.Empty.Message') }}
        </p>
        <div class="pcl-empty-state-actions">
          <PButton type="tint" @click="refresh">{{
            pclText('Instance.Server.RefreshInfo')
          }}</PButton>
          <PButton :disabled="busy || !revision" @click="() => edit()">{{
            pclText('Instance.Server.AddServer')
          }}</PButton>
        </div>
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.server-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
  padding: 0 25px 15px;
}
.server-error {
  padding: 17px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 15px;
  flex-wrap: wrap;
}
.server-error p {
  margin: 0;
}
.server-actions :deep(button) {
  min-width: 140px;
  min-height: 35px;
}
</style>
