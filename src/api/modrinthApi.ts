import { invoke } from '@tauri-apps/api/core'

// 通用 Modrinth API 请求
async function modrinthFetch<T>(endpoint: string): Promise<T> {
  return invoke<T>('fetch_with_modrinth', { endpoint })
}

interface SearchOptions {
  query?: string
  facets?: string[][] // 二维数组，支持 AND/OR 逻辑
  index?: 'relevance' | 'downloads' | 'follows' | 'newest' | 'updated'
  offset?: number
  limit?: number
}

export interface ISearchHit {
  title: string
  description: string
  icon_url: string
  categories: string[]
  downloads: number
  date_modified: string
  project_id: string
  project_type: 'mod' | 'modpack' | 'resourcepack' | 'shader'
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
  project_type: 'mod' | 'modpack' | 'resourcepack' | 'shader'
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
  game_versions: string[]
}

// 搜索项目
async function searchProjects(options?: SearchOptions) {
  if (!options)
    return modrinthFetch<{ hits: ISearchHit[]; offset: number; limit: number; total_hits: number }>(
      `search`,
    )
  const params = new URLSearchParams()
  if (options.query) params.append('query', options.query)
  if (options.facets) params.append('facets', JSON.stringify(options.facets))
  if (options.index) params.append('index', options.index)
  if (options.offset) params.append('offset', options.offset.toString())
  if (options.limit) params.append('limit', options.limit.toString())
  return modrinthFetch<{ hits: ISearchHit[]; offset: number; limit: number; total_hits: number }>(
    `search?${params.toString()}`,
  )
}

// 获取项目详情
async function getProject(id: string): Promise<IProject> {
  return modrinthFetch<IProject>(`project/${id}`)
}

// 获取项目版本
async function getProjectVersions(id: string) {
  return modrinthFetch(`project/${id}/version`)
}

export default {
  searchProjects,
  getProject,
  getProjectVersions,
}
