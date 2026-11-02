import { invoke } from '@tauri-apps/api/core'

export interface ServerEntry {
  name: string
  address: string
  icon: string | null
}

export interface ServerList {
  servers: ServerEntry[]
  revision: string
}

export type ServerChange =
  | { action: 'add'; name: string; address: string }
  | { action: 'edit'; index: number; name: string; address: string }
  | { action: 'remove'; index: number }

export const listServers = (instanceDirectory: string) =>
  invoke<ServerList>('list_servers', { instanceDirectory })

export const changeServer = (instanceDirectory: string, revision: string, change: ServerChange) =>
  invoke<ServerList>('change_server', { instanceDirectory, revision, change })
