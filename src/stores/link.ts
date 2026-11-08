import { defineStore } from 'pinia'
import { computed, reactive, ref } from 'vue'
import { pclText } from '@/api/pclLocale'
import {
  discoverLinkWorlds,
  getLinkAccount,
  getLinkLobbyState,
  getLinkSettings,
  initializeLink,
  lobbyCreate,
  lobbyJoin,
  lobbyLeave,
  lobbyReset,
  logoutLinkNaid,
  resetLinkSettings,
  setLinkEula,
  setLinkSettings,
  startLinkNaidLogin,
  testLinkNat,
  toLinkError,
  type LinkAccount,
  type LinkAnnouncement,
  type LinkLobbyState,
  type LinkNatStatus,
  type LinkSettings,
  type LinkWorld,
} from '@/api/link'

const LINK_ERROR_KEYS: Record<string, string> = {
  lobby_unavailable: 'Link.Mod.LobbyUnavailable',
  eula_required: 'Tools.GameLink.Agreement.Description',
  discover_failed: 'Link.Lobby.InitFailed',
  lobby_failed: 'Link.Lobby.InitFailed',
  config_failed: 'Setup.GameLink.Error.ConfigChangeFailed',
  update_required: 'Tools.GameLink.Error.UpdateRequired',
  login_required: 'Link.Mod.LoginFirst',
  relogin_required: 'Link.Mod.ReLoginRequired',
  realname_required: 'Link.Mod.RealNameRequired',
  account_banned: 'Link.Mod.AccountBanned',
  username_required: 'Link.Mod.UsernameOrLogin',
  invalid_username: 'Link.Mod.InvalidPlayerId',
  easy_tier_unavailable: 'Link.Mod.DownloadingDeps',
  invalid_code: 'Link.Lobby.InvalidCodeFormat',
  join_failed: 'Link.Lobby.JoinFailed',
  create_failed: 'Link.Lobby.CreateLobbyFailed',
  invalid_port: 'Tools.GameLink.Create.NotMcPort',
  not_mc_port: 'Tools.GameLink.Create.NotMcPort',
  naid_not_configured: 'Link.Mod.LoginFirst',
  nat_failed: 'Tools.GameLink.Error.NetworkTestFailed',
  announce_failed: 'Tools.GameLink.Error.ConnectFailed',
}

export function linkErrorKey(code: string): string | undefined {
  const key = LINK_ERROR_KEYS[code]
  if (key) return key
  if (code.startsWith('naid_')) return 'Tools.GameLink.Natayark.LoginFailed'
  return undefined
}

export function linkErrorMessage(error: unknown, fallbackKey?: string): string {
  const failure = toLinkError(error)
  const key = linkErrorKey(failure.code)
  if (key) return pclText(key, failure.args ?? [])
  if (fallbackKey) return pclText(fallbackKey)
  return pclText('Link.Lobby.InitFailed')
}

