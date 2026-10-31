import { invoke, isTauri } from '@tauri-apps/api/core'
export type gameVersionType =
  'snapshot' | 'release' | 'old_beta' | 'old_alpha' | 'old' | 'special' | 'pending'
export interface ManifestVersion {
  id: string
  type: gameVersionType
  releaseTime: string
}
export interface SingleVersionShowInfo {
  id: string
  releaseTime: string
  timestamp: number
}

export function getMinecraftChangelogUrl(id: string, locale: string): string {
  const language = locale.split('-')[0] || 'en'
  const host = ['zh', 'ja', 'fr', 'es'].includes(language)
    ? `${language}.minecraft.wiki`
    : 'minecraft.wiki'
  // Wiki search also resolves historical manifest IDs without guessing page names.
  const url = new URL(`https://${host}/w/Special:Search`)
  const version = id
    .replace(/^b(?=\d)/, 'Beta ')
    .replace(/^a(?=\d)/, 'Alpha v')
    .replace(/^c(?=\d)/, 'Classic ')
  url.searchParams.set(
    'search',
    `${language === 'zh' ? (id.includes('w') ? '' : 'Java版') : 'Java Edition '}${version}`,
  )
  url.searchParams.set('go', 'Go')
  return url.href
}
export interface IVersionShow {
  latest: { release?: SingleVersionShowInfo; snapshot?: SingleVersionShowInfo }
  release: SingleVersionShowInfo[]
  snapshot: SingleVersionShowInfo[]
  old: SingleVersionShowInfo[]
  fool: SingleVersionShowInfo[]
}
// Mirrors PCL2-CE's McVersionClassifier, preserving download IDs from the manifest.
export function classifyVersion(version: ManifestVersion): 'release' | 'snapshot' | 'old' | 'fool' {
  if (version.type === 'release') return 'release'
  if (version.type === 'special') return 'fool'
  if (!['snapshot', 'pending'].includes(version.type)) return 'old'
  const id = version.id.toLowerCase()
  const date = new Date(Date.parse(version.releaseTime) + 2 * 3600000)
  if (
    /^(2(point|\.)0|20w14inf)/.test(id) ||
    [
      '20w14∞',
      '3d shareware v1.34',
      '1.rv-pre1',
      '15w14a',
      '22w13oneblockatatime',
      '23w13a_or_b',
      '24w14potato',
      '25w14craftmine',
      '26w14a',
    ].includes(id) ||
    (date.getUTCMonth() === 3 && date.getUTCDate() === 1)
  )
    return 'fool'
  return id.startsWith('1.') &&
    !['combat', 'rc', 'experimental', 'pre'].some((part) => id.includes(part)) &&
    id !== '1.2'
    ? 'release'
    : 'snapshot'
}
export async function getMinecraftVersions(): Promise<IVersionShow> {
  let data: { versions: ManifestVersion[] }
  if (isTauri()) data = await invoke('get_version_manifest')
  else {
    const response = await fetch('https://piston-meta.mojang.com/mc/game/version_manifest.json')
    if (!response.ok) throw new Error(`Minecraft 版本列表请求失败：HTTP ${response.status}`)
    data = await response.json()
  }
  if (
    !Array.isArray(data?.versions) ||
    data.versions.some(
      (item) =>
        !item ||
        typeof item.id !== 'string' ||
        typeof item.type !== 'string' ||
        typeof item.releaseTime !== 'string' ||
        !Number.isFinite(Date.parse(item.releaseTime)),
    )
  )
    throw new Error('Minecraft 版本列表格式错误')
  const result: IVersionShow = { latest: {}, release: [], snapshot: [], old: [], fool: [] }
  // Reuse the formatter instead of constructing one for every manifest entry.
  const dateFormat = new Intl.DateTimeFormat(undefined, {
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
  })
  for (const version of [...data.versions].sort(
    (a, b) => Date.parse(b.releaseTime) - Date.parse(a.releaseTime),
  )) {
    result[classifyVersion(version)].push({
      id: version.id,
      releaseTime: dateFormat.format(new Date(version.releaseTime)),
      timestamp: Date.parse(version.releaseTime),
    })
  }
  result.latest.release = result.release[0]
  result.latest.snapshot = result.snapshot[0]
  return result
}

interface DownloadInfo {
  sha1: string
  size: number
  url: string
  path?: string
}

export interface VersionDetails {
  id: string
  type: string
  downloads: {
    client: DownloadInfo
    server?: DownloadInfo
  }
  libraries: {
    name: string
    downloads: {
      artifact?: DownloadInfo
      classifiers?: Record<string, DownloadInfo>
    }
    rules?: {
      action: string
      features?: Record<string, boolean>
      os?: {
        name?: 'windows' | 'linux' | 'osx'
        arch?: string
        version?: string
      }
    }[]
  }[]
  assets: string
  assetIndex: DownloadInfo & { id: string }
}
