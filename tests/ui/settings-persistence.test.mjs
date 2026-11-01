import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { build } from 'esbuild'
import { computed, effectScope, nextTick, ref } from 'vue'

const root = fileURLToPath(new URL('../../', import.meta.url))
const outfile = path.join(root, 'node_modules/.tmp/settings-persistence.mjs')
await fs.mkdir(path.dirname(outfile), { recursive: true })
await build({
  stdin: {
    contents: [
      "export { createSettingsPersistence } from './src/util/settingsPersistence'",
      "export { launchArgumentsStorage } from './src/api/launchArguments'",
      "export { memorySettingsStorage } from './src/api/memorySettings'",
      "export { windowSettingsStorage } from './src/api/windowSettings'",
      "export { downloadSettingsStorage } from './src/api/downloadSettings'",
      "export { instanceLaunchSettingsStorage } from './src/api/instanceLaunchSettings'",
      "export { useInstanceLaunchSettings } from './src/composables/useInstanceLaunchSettings'",
    ].join('\n'),
    resolveDir: root,
    loader: 'ts',
  },
  outfile,
  bundle: true,
  packages: 'external',
  platform: 'node',
  format: 'esm',
  alias: { '@': path.join(root, 'src') },
  logLevel: 'silent',
})
const { createSettingsPersistence, ...domains } = await import(pathToFileURL(outfile).href)

function deferred() {
  let resolve, reject
  const promise = new Promise((yes, no) => {
    resolve = yes
    reject = no
  })
  return { promise, resolve, reject }
}

// One scope serializes writes; another scope can save while it is blocked.
const started = deferred()
const release = deferred()
const persisted = new Map()
const calls = []
const storage = createSettingsPersistence(
  async (scope) => persisted.get(scope) ?? { value: 0 },
  async (scope, settings) => {
    calls.push([scope, settings.value])
    if (settings.value === 1) {
      started.resolve()
      await release.promise
    }
    persisted.set(scope, settings)
    return settings
  },
)
const firstInput = { value: 1 }
const first = storage.write(firstInput)
firstInput.value = 99
await started.promise
const second = storage.write({ value: 2 })
await storage.write({ value: 3 }, 'instance')
assert.deepEqual(calls, [
  [null, 1],
  ['instance', 3],
])
let readFinished = false
const reading = storage.read().then((result) => {
  readFinished = true
  return result
})
await Promise.resolve()
assert.equal(readFinished, false)
release.resolve()
await Promise.all([first, second])
assert.equal((await reading).settings.value, 2)
assert.deepEqual(calls, [
  [null, 1],
  ['instance', 3],
  [null, 2],
])

// A failed write remains editable and retryable, without poisoning the queue.
let shouldFail = true
const failureStarted = deferred()
const failureRelease = deferred()
const retryable = createSettingsPersistence(
  async () => ({ value: 0 }),
  async (_, settings) => {
    if (shouldFail) {
      failureStarted.resolve()
      await failureRelease.promise
      throw new Error('disk full')
    }
    return settings
  },
)
const displayedError = computed(() => retryable.errors.get(null) || '')
const failure = assert.rejects(retryable.write({ value: 4 }), /disk full/)
await failureStarted.promise
failureRelease.resolve()
await failure
assert.match(displayedError.value, /disk full/)
const retained = await retryable.read()
assert.equal(retained.settings.value, 4)
assert.equal(retained.dirty, true)
retained.settings.value = 999
assert.equal((await retryable.read()).settings.value, 4)
shouldFail = false
await retryable.save()
assert.equal(displayedError.value, '')
assert.equal((await retryable.read()).dirty, false)

// Older completions cannot discard a newer draft or attach an outdated error.
for (const reject of [false, true]) {
  const started = deferred()
  const release = deferred()
  const storage = createSettingsPersistence(
    async () => ({ value: 0 }),
    async (_, settings) => {
      started.resolve()
      await release.promise
      return settings
    },
  )
  const saving = storage.write({ value: 5 })
  const settled = reject ? assert.rejects(saving, /old failure/) : saving
  await started.promise
  storage.stage({ value: 6 })
  if (reject) release.reject(new Error('old failure'))
  else release.resolve()
  await settled
  assert.deepEqual(await storage.read(), { settings: { value: 6 }, error: '', dirty: true })
}

