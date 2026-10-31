import { invoke } from '@tauri-apps/api/core'
import exportOptions from '@/assets/export-options.json'

const fields: Record<string, Record<string, string | boolean>> = {
  PageInstanceOverall: { ComboDisplayLogo: '0', ComboDisplayType: '0' },
  PageInstanceSavesDatapack: { filter: 'All', SearchBox: '' },
  PageInstanceExport: {
    TextExportName: '',
    TextExportVersion: '',
    CheckAdvancedInclude: false,
    CheckAdvancedModrinth: false,
    ...Object.fromEntries(exportOptions.map((option) => [option.id, option.checked])),
  },
}

export async function migrateLegacyInstancePages(): Promise<number> {
  const records: { key: string; page: string; id: string }[] = []
  for (let index = 0; index < localStorage.length; index++) {
    const key = localStorage.key(index)
    if (!key) continue
    for (const page of Object.keys(fields)) {
      const prefix = `pcl-ui:${page}:`
      if (!key.startsWith(prefix)) continue
      const id = key.slice(prefix.length)
      if (
        id &&
        id !== 'none' &&
        !localStorage.getItem(`pcl-instance-migrated:${encodeURIComponent(key)}`)
      ) {
        records.push({ key, page, id })
      }
    }
  }
  if (!records.length) return 0
  let matches: Record<string, string>
  try {
    matches = await invoke('resolve_legacy_instance_ids', {
      ids: [...new Set(records.map(({ id }) => id))],
    })
  } catch (error) {
    console.warn('Cannot verify legacy instance page settings', error)
    return records.length
  }
  let pending = 0
  for (const { key, page, id } of records) {
    const directory = Object.prototype.hasOwnProperty.call(matches, id) ? matches[id] : undefined
    if (!directory) {
      pending++
      continue
    }
    let old: unknown
    try {
      old = JSON.parse(localStorage.getItem(key) || '')
    } catch {
      pending++
      continue
    }
    if (!old || typeof old !== 'object' || Array.isArray(old)) {
      pending++
      continue
    }
    const target = `pcl-instance-ui:${page}:${encodeURIComponent(directory)}`
    if (localStorage.getItem(target) === null) {
      const allowed = fields[page]!
      const values = Object.fromEntries(
        Object.entries(old).filter(
          ([name, value]) =>
            Object.prototype.hasOwnProperty.call(allowed, name) &&
            typeof value === typeof allowed[name],
        ),
      )
      localStorage.setItem(target, JSON.stringify(values))
    }
    localStorage.setItem(`pcl-instance-migrated:${encodeURIComponent(key)}`, directory)
  }
  return pending
}
