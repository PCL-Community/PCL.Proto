<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PHint from '@/components/widget/PHint.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PErrorRetry from '@/components/widget/PErrorRetry.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PPath from '@/components/widget/PPath.vue'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import { useNavAction } from '@/composables/navAction'
import { pclText } from '@/api/pclLocale'
import { pclImagePath } from '@/api/pclAssets'
import { openExternal } from '@/util/externalLink'
import { isTauri } from '@tauri-apps/api/core'
import { getLinkRuntimeStatus, installLinkRuntime, type LinkRuntimeStatus } from '@/api/linkRuntime'
import type { LinkNatType, LinkPlayer, LinkStateName } from '@/api/link'
import { useLinkStore, linkErrorMessage } from '@/stores/link'
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useRoute } from 'vue-router'

const GRASS_ICON = pclImagePath('images/Blocks/Grass.png', import.meta.env.BASE_URL)
const QUALITY_PATH =
  'M512 128c212.064 0 384 171.936 384 384s-173.632 384-384 384C299.936 896 128 724.064 128 512S299.936 128 512 128z m-126.4 575.968c-1.728 40.8 3.52 76.8 36.32 115.2 28.576 8.32 58.816 12.832 90.08 12.832 81.984 0 156.8-30.848 213.408-81.536-77.12 7.2-161.6-5.952-289.216-61.856-11.616 7.52-18.976 12.608-50.592 15.36z m-193.44-202.368c0 123.744 53.536 214.272 135.744 272.16-6.08-25.6-8.352-48.448-4.96-91.68-21.344-17.6-34.944-39.264-34.624-82.08-50.816-42.88-82.88-75.712-96.16-98.4zM544 448c-14.944 0-29.12-3.424-41.728-9.504-10.976 12.96-21.376 26.432-31.2 40.512-4.832 6.912-15.36 23.776-31.616 50.624a95.872 95.872 0 0 1 36.384 106.432c155.52 68.16 302.56 62.048 325.952 11.904 23.36-50.112-35.968-139.776-189.504-228.48A95.68 95.68 0 0 1 544 448z m-160 128a32 32 0 1 0 0 64 32 32 0 0 0 0-64z m-161.92-199.2c-16.576 35.52 16.96 99.68 94.08 163.2a95.936 95.936 0 0 1 59.616-27.648c19.936-35.008 30.72-52.736 42.88-70.08 10.944-15.648 22.528-30.656 38.432-49.44-5.696-12.128-8.96-25.6-9.088-39.872-118.176-30.4-206.88-16.96-225.92 23.84z m415.136-47.84c1.824 7.36 3.296 19.104 2.336 32.416 88.96 48.704 153.76 105.056 192.32 160.704 0-118.656-34.208-176.064-75.52-210.976-49.792-5.92-69.952-4.064-119.136 17.856zM544 320a32 32 0 1 0 0 64 32 32 0 0 0 0-64z m-32-128c-82.016 0-156.8 30.848-213.44 81.6 47.264-4.384 96-0.544 170.304 18.624A95.84 95.84 0 0 1 544 256c21.568 0 41.472 7.104 57.504 19.104 35.968-17.28 62.016-25.824 87.136-29.984A318.528 318.528 0 0 0 512 192z'
const PING_PATH =
  'M512 409.6a102.4 102.4 0 0 0-51.2 190.464V1024h102.4v-423.936A102.4 102.4 0 0 0 512 409.6z M512 0a512 512 0 0 0-263.168 950.272l51.2-87.552a409.6 409.6 0 1 1 420.864 0l51.2 87.552A512 512 0 0 0 512 0z M819.2 512a307.2 307.2 0 1 0-464.896 263.168l51.2-88.064a204.8 204.8 0 1 1 209.92 0l51.2 88.064A307.2 307.2 0 0 0 819.2 512z'
const LOBBY_ID_PATH =
  'M21.15,2.86a2.89,2.89,0,0,0-3-.71L4,6.88a2.9,2.9,0,0,0-.12,5.47l5.24,2h0a.93.93,0,0,1,.53.52l2,5.25A2.87,2.87,0,0,0,14.36,22h.07a2.88,2.88,0,0,0,2.69-2L21.85,5.83A2.89,2.89,0,0,0,21.15,2.86ZM20,5.2,15.22,19.38a.88.88,0,0,1-.84.62.92.92,0,0,1-.87-.58l-2-5.25a2.91,2.91,0,0,0-1.67-1.68l-5.25-2A.9.9,0,0,1,4,9.62a.88.88,0,0,1,.62-.84L18.8,4.05A.91.91,0,0,1,20,5.2Z'
const CONNECTION_TYPE_PATH =
  'M215.625 18.75A9.375 9.375 0 0 1 225 28.125V249.24375L283.9875 190.2375A9.375 9.375 0 0 1 297.2625000000001 203.5125L222.2625 278.5125000000001A9.375 9.375 0 0 1 208.9875 278.5125000000001L133.9875 203.5125A9.375 9.375 0 1 1 147.2625 190.2375L206.25 249.24375V28.125A9.375 9.375 0 0 1 215.625 18.75zM84.375 281.25A9.375 9.375 0 0 0 93.75 271.875V50.75625L152.7375 109.7625A9.375 9.375 0 0 0 166.0125 96.4875L91.0125 21.4875A9.375 9.375 0 0 0 77.7375 21.4875L2.7375 96.4875A9.375 9.375 0 0 0 16.0125 109.7625L75 50.75625V271.875A9.375 9.375 0 0 0 84.375 281.25z'
