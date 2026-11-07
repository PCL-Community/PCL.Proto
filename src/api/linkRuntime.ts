import { Channel, invoke } from '@tauri-apps/api/core'

export type LinkRuntimeState = 'ready' | 'needs_install' | 'needs_restart' | 'error'

export interface LinkRuntimeStatus {
  state: LinkRuntimeState
  message?: string | null
}

export interface LinkRuntimeEvent {
  phase: string
  progress?: number | null
  speed?: number | null
  message?: string | null
}

export function getLinkRuntimeStatus() {
  return invoke<LinkRuntimeStatus>('get_link_runtime_status')
}

export async function installLinkRuntime(onEvent: (event: LinkRuntimeEvent) => void) {
  const channel = new Channel<LinkRuntimeEvent>()
  channel.onmessage = onEvent
  return invoke<LinkRuntimeStatus>('install_link_runtime', { on_event: channel })
}
