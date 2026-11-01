import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { build } from 'esbuild'
import { computed } from 'vue'

const root = fileURLToPath(new URL('../', import.meta.url))
const languages = JSON.parse(await fs.readFile(path.join(root, 'src/assets/language-names.json')))
const dictionaries = {}
for (const directory of ['languages', 'messages']) {
  for (const file of await fs.readdir(path.join(root, 'src/assets', directory))) {
    if (file.endsWith('.json')) {
      dictionaries[`../assets/${directory}/${file}`] = JSON.parse(
        await fs.readFile(path.join(root, 'src/assets', directory, file)),
      )
    }
  }
}
const zh = dictionaries['../assets/messages/zh-CN.json']
const en = dictionaries['../assets/messages/en-US.json']
assert.deepEqual(Object.keys(en).sort(), Object.keys(zh).sort())
for (const key of Object.keys(zh)) {
  assert.deepEqual(
    [...en[key].matchAll(/\{\d+\}/g)].map(([token]) => token).sort(),
    [...zh[key].matchAll(/\{\d+\}/g)].map(([token]) => token).sort(),
    `inconsistent placeholders: ${key}`,
  )
}
for (const locale of languages) assert.ok(dictionaries[`../assets/languages/${locale}.json`])

const storage = new Map()
const calls = new Map()
const hooks = new Map()
const originals = Object.fromEntries(
  ['localStorage', 'document', 'navigator', '__pclLocaleGlob'].map((name) => [
    name,
    Object.getOwnPropertyDescriptor(globalThis, name),
  ]),
)
const modulePath = path.join(root, 'node_modules/.tmp', `pcl-locale-${process.pid}.mjs`)
try {
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key) => storage.get(key) ?? null,
      setItem: (key, value) => storage.set(key, value),
    },
  })
  globalThis.document = { documentElement: { lang: '' } }
  Object.defineProperty(globalThis, 'navigator', {
    configurable: true,
    value: { language: 'en-AU' },
  })
  // Supply Vite's import adapter while testing the real translation and switching code.
  globalThis.__pclLocaleGlob = (pattern) =>
    Object.fromEntries(
      Object.entries(dictionaries)
        .filter(([key]) => key.startsWith(pattern.slice(0, -'*.json'.length)))
        .map(([key, value]) => [
          key,
          async () => {
            calls.set(key, (calls.get(key) || 0) + 1)
            await hooks.get(key)?.()
            return value
          },
        ]),
    )
  await build({
    entryPoints: [path.join(root, 'src/api/pclLocale.ts')],
    outfile: modulePath,
    bundle: true,
    packages: 'external',
    platform: 'node',
    format: 'esm',
    define: { 'import.meta.glob': 'globalThis.__pclLocaleGlob' },
    logLevel: 'silent',
  })
  const {
    pclText,
    pclLocale,
    pclLanguagePreference,
    pclFormatPreference,
    pclFormatLocale,
    resolveSystemLanguage,
    setPclFormatLocale,
    loadPclLocale,
    setPclLocale,
    initializePclLocale,
  } = await import(pathToFileURL(modulePath))
  storage.set('ce-language', 'EN-us')
  await initializePclLocale()
  assert.equal(pclLocale.value, 'en-US')
  assert.equal(storage.get('pcl-language'), 'en-US')
  assert.equal(document.documentElement.lang, 'en-US')
  assert.equal(resolveSystemLanguage('en-AU'), 'en-US')
  assert.equal(resolveSystemLanguage('zh-TW'), 'zh-TW')
  assert.equal(resolveSystemLanguage('ko-KR'), 'zh-CN')
  await setPclLocale('auto')
  assert.equal(pclLocale.value, 'en-US')
  assert.equal(pclLanguagePreference.value, 'auto')
  assert.equal(storage.get('pcl-language'), 'auto')
  setPclFormatLocale('ui-language')
  assert.equal(pclFormatLocale.value, 'en-US')
  await setPclLocale('en-US')
  assert.equal(pclFormatLocale.value, 'en-US')
  setPclFormatLocale('de-DE')
  assert.equal(pclFormatLocale.value, 'de-DE')
  await setPclLocale('zh-CN')
  assert.equal(pclFormatLocale.value, 'de-DE')
  setPclFormatLocale('auto')
  assert.equal(pclFormatPreference.value, 'auto')
  assert.equal(pclFormatLocale.value, Intl.DateTimeFormat().resolvedOptions().locale)
  await setPclLocale('en-US')
  assert.equal(pclText('Proto.Empty.Tasks'), 'No tasks')
  assert.equal(pclText('{DynamicResource Main.Tab.Launch}'), pclText('Main.Tab.Launch'))
  assert.equal(pclText('{StaticResource Main.Tab.Launch}'), pclText('Main.Tab.Launch'))
  assert.equal(pclText('missing.key'), 'missing.key')
  assert.equal(pclText('{DynamicResource missing.key}'), '{DynamicResource missing.key}')
  assert.equal(pclText('A {0} {0} {1}', ['$&', 0]), 'A $& $& 0')
  assert.equal(pclText('A {0}', ['{1}']), 'A {1}')
  assert.equal(pclText('A {0}'), 'A {0}')
  assert.equal(pclText('PCL CE'), 'PCL.Proto')
  assert.ok(pclText('Proto.Language.Banner.DescriptionLine1', ['PCL CE']).includes('PCL CE'))
  const reactiveText = computed(() => pclText('Proto.Empty.Tasks'))
  assert.equal(reactiveText.value, 'No tasks')
  await setPclLocale('zh-CN')
  assert.ok(pclText('Proto.Language.Banner.DescriptionLine2', ['PCL CE']).includes('PCL CE'))
  assert.equal(reactiveText.value, '暂无任务')
  await setPclLocale('en-US')
  assert.equal(calls.get('../assets/languages/en-US.json'), 1)

  let release
  hooks.set(
    '../assets/languages/fr-FR.json',
    () =>
      new Promise((resolve) => {
        release = resolve
      }),
  )
  const stale = setPclLocale('fr-FR')
  await setPclLocale('zh-CN')
  release()
  await stale
  assert.equal(
    pclLocale.value,
    'zh-CN',
    'a slow previous selection must not overwrite the last one',
  )
  assert.equal(storage.get('pcl-language'), 'zh-CN')
  await setPclLocale('fr-FR')
  assert.equal(pclText('Proto.Empty.Tasks'), zh['Proto.Empty.Tasks'])

  hooks.set('../assets/languages/ja-JP.json', () => {
    throw new Error('network error')
  })
  await assert.rejects(setPclLocale('ja-JP'), /network error/)
  assert.equal(pclLocale.value, 'fr-FR')
  hooks.delete('../assets/languages/ja-JP.json')
  await Promise.all([loadPclLocale('ja-JP'), loadPclLocale('ja-JP')])
  assert.equal(
    calls.get('../assets/languages/ja-JP.json'),
    2,
    'failed loads must be retried and concurrent loads shared',
  )
  await assert.rejects(setPclLocale('unknown'), RangeError)
  storage.set('pcl-language', 'unknown')
  await initializePclLocale()
  assert.equal(pclLocale.value, 'zh-CN')
  const originalWarn = console.warn
  const warnings = []
  console.warn = (...args) => warnings.push(args)
  try {
    storage.set('pcl-language', 'es-ES')
    hooks.set('../assets/languages/es-ES.json', () => {
      throw new Error('startup network error')
    })
    await initializePclLocale()
    assert.equal(
      pclLocale.value,
      'zh-CN',
      'startup loading failures must fall back before mounting',
    )
    hooks.delete('../assets/languages/es-ES.json')
    const getItem = localStorage.getItem
    const setItem = localStorage.setItem
    localStorage.getItem = () => {
      throw new Error('storage disabled')
    }
    localStorage.setItem = () => {
      throw new Error('storage disabled')
    }
    await initializePclLocale()
    await setPclLocale('en-US')
    assert.equal(
      pclLocale.value,
      'en-US',
      'storage failures must not prevent session language changes',
    )
    assert.equal(document.documentElement.lang, 'en-US')
    localStorage.getItem = getItem
    localStorage.setItem = setItem
    assert.ok(warnings.length > 0)
  } finally {
    console.warn = originalWarn
  }
  for (const locale of languages) await setPclLocale(locale)
  storage.delete('pcl-language')
  storage.delete('ce-language')
  storage.set('pcl-format-locale', 'ui-language')
  await initializePclLocale()
  assert.equal(pclLanguagePreference.value, 'auto')
  assert.equal(pclLocale.value, 'en-US')
  assert.equal(pclFormatPreference.value, 'ui-language')
  assert.equal(pclFormatLocale.value, 'en-US')
  storage.set('pcl-language', 'zh-CN')
  storage.set('pcl-format-locale', 'ja-JP')
  await initializePclLocale()
  assert.equal(pclLocale.value, 'zh-CN')
  assert.equal(pclFormatLocale.value, 'ja-JP')
  console.log(
    `Locale checks passed (${languages.length} languages, ${Object.keys(zh).length} application messages).`,
  )
} finally {
  await fs.rm(modulePath, { force: true })
  for (const [name, descriptor] of Object.entries(originals)) {
    if (descriptor) Object.defineProperty(globalThis, name, descriptor)
    else delete globalThis[name]
  }
}