const USER_PATH =
  'M512 447.223c-88.224 0-160-71.776-160-160s71.776-160 160-160c88.225 0 160 71.776 160 160s-71.775 160-160 160z m0-256c-52.935 0-96 43.065-96 96s43.065 96 96 96 96-43.065 96-96-43.065-96-96-96zM454.901 870.594c-96.594 0-184.933-3.802-231.263-49.955C203.308 800.386 193 774.164 193 742.701c0-31.629 10.247-62.812 30.457-92.686 17.978-26.573 42.908-50.741 74.098-71.833C359.256 536.46 437.418 512.53 512 512.53c74.55 0 152.55 23.943 214.002 65.691 31.05 21.094 55.861 45.273 73.746 71.867C819.822 679.937 830 711.096 830 742.701c0 31.552-10.317 57.827-30.664 78.097-50.714 50.521-151.822 50.128-258.88 49.723a7395.45 7395.45 0 0 0-56.914-0.001c-9.605 0.037-19.163 0.074-28.641 0.074zM512 806.447c9.567 0 19.149 0.037 28.701 0.073 49.52 0.191 96.284 0.37 135.808-4.396 38.418-4.633 64.546-13.604 77.659-26.668 5.079-5.06 11.832-13.96 11.832-32.755 0-38.089-27.688-78.744-75.963-111.54C638.933 596.442 574.04 576.53 512 576.53c-126.309 0-255 83.862-255 166.171 0 18.675 6.738 27.547 11.807 32.596 32.045 31.922 128.975 31.55 214.491 31.224 9.556-0.037 19.139-0.074 28.702-0.074z'
const USER_TYPE_PATH =
  'M274.773333 419.498667h154.624v76.8H274.773333v-76.8z m0-154.965334h399.701334V341.333333H274.773333V264.533333z M92.16 105.472v832.170667h179.541333v-76.8H168.96V182.272h678.229333v227.669333h76.8V105.472H92.16z M645.12 720.213333c91.818667-1.024 165.888-75.776 165.888-167.936 0-92.842667-75.093333-167.936-167.936-167.936s-167.936 75.093333-167.936 167.936c0 92.16 74.069333 166.912 165.888 167.936-129.365333 1.024-238.933333 90.794667-279.210667 215.722667h81.92c35.84-81.92 111.616-138.922667 198.997334-138.922667s163.498667 56.661333 198.997333 138.922667h82.261333c-40.277333-124.586667-149.845333-214.698667-278.869333-215.722667z m-2.389333-76.8c-50.176 0-91.477333-40.96-91.477334-91.477333 0-50.176 40.96-91.477333 91.477334-91.477333 50.176 0 91.477333 40.96 91.477333 91.477333s-40.96 91.477333-91.477333 91.477333z'
const PYSIO_HOME = "Pysio's Home"
const CONNECTED_STATES: LinkStateName[] = [
  'host-starting',
  'guest-connecting',
  'guest-starting',
  'host-ok',
  'guest-ok',
]
const NAT_NAME_KEYS: Record<LinkNatType, string> = {
  open: 'Link.Nat.Type.Open',
  no_pat: 'Link.Nat.Type.Open',
  full_cone: 'Link.Nat.Type.FullCone',
  restricted: 'Link.Nat.Type.Restricted',
  port_restricted: 'Link.Nat.Type.PortRestricted',
  symmetric_easy: 'Link.Nat.Type.SymmetricEasy',
  symmetric: 'Link.Nat.Type.Symmetric',
  symmetric_firewall: 'Link.Nat.Type.SymmetricFirewall',
  udp_blocked: 'Link.Nat.Type.UdpBlocked',
  unknown: 'Link.Nat.Type.Unknown',
}
const EXCEPTION_KEYS: Record<number, string> = {
  0: 'Link.Lobby.JoinFailed',
  1: 'Link.Lobby.JoinFailed',
  2: 'Link.Lobby.JoinFailed',
  3: 'Link.Lobby.CreateRoomFailed',
  5: 'Link.Lobby.JoinFailed',
}

const link = useLinkStore()
const route = useRoute()
// 旧书签 /tools/lobby/inner?code=... 重定向后仍带着大厅编号。
const routeCode = computed(() =>
  typeof route.query.code === 'string' && route.query.code ? route.query.code : null,
)
const joinCode = ref<string>()
const selectedPort = ref<string>()
type PendingLobby = { kind: 'create'; port: number } | { kind: 'join'; code: string }
const pending = shallowRef<PendingLobby | null>(null)
const noticeIndex = ref(0)
const actionError = ref<string | null>(null)
const initError = ref<string | null>(null)
const natFailed = ref(false)
const runtimeBusy = ref(false)
const runtimePhase = ref<string | null>(null)
const runtimePercent = ref<number | null>(null)
const handlingException = ref(false)
const canStart = computed(
  () =>
    !pending.value &&
    !runtimeBusy.value &&
    !handlingException.value &&
    !link.busy.creating &&
    !link.busy.joining &&
    !link.busy.leaving &&
    !link.busy.resetting &&
    !link.busy.eula,
)

