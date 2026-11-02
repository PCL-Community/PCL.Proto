/**
 * 旧版把实例页面设置放在 `pcl-ui:<Page>:<实例 ID>` 袋子里。
 * 值迁入 Rust 配置或新 key 后必须删掉，否则同一个设置会长期存在两份存储。
 */
export type LegacyPage = { key: string; id: string; value: Record<string, unknown> | null }

export function legacyInstancePages(prefix: string): LegacyPage[] {
  const entries: LegacyPage[] = []
  for (let index = 0; index < localStorage.length; index++) {
    const key = localStorage.key(index)
    if (!key?.startsWith(prefix)) continue
    const id = key.slice(prefix.length)
    if (!id || id === 'none') continue
    try {
      const parsed: unknown = JSON.parse(localStorage.getItem(key) || 'null')
      entries.push({
        key,
        id,
        value:
          parsed && typeof parsed === 'object' && !Array.isArray(parsed)
            ? (parsed as Record<string, unknown>)
            : null,
      })
    } catch {
      /* Keep malformed legacy data intact. */
    }
  }
  return entries
}

export function dropLegacyFields(key: string, fields: readonly string[]) {
  let stored: unknown
  try {
    stored = JSON.parse(localStorage.getItem(key) || 'null')
  } catch {
    return
  }
  if (!stored || typeof stored !== 'object' || Array.isArray(stored)) return
  const values = stored as Record<string, unknown>
  for (const field of fields) delete values[field]
  if (Object.keys(values).length) localStorage.setItem(key, JSON.stringify(values))
  else localStorage.removeItem(key)
}
