import assert from 'node:assert/strict'
import { build } from 'esbuild'
import 'vue'

const properties = new Map()
const classes = new Set()
let systemChange
globalThis.document = {
  documentElement: {
    style: { setProperty: (key, value) => properties.set(key, value) },
    classList: { toggle: (key, enabled) => (enabled ? classes.add(key) : classes.delete(key)) },
  },
}
const media = {
  matches: false,
  addEventListener: (_event, handler) => {
    systemChange = handler
  },
}
globalThis.window = { matchMedia: () => media }

const result = await build({
  entryPoints: ['src/api/appearance.ts'],
  bundle: true,
  write: false,
  format: 'esm',
  platform: 'node',
  plugins: [
    {
      name: 'settings',
      setup(builder) {
        builder.onResolve({ filter: /^vue$/ }, () => ({
          path: import.meta.resolve('vue'),
          external: true,
        }))
        builder.onResolve({ filter: /^@\/composables\/usePageSettings$/ }, () => ({
          path: 'settings',
          namespace: 'test',
        }))
        builder.onResolve({ filter: /^@\/api\/pclLocale$/ }, () => ({
          path: 'locale',
          namespace: 'test',
        }))
        builder.onResolve({ filter: /^@\/composables\/sideTip$/ }, () => ({
          path: 'sideTip',
          namespace: 'test',
        }))
        builder.onLoad({ filter: /^settings$/, namespace: 'test' }, () => ({
          contents:
            "import { reactive } from 'vue'; export const usePageSettings = (_, defaults) => ({ values: reactive(defaults) })",
          resolveDir: process.cwd(),
        }))
        builder.onLoad({ filter: /^locale$/, namespace: 'test' }, () => ({
          contents: 'export const pclText = (key) => key',
        }))
        builder.onLoad({ filter: /^sideTip$/, namespace: 'test' }, () => ({
          contents: 'export default { show() {} }',
        }))
      },
    },
  ],
})
const { appearance } = await import(
  `data:text/javascript;base64,${Buffer.from(result.outputFiles[0].text).toString('base64')}`
)
function checkPalette() {
  for (let i = 1; i <= 8; i++) {
    assert.match(properties.get(`--ce-color-${i}`), /^#[0-9A-F]{6}$/)
  }
}
assert.equal(classes.has('dark'), false)
const cat = properties.get('--ce-color-3')
const catBackground = properties.get('--base-background')
checkPalette()
appearance.values.CheckBackgroundColorful = false
assert.equal(properties.get('--base-background'), 'var(--color-background)')
appearance.values.CheckBackgroundColorful = true
appearance.values.ComboLightColor = '0'
assert.notEqual(properties.get('--ce-color-3'), cat)
assert.notEqual(properties.get('--base-background'), catBackground)
checkPalette()
assert.equal(properties.get('--ce-color-5'), '#92E0FF')
assert.equal(properties.get('--ce-color-7'), '#D6F8FF')
assert.match(properties.get('--base-background'), /#81D9F5.*#E1F5FF.*#9CD0FF/)
appearance.values.ComboUiFont = 'Example Font'
assert.match(properties.get('--custom-font'), /Example Font/)
appearance.values.ComboUiMotdFont = 'MOTD Font'
assert.match(properties.get('--motd-font'), /MOTD Font/)
media.matches = true
systemChange()
assert.equal(classes.has('dark'), true)
assert.notEqual(properties.get('--base-background'), catBackground)
checkPalette()
appearance.values.ComboDarkMode = '0'
assert.equal(classes.has('dark'), false)
appearance.values.ComboDarkMode = '1'
assert.equal(classes.has('dark'), true)
for (const mode of ['0', '1']) {
  appearance.values.ComboDarkMode = mode
  for (const theme of ['0', '1', '2']) {
    appearance.values[mode === '0' ? 'ComboLightColor' : 'ComboDarkColor'] = theme
    checkPalette()
  }
}
console.log('Appearance modes, colors, fonts and system changes passed.')