const subpage = computed<'eula' | 'select' | 'finish'>(() => {
  if (link.eula !== true) return 'eula'
  if (pending.value !== null) return 'finish'
  if (CONNECTED_STATES.includes(link.lobby?.state ?? 'waiting')) return 'finish'
  return 'select'
})
const isHost = computed(() => {
  if (pending.value) return pending.value.kind === 'create'
  return (link.lobby?.state ?? '').startsWith('host-')
})
const isGuest = computed(() => !isHost.value)
const isConnected = computed(
  () => link.lobby?.state === 'host-ok' || link.lobby?.state === 'guest-ok',
)
const lobbyCode = computed(
  () => link.lobby?.room ?? (pending.value?.kind === 'join' ? pending.value.code : null),
)
const players = computed(() => {
  const list = [...(link.lobby?.players ?? [])]
  return list.sort((a, b) => (a.kind === 'HOST' ? 0 : 1) - (b.kind === 'HOST' ? 0 : 1))
})
const memberTitle = computed(() =>
  isConnected.value
    ? pclText('Tools.GameLink.Member.ListCount', [players.value.length])
    : pclText('Tools.GameLink.Member.ListLoading'),
)
const displayName = computed(() => {
  const custom = link.settings?.username?.trim()
  if (link.announcement?.allowCustomName && custom) return custom
  return link.account?.username ?? null
})
const userName = computed(() => {
  const local = players.value.find((player) => player.kind === 'LOCAL')
  if (local) return local.name
  if (isHost.value) {
    return players.value.find((player) => player.kind === 'HOST')?.name ?? displayName.value
  }
  return displayName.value
})
const userTypeText = computed(() =>
  isHost.value ? pclText('Tools.GameLink.Finish.Host') : pclText('Tools.GameLink.Finish.Guest'),
)
const qualityText = computed(() =>
  isConnected.value
    ? pclText('Tools.GameLink.Finish.Connected')
    : pclText('Tools.GameLink.Finish.Checking'),
)
const pingText = computed(() => {
  const latency = link.lobby?.latencyMs
  return latency == null ? '-ms' : pclText('Tools.GameLink.Finish.PingMs', [latency])
})
const connectionTypeText = computed(() => {
  switch (link.lobby?.connectionType) {
    case 'p2p':
      return pclText('Link.Connection.P2P')
    case 'relay':
      return pclText('Link.Connection.Relay')
    default:
      return pclText('Link.Connection.Unknown')
  }
})
const exitLabel = computed(() =>
  isHost.value
    ? pclText('Tools.GameLink.Finish.CloseLobby')
    : pclText('Tools.GameLink.Finish.Exit'),
)
const worldOptions = computed(() =>
  link.worlds.map((world) => ({
    key: String(world.port),
    text: pclText('Link.Lobby.WorldNameFormat', [
      world.description || world.version || '?',
      world.version || '?',
      world.port,
    ]),
    title: pclText('Link.Lobby.WorldNameFormat', [
      world.description || world.version || '?',
      world.version || '?',
      world.port,
    ]),
  })),
)
const showAccount = computed(() => link.account?.configured === true)
const accountPending = computed(() => link.account?.pending === true || link.busy.account)
const accountText = computed(() => {
  const account = link.account
  if (!account) return pclText('Tools.GameLink.Natayark.Loading')
  if (account.pending) return pclText('Tools.GameLink.Natayark.BrowserContinue')
  if (account.loggedIn && (account.expired || account.error || !account.username)) {
    return pclText('Link.Mod.ReLoginRequired')
  }
  if (account.loggedIn) {
    const name = account.username ?? pclText('Tools.GameLink.Natayark.Loading')
    return account.status !== null && account.status !== 0
      ? name + ' ' + pclText('Tools.GameLink.Natayark.Abnormal')
      : name
  }
  return account.error
    ? pclText('Tools.GameLink.Natayark.FetchFailed')
    : pclText('Tools.GameLink.Natayark.Login')
})
const accountDim = computed(() => {
  const account = link.account
  if (!account) return false
  if (account.pending || account.error) return true
  return account.loggedIn && account.status !== null && account.status !== 0
})
const natText = computed(() => {
  if (link.busy.nat) return pclText('Tools.GameLink.Nat.Testing')
  if (natFailed.value) return pclText('Tools.GameLink.Nat.Failed')
  if (link.nat) {
    const udp = pclText(NAT_NAME_KEYS[link.nat.udp] ?? NAT_NAME_KEYS.unknown)
    const tcp = pclText(NAT_NAME_KEYS[link.nat.tcp] ?? NAT_NAME_KEYS.unknown)
    return pclText('Tools.GameLink.Nat.Result', [udp, tcp])
  }
  return pclText('Tools.GameLink.Nat.Test')
})
const notices = computed(() => link.announcement?.notices ?? [])
const currentNotice = computed(() => {
  if (!notices.value.length) return null
  return notices.value[noticeIndex.value % notices.value.length] ?? null
})
const announcementText = computed<string | null>(() => {
  if (!link.announcement) return pclText('Tools.GameLink.Loading.ConnectingServer')
  if (link.announcement.updateRequired) return pclText('Tools.GameLink.Error.UpdateRequired')
  if (link.announcement.error) return pclText('Tools.GameLink.Error.ConnectFailed')
  const notice = currentNotice.value
  if (!notice) return null
  const prefixKey =
    notice.type === 'important'
      ? 'Tools.GameLink.Announcement.Important'
      : notice.type === 'warning'
        ? 'Tools.GameLink.Announcement.Warning'
        : 'Tools.GameLink.Announcement.Notice'
  return pclText('Tools.GameLink.Announcement.Format', [pclText(prefixKey), notice.content])
})
const announcementSeverity = computed<'info' | 'warning' | 'error'>(() => {
  if (link.announcement?.updateRequired || link.announcement?.error) return 'error'
  if (currentNotice.value?.type === 'important') return 'error'
  if (currentNotice.value?.type === 'warning') return 'warning'
  return 'info'
})
const runtimeText = computed<string | null>(() => {
  if (!runtimeBusy.value) return null
  if (runtimePhase.value === 'downloading') {
    return runtimePercent.value === null
      ? pclText('Link.Mod.DownloadingDeps')
      : pclText('Proto.Link.Runtime.Downloading', [runtimePercent.value])
  }
  if (runtimePhase.value === 'verifying') return pclText('Proto.Link.Runtime.Verifying')
  if (runtimePhase.value === 'installing') return pclText('Proto.Link.Runtime.Installing')
  return pclText('Link.Mod.DownloadingDeps')
})

