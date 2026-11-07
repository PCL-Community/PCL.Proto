import { invoke, isTauri } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export type ProfileType = 'microsoft' | 'offline' | 'authlib' | 'yggdrasilConnect'

/** IPC view of `commands::account::SafeProfile`. */
export interface McProfile {
  profileId: string
  userName: string
  uuid: string
  profileType: ProfileType
  description: string
  server?: string | null
  serverName?: string | null
}

/** `commands::account::ProfileList`. */
export interface ProfileList {
  profiles: McProfile[]
  current?: string | null
}

/** 认证服务器给出的可选角色。 */
export interface Candidate {
  id: string
  name: string
}

/** 离线 UUID 的生成方式，对照 CE 的三个单选项。 */
export type UuidMode = 'standard' | 'legacy' | 'custom'

/** `commands::account::AuthServerInfo`. */
export interface AuthServerInfo {
  apiLocation: string
  serverName?: string | null
  oauthSupported: boolean
  discoveryAddress?: string | null
  clientId?: string | null
  website?: string | null
}

/** 登录请求，对应 Rust 的 `session::LoginRequest`。 */
export interface LoginRequest {
  profileType: ProfileType
  server?: string | null
  discoveryAddress?: string | null
  clientId?: string | null
  username?: string | null
  password?: string | null
  profileId?: string | null
  forceRefresh?: boolean
  forceReselectProfile?: boolean
}

/** 登录会话状态，对照 CE 的 MyMsgLogin 弹窗与角色选择弹窗。 */
export type LoginState =
  | { status: 'idle' }
  | { status: 'working' }
  | {
      status: 'deviceCode'
      userCode: string
      verificationUri: string
      verificationUriComplete: string | null
    }
  | { status: 'selectProfile'; candidates: Candidate[] }
  | { status: 'succeeded' }
  | { status: 'failed'; code: string; message: string }
  | { status: 'cancelled' }

/** 用户取消登录的错误码，与 Rust 的 `session::CANCELLED` 一致。 */
export const CANCELLED = 'login_cancelled'
/** 需要用户交互但当前没有界面（启动流程），与 Rust 的 `session::INTERACTION_REQUIRED` 一致。 */
export const INTERACTION_REQUIRED = 'interaction_required'

/** 后端返回的认证错误。 */
export interface AuthFailure {
  code: string
  message: string
}

interface SkinPayload {
  skin?: string | null
  default: string
}

/** Tauri 把 Rust 的 `AuthError` 原样序列化过来，其它错误按文本处理。 */
export function toAuthFailure(error: unknown): AuthFailure {
  if (error && typeof error === 'object' && 'code' in error) {
    const failure = error as Partial<AuthFailure>
    return { code: String(failure.code ?? 'unknown'), message: String(failure.message ?? error) }
  }
  return { code: 'unknown', message: String(error) }
}

const POLL_INTERVAL = 400

