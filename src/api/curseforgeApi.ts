import { invoke, isTauri } from '@tauri-apps/api/core'
import { pclText } from './pclLocale'
import type { IProject, ISearchHit, IVersion } from '@/types/comp'

// CurseForge 的响应在这里归一化成 Modrinth 的字段形状（project_id / title / categories /
// versions / loaders / files[]），列表、详情与快捷下载因此不需要区分来源。
// 对照 CE：Modules/Minecraft/ModComp.cs 的 _BuildFromCurseForge、CompFile 与 CompFilesGet。

/// CurseForge 工程 ID 是纯数字，Modrinth 是乱码 ID（对照 CE 的 IsFromCurseForge）。
export const isCurseforgeProjectId = (id: string) => /^\d{1,9}$/.test(id)

const GAME_ID = 432
/// 资源种类 → classId，对照 CE 的 GetCurseForgeAddress。
const classIds: Record<string, number> = {
  mod: 6,
  modpack: 4471,
  resourcepack: 12,
  shader: 6552,
  datapack: 6945,
  world: 17,
}
const projectTypes: Record<number, ISearchHit['project_type']> = {
  6: 'mod',
  4471: 'modpack',
  12: 'resourcepack',
  6552: 'shader',
  6945: 'datapack',
  17: 'world',
}
/// 网站栏目段，用于拼项目主页。
const websiteSections: Record<string, string> = {
  mod: 'mc-mods',
  modpack: 'modpacks',
  resourcepack: 'texture-packs',
  shader: 'shaders',
  datapack: 'data-packs',
  world: 'worlds',
}
/// 可用作 CF modLoaderType 的加载器。光影的 iris/optifine/vanilla 不在 CF 的枚举里，
/// 因此对 CF 不附此筛选（CE 同样如此），只影响 Modrinth 一侧的 facets。
const modLoaderTypes: Record<string, number> = { forge: 1, fabric: 4, quilt: 5, neoforge: 6 }
/// values.sort 下标 → sortField，对照 CE：相关性 4 / 下载量 6 / 关注量 2 / 最新 11 / 更新 3。
const sortFields = [2, 4, 6, 2, 11, 3]
/// 归一化时认作加载器标记的 gameVersions 词（对照 CE 的 loader 检测）。
const loaderTokens = ['forge', 'fabric', 'quilt', 'neoforge', 'iris', 'optifine', 'canvas']
/// FileIndex.modLoader 数值 → 加载器名。
const loaderNamesByIndex: Record<number, string> = {
  1: 'forge',
  3: 'liteloader',
  4: 'fabric',
  5: 'quilt',
  6: 'neoforge',
  8: 'canvas',
  9: 'iris',
  10: 'optifine',
}
/// 版本形如 1.20.1 / 26.1 / 24w14a，排除 Forge、Client 等标记词（对照 CE 的 IsFormatFit）。
const versionPattern = /^(?:1\.\d|[2-9]\d\.\d+|\d{2}w\d{2})/
const PAGE_SIZE = 50
/// 文件列表分页上限：再多也只可能是极老的历史版本，界面按分组展示无需全量。
const FILE_PAGES = 8

export interface ICurseforgeFile {
  id: number
  modId: number
  displayName?: string
  fileName: string
  fileDate: string
  fileLength?: number | string
  fileSize?: number | string
  releaseType?: number
  isAvailable?: boolean
  downloadUrl?: string | null
  gameVersions?: string[]
  hashes?: { value: string; algo: number }[]
  dependencies?: { modId: number; relationType: number }[]
}

export interface ICurseforgeProject {
  id: number
  name: string
  slug: string
  summary?: string
  downloadCount?: number
  dateModified?: string
  dateReleased?: string
  classId?: number
  links?: { websiteUrl?: string; issuesUrl?: string; sourceUrl?: string; wikiUrl?: string }
  logo?: { url?: string; thumbnailUrl?: string }
  categories?: { id: number; name?: string; slug?: string; isClass?: boolean | null }[]
  latestFilesIndexes?: {
    fileId?: number
    gameVersion?: string
    modLoader?: number
    loaderName?: string
  }[]
  latestFiles?: ICurseforgeFile[]
}

interface ICurseforgeList<T> {
  data: T[]
  pagination?: { index?: number; pageSize?: number; resultCount?: number; totalCount?: number }
}

/// CurseForge 要求 x-api-key，且不允许浏览器跨域直连，因此只有桌面端可用。
async function curseforgeFetch<T>(endpoint: string): Promise<T> {
  if (!isTauri()) throw new Error(pclText('Proto.Resource.CurseForgeDesktopOnly'))
  return invoke<T>('fetch_with_curseforge', { endpoint })
}

/// 未配置 API Key 时界面需要收起 CurseForge 选项（对照 CE 的 SupportCurseForge）。
export async function curseforgeAvailable(): Promise<boolean> {
  if (!isTauri()) return false
  try {
    return await invoke<boolean>('curseforge_available')
  } catch {
    return false
  }
}