let noticeTimer: ReturnType<typeof setInterval> | null = null
function startNoticeRotation() {
  stopNoticeRotation()
  noticeTimer = setInterval(() => {
    if (noticeIndex.value >= notices.value.length - 1 || notices.value.length === 0) {
      noticeIndex.value = 0
      return
    }
    noticeIndex.value += 1
  }, 10000)
}
function stopNoticeRotation() {
  if (noticeTimer) {
    clearInterval(noticeTimer)
    noticeTimer = null
  }
}

function playerSubtitle(player: LinkPlayer) {
  if (player.kind !== 'HOST') return player.vendor
  return pclText('Tools.GameLink.Player.Details', [
    pclText('Tools.GameLink.Player.Host'),
    player.vendor,
  ])
}

async function confirmDialog(
  title: string,
  content: string,
  confirmText: string,
  cancelText: string,
): Promise<boolean> {
  const modal = useModal()
  const result = await modal.open({
    title,
    content,
    buttons: [
      { type: 'tint', content: confirmText, operation: () => modal.close(true) },
      { type: 'warn', content: cancelText, operation: () => modal.close(false) },
    ],
  })
  return result.confirmed
}

async function alertDialog(title: string, content: string) {
  const modal = useModal()
  await modal.open({
    title,
    content,
    buttons: [
      {
        type: 'tint',
        content: pclText('Common.Action.Confirm'),
        operation: () => modal.close(true),
      },
    ],
  })
}

async function run(task: () => Promise<unknown>, fallbackKey?: string, onFailure?: () => void) {
  actionError.value = null
  try {
    await task()
    return true
  } catch (error) {
    actionError.value = linkErrorMessage(error, fallbackKey)
    sideTip.show(actionError.value, 'warn')
    onFailure?.()
    return false
  }
}

async function copyText(value: string | null | undefined, successKey?: string) {
  if (!value) {
    sideTip.show(pclText('Proto.Clipboard.Unavailable'), 'warn')
    return
  }
  try {
    await navigator.clipboard.writeText(value)
    if (successKey) sideTip.show(pclText(successKey), 'success')
  } catch {
    sideTip.show(pclText('Proto.Clipboard.CopyFailed'), 'warn')
  }
}

async function initialize() {
  initError.value = null
  try {
    await link.initialize()
    if (!joinCode.value && routeCode.value) joinCode.value = routeCode.value
    if (link.eula === true) void refreshWorlds()
    if (link.lobby?.state === 'exception') void handleException()
  } catch (error) {
    initError.value = linkErrorMessage(error)
    sideTip.show(initError.value, 'warn')
  }
}

async function refreshWorlds() {
  await run(() => link.discoverWorlds())
}

async function acceptEula() {
  await run(async () => {
    await link.setEula(true)
    if (link.eula) void refreshWorlds()
  })
}

async function disableLink() {
  const confirmed = await confirmDialog(
    pclText('Tools.GameLink.Eula.RevokeTitle'),
    pclText('Tools.GameLink.Eula.RevokeConfirm'),
    pclText('Common.Action.Confirm'),
    pclText('Common.Action.Cancel'),
  )
  if (!confirmed) return
  pending.value = null
  await run(async () => {
    await link.setEula(false)
    sideTip.show(pclText('Tools.GameLink.Eula.Disabled'), 'default')
  })
}

async function testNat() {
  if (link.busy.nat) return
  natFailed.value = false
  await run(
    () => link.testNat(),
    'Tools.GameLink.Error.NetworkTestFailed',
    () => {
      natFailed.value = true
    },
  )
}

async function toggleAccount() {
  const account = link.account
  if (!account || account.pending || link.busy.account) return
  if (!account.loggedIn) {
    const confirmed = await confirmDialog(
      pclText('Tools.GameLink.Natayark.LoginTitle'),
      pclText('Tools.GameLink.Natayark.LoginPrompt'),
      pclText('Tools.GameLink.Natayark.Continue'),
      pclText('Common.Action.Cancel'),
    )
    if (!confirmed) return
    await run(
      () => link.startLogin(),
      undefined,
      () => sideTip.show(pclText('Tools.GameLink.Natayark.LoginFailed'), 'warn'),
    )
    return
  }
  const confirmed = await confirmDialog(
    pclText('Tools.GameLink.Natayark.LogoutTitle'),
    pclText('Tools.GameLink.Natayark.LogoutConfirm'),
    pclText('Common.Action.Confirm'),
    pclText('Common.Action.Cancel'),
  )
  if (!confirmed) return
  await run(async () => {
    await link.logoutAccount()
    sideTip.show(pclText('Tools.GameLink.Natayark.LogoutComplete'), 'success')
  })
}

