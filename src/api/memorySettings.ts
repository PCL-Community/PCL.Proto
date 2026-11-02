import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

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
export const memorySettingsStorage = createSettingsPersistence(
  (instanceDirectory) => invoke<MemorySnapshot>('get_memory_settings', { instanceDirectory }),
  (instanceDirectory, snapshot) =>
    invoke<MemorySnapshot>('set_memory_settings', { instanceDirectory, ...snapshot }),
)
