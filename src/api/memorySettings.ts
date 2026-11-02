import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'
import { dropLegacyFields, legacyInstancePages } from '@/util/legacySettings'

export type MemorySettings =
  { mode: 'auto' } | { mode: 'custom'; maximum_mib: number; initial_mib: number | null }
export type MemoryCustom = { maximum_mib: number; initial_mib: number | null }
export type MemorySnapshot = { settings: MemorySettings | null; custom: MemoryCustom }
export type MemoryPreview = {
  total_bytes: number
  available_bytes: number
  allocation_mib: number | null
}
export function previewMemorySettings(
  directory: string | null,
  settings: MemorySettings | null,
  is64Bit: boolean | null,
) {
  return invoke<MemoryPreview>('preview_memory_settings', {
    instanceDirectory: directory,
    settings,
    is64Bit,
  })
}
export const memorySettingKeys = [
  'LaunchRamType',
  'VersionRamType',
  'SliderRamCustom',
  'SliderRamInitialCustom',
]

export function memoryTickToMib(tick: number) {
  const gib =
    tick <= 12
      ? tick * 0.1 + 0.3
      : tick <= 25
        ? (tick - 12) * 0.5 + 1.5
        : tick <= 33
          ? tick - 25 + 8
          : (tick - 33) * 2 + 16
  return Math.floor(gib * 1024)
}
export function memoryMibToTick(mib: number) {
  return Array.from({ length: 50 }, (_, tick) => tick).reduce(
    (best, tick) =>
      Math.abs(memoryTickToMib(tick) - mib) < Math.abs(memoryTickToMib(best) - mib) ? tick : best,
    0,
  )
}
function legacyMemory(value: unknown, instance: boolean): MemorySnapshot | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined
  const fields = value as Record<string, unknown>
  const mode = fields[instance ? 'VersionRamType' : 'LaunchRamType']
  if (mode !== '0' && mode !== '1' && !(instance && mode === '2')) return undefined
  const maximum = fields.SliderRamCustom ?? 15
  const initial = fields.SliderRamInitialCustom ?? 0
  if (
    !Number.isInteger(maximum) ||
    !Number.isInteger(initial) ||
    Number(maximum) < 0 ||
    Number(maximum) > 49 ||
    Number(initial) < 0 ||
    Number(initial) > 49
  )
    return undefined
  const max = memoryTickToMib(Number(maximum))
  const custom = {
    maximum_mib: max,
    initial_mib: initial === 0 ? null : memoryTickToMib(Number(initial)),
  }
  const settings: MemorySettings | null =
    mode === '2'
      ? null
      : mode === '0'
        ? { mode: 'auto' }
        : {
            mode: 'custom',
            maximum_mib: max,
            initial_mib: custom.initial_mib === null ? null : Math.min(max, custom.initial_mib),
          }
  return { settings, custom }
}
function parseLegacy(key: string): unknown {
  try {
    return JSON.parse(localStorage.getItem(key) || 'null')
  } catch {
    return null
  }
}

export async function initializeMemorySettings() {
  const globalKey = 'pcl-ui:PageSetupLaunch:global'
  const global = legacyMemory(parseLegacy(globalKey), false)
  if (global) {
    await invoke('set_memory_settings', { instanceDirectory: null, ...global, migrateOnly: true })
    dropLegacyFields(globalKey, memorySettingKeys)
  }
  const legacy: Record<string, MemorySnapshot> = Object.create(null)
  const pages = new Map<string, string>()
  let invalid = 0
  for (const { key, id, value } of legacyInstancePages('pcl-ui:PageInstanceSetup:')) {
    const settings = legacyMemory(value, true)
    if (settings !== undefined) {
      legacy[id] = settings
      pages.set(id, key)
    } else if (value && 'VersionRamType' in value) invalid++
  }
  const pending = Object.keys(legacy).length
    ? await invoke<string[]>('migrate_instance_memory_settings', { legacy })
    : []
  // Rust 已接管的实例删除旧袋副本，pending 的保留供下次重试。
  for (const [id, key] of pages) if (!pending.includes(id)) dropLegacyFields(key, memorySettingKeys)
  return invalid + pending.length
}

export const memorySettingsStorage = createSettingsPersistence(
  (instanceDirectory) => invoke<MemorySnapshot>('get_memory_settings', { instanceDirectory }),
  (instanceDirectory, snapshot) =>
    invoke<MemorySnapshot>('set_memory_settings', {
      instanceDirectory,
      ...snapshot,
      migrateOnly: false,
    }),
)