function requireList<T>(value: unknown, message: string): ICurseforgeList<T> {
  const body = value as Partial<ICurseforgeList<T>> | null
  if (!body || !Array.isArray(body.data)) throw new Error(message)
  return { data: body.data, pagination: body.pagination }
}

const fileNameOf = (file: ICurseforgeFile) => String(file?.fileName ?? '')

/// media.* / mediafilez.* → edge.forgecdn.net（对照 CE 的 HandleCurseForgeDownloadUrls）。
/// 回退地址按 CF 自己的规则用 id/1000 和 id%1000 分段，空格保持 %20。
export function curseforgeDownloadUrl(file: ICurseforgeFile) {
  const direct = String(file?.downloadUrl ?? '').trim()
  if (direct)
    return direct
      .replace('-service.overwolf.wtf', '.forgecdn.net')
      .replace('://media.', '://edge.')
      .replace('://mediafilez.', '://edge.')
      .replace('://mediafiles.', '://edge.')
  return `https://edge.forgecdn.net/files/${Math.floor(file.id / 1000)}/${file.id % 1000}/${encodeURIComponent(fileNameOf(file))}`
}

const usableFile = (file: ICurseforgeFile) =>
  file &&
  Number.isFinite(Number(file.id)) &&
  !!fileNameOf(file) &&
  file.isAvailable !== false &&
  /^https:\/\//i.test(curseforgeDownloadUrl(file))

/// 搜索结果的版本与加载器来自 latestFilesIndexes（对照 CE 的 Drops / ModLoaders）。
function readIndexes(project: ICurseforgeProject) {
  const versions: string[] = []
  const loaders: string[] = []
  for (const index of project.latestFilesIndexes ?? []) {
    const version = String(index?.gameVersion ?? '').trim()
    if (versionPattern.test(version) && !versions.includes(version)) versions.push(version)
    const loader = String(index?.loaderName ?? loaderNamesByIndex[Number(index?.modLoader)] ?? '')
      .toLowerCase()
      .trim()
    if (loader && !loaders.includes(loader)) loaders.push(loader)
  }
  return { versions, loaders }
}

export function curseforgeSearchHit(project: ICurseforgeProject): ISearchHit {
  const { versions, loaders } = readIndexes(project)
  return {
    project_id: String(project.id),
    slug: project.slug,
    project_type: projectTypes[Number(project.classId)] ?? 'mod',
    title: project.name,
    description: project.summary ?? '',
    icon_url: project.logo?.thumbnailUrl || project.logo?.url || '',
    downloads: Number(project.downloadCount) || 0,
    // CE 用 dateReleased 作为“最近更新”，避免只改说明的条目排到前面（CE #1194）。
    date_modified: project.dateReleased || project.dateModified || '',
    // 分类保留 CF 的数字 ID，标签文案由 compCategoryKeys 统一解析。
    categories: (project.categories ?? [])
      .filter((category) => category?.isClass !== true)
      .map((category) => String(category.id)),
    versions,
    loaders,
  }
}

/// 文件列表归一化成 IVersion：CF 的一个文件即一个可下载条目。
export function curseforgeFileVersion(file: ICurseforgeFile): IVersion {
  const versions: string[] = []
  const loaders: string[] = []
  for (const value of file.gameVersions ?? []) {
    const token = String(value).trim()
    const lower = token.toLowerCase()
    if (loaderTokens.includes(lower)) {
      if (!loaders.includes(lower)) loaders.push(lower)
    } else if (versionPattern.test(token) && !versions.includes(token)) versions.push(token)
  }
  return {
    id: String(file.id),
    name: String(file.displayName || file.fileName)
      .replace(/\t/g, '')
      .trim(),
    version_number: fileNameOf(file),
    game_versions: versions,
    loaders,
    date_published: file.fileDate,
    version_type: file.releaseType === 1 ? 'release' : file.releaseType === 2 ? 'beta' : 'alpha',
    files: [
      {
        url: curseforgeDownloadUrl(file),
        filename: fileNameOf(file),
        primary: true,
        size: Number(file.fileLength ?? file.fileSize) || 0,
        hashes: { sha1: file.hashes?.find((hash) => hash.algo === 1)?.value },
      },
    ],
  }
}

export interface CurseforgeSearchOptions {
  query?: string
  type: string
  loader?: string
  version?: string
  /// download-categories.json 的 tag 前半段（CurseForge categoryId），空表示不限制。
  categoryId?: string
  sort?: string
  offset?: number
  limit?: number
}

