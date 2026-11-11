import { invoke, isTauri } from '@tauri-apps/api/core'
import { reactive } from 'vue'
import { instanceIcon } from '@/types/gameInfo'
import type GameInstance from '@/types/gameInstance'
import { customLogo, imageFromSelection, selectionFromLogo } from '@/api/instanceLogo'

export interface InstanceMetadata {
  starred: boolean
  display_type: number
  logo: string
  logo_custom: boolean
  description: string
  launch_count: number
}
export const instanceMetadata = reactive<Record<string, InstanceMetadata>>({})
const customIcons = reactive<Record<string, string>>({})

export function configuredInstanceIcon(instance: GameInstance) {
  const fallback = instanceIcon(instance.version, instance.pluginsVersion || {})
  const selection = selectionFromLogo(instanceMetadata[instance.directory])
  if (selection === customLogo) return customIcons[instance.directory] ?? fallback
  return imageFromSelection(selection) ?? fallback
}
async function loadIcon(directory: string, metadata: InstanceMetadata) {
  delete customIcons[directory]
  if (selectionFromLogo(metadata) !== customLogo) return
  try {
    customIcons[directory] = await invoke<string>('get_instance_custom_icon', {
      instanceDirectory: directory,
    })
  } catch {
    /* Missing icons use the normal instance icon without changing CE settings. */
  }
}

export async function loadInstanceMetadata(directories: string[]) {
  if (!isTauri() || !directories.length) return
  const values = await invoke<Record<string, InstanceMetadata>>('get_instance_metadata', {
    directories,
  })
  Object.assign(instanceMetadata, values)
  await Promise.all(Object.entries(values).map(([path, value]) => loadIcon(path, value)))
}
export async function updateInstanceMetadata(
  directory: string,
  patch: Partial<Pick<InstanceMetadata, 'starred' | 'display_type' | 'logo' | 'logo_custom'>>,
) {
  const saved = await invoke<InstanceMetadata>('set_instance_metadata', {
    instanceDirectory: directory,
    patch,
  })
  instanceMetadata[directory] = saved
  await loadIcon(directory, saved)
}
export async function selectInstanceCustomIcon(directory: string) {
  const saved = await invoke<InstanceMetadata | null>('select_instance_custom_icon', {
    instanceDirectory: directory,
  })
  if (saved) {
    instanceMetadata[directory] = saved
    await loadIcon(directory, saved)
  }
}
