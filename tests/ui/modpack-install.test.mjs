import assert from 'node:assert/strict'
import { build } from 'esbuild'

const mocks = {
  '@tauri-apps/api/core': `export const invoke = (...args) => globalThis.testInvoke(...args)`,
  '@/api/pclLocale': `export const pclText = (key) => key`,
  '@/composables/useModal': `export const useModal = () => ({ open: async (options) => { globalThis.prompts.push(options); return globalThis.choices.shift() } })`,
  '@/composables/sideTip': `export default { show() {} }`,
  '@/stores/task': `export const isTaskCancelled = (error) => String(error) === 'Download cancelled'; export const useTaskManager = () => globalThis.taskManager`,
}
const result = await build({
  entryPoints: ['src/api/modpack.ts'],
  bundle: true,
  write: false,
  platform: 'node',
  format: 'esm',
  plugins: [
    {
      name: 'mocks',
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
const { installOnlineModpack, importLocalModpack } = await import(
  `data:text/javascript;base64,${Buffer.from(result.outputFiles[0].contents).toString('base64')}`
)
const info = {
  name: 'Pack',
  minecraft: '1.20.1',
  fabric: true,
  optional_files: ['one.jar', 'two.jar'],
}
const calls = []
globalThis.prompts = []
globalThis.choices = [
  { confirmed: true, input: 'My Pack' },
  { confirmed: true },
  { confirmed: false },
]
globalThis.testInvoke = async (name, args) => {
  calls.push([name, args])
  if (name === 'inspect_modpack') return info
}
globalThis.taskManager = {
  prepareOnlineModpack: async (...args) => {
    calls.push(['prepare', ...args])
    return { token: 'token', info }
  },
  installOnlineModpack: async (...args) => calls.push(['install', ...args]),
  installModpack: async (...args) => calls.push(['local', ...args]),
}
const file = { url: 'https://cdn.modrinth.com/pack', size: 42, hashes: { sha1: 'abc' } }
await installOnlineModpack(file, 'Pack')
assert.equal(prompts[1].confirmText, 'Minecraft.Download.Modpack.OptionalFile.Download')
assert.equal(prompts[1].cancelText, 'Minecraft.Download.Modpack.OptionalFile.Skip')
assert.deepEqual(calls, [
  ['prepare', 'My Pack', { url: file.url, size: 42, sha1: 'abc' }],
  ['install', 'token', 'My Pack', ['one.jar'], true],
  ['release_online_modpack', { token: 'token' }],
])
calls.length = 0
globalThis.choices = [
  { confirmed: true, input: 'Local' },
  { confirmed: false },
  { confirmed: true },
]
await importLocalModpack('/pack.mrpack')
assert.deepEqual(calls, [
  ['inspect_modpack', { path: '/pack.mrpack' }],
  ['local', '/pack.mrpack', 'Local', ['two.jar'], true],
])
calls.length = 0
globalThis.choices = [
  { confirmed: true, input: 'My Pack' },
  { confirmed: false },
  { confirmed: false },
]
taskManager.installOnlineModpack = async () => {
  throw new Error('install failed')
}
await assert.rejects(installOnlineModpack(file, 'Pack'), /install failed/)
assert.deepEqual(calls.at(-1), ['release_online_modpack', { token: 'token' }])
calls.length = 0
globalThis.choices = [{ confirmed: true, input: 'My Pack' }]
taskManager.prepareOnlineModpack = async () => {
  throw 'Download cancelled'
}
await installOnlineModpack(file, 'Pack')
assert.deepEqual(calls, [])
globalThis.choices = [{ confirmed: true, input: 'CON' }, { confirmed: false }]
const promptCount = prompts.length
await installOnlineModpack(file, 'invalid/name')
assert.equal(prompts[promptCount].defaultInputText, 'invalid name')
assert.equal(prompts[promptCount + 1].defaultInputText, 'CON')
assert.deepEqual(calls, [])
