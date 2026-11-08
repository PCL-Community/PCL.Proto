import { invoke } from '@tauri-apps/api/core'

export type LinkProtocol = 'tcp' | 'udp'

export interface LinkSettings {
  username: string
  protocol: LinkProtocol
  latencyFirst: boolean
  tryPunchSym: boolean
  enableIpv6: boolean
  enableCliOutput: boolean
}

export type LinkNoticeType = 'notice' | 'warning' | 'important'

export interface LinkNotice {
  type: LinkNoticeType
  content: string
}

export interface LinkRelay {
  name: string
  url: string
  official: boolean
}

export interface LinkAnnouncement {
  available: boolean
  allowCustomName: boolean
  requireLogin: boolean
  requireRealname: boolean
  protocolVersion: number
  updateRequired: boolean
  offline: boolean
  notices: LinkNotice[]
  relays: LinkRelay[]
  error: string | null
}

export interface LinkAccount {
  configured: boolean
  loggedIn: boolean
  pending: boolean
  username: string | null
  status: number | null
  realname: boolean
  expired: boolean
  error: string | null
}

export type LinkNatType =
  | 'open'
  | 'no_pat'
  | 'full_cone'
  | 'restricted'
  | 'port_restricted'
  | 'symmetric_easy'
  | 'symmetric'
  | 'symmetric_firewall'
  | 'udp_blocked'
  | 'unknown'

export interface LinkNatStatus {
  udp: LinkNatType
  tcp: LinkNatType
  ipv6: boolean
}

export interface LinkWorld {
  port: number
  description: string
  version: string
  latencyMs: number
}

export type LinkPlayerKind = 'HOST' | 'GUEST' | 'LOCAL'

export interface LinkPlayer {
  name: string
  machineId: string
  vendor: string
  kind: LinkPlayerKind
}

export type LinkStateName =
  | 'waiting'
  | 'host-scanning'
  | 'host-starting'
  | 'guest-connecting'
  | 'guest-starting'
  | 'host-ok'
  | 'guest-ok'
  | 'exception'

export interface LinkLobbyState {
  state: LinkStateName
  index: number
  room?: string | null
  url?: string | null
  latencyMs?: number | null
  connectionType?: 'p2p' | 'relay' | null
  players: LinkPlayer[]
  exception?: number | null
}

export interface LinkMeta {
  version: string
  easyTierVersion: string
  vendor: string
  motd: string
}

export interface LinkInit {
  settings: LinkSettings
  eula: boolean
  announcement: LinkAnnouncement
  account: LinkAccount
  meta: LinkMeta
}

/** invoke 的 reject 值：{ code, args }。 */
export interface LinkError {
  code: string
  args?: string[]
}

export function toLinkError(error: unknown): LinkError {
  if (error && typeof error === 'object') {
    const value = error as { code?: unknown; args?: unknown }
    if (typeof value.code === 'string') {
      const args = Array.isArray(value.args) ? value.args.map(String) : undefined
      return { code: value.code, args }
    }
  }
  return { code: 'unknown', args: [String(error)] }
}

export function initializeLink() {
  return invoke<LinkInit>('initialize_link')
}

export function getLinkSettings() {
  return invoke<LinkSettings>('get_link_settings')
}

export function setLinkSettings(settings: LinkSettings) {
  return invoke<LinkSettings>('set_link_settings', { settings })
}

export function resetLinkSettings() {
  return invoke<LinkSettings>('reset_link_settings')
}

export function setLinkEula(accepted: boolean) {
  return invoke<boolean>('set_link_eula', { accepted })
}

export function refreshLinkAnnouncement() {
  return invoke<LinkAnnouncement>('refresh_link_announcement')
}

export function getLinkAccount() {
  return invoke<LinkAccount>('get_link_account')
}

export function startLinkNaidLogin() {
  return invoke<LinkAccount>('start_link_naid_login')
}

export function logoutLinkNaid() {
  return invoke<LinkAccount>('logout_link_naid')
}

export function refreshLinkNaid() {
  return invoke<LinkAccount>('refresh_link_naid')
}

export function testLinkNat() {
  return invoke<LinkNatStatus>('test_link_nat')
}

export function discoverLinkWorlds() {
  return invoke<LinkWorld[]>('discover_link_worlds')
}

export function verifyLinkPort(port: number) {
  return invoke<LinkWorld>('verify_link_port', { port })
}

export function getLinkLobbyState() {
  return invoke<LinkLobbyState>('get_link_lobby_state')
}

export function lobbyCreate(port: number) {
  return invoke<LinkLobbyState>('lobby_create', { port })
}

export function lobbyJoin(code: string) {
  return invoke<LinkLobbyState>('lobby_join', { code })
}

export function lobbyLeave() {
  return invoke<LinkLobbyState>('lobby_leave')
}

export function lobbyReset() {
  return invoke<LinkLobbyState>('lobby_reset')
}
