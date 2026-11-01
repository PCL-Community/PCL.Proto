// CurseForge 接入的离线检查：归一化规则、请求参数与双来源合并，全部走假 invoke，不联网。
// 样本字段取自 2026-10-31 的 CF 真实响应。对照 ../PCL2-CE 的
// ModComp._BuildFromCurseForge / CompFile / CompFilesGet / CompProjectsGet。
import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { build } from 'esbuild'

const root = fileURLToPath(new URL('../../', import.meta.url))
const outDir = path.join(root, 'node_modules/.tmp')
await fs.mkdir(outDir, { recursive: true })
for (const entry of [
  'src/api/curseforgeApi.ts',
  'src/api/compProjects.ts',
  'src/api/compResources.ts',
])
  await build({
    entryPoints: [path.join(root, entry)],
    outfile: path.join(outDir, path.basename(entry, '.ts') + '.mjs'),
    bundle: true,
    packages: 'external',
    platform: 'node',
    format: 'esm',
    alias: { '@': path.join(root, 'src') },
    // pclLocale 依赖 Vite 的 import.meta.glob，测试里换成空表即可。
    define: { 'import.meta.glob': 'globalThis.__pclLocaleGlob' },
    logLevel: 'silent',
  })
globalThis.__pclLocaleGlob = () => ({})
const curseforge = await import(pathToFileURL(path.join(outDir, 'curseforgeApi.mjs')).href)
const projects = await import(pathToFileURL(path.join(outDir, 'compProjects.mjs')).href)
const resources = await import(pathToFileURL(path.join(outDir, 'compResources.mjs')).href)
const { curseforgeSearchHit, curseforgeFileVersion } = curseforge
const { default: curseforgeApi } = curseforge
const { compSourceOf, getCompVersions, mergeCompHits, rankCompHits, searchProjects } = projects
const testInstance = { version: '1.20.1', pluginsVersion: { fabric: '1' } }
const testVersions = [
  {
    id: 'beta',
    version_type: 'beta',
    game_versions: ['1.20.1'],
    loaders: ['fabric'],
    date_published: '2025-01-01',
    files: [{ url: 'https://cdn.modrinth.com/a', filename: 'a.jar' }],
  },
  {
    id: 'release',
    version_type: 'release',
    game_versions: ['1.20.1'],
    loaders: ['fabric'],
    date_published: '2024-01-01',
    files: [{ url: 'https://cdn.modrinth.com/b', filename: 'b.jar' }],
  },
]
assert.equal(resources.latestResourceVersion(testVersions, testInstance, 'mod').id, 'release')
assert.equal(
  resources.compatibleResource(
    testVersions[0],
    { ...testInstance, pluginsVersion: { forge: '1' } },
    'mod',
  ),
  false,
)
assert.equal(
  resources.compatibleResource(testVersions[0], { ...testInstance, version: '1.19.2' }, 'mod'),
  false,
)

assert.deepEqual(
  resources.instanceCompFilter(
    { version: '1.20.1', pluginsVersion: { fabric: '1', forge: '1' } },
    'mod',
  ),
  { version: '1.20.1', loader: 'forge' },
)
assert.deepEqual(
  resources.instanceCompFilter({ version: '1.20.1', pluginsVersion: { fabric: '1' } }, 'shader'),
  { version: '1.20.1', loader: '' },
)
assert.deepEqual(resources.instanceCompFilter(null, 'mod'), { version: '', loader: '' })
const matchingFiles = [
  { id: 'a', game_versions: ['1.20.1'], loaders: ['fabric'], date_published: '2024-01-01' },
  { id: 'b', game_versions: ['1.20.1'], loaders: ['forge'], date_published: '2024-02-01' },
  { id: 'c', game_versions: ['1.20.2'], loaders: ['fabric'], date_published: '2024-03-01' },
]
assert.deepEqual(
  resources
    .selectedCompVersions(matchingFiles, '1.20.1', 'fabric', '1.20', 'fabric', true, false)
    .map((file) => file.id),
  ['a'],
)
assert.deepEqual(
  resources.selectedCompVersions(matchingFiles, '1.20.1', 'fabric', '1.20', 'forge', true, false),
  [],
)
assert.deepEqual(
  resources.selectedCompVersions(matchingFiles, '1.20.1', 'fabric', '1.21', '', true, false),
  [],
)

