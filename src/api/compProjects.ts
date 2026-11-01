import modrinthApi from './modrinthApi'
import type {
  IProject,
  ISearchHit,
  IVersion,
  CompType,
  CompSource,
  CompSourceFilter,
} from '@/types/comp'
import curseforgeApi, { isCurseforgeProjectId } from './curseforgeApi'
import { compFacets, loaderNames } from './compResources'
import { pclText } from './pclLocale'

// 两个来源的统一入口：界面只表达“要哪些来源”，其余差异都在这里收口。
// 对照 CE：ModComp.CompProjectRequest（CurseForge / Modrinth 双请求 + 去重）。

export interface CompFilters {
  type: CompType
  query: string
  loader: string
  version: string
  /// download-categories.json 的 tag：前半段是 CurseForge categoryId，后半段是 Modrinth 分类。
  tag: string
  sort: string
}

export interface CompSearchResult {
  hits: ISearchHit[]
  total: number
  /// 有一个来源没请求成功，但另一个来源仍有结果时的提示文案。
  warning: string
}

/// 纯数字 ID 来自 CurseForge，乱码 ID 来自 Modrinth（对照 CE 的 IsFromCurseForge）。
export const compSourceOf = (id: string): CompSource =>
  isCurseforgeProjectId(id) ? 'curseforge' : 'modrinth'

export const compSourcesOf = (filter: CompSourceFilter): CompSource[] =>
  filter === 'all' ? ['modrinth', 'curseforge'] : [filter]

const modrinthSorts = ['relevance', 'downloads', 'follows', 'newest', 'updated'] as const

/// 默认“全部”排序：CF 的下载量普遍高于 MR，按 CE 的来源权重折算后再排一次。
const downloadWeight: Record<CompType, [number, number]> = {
  mod: [1, 7],
  modpack: [1, 7],
  datapack: [10, 1],
  resourcepack: [1, 5],
  shader: [1, 5],
  world: [1, 1],
}

const compact = (value: string) => value.toLowerCase().replace(/[^a-z0-9\u4e00-\u9fff]/g, '')
const sameSet = (a: string[], b: string[]) =>
  new Set(a).size === new Set(b).size && a.every((value) => b.includes(value))
const loadersOf = (hit: ISearchHit) =>
  [...(hit.loaders ?? []), ...hit.categories].filter((value) => value in loaderNames)

/// 同一工程的两侧条目：类型相同、更新时间相差不到一周，且名称或简介一致（对照 CE 的 IsLike）。
function isSameProject(a: ISearchHit, b: ISearchHit) {
  if (a.project_id === b.project_id)
    return compSourceOf(a.project_id) === compSourceOf(b.project_id)
  if (a.project_type !== b.project_type) return false
  if (compSourceOf(a.project_id) === compSourceOf(b.project_id)) return false
  if (!sameSet(loadersOf(a), loadersOf(b))) return false
  if (a.project_type !== 'shader' && !sameSet(a.versions ?? [], b.versions ?? [])) return false
  const updated = Date.parse(a.date_modified) - Date.parse(b.date_modified)
  if (Number.isFinite(updated) && Math.abs(updated) > 7 * 86400_000) return false
  return (
    (!!a.title && a.title === b.title) ||
    (!!a.description && a.description === b.description) ||
    (!!a.slug && !!b.slug && !!compact(a.slug) && compact(a.slug) === compact(b.slug))
  )
}

export function mergeCompHits(primary: ISearchHit[], secondary: ISearchHit[]) {
  const merged: ISearchHit[] = []
  for (const hit of [...primary, ...secondary])
    if (!merged.some((kept) => isSameProject(kept, hit))) merged.push(hit)
  return merged
}

/// CE 用来源权重抵消两边的下载量口径差，避免默认列表全部被 CF 占据。
export function rankCompHits(hits: ISearchHit[], type: CompType) {
  const [curseforge, modrinth] = downloadWeight[type]
  return [...hits].sort(
    (a, b) =>
      b.downloads * (compSourceOf(b.project_id) === 'curseforge' ? curseforge : modrinth) -
      a.downloads * (compSourceOf(a.project_id) === 'curseforge' ? curseforge : modrinth),
  )
}

/// tag 只有一半映射时，对应来源必须跳过：CF 缺 categoryId 会退化成无过滤，MR 缺分类同理。
function sourcesForTag(sources: CompSource[], tag: string) {
  if (!tag) return sources
  return sources.filter((source) =>
    source === 'curseforge' ? !!tag.split('/')[0] : !!tag.split('/').slice(1).join('/'),
  )
}

async function searchSource(
  source: CompSource,
  filters: CompFilters,
  offset: number,
  limit: number,
) {
  if (source === 'curseforge')
    return curseforgeApi.searchProjects({
      type: filters.type,
      query: filters.query,
      loader: filters.loader,
      version: filters.version,
      categoryId: filters.tag.split('/')[0] ?? '',
      sort: filters.sort,
      offset,
      limit,
    })
  const index =
    modrinthSorts[Number(filters.sort) - 1] ?? (filters.query.trim() ? 'relevance' : 'downloads')
  return modrinthApi.searchProjects({
    query: filters.query,
    facets: compFacets(filters.type, filters),
    index,
    offset,
    limit,
  })
}

type SourceQuery = { source: CompSource; hits: ISearchHit[]; total: number }
type SourceFailure = { source: CompSource; error: unknown }

export async function searchProjects(
  filter: CompSourceFilter,
  filters: CompFilters,
  offset: number,
  limit: number,
): Promise<CompSearchResult> {
  const wanted = sourcesForTag(compSourcesOf(filter), filters.tag)
  const results = await Promise.all(
    wanted.map(async (source): Promise<SourceQuery | SourceFailure> => {
      try {
        const data = await searchSource(source, filters, offset, limit)
        return { source, hits: data.hits, total: data.total_hits }
      } catch (error) {
        return { source, error }
      }
    }),
  )
  const done = results.filter((item): item is SourceQuery => 'hits' in item)
  const failed = results.filter((item): item is SourceFailure => !('hits' in item))
  if (!done.length) throw failed[0]?.error ?? new Error(pclText('Proto.Error.Resources'))
  // 只有一个来源时直接用 API 顺序；两个来源都返回结果才须合并加权重排。
  const merged = done.reduce<ISearchHit[]>((all, item) => mergeCompHits(all, item.hits), [])
  return {
    hits: wanted.length > 1 && filters.sort === '0' ? rankCompHits(merged, filters.type) : merged,
    total: done.reduce((sum, item) => sum + item.total, 0),
    warning:
      failed.length && merged.length
        ? pclText(
            failed[0].source === 'curseforge'
              ? 'Download.Comp.List.Source.CurseForgeUnavailable'
              : 'Download.Comp.List.Source.ModrinthUnavailable',
          )
        : '',
  }
}

export const getCompProject = (id: string): Promise<IProject> =>
  compSourceOf(id) === 'curseforge' ? curseforgeApi.getProject(id) : modrinthApi.getProject(id)

export const getCompVersions = (
  id: string,
  filters?: { version?: string; loader?: string },
): Promise<IVersion[]> =>
  compSourceOf(id) === 'curseforge'
    ? curseforgeApi.getProjectVersions(id, filters)
    : modrinthApi.getProjectVersions(id)

/// 项目主页，列表来源标记与详情的“在浏览器打开”共用。
export function compProjectUrl(project: Pick<IProject, 'id' | 'slug' | 'project_type'>) {
  return compSourceOf(project.id) === 'curseforge'
    ? curseforgeApi.projectUrl(project)
    : `https://modrinth.com/${project.project_type}/${project.slug}`
}
