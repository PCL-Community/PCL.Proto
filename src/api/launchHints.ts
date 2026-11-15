import { invoke, isTauri } from '@tauri-apps/api/core'
import { pclLocale } from './pclLocale'
import hintsZh from '@/assets/homepage/hints-zh-CN.txt?raw'
import hintsEn from '@/assets/homepage/hints-en-US.txt?raw'

const PAGE_SETTINGS_KEY = 'pcl-ui:PageSetupUI:global'

/** 「显示启动提示」与设置页共用同一份页面设置，默认开启。 */
export function launchHintEnabled(): boolean {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(PAGE_SETTINGS_KEY) || '{}')
    if (parsed && typeof parsed === 'object' && 'CheckShowLaunchingHint' in parsed)
      return parsed.CheckShowLaunchingHint !== false
  } catch {
    // 损坏的设置按默认值处理。
  }
  return true
}

function parseHints(text: string) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
}

/**
 * 抽取一条启动提示，对照 CE `PageLaunchRight.GetRandomHint`：
 * 优先读取配置目录下的 `hints.txt`，否则使用内置列表；优先 50 字以内的短句。
 */
export async function pickLaunchHint(): Promise<string> {
  let lines: string[] = []
  if (isTauri()) {
    try {
      const external = await invoke<string | null>('read_launch_hints')
      if (external) lines = parseHints(external)
    } catch {
      // 读取失败时退回内置列表。
    }
  }
  if (!lines.length) {
    const bundled = pclLocale.value.startsWith('en') ? hintsEn : hintsZh
    // 内置列表混有 CE 自身的介绍和自定义说明，不适合出现在启动提示里。
    lines = parseHints(bundled).filter((line) => !/pcl|hints\.txt/i.test(line))
  }
  if (!lines.length) return ''
  const short = lines.filter((line) => line.length < 50)
  const pool = short.length ? short : lines
  return pool[Math.floor(Math.random() * pool.length)]
}