function selectionAfterWorldsChange() {
  if (!link.worlds.length) {
    selectedPort.value = undefined
    return
  }
  if (!link.worlds.some((world) => String(world.port) === selectedPort.value)) {
    selectedPort.value = String(link.worlds[0].port)
  }
}

async function pasteCode() {
  try {
    const text = await navigator.clipboard.readText()
    if (!text) {
      sideTip.show(pclText('Tools.GameLink.Join.InvalidText'), 'warn')
      return
    }
    joinCode.value = text
  } catch {
    sideTip.show(pclText('Proto.Clipboard.ReadLobbyFailed'), 'warn')
  }
}

function clearCode() {
  joinCode.value = undefined
}

async function ensureRuntime(): Promise<boolean> {
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.Link'), 'warn')
    return false
  }
  let status: LinkRuntimeStatus
  try {
    status = await getLinkRuntimeStatus()
  } catch (error) {
    sideTip.show(pclText('Proto.Link.Runtime.Failed', [String(error)]), 'warn')
    return false
  }
  if (status.state === 'ready') return true
  if (status.state === 'needs_restart') {
    sideTip.show(pclText('Proto.Link.Runtime.StatusNeedsRestart'), 'warn')
    return false
  }
  const modal = useModal()
  const choice = await modal.open({
    title: pclText('Proto.Link.Runtime.Title'),
    content: pclText('Proto.Link.Runtime.Message'),
    buttons: [
      {
        type: 'tint',
        content: pclText('Proto.Link.Runtime.Install'),
        operation: () => modal.close(true),
      },
      {
        type: 'default',
        content: pclText('Proto.Link.Runtime.LearnMore'),
        operation: () => {
          void openExternal('https://npcap.com/')
        },
      },
      {
        type: 'warn',
        content: pclText('Common.Action.Cancel'),
        operation: () => modal.close(false),
      },
    ],
  })
  if (!choice.confirmed) {
    sideTip.show(pclText('Proto.Link.Runtime.Cancelled'), 'default')
    return false
  }
  runtimeBusy.value = true
  runtimePhase.value = 'checking'
  runtimePercent.value = null
  let cancelled = false
  try {
    const next = await installLinkRuntime((event) => {
      runtimePhase.value = event.phase
      if (event.phase === 'cancelled') cancelled = true
      if (event.phase === 'downloading' && typeof event.progress === 'number') {
        runtimePercent.value = Math.round(event.progress * 100)
      }
    })
    if (cancelled) {
      sideTip.show(pclText('Proto.Link.Runtime.Cancelled'), 'default')
      return false
    }
    if (next.state === 'ready') {
      sideTip.show(pclText('Proto.Link.Runtime.Ready'), 'success')
      return true
    }
    if (next.state === 'needs_restart') {
      sideTip.show(pclText('Proto.Link.Runtime.NeedsRestart'), 'warn')
      return false
    }
    sideTip.show(pclText('Proto.Link.Runtime.Failed', [next.message ?? '']), 'warn')
    return false
  } catch (error) {
    sideTip.show(pclText('Proto.Link.Runtime.Failed', [String(error)]), 'warn')
    return false
  } finally {
    runtimeBusy.value = false
    runtimePhase.value = null
    runtimePercent.value = null
  }
}

async function enterLobby(request: PendingLobby) {
  pending.value = request
  try {
    if (!(await ensureRuntime()) || pending.value !== request || link.eula !== true) return
    if (request.kind === 'create') await link.create(request.port)
    else await link.join(request.code)
  } catch (error) {
    if (pending.value !== request) return
    actionError.value = linkErrorMessage(error)
    sideTip.show(actionError.value, 'warn')
  } finally {
    if (pending.value === request) pending.value = null
  }
}

async function createLobby() {
  if (!canStart.value) return
  const port = Number(selectedPort.value)
  if (!Number.isInteger(port) || port < 1024 || port > 65535) {
    sideTip.show(pclText('Tools.GameLink.Create.NoWorld'), 'warn')
    return
  }
  actionError.value = null
  await enterLobby({ kind: 'create', port })
}

async function manualCreate() {
  if (!canStart.value) return
  const modal = useModal()
  const result = await modal.open({
    title: pclText('Tools.GameLink.Create.EnterPort'),
    showInput: true,
    buttons: [
      {
        type: 'tint',
        content: pclText('Common.Action.Confirm'),
        operation: () => modal.close(true),
      },
      {
        type: 'warn',
        content: pclText('Common.Action.Cancel'),
        operation: () => modal.close(false),
      },
    ],
  })
  if (!result.confirmed || !result.input || !canStart.value) return
  const raw = result.input.trim()
  const port = Number(raw)
  if (!/^\d+$/.test(raw) || port < 1024 || port > 65535) {
    sideTip.show(pclText('Tools.GameLink.Create.NotMcPort'), 'warn')
    return
  }
  actionError.value = null
  await enterLobby({ kind: 'create', port })
}

async function joinLobby() {
  if (!canStart.value) return
  const code = joinCode.value?.trim()
  if (!code) {
    sideTip.show(pclText('Tools.GameLink.Join.InvalidText'), 'warn')
    return
  }
  actionError.value = null
  await enterLobby({ kind: 'join', code })
}

async function copyLobbyCode() {
  await copyText(lobbyCode.value, 'Common.Hint.Copied')
}