export const useLinkStore = defineStore('link', () => {
  const settings = ref<LinkSettings | null>(null)
  const eula = ref<boolean | null>(null)
  const announcement = ref<LinkAnnouncement | null>(null)
  const account = ref<LinkAccount | null>(null)
  const lobby = ref<LinkLobbyState | null>(null)
  const worlds = ref<LinkWorld[]>([])
  const nat = ref<LinkNatStatus | null>(null)
  const pollError = ref<string | null>(null)
  const busy = reactive({
    resetting: false,
    nat: false,
    discovering: false,
    creating: false,
    joining: false,
    leaving: false,
    account: false,
    eula: false,
  })
  const ready = computed(() => settings.value !== null && eula.value !== null)

  let revision = 0
  let mutations = 0
  let queue: Promise<unknown> = Promise.resolve()
  // 退出和重置必须等已发送的创建/保存完成，避免旧命令随后重新建房或写回配置。
  function enqueue<T>(task: () => Promise<T>): Promise<T> {
    revision++
    mutations++
    const result = queue.then(task).finally(() => mutations--)
    queue = result.catch(() => undefined)
    return result
  }

  let initialization: Promise<void> | null = null
  function initialize() {
    if (initialization) return initialization
    initialization = enqueue(async () => {
      const next = await initializeLink()
      const nextLobby = await getLinkLobbyState()
      settings.value = next.settings
      eula.value = next.eula
      announcement.value = next.announcement
      account.value = next.account
      lobby.value = nextLobby
    }).finally(() => {
      initialization = null
    })
    return initialization
  }

  function loadSettings() {
    return enqueue(async () => {
      settings.value = await getLinkSettings()
      return settings.value
    })
  }

  function saveSettings(next: LinkSettings) {
    if (busy.resetting) return Promise.resolve()
    const target = { ...next }
    return enqueue(async () => {
      settings.value = await setLinkSettings(target)
    })
  }

  async function resetSettings() {
    if (busy.resetting) return settings.value
    busy.resetting = true
    try {
      return await enqueue(async () => {
        settings.value = await resetLinkSettings()
        eula.value = false
        account.value = null
        lobby.value = null
        announcement.value = null
        return settings.value
      })
    } finally {
      busy.resetting = false
    }
  }

  async function setEula(accepted: boolean) {
    if (busy.eula || busy.resetting) return
    busy.eula = true
    try {
      return await enqueue(async () => {
        eula.value = await setLinkEula(accepted)
        if (!accepted) {
          lobby.value = null
          account.value = null
        }
        account.value = await getLinkAccount()
        return eula.value
      })
    } finally {
      busy.eula = false
    }
  }

  async function testNat() {
    if (busy.nat) return nat.value
    busy.nat = true
    try {
      nat.value = await testLinkNat()
      return nat.value
    } finally {
      busy.nat = false
    }
  }

  let discovery: Promise<LinkWorld[]> | null = null
  function discoverWorlds() {
    if (discovery) return discovery
    busy.discovering = true
    discovery = discoverLinkWorlds()
      .then((next) => {
        worlds.value = next
        return worlds.value
      })
      .finally(() => {
        discovery = null
        busy.discovering = false
      })
    return discovery
  }

  async function create(port: number) {
    if (busy.creating || busy.joining || busy.leaving || busy.resetting || busy.eula) return
    busy.creating = true
    try {
      return await enqueue(async () => {
        lobby.value = await lobbyCreate(port)
        return lobby.value
      })
    } finally {
      busy.creating = false
    }
  }

  async function join(code: string) {
    if (busy.creating || busy.joining || busy.leaving || busy.resetting || busy.eula) return
    busy.joining = true
    try {
      return await enqueue(async () => {
        lobby.value = await lobbyJoin(code)
        return lobby.value
      })
    } finally {
      busy.joining = false
    }
  }

  async function leave() {
    if (busy.leaving) return
    busy.leaving = true
    try {
      return await enqueue(async () => {
        lobby.value = await lobbyLeave()
        return lobby.value
      })
    } finally {
      busy.leaving = false
    }
  }

  function resetLobby() {
    return enqueue(async () => {
      lobby.value = await lobbyReset()
      return lobby.value
    })
  }

  async function startLogin() {
    if (busy.account || busy.eula || busy.resetting) return
    busy.account = true
    try {
      return await enqueue(async () => {
        account.value = await startLinkNaidLogin()
        return account.value
      })
    } finally {
      busy.account = false
    }
  }

  async function logoutAccount() {
    if (busy.account || busy.eula || busy.resetting) return
    busy.account = true
    try {
      return await enqueue(async () => {
        account.value = await logoutLinkNaid()
        return account.value
      })
    } finally {
      busy.account = false
    }
  }

  let pollTimer: ReturnType<typeof setTimeout> | null = null
  let polling = false
  let pollingNow = false
  async function poll() {
    pollTimer = null
    pollingNow = true
    const current = revision
    try {
      if (mutations || (typeof document !== 'undefined' && document.visibilityState === 'hidden'))
        return
      let failure: string | null = null
      const nextLobby = await getLinkLobbyState().catch((error) => {
        failure = linkErrorMessage(error)
        return null
      })
      if (!polling || current !== revision) return
      if (nextLobby) lobby.value = nextLobby
      if (account.value?.pending) {
        const nextAccount = await getLinkAccount().catch((error) => {
          failure = linkErrorMessage(error, 'Tools.GameLink.Natayark.FetchFailed')
          return null
        })
        if (!polling || current !== revision) return
        if (nextAccount) account.value = nextAccount
      }
      pollError.value = failure
    } finally {
      pollingNow = false
      if (polling) pollTimer = setTimeout(poll, 1000)
    }
  }

  function stopPolling() {
    polling = false
    revision++
    if (pollTimer) clearTimeout(pollTimer)
    pollTimer = null
  }

  function startPolling() {
    polling = true
    if (!pollTimer && !pollingNow) void poll()
  }

  return {
    settings,
    eula,
    announcement,
    account,
    lobby,
    worlds,
    nat,
    pollError,
    busy,
    ready,
    initialize,
    loadSettings,
    saveSettings,
    resetSettings,
    setEula,
    testNat,
    discoverWorlds,
    create,
    join,
    leave,
    resetLobby,
    startLogin,
    logoutAccount,
    startPolling,
    stopPolling,
  }
})
