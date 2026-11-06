import { invoke, isTauri } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import sideTip from '@/composables/sideTip'
import { pclText } from './pclLocale'

export interface LogFileEntry {
  name: string
  path: string
  size: number
  /** Seconds since the Unix epoch. */
  modified: number
  current: boolean
}

export function listLogFiles() {
  return invoke<LogFileEntry[]>('list_log_files')
}

export function openLogDirectory() {
  return invoke<void>('open_log_directory')
}

/** Open a log file with the system default program. */
export function openLogPath(path: string) {
  return invoke<void>('open_log_path', { path })
}

export function clearLogFiles() {
  return invoke<void>('clear_log_files')
}

function exportFilter() {
  // PCL 语言包的过滤器格式为 "名称|*.zip"，转换成 Tauri 保存对话框的过滤器
  const [name, pattern] = pclText('Setup.Log.ExportFilter').split('|')
  return {
    name: name?.trim() || 'ZIP',
    extensions: [pattern?.trim().replace(/^\*\./, '') || 'zip'],
  }
}

/** Pack the given log files into a zip, CE 的 `PCL_CE_Logs_时间戳.zip`（默认桌面）。 */
export async function exportLogFiles(paths: string[]) {
  if (!isTauri() || paths.length === 0) return
  const destination = await save({
    title: pclText('Setup.Log.ExportSaveTitle'),
    defaultPath: await invoke<string>('log_export_destination'),
    filters: [exportFilter()],
  })
  if (!destination) return
  try {
    await invoke('export_log_files', { paths, destination })
    sideTip.show(pclText('Setup.Log.ExportSuccess'), 'success')
  } catch (error) {
    sideTip.show(`${pclText('Setup.Log.ExportFailed')} ${error}`, 'warn')
  }
}
