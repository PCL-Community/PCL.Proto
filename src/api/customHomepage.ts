import { invoke, isTauri } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { appearance } from '@/api/appearance'
import { homepageLimit, homepageUrl, parseHomepage } from '@/util/homepageXaml'

export const homepageRevision = ref(0)
export const homepagePresets = [
  { key: '0', name: 'Trivia' },
  { key: '2', name: 'McNews', url: 'https://news.bugjump.net' },
  { key: '3', name: 'DailyModpack', url: 'https://pclsub.sodamc.com/' },
  { key: '4', name: 'McSkin', url: 'https://forgepixel.com/pcl_sub_file' },
  { key: '5', name: 'OpenBmclapi', url: 'https://pcl-bmcl.milu.ink/' },
  {
    key: '6',
    name: 'PclManual',
    url: 'https://raw.gitcode.com/WForst-Breeze/WhatsNewPCL/raw/main/Custom.xaml',
  },
  {
    key: '7',
    name: 'Magazine',
    url: 'https://pclhomeplazaoss.lingyunawa.top:26994/d/Homepages/Ext1nguisher/Custom.xaml',
  },
  { key: '8', name: 'GitHub', url: 'https://ddf.pcl-community.org/Custom.xaml' },
  {
    key: '9',
    name: 'McUpdateSummary',
    url: 'https://raw.gitcode.com/ENC_Euphony/PCL-AI-Summary-HomePage/raw/master/Custom.xaml',
  },
  { key: '10', name: 'NewsToday', url: 'https://pcl.wyc-w.top/index.xaml' },
  { key: '11', name: 'MinecraftKnowledge', url: 'https://www.xxag.top/mkss' },
  {
    key: '12',
    name: 'ModpackRecommendation',
    url: 'https://qawsedrftgyhujiko.fun/pcl2/Custom.xaml',
  },
  { key: '13', name: 'Bangumi', url: 'https://bangumi.p.kaphia.qzz.io' },
  {
    key: '14',
    name: 'PclAnnouncement',
    url: 'https://s3.pysio.online/pcl2-ce/apiv2/pages/announce.xaml',
  },
  { key: '15', name: 'McOfficialFeed' },
]
const localKey = 'pcl-homepage:local'
const cacheKey = 'pcl-homepage:network-cache'

export async function readLocalHomepage() {
  if (isTauri()) return invoke<string>('load_custom_homepage', { url: null })
  const content = localStorage.getItem(localKey)
  if (content === null) throw new Error('Import a Custom.xaml file in the homepage editor first.')
  return content
}

export async function saveLocalHomepage(content: string) {
  parseHomepage(content)
  if (isTauri()) await invoke('save_custom_homepage', { content })
  else localStorage.setItem(localKey, content)
  appearance.values.UiCustomType = '1'
  homepageRevision.value++
}

export function cachedHomepage(url: string): string | undefined {
  try {
    const cache = JSON.parse(localStorage.getItem(cacheKey) || 'null')
    if (cache?.url === homepageUrl(url) && typeof cache.content === 'string') {
      parseHomepage(cache.content)
      return cache.content
    }
  } catch {
    /* An invalid cache is replaced after the next successful download. */
  }
}

export async function downloadHomepageText(value: string, signal?: AbortSignal) {
  const url = homepageUrl(value)
  if (!url) throw new Error('Invalid homepage URL')
  let content: string
  if (isTauri()) content = await invoke<string>('load_custom_homepage', { url })
  else {
    const response = await fetch(url, {
      signal: signal
        ? AbortSignal.any([signal, AbortSignal.timeout(20000)])
        : AbortSignal.timeout(20000),
    })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    if (!response.body) throw new Error('Empty homepage response')
    const reader = response.body.getReader()
    const chunks: Uint8Array[] = []
    let size = 0
    try {
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        size += value.length
        if (size > homepageLimit) throw new Error('Homepage exceeds 2 MiB')
        chunks.push(value)
      }
    } finally {
      await reader.cancel()
    }
    const bytes = new Uint8Array(size)
    let offset = 0
    for (const chunk of chunks) {
      bytes.set(chunk, offset)
      offset += chunk.length
    }
    content = new TextDecoder('utf-8', { fatal: true }).decode(bytes)
  }
  if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
  return content
}

export async function downloadHomepage(value: string, signal?: AbortSignal) {
  const url = homepageUrl(value)
  if (!url) throw new Error('Invalid homepage URL')
  const content = await downloadHomepageText(url, signal)
  parseHomepage(content)
  try {
    localStorage.setItem(cacheKey, JSON.stringify({ url, content }))
  } catch {
    /* Storage quotas must not prevent a downloaded homepage from rendering. */
  }
  return content
}
