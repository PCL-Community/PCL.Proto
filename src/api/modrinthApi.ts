import { invoke, isTauri } from '@tauri-apps/api/core'
import type { IProject, ISearchHit, IVersion } from '@/types/comp'

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
