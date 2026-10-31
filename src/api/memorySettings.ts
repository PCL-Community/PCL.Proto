import { invoke } from '@tauri-apps/api/core'

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
  const global = legacyMemory(parseLegacy('pcl-ui:PageSetupLaunch:global'), false)
  if (global)
    await invoke('set_memory_settings', { instanceDirectory: null, ...global, migrateOnly: true })
  const prefix = 'pcl-ui:PageInstanceSetup:'
  const legacy: Record<string, MemorySnapshot> = Object.create(null)
  let invalid = 0
  for (let index = 0; index < localStorage.length; index++) {
    const key = localStorage.key(index)
    if (!key?.startsWith(prefix)) continue
    const id = key.slice(prefix.length)
    if (!id || id === 'none') continue
    const parsed = parseLegacy(key)
    const settings = legacyMemory(parsed, true)
    if (settings !== undefined) legacy[id] = settings
    else if (parsed && typeof parsed === 'object' && 'VersionRamType' in parsed) invalid++
  }
  const pending = Object.keys(legacy).length
    ? await invoke<string[]>('migrate_instance_memory_settings', { legacy })
    : []
  return invalid + pending.length
}

const queues = new Map<string, Promise<unknown>>()
const drafts = new Map<string, MemorySnapshot>()
const errors = new Map<string, string>()
const revisions = new Map<string, number>()
const keyFor = (directory: string | null) =>
  directory === null ? 'global' : `instance:${directory}`
export async function readMemorySettings(directory: string | null) {
  const key = keyFor(directory)
  await queues.get(key)
  const snapshot = drafts.has(key)
    ? drafts.get(key)!
    : await invoke<MemorySnapshot>('get_memory_settings', { instanceDirectory: directory })
  return { ...snapshot, error: errors.get(key) || '' }
}
export function saveMemorySettings(directory: string | null, snapshot: MemorySnapshot) {
  const key = keyFor(directory)
  const revision = (revisions.get(key) || 0) + 1
  revisions.set(key, revision)
  drafts.set(key, snapshot)
  errors.delete(key)
  const result = (queues.get(key) || Promise.resolve())
    .then(() =>
      invoke<MemorySnapshot>('set_memory_settings', {
        instanceDirectory: directory,
        ...snapshot,
        migrateOnly: false,
      }),
    )
    .then((saved) => {
      if (revisions.get(key) === revision) drafts.delete(key)
      return saved
    })
    .catch((error) => {
      if (revisions.get(key) === revision) errors.set(key, String(error))
      throw error
    })
  queues.set(
    key,
    result.catch(() => {}),
  )
  return result
}