export const useAccountInfo = defineStore('account-info', () => {
  const profiles = ref<McProfile[]>([])
  const currentId = ref<string | null>(null)
  const skins = ref<Record<string, SkinPayload>>({})
  const loginState = ref<LoginState>({ status: 'idle' })
  const loggingIn = ref(false)
  let sessionId: string | null = null
  let pollTimer: ReturnType<typeof setTimeout> | null = null

  const profile = computed(
    () => profiles.value.find((item) => item.profileId === currentId.value) ?? null,
  )
  const username = computed(() => profile.value?.userName)
  const uuid = computed(() => profile.value?.uuid)

  function builtInSkin(name: string) {
    return `${import.meta.env.BASE_URL}pcl/images/Skins/${name}.png`
  }

  /** 某个档案要显示的皮肤：自定义皮肤优先，否则用内置的 Steve/Alex。 */
  function skinFor(profileId: string | null | undefined): string {
    const payload = profileId ? skins.value[profileId] : undefined
    return payload?.skin || builtInSkin(payload?.default ?? 'Steve')
  }

  const skinUrl = computed(() => skinFor(currentId.value))

  async function refresh() {
    if (!isTauri()) return
    const list = await invoke<ProfileList>('get_profiles')
    profiles.value = list.profiles
    currentId.value = list.current ?? null
  }

  async function loadSkin(profileId: string, force = false) {
    if (!isTauri()) return
    try {
      const payload = await invoke<SkinPayload>('fetch_profile_skin', { profileId, refresh: force })
      skins.value = { ...skins.value, [profileId]: payload }
    } catch (error) {
      console.warn('Failed to load profile skin:', error)
    }
  }

  async function loadSkins(force = false) {
    await Promise.all(profiles.value.map((item) => loadSkin(item.profileId, force)))
  }

  async function initialize() {
    try {
      await refresh()
      await loadSkins()
    } catch (error) {
      console.error('Failed to initialize account:', error)
    }
  }

  /** 档案列表变化后统一刷新界面数据。 */
  async function applyList(list: ProfileList) {
    profiles.value = list.profiles
    currentId.value = list.current ?? null
    await loadSkins()
  }

  /** 选择档案；传 `null` 取消选择（CE 的“选择档案”按钮）。 */
  async function select(profileId: string | null) {
    if (!isTauri()) return
    await applyList(await invoke<ProfileList>('select_profile', { profileId }))
  }

  async function remove(profileId: string) {
    if (!isTauri()) return
    await applyList(await invoke<ProfileList>('delete_profile', { profileId }))
  }

  async function createOffline(userName: string, uuidMode: UuidMode, customUuid?: string) {
    await applyList(
      await invoke<ProfileList>('create_offline_profile', { userName, uuidMode, customUuid }),
    )
  }

  async function setOfflineUuid(profileId: string, uuidMode: UuidMode, customUuid?: string) {
    await applyList(
      await invoke<ProfileList>('set_offline_uuid', { profileId, uuidMode, customUuid }),
    )
  }

  async function renameMicrosoft(profileId: string, userName: string) {
    await applyList(await invoke<ProfileList>('rename_microsoft_profile', { profileId, userName }))
  }

  async function probeServer(address: string) {
    return invoke<AuthServerInfo>('probe_auth_server', { address })
  }

  function stopPolling() {
    if (pollTimer) clearTimeout(pollTimer)
    pollTimer = null
    sessionId = null
    loggingIn.value = false
  }

  async function poll() {
    const id = sessionId
    if (!id) return
    try {
      const state = await invoke<LoginState>('login_state', { sessionId: id })
      loginState.value = state
      if (state.status === 'succeeded') {
        stopPolling()
        await refresh()
        await loadSkins(true)
        return
      }
      if (state.status === 'failed' || state.status === 'cancelled') {
        stopPolling()
        return
      }
    } catch (error) {
      loginState.value = { status: 'failed', ...toAuthFailure(error) }
      stopPolling()
      return
    }
    pollTimer = setTimeout(() => void poll(), POLL_INTERVAL)
  }

  /** 开始一次登录，之后由 [`loginState`] 驱动界面。 */
  async function startLogin(request: LoginRequest) {
    if (!isTauri() || loggingIn.value) return
    loggingIn.value = true
    loginState.value = { status: 'working' }
    try {
      sessionId = await invoke<string>('start_login', { request })
    } catch (error) {
      loginState.value = { status: 'failed', ...toAuthFailure(error) }
      stopPolling()
      return
    }
    pollTimer = setTimeout(() => void poll(), POLL_INTERVAL)
  }

  async function cancelLogin() {
    if (sessionId) {
      try {
        await invoke('cancel_login', { sessionId })
      } catch (error) {
        console.warn('Failed to cancel login:', error)
      }
    }
    loginState.value = { status: 'cancelled' }
    stopPolling()
  }

  /** 提交角色选择；不传 ID 表示放弃登录。 */
  async function chooseCandidate(candidateId?: string | null) {
    if (!sessionId) return
    try {
      await invoke('select_login_profile', { sessionId, candidateId: candidateId ?? null })
    } catch (error) {
      loginState.value = { status: 'failed', ...toAuthFailure(error) }
      stopPolling()
    }
  }

  /** 一轮登录结束后由界面调用，避免残留状态影响下一次登录。 */
  function resetLoginState() {
    loginState.value = { status: 'idle' }
  }

  return {
    profiles,
    currentId,
    profile,
    username,
    uuid,
    skinUrl,
    skins,
    loginState,
    loggingIn,
    skinFor,
    initialize,
    refresh,
    loadSkin,
    loadSkins,
    select,
    remove,
    createOffline,
    setOfflineUuid,
    renameMicrosoft,
    probeServer,
    startLogin,
    cancelLogin,
    chooseCandidate,
    resetLoginState,
  }
})
