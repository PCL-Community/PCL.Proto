import assert from 'node:assert/strict'
import { build } from 'esbuild'

const mocks = {
  '@/api/pclLocale': `export const pclText = (key) => key`,
  '@/composables/useFloatButton': `export const FloatButtonType = { TaskManage: 1 }; export const useFloatButton = () => ({ setFloatButton() {}, floatButtonState: {} })`,
  '@/stores/gameLaunch': `export const useSelectedInstance = () => ({ fetch: async () => {} })`,
  '@/composables/sideTip': `export default { show() {} }`,
  vue: `export const ref = (value) => ({ __ref: true, value }); export const computed = (fn) => ({ __ref: true, get value() { return fn() } })`,
  pinia: `export const defineStore = (_, setup) => { let store; return () => store ||= new Proxy(setup(), { get: (target, key) => target[key]?.__ref ? target[key].value : target[key] }) }`,
  '@tauri-apps/api/core': `
    export class Channel {}
    export const invoke = (name, args) => globalThis.testInvoke(name, args)
  `,
}
const result = await build({
  entryPoints: ['src/stores/task.ts'], bundle: true, write: false, platform: 'node', format: 'esm',
  plugins: [{ name: 'mocks', setup(builder) {
    builder.onResolve({ filter: /.*/ }, (args) => Object.hasOwn(mocks, args.path) ? { path: args.path, namespace: 'mock' } : undefined)
    builder.onLoad({ filter: /.*/, namespace: 'mock' }, (args) => ({ contents: mocks[args.path] }))
  } }],
})
const { useTaskManager, isTaskCancelled } = await import(`data:text/javascript;base64,${Buffer.from(result.outputFiles[0].contents).toString('base64')}`)
let rejectDownload
const calls = []
globalThis.testInvoke = (name, args) => {
  calls.push([name, args])
  if (name === 'download_minecraft_version') return new Promise((_, reject) => { rejectDownload = reject })
  if (name === 'cancel_download_task') rejectDownload('Download cancelled')
  return Promise.resolve()
}
const store = useTaskManager()
const pending = store.startDownloadMCVersion('1.20.1', 'Example', {
  family: 'fabric',
  version: '0.16',
  api: {
    url: 'https://cdn.modrinth.com/data/P7dR8mSH/versions/test/fabric-api.jar',
    size: 1,
    sha1: 'a'.repeat(40),
    filename: 'fabric-api.jar',
  },
})
await Promise.resolve()
const channel = calls.find(([name]) => name === 'download_minecraft_version')[1].on_event
channel.onmessage({ task_id: store.tasks[0].id, item_id: 3, progress: 0.5, status: 1, files_remaining: 1, speed: 0 })
assert.equal(store.tasks[0].progress, 9 / 44)
await store.cancelTask(store.tasks[0].id)
await assert.rejects(pending, isTaskCancelled)
assert.equal(store.tasks.length, 0)
assert.deepEqual(calls.map(([name]) => name), ['prepare_download_task', 'download_minecraft_version', 'cancel_download_task', 'release_download_task'])