/// 构造搜索地址，参数顺序与 CE 的 GetCurseForgeAddress 一致，便于对照。
export function searchEndpoint(options: CurseforgeSearchOptions) {
  const limit = Math.min(Math.max(options.limit ?? 40, 1), PAGE_SIZE)
  const params = new URLSearchParams({
    gameId: String(GAME_ID),
    sortOrder: 'desc',
    pageSize: String(limit),
  })
  if (classIds[options.type]) params.set('classId', String(classIds[options.type]))
  if (options.categoryId) params.set('categoryId', options.categoryId)
  const loader = modLoaderTypes[options.loader ?? '']
  if (loader) params.set('modLoaderType', String(loader))
  if (options.version?.trim()) params.set('gameVersion', options.version.trim())
  if (options.query?.trim()) params.set('searchFilter', options.query.trim())
  params.set('sortField', String(sortFields[Number(options.sort)] ?? sortFields[0]))
  // index 是结果偏移量，且 index + pageSize 超过 10000 会被直接拒绝。
  const offset = Math.min(Math.max(options.offset ?? 0, 0), 10000 - limit)
  if (offset > 0) params.set('index', String(offset))
  return `mods/search?${params.toString()}`
}

async function searchProjects(
  options: CurseforgeSearchOptions,
): Promise<{ hits: ISearchHit[]; total_hits: number }> {
  const list = requireList<ICurseforgeProject>(
    await curseforgeFetch<unknown>(searchEndpoint(options)),
    'CurseForge 搜索结果格式错误',
  )
  const total = Number(list.pagination?.totalCount)
  return {
    hits: list.data.map(curseforgeSearchHit),
    // CF 的 totalCount 在浏览型查询里饱和在 10000，直接作为可翻页上限。
    total_hits: Number.isFinite(total) && total > 0 ? Math.min(total, 10000) : list.data.length,
  }
}

/// 项目详情，归一化成 IProject。CF 的详情包在 { data: ... } 里。
async function getProject(id: string): Promise<IProject> {
  const response = await curseforgeFetch<{ data?: ICurseforgeProject }>(
    `mods/${encodeURIComponent(id)}`,
  )
  const project = response?.data
  if (!project || !Number.isFinite(Number(project.id)) || typeof project.name !== 'string')
    throw new Error('CurseForge 项目信息格式错误')
  const hit = curseforgeSearchHit(project)
  return {
    id: hit.project_id,
    slug: project.slug,
    title: project.name,
    description: hit.description,
    categories: hit.categories,
    client_side: 'unknown',
    server_side: 'unknown',
    body: '',
    status: 'approved',
    requested_status: null,
    additional_categories: [],
    issues_url: project.links?.issuesUrl ?? null,
    source_url: project.links?.sourceUrl ?? null,
    wiki_url: project.links?.wikiUrl ?? null,
    donation_urls: [],
    project_type: hit.project_type,
    downloads: hit.downloads,
    published: project.dateModified ?? '',
    updated: project.dateModified ?? '',
    // 界面不读 IProject.versions（Modrinth 那里是版本 ID 列表），CF 直接放支持的 MC 版本。
    versions: hit.versions ?? [],
    loaders: hit.loaders ?? [],
    icon_url: hit.icon_url,
    team: '',
    organization: null,
  }
}

async function getProjectDescription(id: string): Promise<string> {
  const response = await curseforgeFetch<{ data?: unknown }>(
    `mods/${encodeURIComponent(id)}/description`,
  )
  if (typeof response?.data !== 'string') throw new Error('CurseForge 项目介绍格式错误')
  return response.data
}

/// 文件列表：CF 分页返回。首页拿到总量后并发取回剩下的页，
/// 最多 FILE_PAGES 页；已知目标版本/加载器时直接让上游筛，省掉无用的翻页。
async function getProjectVersions(
  id: string,
  filters: { version?: string; loader?: string } = {},
): Promise<IVersion[]> {
  const query = new URLSearchParams({ pageSize: String(PAGE_SIZE) })
  if (filters.version) query.set('gameVersion', filters.version)
  const loader = modLoaderTypes[filters.loader ?? '']
  if (loader) query.set('modLoaderType', String(loader))
  const page = async (index: number) =>
    requireList<ICurseforgeFile>(
      await curseforgeFetch<unknown>(
        `mods/${encodeURIComponent(id)}/files?${query.toString()}&index=${index}`,
      ),
      'CurseForge 文件列表格式错误',
    )
  const first = await page(0)
  const total = Number(first.pagination?.totalCount) || first.data.length
  const pages = Math.min(Math.ceil(total / PAGE_SIZE), FILE_PAGES)
  const rest =
    pages > 1
      ? await Promise.all(Array.from({ length: pages - 1 }, (_, i) => page((i + 1) * PAGE_SIZE)))
      : []
  return [...first.data, ...rest.flatMap((item) => item.data)]
    .filter(usableFile)
    .map(curseforgeFileVersion)
}

/// 项目主页（列表与详情共用）。
export function curseforgeProjectUrl(project: Pick<IProject, 'slug' | 'project_type'>) {
  return `https://www.curseforge.com/minecraft/${websiteSections[project.project_type] ?? 'mc-mods'}/${encodeURIComponent(project.slug)}`
}

export default {
  isProjectId: isCurseforgeProjectId,
  available: curseforgeAvailable,
  projectUrl: curseforgeProjectUrl,
  searchEndpoint,
  searchProjects,
  getProject,
  getProjectDescription,
  getProjectVersions,
}