const cfProject = {
  id: 322115,
  name: 'Just Enough Items (JEI)',
  slug: 'jei',
  summary: 'View items and recipes',
  downloadCount: 60,
  dateModified: '2024-02-02T00:00:00.000Z',
  dateReleased: '2024-01-01T00:00:00.000Z',
  classId: 6,
  links: { websiteUrl: 'https://www.curseforge.com/minecraft/mc-mods/jei' },
  logo: {
    thumbnailUrl: 'https://image.cursecdn.com/jei-thumb.png',
    url: 'https://image.cursecdn.com/jei.png',
  },
  categories: [
    { id: 6, name: 'Mods', isClass: true },
    { id: 412, name: 'Technology', isClass: false },
    { id: 421, name: 'Library', slug: 'library' },
  ],
  // CF 的搜索结果不给 loaderName，只给 modLoader 数字枚举。
  latestFilesIndexes: [
    { fileId: 1, gameVersion: '1.20.1', modLoader: 1 },
    { fileId: 1, gameVersion: 'Forge', modLoader: 1 },
    { fileId: 2, gameVersion: '1.19.2', modLoader: 4 },
    { fileId: 2, gameVersion: '26.3', modLoader: 6 },
  ],
}

// 搜索结果归一化：分类保留 CF 数字 ID，版本与加载器来自 latestFilesIndexes。
const normalized = curseforgeSearchHit(cfProject)
assert.equal(normalized.project_id, '322115')
assert.equal(normalized.project_type, 'mod')
assert.equal(normalized.icon_url, 'https://image.cursecdn.com/jei-thumb.png')
assert.equal(normalized.date_modified, '2024-01-01T00:00:00.000Z')
assert.deepEqual(normalized.categories, ['412', '421'])
assert.deepEqual(normalized.versions, ['1.20.1', '1.19.2', '26.3'])
assert.deepEqual(normalized.loaders, ['forge', 'fabric', 'neoforge'])
assert.equal(compSourceOf(normalized.project_id), 'curseforge')
assert.equal(compSourceOf('ehG7t1wE'), 'modrinth')
assert.equal(
  curseforgeApi.projectUrl({ slug: 'jei', project_type: 'mod' }),
  'https://www.curseforge.com/minecraft/mc-mods/jei',
)

// 文件归一化：gameVersions 混着版本、加载器和 Client/Server，需要拆开。
const version = curseforgeFileVersion({
  id: 9195226,
  modId: 394468,
  displayName: 'Sodium 0.9.3-beta.1 for Fabric 26.3',
  fileName: 'sodium-fabric-0.9.3-beta.1+mc26.3.jar',
  fileDate: '2026-10-15T17:51:37.020Z',
  fileLength: 1912496,
  releaseType: 2,
  downloadUrl: 'https://edge.forgecdn.net/files/9195/226/sodium-fabric-0.9.3-beta.1%2bmc26.3.jar',
  gameVersions: ['Client', 'Fabric', '26.3'],
})
assert.deepEqual(version.game_versions, ['26.3'])
assert.deepEqual(version.loaders, ['fabric'])
assert.equal(version.files[0].size, 1912496)
// CF 已把文件名里的 + 编成 %2b，不能改写掉，所以直接用接口给出的地址。
assert.equal(
  version.files[0].url,
  'https://edge.forgecdn.net/files/9195/226/sodium-fabric-0.9.3-beta.1%2bmc26.3.jar',
)
assert.deepEqual(
  curseforgeFileVersion({
    id: 3909837,
    modId: 322115,
    displayName: 'JEI 1.20.1-14.2.1',
    fileName: 'jei 14.2.1.jar',
    fileDate: '2024-01-01T00:00:00.000Z',
    fileLength: '9443225',
    downloadUrl: null,
    gameVersions: ['Client', '1.20.1', 'Forge', 'Server', 'OptiFine'],
  }).files[0].url,
  // 缺 downloadUrl 时按 id 补 CDN 地址，空格保持 %20。
  'https://edge.forgecdn.net/files/3909/837/jei%2014.2.1.jar',
)
assert.equal(
  curseforgeFileVersion({
    id: 1,
    modId: 1,
    fileName: 'a.jar',
    fileDate: '2024-01-01T00:00:00.000Z',
    downloadUrl: 'https://mediafiles.forgecdn.net/files/1/1/a.jar',
    gameVersions: ['1.20.1'],
  }).files[0].url,
  // media* 域名统一改写成 edge.forgecdn.net（对照 CE 的 HandleCurseForgeDownloadUrls）。
  'https://edge.forgecdn.net/files/1/1/a.jar',
)

