import categories from '@/assets/download-categories.json'
import { pclLocale, pclText } from './pclLocale'
import type { ISearchHit, IVersion } from './modrinthApi'
import type GameInstance from '@/types/gameInstance'

export type CompType = keyof typeof categories

const categoryKeys: Record<string, string> = Object.fromEntries(
  Object.values(categories).flatMap((items) =>
    items
      .filter((item) => item.tag.split('/')[1])
      .map((item) => [item.tag.split('/')[1], item.text]),
  ),
)
// tag 的前半段是 CurseForge 分类 ID（例如 "406/worldgen"），归一化后的分类同样按这张表取文案。
for (const items of Object.values(categories))
  for (const item of items) {
    const id = item.tag.split('/')[0]
    if (id && !categoryKeys[id]) categoryKeys[id] = item.text
  }
Object.assign(categoryKeys, {
  iris: 'Download.Comp.Category.Iris',
  optifine: 'Download.Comp.Category.Optifine',
  vanilla: 'Download.Comp.Filter.Loader.VanillaAvailable',
})
const hiddenTags = ['datapack', 'themed', 'cursed', 'entities', 'potato', 'low', 'medium', 'high']
export const loaderNames: Record<string, string> = {
  forge: 'Forge',
  neoforge: 'NeoForge',
  fabric: 'Fabric',
  quilt: 'Quilt',
}

export function compTagOptions(type: CompType) {
  // 只要有一侧能映射就保留：CF 独有的分类（如 "407/"）也要可选。
  return categories[type]
    .filter((item) => !item.tag || item.tag.replace('/', ''))
    .map((item) => ({
      key: item.tag || (item.text === 'Common.Option.All' ? '' : item.text),
      text: pclText(item.text),
      disabled: !item.tag && item.text !== 'Common.Option.All',
    }))
}

export function compTags(project: ISearchHit) {
  const tags = project.categories.flatMap((category) => {
    if (hiddenTags.includes(category)) return []
    if (
      project.project_type === 'resourcepack' &&
      ['decoration', 'mobs', 'equipment'].includes(category)
    )
      return []
    const key = categoryKeys[category]
    return key ? [pclText(key)] : []
  })
  return tags.length
    ? [...new Set(tags)].sort((a, b) => a.localeCompare(b, pclLocale.value))
    : [pclText('Download.Comp.Category.Other')]
}

export function compFacets(
  type: CompType,
  values: { loader: string; version: string; tag: string },
) {
  const facets = [[`project_type:${type === 'datapack' ? 'mod' : type}`]]
  if (type === 'datapack') facets.push(['categories:datapack'])
  if (values.loader && ['mod', 'modpack', 'shader'].includes(type))
    facets.push([`categories:${values.loader}`])
  if (values.version.trim()) facets.push([`versions:${values.version.trim()}`])
  const tag = values.tag.split('/')[1]
  if (tag) facets.push([`categories:${tag}`])
  return facets
}

export function compLoaders(project: ISearchHit) {
  return [...new Set([...(project.loaders || []), ...project.categories])]
    .flatMap((loader) => (loaderNames[loader] ? [loaderNames[loader]] : []))
    .join(' / ')
}

export function compVersions(versions: string[] = []) {
  const releases = [
    ...new Set(
      versions.flatMap((version) =>
        /^\d+\.\d+(?:\.\d+)?$/.test(version) ? [version.split('.').slice(0, 2).join('.')] : [],
      ),
    ),
  ].sort((a, b) => b.localeCompare(a, 'en', { numeric: true }))
  if (!releases.length)
    return pclText(
      versions.length
        ? 'Download.Comp.Detail.CompItem.SnapshotOnly'
        : 'Download.Comp.Detail.CompItem.UnknownVersion',
    )
  const segments: string[] = []
  for (let i = 0; i < releases.length; i++) {
    const start = releases[i]!
    let end = start
    while (i + 1 < releases.length) {
      const next = releases[i + 1]!
      if (
        next.split('.')[0] !== end.split('.')[0] ||
        Number(next.split('.')[1]) !== Number(end.split('.')[1]) - 1
      )
        break
      end = next
      i++
    }
    segments.push(start === end ? start : `${end}~${start}`)
  }
  return segments.join(', ')
}

export function latestCompFile(versions: IVersion[], gameVersion = '', loader = '') {
  const compatible = versions
    .filter(
      (version) =>
        (!gameVersion || version.game_versions.includes(gameVersion)) &&
        (!loader || version.loaders.includes(loader)),
    )
    .sort((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published))
  const files = compatible[0]?.files || []
  const file = files.find((file) => file.primary) || files[0]
  return file && /^https:\/\//i.test(file.url) ? file : undefined
}

export function compatibleResource(version: IVersion, instance: GameInstance, type: CompType) {
  if (
    ['mod', 'datapack', 'world'].includes(type) &&
    version.game_versions.some((v) => v.includes('.')) &&
    !version.game_versions.includes(instance.version)
  )
    return false
  const loaders = version.loaders.filter((value) =>
    ['forge', 'fabric', 'neoforge', 'quilt', 'liteloader'].includes(value),
  )
  if (!loaders.length) return true
  const plugins = instance.pluginsVersion || {}
  return loaders.some(
    (loader) =>
      loader in plugins ||
      (loader === 'forge' && 'cleanroom' in plugins) ||
      (loader === 'fabric' && 'legacyfabric' in plugins),
  )
}

