import assert from 'node:assert/strict'
import { build } from 'esbuild'

const mocks = {
  '@tauri-apps/api/core': `
    export const isTauri = () => true;
    export const invoke = async (name, args) => { globalThis.testCalls.push([name, args]); return '/game/saves/world/datapacks/pack.zip'; };
  `,
  '@tauri-apps/api/path': `export const join = async (...parts) => parts.join('/');`,
  '@tauri-apps/plugin-dialog': `
    export const save = async (options) => { globalThis.testDialogs.push(options); return null; };
    export const open = async () => null;
  `,
  './pclLocale': `export const pclText = (key) => key;`,
  './compProjects': `export const getCompVersions = async () => [globalThis.testVersion]; export const compSourceOf = (id) => /^\\d+$/.test(id) ? 'curseforge' : 'modrinth';`,
  './compResources': `
    export const compatibleResource = () => true;
    export const latestResourceVersion = (versions) => versions[0];
  `,
  '@/stores/gameLaunch': `export const useSelectedInstance = () => ({ instance_info: globalThis.testInstance });`,
  '@/stores/repositories': `export const useRepositoriesStore = () => ({});`,
  '@/stores/task': `export const isTaskCancelled = (error) => String(error) === 'Download cancelled'; export const useTaskManager = () => ({ downloadResource: async (...args) => globalThis.testDownloads.push(args) });`,
  '@/api/modpack': `export const installOnlineModpack = async (...args) => globalThis.testModpacks.push(args);`,
  '@/composables/useModal': `export const useModal = () => ({});`,
  '@/composables/sideTip': `export default { show() {} };`,
}
const result = await build({
  entryPoints: ['src/api/compDownload.ts'],
  bundle: true,
  write: false,
  platform: 'node',
  format: 'esm',
  logLevel: 'silent',
  plugins: [
    {
      name: 'download-mocks',
      setup(builder) {
        builder.onResolve({ filter: /.*/ }, (args) =>
          Object.hasOwn(mocks, args.path) ? { path: args.path, namespace: 'mock' } : undefined,
        )
        builder.onLoad({ filter: /.*/, namespace: 'mock' }, (args) => ({
          contents: mocks[args.path],
        }))
      },
    },
  ],
})
const download = await import(
  `data:text/javascript;base64,${Buffer.from(result.outputFiles[0].contents).toString('base64')}`
)
globalThis.testCalls = []
globalThis.testDialogs = []
globalThis.testDownloads = []
globalThis.testModpacks = []
globalThis.localStorage = { getItem: () => JSON.stringify({ ComboDownloadQuickBehavior: '1' }) }
globalThis.testInstance = { directory: '/game/instance', version: '1.20.1' }
const file = {
  url: 'https://cdn.modrinth.com/pack.zip',
  filename: 'pack.zip',
  size: 20,
  hashes: {},
}
globalThis.testVersion = { name: 'Pack', files: [file] }
const target = { instanceDirectory: '/game/instance', save: 'world' }

await download.saveResource(file, testVersion, 'modpack', undefined, 'My Pack')
assert.deepEqual(testModpacks, [[file, 'My Pack']])
await download.quickDownload({ project_id: 'abc', project_type: 'modpack', title: 'Online Pack' })
assert.deepEqual(testModpacks.at(-1), [file, 'Online Pack'])
await download.quickDownload({
  project_id: '123',
  project_type: 'modpack',
  title: 'CurseForge Pack',
})
assert.equal(testModpacks.length, 2)

assert.deepEqual(
  download.datapackTargetFromQuery({ targetInstance: '/game/instance', save: 'world' }),
  target,
)
assert.equal(
  download.datapackTargetFromQuery({ targetInstance: '/game/instance', save: ['world'] }),
  undefined,
)
assert.equal(
  download.datapackTargetFromQuery({ targetInstance: '/game/instance', save: '' }),
  undefined,
)
await download.saveResource(file, testVersion, 'datapack', target)
assert.deepEqual(testCalls[0], [
  'instance_resource_destination',
  {
    instanceDirectory: '/game/instance',
    kind: 'datapack',
    filename: 'pack.zip',
    save: 'world',
  },
])
assert.equal(testDialogs[0].defaultPath, '/game/saves/world/datapacks/pack.zip')
await download.quickDownload({ project_id: 'abc', project_type: 'datapack', title: 'Pack' }, target)
assert.equal(testDownloads[0][2], '/game/saves/world/datapacks/pack.zip')
globalThis.testInstance = { directory: '/other/instance', version: '1.20.1' }
await assert.rejects(
  download.saveResource(file, testVersion, 'datapack', target),
  /selected instance changed/,
)
await assert.rejects(
  download.quickDownload({ project_id: 'abc', project_type: 'datapack', title: 'Pack' }, target),
  /selected instance changed/,
)