// 请求参数对照 CE 的 GetCurseForgeAddress。
assert.equal(
  curseforgeApi.searchEndpoint({ type: 'mod', sort: '0', offset: 0, limit: 40 }),
  'mods/search?gameId=432&sortOrder=desc&pageSize=40&classId=6&sortField=2',
)
assert.equal(
  curseforgeApi.searchEndpoint({
    type: 'resourcepack',
    query: 'a & b',
    loader: 'fabric',
    version: '1.20.1',
    categoryId: '403',
    sort: '3',
    offset: 80,
    limit: 100,
  }),
  'mods/search?gameId=432&sortOrder=desc&pageSize=50&classId=12&categoryId=403&modLoaderType=4' +
    '&gameVersion=1.20.1&searchFilter=a+%26+b&sortField=2&index=80',
)
// index + pageSize 超过 10000 会被上游直接拒绝，所以要钳住偏移量。
assert.match(curseforgeApi.searchEndpoint({ type: 'mod', offset: 99999, limit: 40 }), /index=9960$/)

const requested = []
const file = (index) => ({
  id: 1000 + index,
  modId: 322115,
  displayName: `file ${index}`,
  fileName: `file-${index}.jar`,
  fileDate: '2024-01-01T00:00:00.000Z',
  fileLength: 1,
  isAvailable: true,
  downloadUrl: `https://mediafilez.forgecdn.net/files/${Math.floor((1000 + index) / 1000)}/${(1000 + index) % 1000}/file-${index}.jar`,
  gameVersions: ['1.20.1', 'Forge'],
})
const unavailable = {
  id: 7,
  modId: 322115,
  fileName: 'gone.jar',
  fileDate: '2024-01-01T00:00:00.000Z',
  isAvailable: false,
  downloadUrl: 'https://edge.forgecdn.net/files/0/7/gone.jar',
  gameVersions: ['1.20.1'],
}
const mrHit = (id, title, downloads) => ({
  project_id: id,
  project_type: 'mod',
  title,
  description: 'View items and recipes',
  icon_url: 'https://cdn.modrinth.com/icon.png',
  downloads,
  date_modified: '2024-01-02T00:00:00.000Z',
  categories: ['technology'],
  versions: ['1.20.1', '1.19.2', '26.3'],
  loaders: ['forge', 'fabric', 'neoforge'],
})
let handler = async (command, args) => {
  if (command === 'curseforge_available') return true
  const endpoint = String(args.endpoint)
  requested.push(endpoint)
  if (endpoint.startsWith('mods/search'))
    return { data: [cfProject], pagination: { totalCount: 100000, pageSize: 40 } }
  if (endpoint === 'mods/322115/files?pageSize=50&index=0')
    return {
      data: Array.from({ length: 50 }, (_, i) => file(i)).concat(unavailable),
      pagination: { index: 0, pageSize: 50, totalCount: 55 },
    }
  if (endpoint === 'mods/322115/files?pageSize=50&index=50')
    return {
      data: Array.from({ length: 5 }, (_, i) => file(50 + i)),
      pagination: { index: 50, pageSize: 50, totalCount: 55 },
    }
  if (endpoint.startsWith('mods/322115')) return { data: cfProject }
  if (endpoint.startsWith('search?'))
    return {
      hits: [
        mrHit('ehG7t1wE', 'Just Enough Items (JEI)', 20),
        mrHit('otherMod', 'Other Mod', 1000),
      ],
      total_hits: 2,
      offset: 0,
      limit: 40,
    }
  throw new Error(`unexpected endpoint ${endpoint}`)
}
globalThis.isTauri = true
globalThis.window = {
  __TAURI_INTERNALS__: { invoke: (command, args) => handler(command, args) },
}

const search = await curseforgeApi.searchProjects({ type: 'mod', sort: '2', limit: 40 })
// CF 的 totalCount 饱和在 10000，不能当成目录总量展示。
assert.equal(search.total_hits, 10000)

