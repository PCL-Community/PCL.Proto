import { invoke, isTauri } from '@tauri-apps/api/core'

// 通用 Modrinth API 请求
async function modrinthFetch<T>(endpoint: string): Promise<T> {
  if (isTauri()) return invoke<T>('fetch_with_modrinth', { endpoint })
  const response = await fetch(`https://api.modrinth.com/v2/${endpoint}`)
  if (!response.ok) throw new Error(`Modrinth 请求失败：HTTP ${response.status}`)
  return response.json()
}

interface SearchOptions {
  query?: string
  facets?: string[][] // 二维数组，支持 AND/OR 逻辑
  index?: 'relevance' | 'downloads' | 'follows' | 'newest' | 'updated'
  offset?: number
  limit?: number
}

/// 归一化后的资源类型：Modrinth 只会给前四种，CurseForge 按 classId 补齐后两种。
export type CompProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' | 'world'

export interface ISearchHit {
  slug?: string
  title: string
  description: string
  icon_url: string
  categories: string[]
  downloads: number
  date_modified: string
  project_id: string
  project_type: CompProjectType
  versions?: string[]
  loaders?: string[]
}

export interface IProject {
  slug: string
  id: string
  title: string
  description: string
  categories: string[]
  client_side: 'required' | 'optional' | 'unsupported' | 'unknown'
  server_side: 'required' | 'optional' | 'unsupported' | 'unknown'
  body: string
  status:
    | 'approved'
    | 'archived'
    | 'rejected'
    | 'draft'
    | 'unlisted'
    | 'processing'
    | 'withheld'
    | 'scheduled'
    | 'private'
    | 'unknown'
  requested_status: 'approved' | 'archived' | 'unlisted' | 'private' | 'draft' | null
  additional_categories: string[]
  issues_url: string | null
  source_url: string | null
  wiki_url: string | null
  donation_urls: string[]
  project_type: CompProjectType
  downloads: number
  published: string
  updated: string
  versions: string[]
  loaders: string[]
  team: string
  organization: string | null
  icon_url: string
}

export interface IVersion {
  id: string
  name: string
  version_number: string
  game_versions: string[]
  loaders: string[]
  date_published: string
  version_type?: 'release' | 'beta' | 'alpha'
  files: {
    url: string
    filename: string
    primary: boolean
    size: number
    hashes?: { sha1?: string }
  }[]
}

// 搜索项目
async function searchProjects(options: SearchOptions = {}) {
  const params = new URLSearchParams()
  if (options.query) params.append('query', options.query)
  if (options.facets) params.append('facets', JSON.stringify(options.facets))
  if (options.index) params.append('index', options.index)
  if (options.offset) params.append('offset', options.offset.toString())
  if (options.limit) params.append('limit', options.limit.toString())
  const data = await modrinthFetch<{
    hits: ISearchHit[]
    offset: number
    limit: number
    total_hits: number
  }>(`search?${params.toString()}`)
  if (
    !data ||
    !Array.isArray(data.hits) ||
    !Number.isFinite(data.total_hits) ||
    data.hits.some(
      (item) =>
        !item ||
        typeof item.project_id !== 'string' ||
        typeof item.title !== 'string' ||
        typeof item.description !== 'string' ||
        !Number.isFinite(item.downloads) ||
        typeof item.date_modified !== 'string' ||
        !Array.isArray(item.categories) ||
        item.categories.some((category) => typeof category !== 'string') ||
        ['versions', 'loaders'].some((field) => {
          const values = item[field as 'versions' | 'loaders']
          return (
            values !== undefined &&
            (!Array.isArray(values) || values.some((value) => typeof value !== 'string'))
          )
        }),
    )
  )
    throw new Error('Modrinth 搜索结果格式错误')
  return data
}

// 获取项目详情
async function getProject(id: string): Promise<IProject> {
  const data = await modrinthFetch<IProject>(`project/${encodeURIComponent(id)}`)
  if (
    !data ||
    typeof data.id !== 'string' ||
    typeof data.title !== 'string' ||
    typeof data.description !== 'string' ||
    typeof data.slug !== 'string' ||
    !Number.isFinite(data.downloads) ||
    typeof data.updated !== 'string' ||
    !Array.isArray(data.categories)
  )
    throw new Error('Modrinth 项目信息格式错误')
  return data
}

// 获取项目版本
async function getProjectVersions(id: string) {
  const data = await modrinthFetch<IVersion[]>(`project/${encodeURIComponent(id)}/version`)
  if (
    !Array.isArray(data) ||
    data.some(
      (item) =>
        !item ||
        typeof item.id !== 'string' ||
        typeof item.name !== 'string' ||
        !Array.isArray(item.game_versions) ||
        item.game_versions.some((value) => typeof value !== 'string') ||
        !Array.isArray(item.loaders) ||
        item.loaders.some((value) => typeof value !== 'string') ||
        !Array.isArray(item.files) ||
        item.files.some(
          (file) => !file || typeof file.url !== 'string' || typeof file.filename !== 'string',
        ),
    )
  )
    throw new Error('Modrinth 版本列表格式错误')
  return data
}

export default {
  searchProjects,
  getProject,
  getProjectVersions,
}