export function latestResourceVersion(
  versions: IVersion[],
  instance?: GameInstance,
  type?: CompType,
) {
  return [...versions]
    .filter((version) => !instance || compatibleResource(version, instance, type!))
    .sort(
      (a, b) =>
        Number(b.version_type === 'release') - Number(a.version_type === 'release') ||
        Date.parse(b.date_published) - Date.parse(a.date_published),
    )
    .find((version) => version.files.some((file) => file.url && file.filename))
}

export function instanceCompFilter(instance: GameInstance | null, type: CompType) {
  const loader = ['forge', 'fabric', 'neoforge', 'quilt'].find(
    (name) => name in (instance?.pluginsVersion ?? {}),
  )
  return {
    version: instance?.version ?? '',
    loader: type === 'mod' || type === 'modpack' ? (loader ?? '') : '',
  }
}

export function selectedCompVersions(
  versions: readonly IVersion[],
  targetVersion: string,
  targetLoader: string,
  gameVersion: string,
  loader: string,
  byDrop: boolean,
  foldOld: boolean,
) {
  if (!targetVersion || (loader && loader !== targetLoader)) return []
  if (gameVersion && groupedVersionName(targetVersion, byDrop, foldOld) !== gameVersion) return []
  return versions
    .filter(
      (version) =>
        version.game_versions.includes(targetVersion) &&
        (!targetLoader || version.loaders.includes(targetLoader)),
    )
    .sort((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published))
}

/// 版本名 → Drop 序数（1.21 → 121，快照按年份估算），无法识别时为 -1。对照 CE 的 McInstanceInfo.VersionToDrop。
export function mcVersionDrop(version: string): number {
  const lower = version.toLowerCase()
  if (lower.startsWith('1.rv')) return 90
  if (lower.startsWith('3d shareware')) return 140
  const base = lower.split('-')[0] ?? ''
  const snapshot = /^(\d\d)w/.exec(base)
  if (snapshot) {
    const year = Number(snapshot[1])
    if (year <= 16) return Math.max(year * 10 - 70, 20)
    if (year === 17) return 120
    return Math.min(130 + (year - 18) * 10, 210)
  }
  const parts = /^(\d+)\.(\d+)/.exec(base)
  if (!parts) return -1
  const major = Number(parts[1])
  const minor = Number(parts[2])
  if (major === 1) return minor * 10
  if (major >= 25) return major * 10 + minor
  if (major === 2) return 50
  return -1
}

export function dropToVersion(drop: number): string {
  return drop >= 250 ? `${Math.floor(drop / 10)}.${drop % 10}` : `1.${Math.floor(drop / 10)}`
}

/// 按 CE 规则把版本名归入筛选项名称：快照、远古版、Drop（1.21）或原样。
export function groupedVersionName(name: string, byDrop = false, foldOld = false): string {
  if (!name) return pclText('Download.Comp.Detail.VersionGroup.Other')
  if (name.includes('w')) return pclText('Download.Comp.Detail.VersionGroup.Snapshot')
  const drop = mcVersionDrop(name)
  if (foldOld && drop < 120) return pclText('Download.Comp.Detail.VersionGroup.Old')
  if (byDrop) return drop >= 0 ? dropToVersion(drop) : name
  return name
}

/// 生成筛选器的全部选项；选项过多时逐级合并为大版本 / 远古版。
export function compVersionFilters(names: readonly string[]) {
  const comparer = new Intl.Collator('en', { numeric: true })
  const build = (byDrop: boolean, foldOld: boolean) =>
    [...new Set(names.map((name) => groupedVersionName(name, byDrop, foldOld)))].sort(
      (a, b) => Number(/^\d/.test(b)) - Number(/^\d/.test(a)) || comparer.compare(b, a),
    )
  let byDrop = false
  let foldOld = false
  let list = build(false, false)
  if (list.length >= 9) {
    byDrop = true
    list = build(true, false)
    if (list.length >= 9) {
      byDrop = false
      foldOld = true
      list = build(false, true)
      if (list.length >= 9) {
        byDrop = true
        list = build(true, true)
      }
    }
  }
  return { list, byDrop, foldOld }
}

export function groupCompVersions(
  versions: readonly IVersion[],
  gameVersion = '',
  loader = '',
  splitLoaders = false,
  byDrop = false,
  foldOld = false,
) {
  const groups = new Map<
    string,
    { key: string; name: string; gameVersion: string; versions: IVersion[] }
  >()
  const comparer = new Intl.Collator('en', { numeric: true })
  for (const version of [...versions].sort(
    (a, b) => Date.parse(b.date_published) - Date.parse(a.date_published),
  )) {
    if (loader && !version.loaders.includes(loader)) continue
    for (const game of new Set(version.game_versions.length ? version.game_versions : [''])) {
      if (gameVersion && groupedVersionName(game, byDrop, foldOld) !== gameVersion) continue
      const name = groupedVersionName(game)
      const loaders =
        splitLoaders && game.includes('.')
          ? [...new Set(version.loaders)].filter((value) => !loader || value === loader)
          : ['']
      for (const value of loaders.length ? loaders : ['']) {
        const key = `${value}:${name}`
        let group = groups.get(key)
        if (!group) {
          group = {
            key,
            name: `${loaderNames[value] || value}${value ? ' ' : ''}${name}`,
            gameVersion: name,
            versions: [],
          }
          groups.set(key, group)
        }
        if (group.versions[group.versions.length - 1] !== version) group.versions.push(version)
      }
    }
  }
  return [...groups.values()].sort(
    (a, b) =>
      Number(b.gameVersion.includes('.')) - Number(a.gameVersion.includes('.')) ||
      comparer.compare(b.gameVersion, a.gameVersion) ||
      comparer.compare(b.name, a.name),
  )
}
