import { invoke } from '@tauri-apps/api/core'

export type ProxyMode = 'off' | 'system' | 'custom'

export interface ProxySettings {
  mode: ProxyMode
  address: string
  username: string
  password: string
}

export const getProxySettings = () => invoke<ProxySettings>('get_proxy_settings')

export const setProxySettings = (settings: ProxySettings) =>
  invoke<ProxySettings>('set_proxy_settings', { settings })