async function copyIp() {
  const url = link.lobby?.url
  if (!url) return
  const modal = useModal()
  await modal.open({
    title: pclText('Tools.GameLink.CopyIp.Title'),
    content: pclText('Tools.GameLink.CopyIp.Message', [url]),
    buttons: [
      {
        type: 'tint',
        content: pclText('Common.Action.Copy'),
        operation: async () => {
          await copyText(url)
          modal.close(true)
        },
      },
      {
        type: 'default',
        content: pclText('Tools.GameLink.CopyIp.Back'),
        operation: () => modal.close(false),
      },
    ],
  })
}

async function exitLobby() {
  const bodyKey = isHost.value
    ? 'Tools.GameLink.Exit.ConfirmMessageWithHost'
    : 'Tools.GameLink.Exit.ConfirmMessage'
  const confirmed = await confirmDialog(
    pclText('Tools.GameLink.Exit.ConfirmTitle'),
    pclText(bodyKey),
    pclText('Common.Action.Confirm'),
    pclText('Common.Action.Cancel'),
  )
  if (!confirmed) return
  pending.value = null
  if (await run(() => link.leave())) void refreshWorlds()
}

async function openPlayer(player: LinkPlayer) {
  await alertDialog(
    pclText('Tools.GameLink.Player.InfoTitle', [player.name]),
    pclText('Tools.GameLink.Player.InfoMessage', [player.name, player.vendor]),
  )
}

async function handleException() {
  if (handlingException.value) return
  handlingException.value = true
  try {
    const exception = link.lobby?.exception ?? null
    if (exception === 4) {
      await alertDialog(
        pclText('Tools.GameLink.Exit.DisbandedTitle'),
        pclText('Tools.GameLink.Exit.Disbanded'),
      )
    } else {
      sideTip.show(pclText(EXCEPTION_KEYS[exception ?? 0] ?? EXCEPTION_KEYS[0]), 'warn')
    }
    if (link.lobby?.state !== 'exception') return
    pending.value = null
    await link.resetLobby()
    void refreshWorlds()
  } catch (error) {
    sideTip.show(linkErrorMessage(error), 'warn')
  } finally {
    handlingException.value = false
  }
}

watch(() => link.worlds, selectionAfterWorldsChange, { immediate: true })
watch(
  () => link.announcement?.notices.length,
  () => {
    noticeIndex.value = 0
  },
)
watch(
  () => link.account,
  (next, previous) => {
    if (!next || !previous) return
    if (!previous.pending || next.pending) return
    if (next.error || next.expired)
      sideTip.show(pclText('Tools.GameLink.Natayark.LoginFailed'), 'warn')
    else if (next.loggedIn && next.username)
      sideTip.show(pclText('Tools.GameLink.Natayark.LoginComplete'), 'success')
  },
)
watch(
  () => link.lobby?.state,
  (state) => {
    if (state === 'exception') void handleException()
  },
)

useNavAction('refresh', initialize)

onMounted(() => {
  link.startPolling()
  startNoticeRotation()
  void initialize()
})
onUnmounted(() => {
  pending.value = null
  link.stopPolling()
  stopNoticeRotation()
})
</script>

