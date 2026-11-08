import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

export interface DownloadSettings {
  max_connections: number
  file_connections: number
  speed_limit: number
  http_mode: number
  file_source: number
  version_source: number
  community_source: number
  auto_select_instance: boolean | null
}

export const downloadSettingKeys = [
  'ComboDownloadSource',
  'ComboDownloadVersion',
  'ComboDownloadMod',
  'ComboDownloadHttpMode',
  'SliderDownloadThread',
  'SliderDownloadFileConnection',
  'SliderDownloadSpeed',
  'CheckDownloadAutoSelectVersion',
]

export async function initializeDownloadSettings() {
  return (
    (await invoke<DownloadSettings | null>('get_download_settings')) ||
    invoke<DownloadSettings>('set_download_settings', { settings: downloadSettings({}) })
  )
}

export const downloadSettingsStorage = createSettingsPersistence(
  async () => {
    const settings = await invoke<DownloadSettings | null>('get_download_settings')
    if (!settings) throw new Error('Download settings are not initialized')
    return settings
  },
  (_, settings) => invoke<DownloadSettings>('set_download_settings', { settings }),
)

export function downloadSettingValues(settings: DownloadSettings) {
  const speed = Array.from({ length: 43 }, (_, index) => ({
    index,
    value: downloadSettings({ SliderDownloadSpeed: index }).speed_limit,
  })).reduce((best, item) =>
    Math.abs(item.value - settings.speed_limit) < Math.abs(best.value - settings.speed_limit)
      ? item
      : best,
  )
  return {
    ComboDownloadSource: String(settings.file_source),
    ComboDownloadVersion: String(settings.version_source),
    ComboDownloadMod: String(settings.community_source),
    ComboDownloadHttpMode: String(settings.http_mode),
    SliderDownloadThread: settings.max_connections - 1,
    SliderDownloadFileConnection: settings.file_connections - 1,
    SliderDownloadSpeed: speed.index,
    CheckDownloadAutoSelectVersion: settings.auto_select_instance ?? true,
  }
}

export interface DownloadStats {
  active_connections: number
  max_connections: number
  speed: number
  downloaded_bytes: number
}

export function downloadSettings(values: Record<string, unknown> = {}): DownloadSettings {
  const settings = values
  const integer = (key: string, fallback: number, max: number) => {
    const raw = settings[key]
    const value =
      typeof raw === 'number' || (typeof raw === 'string' && raw.trim()) ? Number(raw) : NaN
    return Number.isInteger(value) && value >= 0 && value <= max ? value : fallback
  }
  const speed = integer('SliderDownloadSpeed', 42, 42)
  const mib = 1024 * 1024
  return {
    max_connections: integer('SliderDownloadThread', 63, 255) + 1,
    file_connections: integer('SliderDownloadFileConnection', 7, 7) + 1,
    speed_limit:
      speed <= 14
        ? Math.round((speed + 1) * 0.1 * mib)
        : speed <= 31
          ? Math.round((speed - 11) * 0.5 * mib)
          : speed <= 41
            ? (speed - 21) * mib
            : 0,
    http_mode: integer('ComboDownloadHttpMode', 0, 2),
    file_source: integer('ComboDownloadSource', 1, 2),
    version_source: integer('ComboDownloadVersion', 1, 2),
    community_source: integer('ComboDownloadMod', 1, 2),
    auto_select_instance:
      typeof settings.CheckDownloadAutoSelectVersion === 'boolean'
        ? settings.CheckDownloadAutoSelectVersion
        : true,
  }
}