requested.length = 0
const allFiles = await curseforgeApi.getProjectVersions('322115')
// 56 条里去掉 1 条不可用文件，并且只请求了两页。
assert.equal(allFiles.length, 55)
assert.match(allFiles[0].files[0].url, /^https:\/\/edge\.forgecdn\.net\/files\/1\/\d+\//)
assert.deepEqual(requested, [
  'mods/322115/files?pageSize=50&index=0',
  'mods/322115/files?pageSize=50&index=50',
])

// 已知目标和加载器时让上游先筛。
requested.length = 0
handler = async (command, args) => {
  requested.push(String(args.endpoint))
  // CF 分页返回对象，Modrinth 的版本接口直接返回数组。
  return String(args.endpoint).startsWith('project/')
    ? []
    : { data: [], pagination: { totalCount: 0 } }
}
await getCompVersions('322115', { version: '1.20.1', loader: 'fabric' })
assert.deepEqual(requested, [
  'mods/322115/files?pageSize=50&gameVersion=1.20.1&modLoaderType=4&index=0',
])
// Modrinth 的版本接口没有筛选参数，来源派发仍然正确。
requested.length = 0
await getCompVersions('ehG7t1wE')
assert.deepEqual(requested, ['project/ehG7t1wE/version'])

// 双来源合并：同名同简介且更新时间相差不到一周的算同一个工程，Modrinth 在前。
requested.length = 0
handler = async (command, args) => {
  const endpoint = String(args.endpoint)
  requested.push(endpoint)
  if (endpoint.startsWith('mods/search'))
    return { data: [cfProject], pagination: { totalCount: 100000, pageSize: 40 } }
  return {
    hits: [mrHit('ehG7t1wE', 'Just Enough Items (JEI)', 20), mrHit('otherMod', 'Other Mod', 1000)],
    total_hits: 2,
    offset: 0,
    limit: 40,
  }
}
const merged = await searchProjects(
  'all',
  { type: 'mod', query: 'jei', loader: '', version: '', tag: '', sort: '1' },
  0,
  40,
)
assert.deepEqual(
  merged.hits.map((item) => item.project_id),
  ['ehG7t1wE', 'otherMod'],
)
assert.equal(merged.warning, '')
assert.equal(merged.total, 10002)
assert.equal(
  mergeCompHits(
    [{ ...mrHit('abc', 'One', 1), description: '' }],
    [{ ...normalized, name: 'Two', title: 'Two', description: '' }],
  ).length,
  2,
)
assert.equal(
  mergeCompHits([mrHit('abc', normalized.title, 1)], [{ ...normalized, loaders: ['fabric'] }])
    .length,
  2,
)
assert.equal(
  mergeCompHits([mrHit('abc', normalized.title, 1)], [{ ...normalized, versions: ['1.20.1'] }])
    .length,
  2,
)
// 同一来源内部不去重，即便名字一模一样。
assert.deepEqual(
  mergeCompHits(
    [],
    [
      { ...normalized, project_id: '1' },
      { ...normalized, project_id: '2' },
    ],
  ).map((item) => item.project_id),
  ['1', '2'],
)

// 一个来源失败时仍然展示另一个来源的结果，并给出 CE 的提示文案。
handler = async (command, args) => {
  if (String(args.endpoint).startsWith('mods/search')) throw new Error('CurseForge 403')
  return { hits: [normalized], total_hits: 1, offset: 0, limit: 40 }
}
const partial = await searchProjects(
  'all',
  { type: 'mod', query: 'jei', loader: '', version: '', tag: '', sort: '1' },
  0,
  40,
)
assert.equal(partial.hits.length, 1)
assert.ok(partial.warning.includes('CurseForge'))
await assert.rejects(
  searchProjects(
    'curseforge',
    { type: 'mod', query: '', loader: '', version: '', tag: '', sort: '0' },
    0,
    40,
  ),
  /CurseForge 403/,
)

// tag 只映射到一侧时，另一侧不请求：CF 缺 categoryId 会退化成无过滤，Modrinth 同理。
requested.length = 0
handler = async (command, args) => {
  requested.push(String(args.endpoint))
  if (String(args.endpoint).startsWith('mods/search'))
    return { data: [], pagination: { totalCount: 0 } }
  return { hits: [], total_hits: 0, offset: 0, limit: 40 }
}
await searchProjects(
  'all',
  { type: 'world', query: '', loader: '', version: '', tag: '248/', sort: '0' },
  0,
  40,
)
assert.ok(requested.length && requested.every((endpoint) => endpoint.startsWith('mods/search')))
requested.length = 0
await searchProjects(
  'all',
  { type: 'mod', query: '', loader: '', version: '', tag: '/game-mechanics', sort: '0' },
  0,
  40,
)
assert.ok(requested.length && requested.every((endpoint) => endpoint.startsWith('search?')))

// 默认顺序按 CE 的来源权重折算下载量，避免 CF 的口径压过 Modrinth。
const ranked = rankCompHits(
  [
    { ...normalized, downloads: 100 },
    { ...normalized, project_id: 'ehG7t1wE', downloads: 20 },
  ],
  'mod',
)
assert.equal(ranked[0].project_id, 'ehG7t1wE')

await Promise.all(
  ['curseforgeApi.mjs', 'compProjects.mjs', 'compResources.mjs'].map((file) =>
    fs.rm(path.join(outDir, file), { force: true }),
  ),
)
console.log('CurseForge integration checks passed.')