<template>
  <section class="pcl-page lobby-page">
    <PErrorRetry v-if="initError" :error="initError" @retry="initialize" />
    <template v-if="link.ready">
      <PHint v-if="announcementText" class="announcement-hint" :severity="announcementSeverity">
        <span class="announcement-text">{{ announcementText }}</span>
      </PHint>
      <PHint v-if="runtimeText" severity="info">{{ runtimeText }}</PHint>
      <PHint v-if="link.pollError" severity="error">{{ link.pollError }}</PHint>
      <PHint v-if="actionError" severity="error" closable @close="actionError = null">
        {{ actionError }}
      </PHint>

      <template v-if="subpage === 'eula'">
        <PCard :title="pclText('Tools.GameLink.Agreement.Title')">
          <div class="card-body eula-body">
            <p class="eula-description">
              {{ pclText('Tools.GameLink.Agreement.Description') }}
            </p>
            <PListItem
              :icon="GRASS_ICON"
              :title="pclText('Tools.GameLink.Agreement.PrivacyPolicy')"
              :subtitle="pclText('Tools.GameLink.Agreement.PrivacyPolicyInfo')"
              clickable
              @click="openExternal('https://www.pclc.cc/privacy/personal-info-brief.html')"
            />
            <PListItem
              :icon="GRASS_ICON"
              :title="pclText('Tools.GameLink.Agreement.NatayarkTerms')"
              :subtitle="pclText('Tools.GameLink.Agreement.NatayarkTermsInfo')"
              clickable
              @click="openExternal('https://account.naids.com/policy')"
            />
            <p
              v-for="paragraph in pclText('Tools.GameLink.Agreement.Body').split(/\n+/)"
              :key="paragraph"
            >
              {{ paragraph }}
            </p>
            <div class="eula-actions">
              <PButton type="tint" :disabled="link.busy.eula" @click="acceptEula">
                {{ pclText('Tools.GameLink.Agreement.Accept') }}
              </PButton>
            </div>
          </div>
        </PCard>
      </template>

      <template v-else-if="subpage === 'select'">
        <div class="status-chips">
          <PCard
            hide-title
            class="status-chip"
            role="button"
            tabindex="0"
            :title="pclText('Tools.GameLink.Nat.Title')"
            @click="testNat"
            @keydown.enter.prevent="testNat"
          >
            <div class="status-chip-body">
              <PSvgIcon class="status-chip-icon" name="lucide/ethernet-port" :size="14" />
              <span class="status-chip-text">{{ natText }}</span>
            </div>
          </PCard>
          <PCard
            v-if="showAccount"
            hide-title
            class="status-chip"
            :class="{ 'status-chip-dim': accountDim, 'status-chip-disabled': accountPending }"
            role="button"
            tabindex="0"
            :title="pclText('Tools.GameLink.Natayark.Account')"
            @click="toggleAccount"
            @keydown.enter.prevent="toggleAccount"
          >
            <div class="status-chip-body">
              <PSvgIcon class="status-chip-icon" name="lucide/user" :size="14" />
              <span class="status-chip-text">{{ accountText }}</span>
            </div>
          </PCard>
        </div>

        <PCard :title="pclText('Tools.GameLink.Join.Title')">
          <div class="card-body">
            <p class="pcl-text card-description join-description">
              {{ pclText('Tools.GameLink.Join.Description') }}
            </p>
            <div class="lobby-row">
              <PTextBox
                v-model="joinCode"
                :placeholder="pclText('Tools.GameLink.Join.IdHint')"
                :aria-label="pclText('Tools.GameLink.Join.IdHint')"
                @keydown.enter="joinLobby"
              />
              <PButton compact class="lobby-row-button" @click="clearCode">
                {{ pclText('Tools.GameLink.Join.Clear') }}
              </PButton>
              <PButton compact class="lobby-row-button" @click="pasteCode">
                {{ pclText('Tools.GameLink.Join.Paste') }}
              </PButton>
              <PButton
                compact
                type="tint"
                class="lobby-row-button"
                :disabled="!canStart"
                @click="joinLobby"
              >
                {{ pclText('Tools.GameLink.Join.Action') }}
              </PButton>
            </div>
          </div>
        </PCard>

        <PCard
          :title="pclText('Tools.GameLink.Create.Title')"
          default-fold-status="unfold"
        >
          <div class="card-body">
            <p class="pcl-text card-description">
              {{ pclText('Tools.GameLink.Create.Description') }}
            </p>
            <div class="lobby-row">
              <PComboBox
                v-model="selectedPort"
                :options="worldOptions"
                :disabled="!link.worlds.length || !canStart || link.busy.discovering"
                :aria-label="pclText('Tools.GameLink.Create.Title')"
              />
              <PButton
                compact
                class="lobby-row-button lobby-row-button-wide"
                :disabled="!canStart"
                @click="manualCreate"
              >
                {{ pclText('Tools.GameLink.Create.ManualInput') }}
              </PButton>
              <PButton
                compact
                class="lobby-row-button"
                :disabled="link.busy.discovering || !canStart"
                @click="refreshWorlds"
              >
                {{ pclText('Common.Action.Refresh') }}
              </PButton>
              <PButton
                compact
                type="tint"
                class="lobby-row-button"
                :disabled="!selectedPort || !canStart || link.busy.discovering"
                @click="createLobby"
              >
                {{ pclText('Tools.GameLink.Create.Action') }}
              </PButton>
            </div>
          </div>
        </PCard>
      </template>

      <template v-else>
        <div class="finish-grid">
          <PCard hide-title class="finish-users">
            <div class="finish-user-row">
              <div class="status-line" :title="pclText('Tools.GameLink.Finish.UserName')">
                <PPath class="status-icon" :data="USER_PATH" />
                <span>
                  {{ userName ?? pclText('Tools.GameLink.Finish.UserName') }}
                </span>
              </div>
              <span class="user-divider" aria-hidden="true"></span>
              <div class="status-line" :title="pclText('Tools.GameLink.Finish.UserType')">
                <PPath class="status-icon" :data="USER_TYPE_PATH" />
                <span>{{ userTypeText }}</span>
              </div>
            </div>
          </PCard>

          <PCard :title="pclText('Tools.GameLink.Finish.InfoTitle')" class="finish-info">
            <div class="finish-info-body">
              <div class="status-line" :title="pclText('Tools.GameLink.Finish.ConnectionStatus')">
                <PPath class="status-icon" :data="QUALITY_PATH" />
                <span>{{ qualityText }}</span>
              </div>
              <div
                v-if="isGuest"
                class="status-line"
                :title="pclText('Tools.GameLink.Finish.PingToHost')"
              >
                <PPath class="status-icon" :data="PING_PATH" />
                <span>{{ pingText }}</span>
              </div>
              <div
                class="status-line selectable"
                :title="pclText('Tools.GameLink.Finish.LobbyId')"
              >
                <PPath class="status-icon" :data="LOBBY_ID_PATH" />
                <span>{{ lobbyCode ?? '-' }}</span>
              </div>
              <div
                v-if="isGuest"
                class="status-line"
                :title="pclText('Tools.GameLink.Finish.ConnectionType')"
              >
                <PPath class="status-icon" :data="CONNECTION_TYPE_PATH" />
                <span>{{ connectionTypeText }}</span>
              </div>
            </div>
          </PCard>

          <PCard
            :title="pclText('Tools.GameLink.Finish.Actions')"
            class="finish-actions"
          >
            <div class="finish-actions-body">
              <PIconTextButton
                v-if="isGuest"
                icon="lucide/copy"
                :disabled="!link.lobby?.url"
                @click="copyIp"
              >
                {{ pclText('Tools.GameLink.Finish.CopyVirtualIp') }}
              </PIconTextButton>
              <PIconTextButton icon="lucide/copy" :disabled="!lobbyCode" @click="copyLobbyCode">
                {{ pclText('Tools.GameLink.Finish.CopyLobbyId') }}
              </PIconTextButton>
              <PIconTextButton
                icon="lucide/power"
                :disabled="link.busy.leaving || link.busy.eula"
                @click="exitLobby"
              >
                {{ exitLabel }}
              </PIconTextButton>
            </div>
          </PCard>

          <PCard :title="memberTitle" class="finish-members">
            <div class="card-body">
              <PListItem
                v-for="player in players"
                :key="player.machineId"
                :title="player.name"
                :subtitle="playerSubtitle(player)"
                clickable
                @click="openPlayer(player)"
              />
            </div>
          </PCard>
        </div>
      </template>

      <PCard hide-title>
        <div class="lobby-footer">
          <div class="footer-line">
            <PButton inline @click="openExternal('https://qm.qq.com/q/zfml1KaPWS')">
              {{ pclText('Tools.GameLink.Link.Report') }}
            </PButton>
            <span class="pcl-text footer-sep">|</span>
            <PButton inline @click="openExternal('https://account.naids.com/policy')">
              {{ pclText('Tools.GameLink.Link.Terms') }}
            </PButton>
            <span class="pcl-text footer-sep">|</span>
            <PButton
              inline
              @click="openExternal('https://www.pclc.cc/privacy/personal-info-brief.html')"
            >
              {{ pclText('Tools.GameLink.Link.Privacy') }}
            </PButton>
            <span class="pcl-text footer-sep">|</span>
            <PButton inline :disabled="link.busy.eula" @click="disableLink">
              {{ pclText('Tools.GameLink.Link.Disable') }}
            </PButton>
          </div>
          <div class="footer-line">
            <span class="pcl-text">{{ pclText('Tools.GameLink.Link.Friends') }}</span>
            <PButton inline @click="openExternal('https://easytier.cn/')">
              {{ pclText('Tools.GameLink.Link.EasyTier') }}
            </PButton>
            <span class="pcl-text footer-sep">|</span>
            <PButton inline @click="openExternal('https://pysio.online/')">
              {{ PYSIO_HOME }}
            </PButton>
          </div>
        </div>
      </PCard>
    </template>
    <template v-else>
      <PLoading
        v-if="!initError"
        class="lobby-loading"
        :loading-text="pclText('Tools.GameLink.Loading.ConnectingServer')"
      />
    </template>
  </section>