// A draft entered during an outstanding backend read takes precedence.
const loading = deferred()
const readStorage = createSettingsPersistence(
  () => loading.promise,
  async (_, value) => value,
)
const loadingRead = readStorage.read()
await Promise.resolve()
readStorage.stage({ value: 7 })
loading.resolve({ value: 0 })
assert.equal((await loadingRead).settings.value, 7)

// Exercise the actual domain adapters and remounted instance editor through fake IPC.
const originals = Object.fromEntries(
  ['window', 'isTauri'].map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]),
)
const scopes = []
try {
  const calls = []
  const instanceWrite = deferred()
  const instanceStarted = deferred()
  const failedWrite = deferred()
  const failureStarted = deferred()
  const instances = new Map()
  globalThis.isTauri = true
  globalThis.window = {
    __TAURI_INTERNALS__: {
      invoke: async (command, args) => {
        calls.push([command, args])
        if (command === 'set_instance_launch_settings' && args.settings.info === 'first') {
          instanceStarted.resolve()
          await instanceWrite.promise
        }
        if (command === 'set_instance_launch_settings' && args.settings.info === 'old-failure') {
          failureStarted.resolve()
          await failedWrite.promise
          throw new Error('outdated save failure')
        }
        if (command === 'set_instance_launch_settings')
          instances.set(args.instanceDirectory, args.settings)
        if (command === 'get_instance_launch_settings')
          return instances.get(args.instanceDirectory) ?? { info: 'original' }
        if (command === 'get_window_settings' || command === 'get_download_settings') return null
        return args.settings ?? args
      },
    },
  }
  await assert.rejects(domains.windowSettingsStorage.read(), /not initialized/)
  await assert.rejects(domains.downloadSettingsStorage.read(), /not initialized/)
  const argumentsValue = { jvm: '-Xmx2G', game: '', classpath: '' }
  domains.launchArgumentsStorage.stage(argumentsValue, 'instance')
  assert.equal((await domains.launchArgumentsStorage.read('instance')).dirty, true)
  await domains.launchArgumentsStorage.save('instance')
  assert.deepEqual(calls.at(-1), [
    'set_launch_arguments',
    {
      instanceDirectory: 'instance',
      settings: argumentsValue,
    },
  ])
  const snapshot = { settings: { mode: 'auto' }, custom: { maximum_mib: 4096, initial_mib: null } }
  await domains.memorySettingsStorage.write(snapshot)
  assert.deepEqual(calls.at(-1), [
    'set_memory_settings',
    {
      instanceDirectory: null,
      ...snapshot,
      migrateOnly: false,
    },
  ])
  await domains.windowSettingsStorage.write({ mode: 'custom', width: 854, height: 480 })
  assert.equal(calls.at(-1)[1].migrateOnly, false)
  await domains.downloadSettingsStorage.write({ max_connections: 8 })
  assert.deepEqual(calls.at(-1), ['set_download_settings', { settings: { max_connections: 8 } }])

  function editor() {
    const scope = effectScope()
    scopes.push(scope)
    return scope.run(() => domains.useInstanceLaunchSettings(ref('instance')))
  }
  const firstEditor = editor()
  await firstEditor.load()
  firstEditor.settings.info = 'first'
  firstEditor.save()
  await instanceStarted.promise
  scopes[0].stop()
  const remounted = editor()
  const reloading = remounted.load()
  instanceWrite.resolve()
  await reloading
  assert.equal(remounted.settings.info, 'first')
  remounted.settings.info = 'second'
  remounted.save()
  await domains.instanceLaunchSettingsStorage.read('instance')
  assert.deepEqual(
    calls
      .filter(([command]) => command === 'set_instance_launch_settings')
      .map(([, args]) => args.settings.info),
    ['first', 'second'],
  )
  await nextTick()
  assert.equal(remounted.error.value, '')
  remounted.settings.info = 'old-failure'
  remounted.save()
  await failureStarted.promise
  remounted.settings.info = 'latest'
  remounted.save()
  failedWrite.resolve()
  const latest = await domains.instanceLaunchSettingsStorage.read('instance')
  assert.equal(latest.settings.info, 'latest')
  assert.equal(remounted.error.value, '')
} finally {
  for (const scope of scopes) scope.stop()
  for (const [key, descriptor] of Object.entries(originals)) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else delete globalThis[key]
  }
}
console.log(
  'Settings queues, isolated scopes, drafts, failures, retries and remounted editors passed.',
)