</template>

<style scoped>
.lobby-loading {
  margin: auto;
}

.card-body {
  margin: 0 10px 10px;
  display: flex;
  flex-direction: column;
}

.card-description {
  margin: 10px 10px -2px;
  font-size: 14px;
  line-height: 23px;
  white-space: pre-wrap;
}

.join-description {
  margin-bottom: 0;
}

.eula-body {
  gap: 6px;
  margin: 0 20px 16px;
  line-height: 1.5;
  text-align: start;
}

.eula-body p {
  margin: 0;
}

.eula-description {
  font-size: 14px;
}

.eula-actions {
  display: flex;
  justify-content: center;
  margin-top: 4px;
}

.announcement-hint {
  max-height: 64px;
  overflow: hidden;
}

.announcement-text {
  white-space: pre-wrap;
}

.status-chips {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  margin: 0;
}

.status-chip {
  margin: 0;
  cursor: pointer;
}

.status-chip-dim {
  opacity: 0.6;
}

.status-chip-disabled {
  cursor: default;
}

.status-chip-body {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 12px;
}

.status-chip-icon {
  margin-right: 7px;
  color: var(--color-text-grey);
}

.status-chip-text {
  margin-right: 0;
  color: var(--color-text-grey);
  font-size: 13px;
  white-space: nowrap;
}

.lobby-row {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 28px;
  margin: 10px;
}

.lobby-row > .input-wrapper,
.lobby-row > .dropdown-wrapper {
  flex: 1 1 0;
  min-width: 0;
}

.lobby-row-button {
  flex: 0 0 auto;
  width: 50px;
}

.lobby-row-button-wide {
  width: 80px;
}

.finish-grid {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  grid-template-areas:
    'users members'
    'info members'
    'actions members';
  gap: var(--pcl-page-gap);
  align-items: start;
}

.finish-users {
  grid-area: users;
  align-self: start;
}

.finish-info {
  grid-area: info;
  align-self: start;
}

.finish-actions {
  grid-area: actions;
  align-self: start;
}

.finish-members {
  grid-area: members;
  align-self: stretch;
}

.finish-user-row {
  display: flex;
  align-items: center;
  padding: 4px 10px;
  margin: 0 0 0 0;
}

.user-divider {
  flex: 0 0 auto;
  align-self: center;
  width: 1px;
  height: 14px;
  margin: 0 10px;
  border-radius: 1px;
  background: var(--color-text-grey);
  opacity: 0.35;
}

.status-line {
  display: flex;
  align-items: center;
  margin: 4px 5px;
}

.status-line.selectable {
  user-select: text;
}

.status-icon {
  width: 14px;
  height: 14px;
  margin-right: 7px;
}

.finish-info-body,
.finish-actions-body {
  margin: 0 10px 8px;
  display: flex;
  flex-direction: column;
}

.finish-actions-body {
  gap: 2px;
  align-items: flex-start;
}

.lobby-footer {
  margin: 8px;
  display: flex;
  flex-direction: column;
}

.footer-line {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-wrap: wrap;
  margin: 4px 15px;
}

.footer-sep {
  margin: 0 4px;
}
</style>
